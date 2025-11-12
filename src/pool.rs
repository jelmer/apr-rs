//! Memory pool management.
use apr_sys;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

/// A memory pool.
///
/// The lifetime parameter `'pool` ensures that subpools cannot outlive their parent pool.
/// Root pools (created with `Pool::new()`) have a `'static` lifetime.
#[derive(Debug)]
#[repr(transparent)]
pub struct Pool<'pool> {
    raw: *mut apr_sys::apr_pool_t,
    // Pools are not Send or Sync - they are single-threaded
    // Lifetime ensures subpools don't outlive parent
    _marker: std::marker::PhantomData<&'pool ()>,
}

#[cfg(feature = "pool-debug")]
/// Macro for creating debug pools with location tracking.
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

impl Default for Pool<'static> {
    fn default() -> Self {
        Self::new()
    }
}

impl Pool<'static> {
    /// Create a new root pool.
    ///
    /// Root pools have a `'static` lifetime since they have no parent.
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
            _marker: std::marker::PhantomData,
        }
    }

    #[cfg(feature = "pool-debug")]
    /// Create a new root pool with debug information.
    ///
    /// This is used for debugging memory pool issues and tracking where pools are created.
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
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'pool> Pool<'pool> {
    /// Create a pool from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure the pointer is valid and the lifetime is appropriate.
    /// For pools passed from C callbacks, consider using `PoolHandle::from_borrowed_raw` instead.
    pub unsafe fn from_raw(ptr: *mut apr_sys::apr_pool_t) -> Self {
        Pool {
            raw: ptr,
            _marker: std::marker::PhantomData,
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
    ///
    /// The returned pool's lifetime is bounded by this pool, ensuring the subpool
    /// cannot outlive its parent.
    pub fn subpool(&self) -> Pool<'_> {
        let mut subpool: *mut apr_sys::apr_pool_t = std::ptr::null_mut();
        unsafe {
            apr_sys::apr_pool_create_ex(&mut subpool, self.raw, None, std::ptr::null_mut());
        }
        Pool {
            raw: subpool,
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a subpool, run a function with it, then destroy the subpool.
    pub fn with_child<R>(&self, f: impl FnOnce(&Pool<'_>) -> R) -> R {
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
    pub fn is_ancestor(&self, other: &Pool<'_>) -> bool {
        unsafe { apr_sys::apr_pool_is_ancestor(self.raw, other.raw) != 0 }
    }

    /// Set a tag for the pool.
    pub fn tag(&self, tag: &str) {
        let tag = std::ffi::CString::new(tag).unwrap();
        unsafe {
            apr_sys::apr_pool_tag(self.raw, tag.as_ptr() as *const std::ffi::c_char);
        }
    }

    /// Allocate a C string in the pool.
    ///
    /// The string is copied into pool-managed memory and will live as long as the pool.
    pub fn pstrdup(&self, s: &str) -> *const std::ffi::c_char {
        let c_str = std::ffi::CString::new(s).expect("Invalid C string");
        unsafe { apr_sys::apr_pstrdup(self.raw, c_str.as_ptr()) }
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
    pub fn join(&self, other: &Pool<'_>) {
        unsafe { apr_sys::apr_pool_join(self.raw, other.raw) }
    }

    #[cfg(feature = "pool-debug")]
    /// Get the number of bytes allocated in this pool.
    ///
    /// If `recurse` is true, includes bytes from all subpools.
    pub fn num_bytes(&self, recurse: bool) -> usize {
        unsafe { apr_sys::apr_pool_num_bytes(self.raw, if recurse { 1 } else { 0 }) }
    }

    #[cfg(feature = "pool-debug")]
    /// Find the pool that owns the given pointer.
    ///
    /// # Safety
    ///
    /// The pointer must be a valid pointer that was allocated from an APR pool.
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
    pub fn join(&self, _other: &Pool<'_>) {}
}

impl Drop for Pool<'_> {
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
pub fn with_tmp_pool<R>(f: impl FnOnce(&Pool<'_>) -> R) -> R {
    let tmp_pool = Pool::new();
    f(&tmp_pool)
}

/// A pool that may be owned or borrowed.
///
/// This is similar to `Cow` but for APR pools, which cannot be cloned.
/// When borrowed, the pool is not destroyed on drop.
///
/// # Use Cases
///
/// ## C Library Callbacks
///
/// When C libraries call Rust callbacks and pass pool pointers, the C library
/// typically owns the pool. Using `PoolHandle::from_borrowed_raw()` prevents
/// the pool from being destroyed when the Rust wrapper goes out of scope:
///
/// ```ignore
/// extern "C" fn callback(pool: *mut apr_pool_t) -> *mut svn_error_t {
///     // Safe: Borrowed pool won't be destroyed
///     let pool = unsafe { PoolHandle::from_borrowed_raw(pool) };
///     // ... use pool ...
///     std::ptr::null_mut()
/// }  // pool handle dropped, but C library's pool is NOT destroyed
/// ```
///
/// ## Flexible Pool Ownership
///
/// Structs can use `PoolHandle` to support both owned and borrowed pools:
///
/// ```ignore
/// pub struct SslServerCertInfo {
///     ptr: *const svn_auth_ssl_server_cert_info_t,
///     pool: PoolHandle,  // Can be owned OR borrowed
/// }
/// ```
///
/// # Why Not Use `Cow`?
///
/// Standard library's `Cow` requires `ToOwned`, but `Pool` cannot implement it
/// because pools cannot be cloned. APR only supports creating subpools, not
/// copying pool contents.
#[derive(Debug)]
pub enum PoolHandle<'pool> {
    /// An owned pool that will be destroyed when dropped.
    Owned(Pool<'pool>),
    /// A borrowed pool that will NOT be destroyed when dropped.
    /// The pool is owned by the caller (typically a C library).
    Borrowed(ManuallyDrop<Pool<'pool>>),
}

impl<'pool> PoolHandle<'pool> {
    /// Create a handle to an owned pool.
    ///
    /// The pool will be destroyed when the handle is dropped.
    ///
    /// # Example
    ///
    /// ```
    /// use apr::PoolHandle;
    /// use apr::Pool;
    ///
    /// let pool = Pool::new();
    /// let handle = PoolHandle::owned(pool);
    /// // handle will destroy the pool when dropped
    /// ```
    pub fn owned(pool: Pool<'pool>) -> Self {
        PoolHandle::Owned(pool)
    }

    /// Create a handle to a borrowed pool from a raw pointer.
    ///
    /// The pool will NOT be destroyed when the handle is dropped.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// - `ptr` is a valid pool pointer
    /// - The pool remains valid for the lifetime of this handle
    /// - The pool is owned and will be destroyed by the caller
    ///
    /// # Example
    ///
    /// ```ignore
    /// // In a C callback that receives a pool pointer
    /// let handle = unsafe { PoolHandle::from_borrowed_raw(c_pool_ptr) };
    /// // Use the pool without worrying about destroying it
    /// ```
    pub unsafe fn from_borrowed_raw(ptr: *mut apr_sys::apr_pool_t) -> Self {
        PoolHandle::Borrowed(ManuallyDrop::new(Pool::from_raw(ptr)))
    }

    /// Get the raw pointer to the pool.
    pub fn as_ptr(&self) -> *const apr_sys::apr_pool_t {
        match self {
            PoolHandle::Owned(pool) => pool.as_ptr(),
            PoolHandle::Borrowed(pool) => pool.as_ptr(),
        }
    }

    /// Get the raw mutable pointer to the pool.
    pub fn as_mut_ptr(&self) -> *mut apr_sys::apr_pool_t {
        match self {
            PoolHandle::Owned(pool) => pool.as_mut_ptr(),
            PoolHandle::Borrowed(pool) => pool.as_mut_ptr(),
        }
    }

    /// Create a subpool.
    ///
    /// The returned pool is always owned and will be destroyed on drop,
    /// regardless of whether the parent handle is owned or borrowed.
    ///
    /// # Example
    ///
    /// ```
    /// use apr::PoolHandle;
    /// use apr::Pool;
    ///
    /// let pool = Pool::new();
    /// let handle = PoolHandle::owned(pool);
    /// let subpool = handle.subpool();
    /// // subpool is owned and will be destroyed when dropped
    /// ```
    pub fn subpool(&self) -> Pool<'static> {
        let mut subpool: *mut apr_sys::apr_pool_t = std::ptr::null_mut();
        unsafe {
            apr_sys::apr_pool_create_ex(
                &mut subpool,
                self.as_mut_ptr(),
                None,
                std::ptr::null_mut(),
            );
            Pool::from_raw(subpool)
        }
    }

    /// Returns true if this handle owns the pool.
    ///
    /// # Example
    ///
    /// ```
    /// use apr::PoolHandle;
    /// use apr::Pool;
    ///
    /// let pool = Pool::new();
    /// let handle = PoolHandle::owned(pool);
    /// assert!(handle.is_owned());
    /// assert!(!handle.is_borrowed());
    /// ```
    pub fn is_owned(&self) -> bool {
        matches!(self, PoolHandle::Owned(_))
    }

    /// Returns true if this handle borrows the pool.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let handle = unsafe { PoolHandle::from_borrowed_raw(c_pool_ptr) };
    /// assert!(handle.is_borrowed());
    /// assert!(!handle.is_owned());
    /// ```
    pub fn is_borrowed(&self) -> bool {
        matches!(self, PoolHandle::Borrowed(_))
    }
}

impl Drop for PoolHandle<'_> {
    fn drop(&mut self) {
        // Only Owned variant destroys the pool (via Pool's Drop)
        // Borrowed variant does nothing (ManuallyDrop prevents Pool::drop)
    }
}

impl<'pool> From<Pool<'pool>> for PoolHandle<'pool> {
    /// Convert an owned pool into a pool handle.
    ///
    /// # Example
    ///
    /// ```
    /// use apr::PoolHandle;
    /// use apr::Pool;
    ///
    /// let pool = Pool::new();
    /// let handle: PoolHandle = pool.into();
    /// ```
    fn from(pool: Pool<'pool>) -> Self {
        PoolHandle::Owned(pool)
    }
}

impl<'pool> Deref for PoolHandle<'pool> {
    type Target = Pool<'pool>;

    /// Dereference to the underlying pool.
    ///
    /// This allows `PoolHandle` to be used transparently as a `Pool`.
    ///
    /// # Example
    ///
    /// ```
    /// use apr::PoolHandle;
    /// use apr::Pool;
    ///
    /// let pool = Pool::new();
    /// let handle = PoolHandle::owned(pool);
    /// // Can call Pool methods directly on the handle
    /// handle.tag("my-pool");
    /// ```
    fn deref(&self) -> &Pool<'pool> {
        match self {
            PoolHandle::Owned(pool) => pool,
            PoolHandle::Borrowed(pool) => pool,
        }
    }
}

impl<'pool> DerefMut for PoolHandle<'pool> {
    /// Mutably dereference to the underlying pool.
    fn deref_mut(&mut self) -> &mut Pool<'pool> {
        match self {
            PoolHandle::Owned(pool) => pool,
            PoolHandle::Borrowed(pool) => pool,
        }
    }
}

/// A reference-counted shared pool.
///
/// This is a wrapper around `Rc<Pool>` that provides a convenient way to share
/// a pool across multiple owners. The pool will be destroyed when the last
/// reference is dropped.
///
/// # Use Cases
///
/// `SharedPool` is useful when:
/// - Multiple components need to share the same pool
/// - The pool's lifetime cannot be statically determined
/// - You want automatic cleanup when all references are dropped
///
/// # Example
///
/// ```
/// use apr::SharedPool;
/// use apr::Pool;
///
/// let pool = SharedPool::new();
/// let pool_clone = pool.clone();
///
/// // Both pool and pool_clone refer to the same underlying pool
/// pool.tag("shared-pool");
///
/// // Pool is destroyed when both pool and pool_clone are dropped
/// ```
///
/// # Note
///
/// Since `Pool` is not `Send` or `Sync`, `SharedPool` is also not `Send` or `Sync`.
/// This means it can only be shared within a single thread. For multi-threaded
/// scenarios, the pool must be created in each thread separately.
#[derive(Debug, Clone)]
pub struct SharedPool<'pool> {
    inner: Rc<Pool<'pool>>,
}

impl<'pool> SharedPool<'pool> {
    /// Create a new shared pool.
    ///
    /// # Example
    ///
    /// ```
    /// use apr::SharedPool;
    ///
    /// let pool = SharedPool::new();
    /// ```
    pub fn new() -> SharedPool<'static> {
        SharedPool {
            inner: Rc::new(Pool::new()),
        }
    }

    /// Create a shared pool from an existing pool.
    ///
    /// # Example
    ///
    /// ```
    /// use apr::{SharedPool, Pool};
    ///
    /// let pool = Pool::new();
    /// let shared = SharedPool::from_pool(pool);
    /// ```
    pub fn from_pool(pool: Pool<'pool>) -> Self {
        SharedPool {
            inner: Rc::new(pool),
        }
    }

    /// Get the number of strong references to this pool.
    ///
    /// # Example
    ///
    /// ```
    /// use apr::SharedPool;
    ///
    /// let pool = SharedPool::new();
    /// assert_eq!(pool.strong_count(), 1);
    ///
    /// let pool2 = pool.clone();
    /// assert_eq!(pool.strong_count(), 2);
    /// ```
    pub fn strong_count(&self) -> usize {
        Rc::strong_count(&self.inner)
    }

    /// Get the raw pointer to the pool.
    pub fn as_ptr(&self) -> *const apr_sys::apr_pool_t {
        self.inner.as_ptr()
    }

    /// Get the raw mutable pointer to the pool.
    pub fn as_mut_ptr(&self) -> *mut apr_sys::apr_pool_t {
        self.inner.as_mut_ptr()
    }
}

impl Default for SharedPool<'static> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'pool> Deref for SharedPool<'pool> {
    type Target = Pool<'pool>;

    /// Dereference to the underlying pool.
    ///
    /// This allows `SharedPool` to be used transparently as a `Pool`.
    ///
    /// # Example
    ///
    /// ```
    /// use apr::SharedPool;
    ///
    /// let pool = SharedPool::new();
    /// // Can call Pool methods directly on the shared pool
    /// pool.tag("shared-pool");
    /// ```
    fn deref(&self) -> &Pool<'pool> {
        &self.inner
    }
}

