pub use crate::generated::apr_hash_t;
use std::marker::PhantomData;

pub struct Hash<'pool> {
    hash: *mut apr_hash_t,
    _marker: PhantomData<&'pool ()>,
}

impl<'pool> Hash<'pool> {
    pub fn new(pool: &'pool mut crate::Pool) -> Self {
        let hash = unsafe { crate::generated::apr_hash_make(pool.into()) };
        Self {
            hash,
            _marker: PhantomData,
        }
    }

    pub fn copy(&self, pool: &'pool mut crate::Pool) -> Self {
        let hash = unsafe { crate::generated::apr_hash_copy(pool.into(), self.hash) };
        Self {
            hash,
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        unsafe { crate::generated::apr_hash_count(self.hash) as usize }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        unsafe { crate::generated::apr_hash_clear(self.hash) }
    }

    pub fn iter(&self) {
        unimplemented!();
    }
}
