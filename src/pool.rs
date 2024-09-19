use crate::generated;

#[derive(Debug)]
#[repr(transparent)]
pub struct Pool(*mut generated::apr_pool_t);

// Pools are Send, but not Sync
unsafe impl Send for Pool {}

#[cfg(feature = "pool-debug")]
#[macro_export]
macro_rules! pool_debug {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        pub fn $name(&mut self) -> Self {
            let mut subpool: *mut generated::apr_pool_t = std::ptr::null_mut();
            let location = std::concat!(file!(), ":", line!());
            Pool::new_debug(location)
        }
    };
}

impl Pool {
    /// Create a new pool.
    pub fn new() -> Self {
        let mut pool: *mut generated::apr_pool_t = std::ptr::null_mut();
        unsafe {
            generated::apr_pool_create_ex(
                &mut pool,
                std::ptr::null_mut(),
                None,
                std::ptr::null_mut(),
            );
        }
        Pool(pool)
    }

    #[cfg(feature = "pool-debug")]
    pub fn new_debug(location: &str) -> Self {
        let mut pool: *mut generated::apr_pool_t = std::ptr::null_mut();
        unsafe {
            generated::apr_pool_create_ex_debug(
                &mut pool,
                std::ptr::null_mut(),
                None,
                std::ptr::null_mut(),
                location.as_ptr() as *const std::ffi::c_char,
            );
        }
        Pool(pool)
    }

    pub fn from_raw(ptr: *mut generated::apr_pool_t) -> Self {
        Pool(ptr)
    }

    pub unsafe fn as_ptr(&self) -> *const generated::apr_pool_t {
        self.0
    }

    pub unsafe fn as_mut_ptr(&self) -> *mut generated::apr_pool_t {
        self.0
    }

    /// Create a subpool.
    pub fn subpool(&self) -> Self {
        let mut subpool: *mut generated::apr_pool_t = std::ptr::null_mut();
        unsafe {
            generated::apr_pool_create_ex(&mut subpool, self.0, None, std::ptr::null_mut());
        }
        Pool(subpool)
    }

    #[allow(clippy::mut_from_ref)]
    /// Allocate memory in the pool.
    pub fn alloc<T: Sized>(&self) -> &mut std::mem::MaybeUninit<T> {
        let size = std::mem::size_of::<T>();
        unsafe {
            let x = generated::apr_palloc(self.0, size) as *mut std::mem::MaybeUninit<T>;
            &mut *x
        }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn calloc<T: Sized>(&self) -> &mut T {
        let size = std::mem::size_of::<T>();
        unsafe {
            let x = generated::apr_palloc(self.0, size) as *mut T;
            std::ptr::write_bytes(x as *mut u8, 0, size);
            &mut *x
        }
    }

    /// Check if the pool is an ancestor of another pool.
    pub fn is_ancestor(&self, other: &Pool) -> bool {
        unsafe { generated::apr_pool_is_ancestor(self.0, other.0) != 0 }
    }

    /// Set a tag for the pool.
    pub fn tag(&self, tag: &str) {
        let tag = std::ffi::CString::new(tag).unwrap();
        unsafe {
            generated::apr_pool_tag(self.0, tag.as_ptr() as *const std::ffi::c_char);
        }
    }

    /// Clear all memory in the pool.
    ///
    /// This does not actually free the memory, it just allows the pool to reuse this memory for the next allocation.
    ///
    /// # Safety
    ///
    /// This is unsafe because it is possible to create a dangling pointer to memory that has been cleared.
    pub unsafe fn clear(&mut self) {
        unsafe {
            generated::apr_pool_clear(self.0);
        }
    }

    /// Clear all memory in the pool, with debug information.
    ///
    /// This does not actually free the memory, it just allows the pool to reuse this memory for the next allocation.
    ///
    /// # Safety
    ///
    /// This is unsafe because it is possible to create a dangling pointer to memory that has been cleared.
    #[cfg(feature = "pool-debug")]
    pub unsafe fn fn_clear_debug(&mut self, location: &str) {
        unsafe {
            generated::apr_pool_clear_debug(self.0, location.as_ptr() as *const std::ffi::c_char);
        }
    }

    /// Get the parent pool, if any.
    pub fn parent(&self) -> Option<&Self> {
        let parent = unsafe { generated::apr_pool_parent_get(self.0) };
        if parent.is_null() {
            None
        } else {
            Some(unsafe { &*(parent as *const Pool) })
        }
    }

    /// Run all registered child cleanups, in preparation for an exec() call.
    pub fn cleanup_for_exec(&self) {
        unsafe {
            generated::apr_pool_cleanup_for_exec();
        }
    }

    #[cfg(feature = "pool-debug")]
    pub fn join(&self, other: &Pool) {
        unsafe { generated::apr_pool_join(self.0, other.0) }
    }

    #[cfg(feature = "pool-debug")]
    pub fn num_bytes(&self, recurse: bool) -> usize {
        unsafe { generated::apr_pool_num_bytes(self.0, if recurse { 1 } else { 0 }) }
    }

    #[cfg(feature = "pool-debug")]
    pub unsafe fn find(&self, ptr: *const std::ffi::c_void) -> Option<Pool> {
        let pool = generated::apr_pool_find(ptr);
        if pool.is_null() {
            None
        } else {
            Some(Pool(pool))
        }
    }

    #[cfg(not(feature = "pool-debug"))]
    pub fn join(&self, _other: &Pool) {}
}

impl Default for Pool {
    fn default() -> Self {
        Pool::new()
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        unsafe {
            generated::apr_pool_destroy(self.0);
        }
    }
}
pub struct Allocator(*mut generated::apr_allocator_t);

impl Allocator {
    pub fn new() -> Self {
        let mut allocator: *mut generated::apr_allocator_t = std::ptr::null_mut();
        unsafe {
            generated::apr_allocator_create(&mut allocator);
        }
        Allocator(allocator)
    }

