use crate::generated;

#[derive(Debug)]
pub struct Pool(*mut generated::apr_pool_t);

impl From<Pool> for *mut generated::apr_pool_t {
    fn from(p: Pool) -> Self {
        p.0
    }
}

impl Pool {
    pub fn new() -> Self {
        let mut pool: *mut generated::apr_pool_t = std::ptr::null_mut();
        unsafe {
            generated::apr_pool_create_ex(
                &mut pool,
                std::ptr::null_mut(),
                None,
                std::ptr::null_mut() as *mut generated::apr_allocator_t,
            );
        }
        Pool(pool)
    }

    pub fn subpool(&mut self) -> Self {
        let mut subpool: *mut generated::apr_pool_t = std::ptr::null_mut();
        unsafe {
            generated::apr_pool_create_ex(
                &mut subpool,
                self.0,
                None,
                std::ptr::null_mut() as *mut generated::apr_allocator_t,
            );
        }
        Pool(subpool)
    }

    pub fn alloc<T: Sized>(&mut self) -> *mut std::mem::MaybeUninit<T> {
        let size = std::mem::size_of::<T>();
        unsafe { generated::apr_palloc(self.0, size) as *mut std::mem::MaybeUninit<T> }
    }

    pub fn is_ancestor(&self, other: &Pool) -> bool {
        unsafe { generated::apr_pool_is_ancestor(self.0, other.0) != 0 }
    }

    pub fn tag(&self, tag: &str) {
        unsafe {
            generated::apr_pool_tag(self.0, tag.as_ptr() as *const i8);
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

    pub fn parent(&self) -> Self {
        let parent = unsafe { generated::apr_pool_parent_get(self.0) };
        Pool(parent)
    }
}

impl From<&mut Pool> for *mut generated::apr_pool_t {
    fn from(p: &mut Pool) -> Self {
        p.0
    }
}

impl From<&Pool> for *mut generated::apr_pool_t {
    fn from(p: &Pool) -> Self {
        p.0
    }
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

impl From<Allocator> for *mut generated::apr_allocator_t {
    fn from(a: Allocator) -> Self {
        a.0
    }
}

impl Allocator {
    pub fn new() -> Self {
        let mut allocator: *mut generated::apr_allocator_t = std::ptr::null_mut();
        unsafe {
            generated::apr_allocator_create(&mut allocator);
        }
        Allocator(allocator)
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
        let subpool = pool.subpool();
        assert!(pool.is_ancestor(&subpool));
        assert!(!subpool.is_ancestor(&pool));
        assert!(subpool.parent().is_ancestor(&subpool));
        subpool.tag("subpool");
        pool.tag("pool");
    }
}