impl<'pool> From<Pool<'pool>> for SharedPool<'pool> {
    /// Convert a pool into a shared pool.
    ///
    /// # Example
    ///
    /// ```
    /// use apr::{SharedPool, Pool};
    ///
    /// let pool = Pool::new();
    /// let shared: SharedPool = pool.into();
    /// ```
    fn from(pool: Pool<'pool>) -> Self {
        SharedPool::from_pool(pool)
    }
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

    #[test]
    fn test_pool_handle_owned() {
        // Create an owned pool handle
        let pool = Pool::new();
        let handle = PoolHandle::owned(pool);

        // Verify it's owned
        assert!(handle.is_owned());
        assert!(!handle.is_borrowed());

        // Test pointer access
        assert!(!handle.as_ptr().is_null());
        assert!(!handle.as_mut_ptr().is_null());

        // Test that we can use Pool methods through Deref
        handle.tag("owned-pool");

        // Pool will be destroyed when handle is dropped
    }

    #[test]
    fn test_pool_handle_borrowed() {
        // Create a real pool that we'll keep alive
        let pool = Pool::new();
        let pool_ptr = pool.as_mut_ptr();

        // Create a borrowed handle from the raw pointer
        let handle = unsafe { PoolHandle::from_borrowed_raw(pool_ptr) };

        // Verify it's borrowed
        assert!(handle.is_borrowed());
        assert!(!handle.is_owned());

        // Test pointer access
        assert_eq!(handle.as_ptr(), pool_ptr);
        assert_eq!(handle.as_mut_ptr(), pool_ptr);

        // Test that we can use Pool methods through Deref
        handle.tag("borrowed-pool");

        // Drop the borrowed handle - should NOT destroy the pool
        drop(handle);

        // Pool is still valid and can be used
        pool.tag("still-valid");

        // Pool will be destroyed when the original pool is dropped
    }

    #[test]
    fn test_pool_handle_subpool() {
        // Test subpool creation from owned handle
        let pool = Pool::new();
        let handle = PoolHandle::owned(pool);
        let subpool = handle.subpool();

        // Verify the subpool is valid and is a child of the handle's pool
        subpool.tag("subpool");

        // Test subpool creation from borrowed handle
        let pool2 = Pool::new();
        let pool2_ptr = pool2.as_mut_ptr();
        let borrowed_handle = unsafe { PoolHandle::from_borrowed_raw(pool2_ptr) };
        let subpool2 = borrowed_handle.subpool();

        subpool2.tag("subpool2");
    }

    #[test]
    fn test_pool_handle_from_pool() {
        // Test From<Pool> conversion
        let pool = Pool::new();
        let handle: PoolHandle = pool.into();

        assert!(handle.is_owned());
        assert!(!handle.is_borrowed());
    }

    #[test]
    fn test_pool_handle_deref() {
        // Test that we can call Pool methods directly on PoolHandle
        let pool = Pool::new();
        let handle = PoolHandle::owned(pool);

        // These should all work through Deref/DerefMut
        handle.tag("test-deref");
        let _subpool = handle.subpool();
        let _parent = handle.parent();

        // Test with borrowed handle
        let pool2 = Pool::new();
        let pool2_ptr = pool2.as_mut_ptr();
        let borrowed_handle = unsafe { PoolHandle::from_borrowed_raw(pool2_ptr) };

        borrowed_handle.tag("test-deref-borrowed");
        let _subpool2 = borrowed_handle.subpool();
    }

    #[test]
    fn test_pool_handle_no_double_free() {
        // This test ensures that dropping a borrowed handle doesn't destroy the pool
        let pool = Pool::new();
        let pool_ptr = pool.as_mut_ptr();

        // Create and drop multiple borrowed handles to the same pool
        for i in 0..10 {
            let handle = unsafe { PoolHandle::from_borrowed_raw(pool_ptr) };
            handle.tag(&format!("iteration-{}", i));
            // handle is dropped here, but pool should NOT be destroyed
        }

        // Pool should still be valid
        pool.tag("still-valid-after-multiple-borrows");
    }

    #[test]
    fn test_pool_handle_mixed_ownership() {
        // Test a scenario with mixed owned and borrowed handles
        let long_lived_pool = Pool::new();
        let long_lived_ptr = long_lived_pool.as_mut_ptr();

        // Borrowed handle for long-lived allocations
        let result_handle = unsafe { PoolHandle::from_borrowed_raw(long_lived_ptr) };

        // Owned handle for temporary work
        let scratch_pool = Pool::new();
        let scratch_handle = PoolHandle::owned(scratch_pool);

        // Use both handles
        result_handle.tag("result-pool");
        scratch_handle.tag("scratch-pool");

        // Drop scratch handle (destroys the pool)
        drop(scratch_handle);

        // Drop result handle (does NOT destroy the pool)
        drop(result_handle);

        // Original pool still valid
        long_lived_pool.tag("still-valid");
    }

    #[test]
    fn test_shared_pool_basic() {
        // Create a new shared pool
        let pool = SharedPool::new();

        // Verify initial reference count
        assert_eq!(pool.strong_count(), 1);

        // Test pointer access
        assert!(!pool.as_ptr().is_null());
        assert!(!pool.as_mut_ptr().is_null());

        // Test that we can use Pool methods through Deref
        pool.tag("shared-pool");
    }

    #[test]
    fn test_shared_pool_clone() {
        // Create a shared pool
        let pool1 = SharedPool::new();
        assert_eq!(pool1.strong_count(), 1);

        // Clone it
        let pool2 = pool1.clone();
        assert_eq!(pool1.strong_count(), 2);
        assert_eq!(pool2.strong_count(), 2);

        // Both refer to the same underlying pool
        assert_eq!(pool1.as_ptr(), pool2.as_ptr());

        // Can use both independently
        pool1.tag("shared-pool-1");
        pool2.tag("shared-pool-2");

        // Create another clone
        let pool3 = pool2.clone();
        assert_eq!(pool1.strong_count(), 3);
        assert_eq!(pool2.strong_count(), 3);
        assert_eq!(pool3.strong_count(), 3);

        // Drop one reference
        drop(pool3);
        assert_eq!(pool1.strong_count(), 2);
        assert_eq!(pool2.strong_count(), 2);
    }

    #[test]
    fn test_shared_pool_from_pool() {
        // Create a regular pool
        let pool = Pool::new();
        pool.tag("original");

        // Convert to shared pool
        let shared = SharedPool::from_pool(pool);
        assert_eq!(shared.strong_count(), 1);

        // Pool is destroyed when shared is dropped
    }

    #[test]
    fn test_shared_pool_from_conversion() {
        // Test From<Pool> trait
        let pool = Pool::new();
        let shared: SharedPool = pool.into();

        assert_eq!(shared.strong_count(), 1);
        shared.tag("converted");
    }

    #[test]
    fn test_shared_pool_default() {
        // Test Default trait
        let pool = SharedPool::default();
        assert_eq!(pool.strong_count(), 1);
        pool.tag("default-pool");
    }

    #[test]
    fn test_shared_pool_subpool() {
        // Create a shared pool
        let shared = SharedPool::new();
        shared.tag("shared-parent");

        // Create a subpool (through Deref)
        let subpool = shared.subpool();
        subpool.tag("subpool");

        // Shared pool is the ancestor of the subpool
        assert!(shared.is_ancestor(&subpool));
        assert!(!subpool.is_ancestor(&*shared));
    }

    #[test]
    fn test_shared_pool_multiple_owners() {
        // Simulate multiple components sharing a pool
        let pool = SharedPool::new();
        pool.tag("shared");

        let component1 = pool.clone();
        let component2 = pool.clone();
        let component3 = pool.clone();

        assert_eq!(pool.strong_count(), 4);

        // All components can use the pool
        component1.pstrdup("component1");
        component2.pstrdup("component2");
        component3.pstrdup("component3");

        // Drop components one by one
        drop(component1);
        assert_eq!(pool.strong_count(), 3);

        drop(component2);
        assert_eq!(pool.strong_count(), 2);

        drop(component3);
        assert_eq!(pool.strong_count(), 1);

        // Pool is still valid
        pool.pstrdup("still-valid");
    }
}