    pub fn as_ptr(&self) -> *const generated::apr_allocator_t {
        self.0
    }
}

impl Default for Allocator {
    fn default() -> Self {
        Allocator::new()
    }
}

impl Drop for Allocator {
    fn drop(&mut self) {
        unsafe {
            generated::apr_allocator_destroy(self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool() {
        let mut pool = Pool::new();
        let parent = pool.parent();
        assert!(parent.unwrap().is_ancestor(&pool));
        let subpool = pool.subpool();
        assert!(pool.is_ancestor(&subpool));
        assert!(!subpool.is_ancestor(&pool));
        assert!(subpool.parent().unwrap().is_ancestor(&subpool));
        subpool.tag("subpool");
        pool.tag("pool");
    }
}

/// A wrapper around a value that is allocated in a pool.
pub struct Pooled<T> {
    pool: std::rc::Rc<Pool>,
    pub data: T,
}

impl<T> Pooled<T> {
    /// Create a pooled value, using the given closure to initialize it.
    pub fn initialize<E: std::error::Error>(
        cb: impl FnOnce(&mut Pool) -> Result<T, E>,
    ) -> Result<Self, E> {
        let mut pool = std::rc::Rc::new(Pool::new());
        let data = cb(std::rc::Rc::get_mut(&mut pool).as_mut().unwrap())?;
        Ok(Pooled { pool, data })
    }

    /// Create a pooled value from a value allocated in a pool.
    ///
    /// # Safety
    ///
    /// The data must be allocated in the pool.
    pub unsafe fn in_pool(pool: std::rc::Rc<Pool>, data: T) -> Self {
        // Assert that the data is allocated in the pool.
        Pooled { pool, data }
    }

    /// Get a reference to the pool that the value is allocated in.
    pub fn pool(&self) -> std::rc::Rc<Pool> {
        self.pool.clone()
    }
}

impl<T> AsRef<T> for Pooled<T> {
    fn as_ref(&self) -> &T {
        &self.data
    }
}

impl<T> std::ops::Deref for Pooled<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> std::ops::DerefMut for Pooled<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Pooled<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pooled")
            .field("pool", &self.pool)
            .field("data", &self.data)
            .finish()
    }
}

/// A wrapper around a pointer to a value that is allocated in a pool.
pub struct PooledPtr<T> {
    pool: std::rc::Rc<Pool>,
    data: *mut T,
}

impl<T> PooledPtr<T> {
    /// Create a pooled value, using the given closure to initialize it.
    pub fn initialize<E: std::error::Error>(
        cb: impl FnOnce(&mut Pool) -> Result<*mut T, E>,
    ) -> Result<Self, E> {
        let mut pool = std::rc::Rc::new(Pool::new());
        let data = cb(std::rc::Rc::get_mut(&mut pool).as_mut().unwrap())?;
        Ok(PooledPtr { pool, data })
    }

    /// Create a pooled value from a value allocated in a pool.
    ///
    /// # Safety
    ///
    /// The data must be allocated in the pool.
    pub unsafe fn in_pool(pool: std::rc::Rc<Pool>, data: *mut T) -> Self {
        // TODO: Assert that the data is allocated in the pool.
        PooledPtr { pool, data }
    }

    pub fn is_null(&self) -> bool {
        self.data.is_null()
    }

    /// Get a reference to the pool that the value is allocated in.
    pub fn pool(&self) -> std::rc::Rc<Pool> {
        self.pool.clone()
    }

    pub fn as_ptr(&self) -> *const T {
        self.data
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.data
    }
}

impl<T> AsRef<T> for PooledPtr<T> {
    fn as_ref(&self) -> &T {
        unsafe { &*self.data }
    }
}
impl<T> std::ops::Deref for PooledPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<T> std::ops::DerefMut for PooledPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}

impl<T> std::fmt::Debug for PooledPtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PooledPtr")
            .field("pool", &self.pool)
            .field("data", &self.data)
            .finish()
    }
}

pub unsafe fn terminate() {
    unsafe {
        generated::apr_pool_terminate();
    }
}

#[cfg(test)]
mod pooled_tests {
    #[test]
    fn test_pooled() {
        let pooled = super::Pooled::initialize(|_pool| Ok::<_, std::io::Error>(1)).unwrap();
        assert_eq!(*pooled, 1);
    }
}
