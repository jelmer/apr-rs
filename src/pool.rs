//! Memory pool management.
use crate::generated;

/// A memory pool.
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

    /// Create a pool from a raw pointer.
    pub fn from_raw(ptr: *mut generated::apr_pool_t) -> Self {
        Pool(ptr)
    }

    /// Get the raw pointer to the pool.
    pub fn as_ptr(&self) -> *const generated::apr_pool_t {
        self.0
    }

    /// Get the raw mutable pointer to the pool.
    pub fn as_mut_ptr(&self) -> *mut generated::apr_pool_t {
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
    pub fn alloc<T: Sized>(&self) -> Pooled<'_, std::mem::MaybeUninit<T>> {
        let size = std::mem::size_of::<T>();
        unsafe {
            let x = generated::apr_palloc(self.0, size) as *mut std::mem::MaybeUninit<T>;
            Pooled {
                ptr: x,
                _marker: std::marker::PhantomData,
            }
        }
    }

    /// Allocate memory in the pool and zero it.
    #[allow(clippy::mut_from_ref)]
    pub fn calloc<T: Sized>(&self) -> Pooled<'_, T> {
        let size = std::mem::size_of::<T>();
        unsafe {
            let x = generated::apr_palloc(self.0, size) as *mut T;
            std::ptr::write_bytes(x as *mut u8, 0, size);
            Pooled {
                ptr: x,
                _marker: std::marker::PhantomData,
            }
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
    pub fn parent<'a, 'b>(&'a self) -> Option<&'b Self>
    where
        'b: 'a,
    {
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

    /// Try to join two pools.
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

    /// Try to join two pools.
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

/// An allocator.
pub struct Allocator(*mut generated::apr_allocator_t);

impl Allocator {
    /// Create a new allocator.
    pub fn new() -> Self {
        let mut allocator: *mut generated::apr_allocator_t = std::ptr::null_mut();
        unsafe {
            generated::apr_allocator_create(&mut allocator);
        }
        Allocator(allocator)
    }

    /// Return the raw pointer to the allocator.
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

pub struct Pooled<'a, T> {
    ptr: *mut T,
    _marker: std::marker::PhantomData<&'a Pool>,
}

impl<T> Pooled<'_, T> {
    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }

    pub fn from_ptr(ptr: *mut T) -> Self {
        assert!(!ptr.is_null());
        Pooled {
            ptr,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn option_from_ptr(ptr: *mut T) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Pooled {
                ptr,
                _marker: std::marker::PhantomData,
            })
        }
    }
}

impl<T> std::ops::Deref for Pooled<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T> std::ops::DerefMut for Pooled<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

pub struct OwnedPooled<T> {
    ptr: *mut T,
    pool: std::rc::Rc<Pool>,
}

impl<T> OwnedPooled<T> {
    pub fn new(pool: std::rc::Rc<Pool>, ptr: *mut T) -> Self {
        OwnedPooled { ptr, pool }
    }

    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }

    pub fn pool(&self) -> std::rc::Rc<Pool> {
        self.pool.clone()
    }
}

impl<T> std::ops::Deref for OwnedPooled<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T> std::ops::DerefMut for OwnedPooled<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

/// Terminate the apr pool subsystem.
///
/// # Safety
///
/// This function is unsafe because it is possible to create a dangling pointer to memory that has been cleared.
pub unsafe fn terminate() {
    unsafe {
        generated::apr_pool_terminate();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool() {
        let pool = Pool::new();
        assert!(pool.parent().unwrap().is_ancestor(&pool));
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
