//! Memory pool management.
use crate::generated;

/// A memory pool.
#[derive(Debug)]
#[repr(transparent)]
pub struct Pool {
    raw: *mut apr_sys::apr_pool_t,
    // Pools are not Send or Sync - they are single-threaded
    _no_send: std::marker::PhantomData<*mut ()>,
}

#[cfg(feature = "pool-debug")]
#[macro_export]
macro_rules! pool_debug {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        pub fn $name(&mut self) -> Self {
            let mut subpool: *mut apr_sys::apr_pool_t = std::ptr::null_mut();
            let location = std::concat!(file!(), ":", line!());
            Pool::new_debug(location)
        }
    };
}

impl Pool {
    /// Create a new pool.
    pub fn new() -> Self {
        let mut pool: *mut apr_sys::apr_pool_t = std::ptr::null_mut();
        unsafe {
            apr_sys::apr_pool_create_ex(
                &mut pool,
                std::ptr::null_mut(),
                None,
                std::ptr::null_mut(),
            );
        }
        Pool {
            raw: pool,
            _no_send: std::marker::PhantomData,
        }
    }

    #[cfg(feature = "pool-debug")]
    pub fn new_debug(location: &str) -> Self {
        let mut pool: *mut apr_sys::apr_pool_t = std::ptr::null_mut();
        unsafe {
            apr_sys::apr_pool_create_ex_debug(
                &mut pool,
                std::ptr::null_mut(),
                None,
                std::ptr::null_mut(),
                location.as_ptr() as *const std::ffi::c_char,
            );
        }
        Pool {
            raw: pool,
            _no_send: std::marker::PhantomData,
        }
    }

    /// Create a pool from a raw pointer.
    pub fn from_raw(ptr: *mut apr_sys::apr_pool_t) -> Self {
        Pool {
            raw: ptr,
            _no_send: std::marker::PhantomData,
        }
    }

    /// Get the raw pointer to the pool.
    pub fn as_ptr(&self) -> *const apr_sys::apr_pool_t {
        self.raw
    }

    /// Get the raw mutable pointer to the pool.
    pub fn as_mut_ptr(&self) -> *mut apr_sys::apr_pool_t {
        self.raw
    }

    /// Create a subpool.
    pub fn subpool(&self) -> Self {
        let mut subpool: *mut apr_sys::apr_pool_t = std::ptr::null_mut();
        unsafe {
            apr_sys::apr_pool_create_ex(&mut subpool, self.raw, None, std::ptr::null_mut());
        }
        Pool {
            raw: subpool,
            _no_send: std::marker::PhantomData,
        }
    }

    /// Create a subpool, run a function with it, then destroy the subpool.
    pub fn with_child<R>(&self, f: impl FnOnce(&Pool) -> R) -> R {
        let child = self.subpool();
        f(&child)
    }

    #[allow(clippy::mut_from_ref)]
    /// Allocate memory in the pool.
    pub fn alloc<T: Sized>(&self) -> *mut std::mem::MaybeUninit<T> {
        let size = std::mem::size_of::<T>();
        unsafe { apr_sys::apr_palloc(self.raw, size) as *mut std::mem::MaybeUninit<T> }
    }

    /// Allocate memory in the pool and zero it.
    #[allow(clippy::mut_from_ref)]
    pub fn calloc<T: Sized>(&self) -> *mut T {
        let size = std::mem::size_of::<T>();
        unsafe {
            let x = apr_sys::apr_palloc(self.raw, size) as *mut T;
            std::ptr::write_bytes(x as *mut u8, 0, size);
            x
        }
    }

    /// Check if the pool is an ancestor of another pool.
    pub fn is_ancestor(&self, other: &Pool) -> bool {
        unsafe { apr_sys::apr_pool_is_ancestor(self.raw, other.raw) != 0 }
    }

    /// Set a tag for the pool.
    pub fn tag(&self, tag: &str) {
        let tag = std::ffi::CString::new(tag).unwrap();
        unsafe {
            apr_sys::apr_pool_tag(self.raw, tag.as_ptr() as *const std::ffi::c_char);
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
            apr_sys::apr_pool_clear(self.raw);
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
            apr_sys::apr_pool_clear_debug(self.raw, location.as_ptr() as *const std::ffi::c_char);
        }
    }

    /// Get the parent pool, if any.
    pub fn parent<'a, 'b>(&'a self) -> Option<&'b Self>
    where
        'b: 'a,
    {
        let parent = unsafe { apr_sys::apr_pool_parent_get(self.raw) };
        if parent.is_null() {
            None
        } else {
            Some(unsafe { &*(parent as *const Pool) })
        }
    }

    /// Run all registered child cleanups, in preparation for an exec() call.
    pub fn cleanup_for_exec(&self) {
        unsafe {
            apr_sys::apr_pool_cleanup_for_exec();
        }
    }

    /// Try to join two pools.
    #[cfg(feature = "pool-debug")]
    pub fn join(&self, other: &Pool) {
        unsafe { apr_sys::apr_pool_join(self.raw, other.raw) }
    }

    #[cfg(feature = "pool-debug")]
    pub fn num_bytes(&self, recurse: bool) -> usize {
        unsafe { apr_sys::apr_pool_num_bytes(self.raw, if recurse { 1 } else { 0 }) }
    }

    #[cfg(feature = "pool-debug")]
    pub unsafe fn find(&self, ptr: *const std::ffi::c_void) -> Option<Pool> {
        let pool = apr_sys::apr_pool_find(ptr);
        if pool.is_null() {
            None
        } else {
            Some(Pool {
                raw: pool,
                _no_send: std::marker::PhantomData,
            })
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
            apr_sys::apr_pool_destroy(self.raw);
        }
    }
}

/// An allocator.
pub struct Allocator {
    raw: *mut apr_sys::apr_allocator_t,
    // Allocators are not Send or Sync
    _no_send: std::marker::PhantomData<*mut ()>,
}

impl Allocator {
    /// Create a new allocator.
    pub fn new() -> Self {
        let mut allocator: *mut apr_sys::apr_allocator_t = std::ptr::null_mut();
        unsafe {
            apr_sys::apr_allocator_create(&mut allocator);
        }
        Allocator {
            raw: allocator,
            _no_send: std::marker::PhantomData,
        }
    }

    /// Return the raw pointer to the allocator.
    pub fn as_ptr(&self) -> *const apr_sys::apr_allocator_t {
        self.raw
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
            apr_sys::apr_allocator_destroy(self.raw);
        }
    }
}

/// Create a temporary pool, run a function with it, then destroy the pool.
///
/// This is useful for short-lived operations that need a pool for temporary allocations.
pub fn with_tmp_pool<R>(f: impl FnOnce(&Pool) -> R) -> R {
    let tmp_pool = Pool::new();
    f(&tmp_pool)
}

/// Terminate the apr pool subsystem.
///
/// # Safety
///
/// This function is unsafe because it is possible to create a dangling pointer to memory that has been cleared.
pub unsafe fn terminate() {
    unsafe {
        apr_sys::apr_pool_terminate();
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
