pub use crate::generated::apr_hash_t;
use std::marker::PhantomData;

pub struct Hash<'pool, K: Into<Vec<u8>>, V> {
    hash: *mut apr_hash_t,
    _marker: PhantomData<&'pool ()>,
    _marker2: PhantomData<(K, V)>,
}

impl<'pool, K: Into<Vec<u8>>, V> Hash<'pool, K, V> {
    pub fn new(pool: &'pool mut crate::Pool) -> Self {
        let hash = unsafe { crate::generated::apr_hash_make(pool.into()) };
        Self {
            hash,
            _marker: PhantomData,
            _marker2: PhantomData,
        }
    }

    pub fn copy(&self, pool: &'pool mut crate::Pool) -> Self {
        let hash = unsafe { crate::generated::apr_hash_copy(pool.into(), self.hash) };
        Self {
            hash,
            _marker: PhantomData,
            _marker2: PhantomData,
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

    pub fn get(&self, key: K) -> Option<&V> {
        let key = key.into();
        unsafe {
            let val = crate::generated::apr_hash_get(
                self.hash,
                key.as_ptr() as *mut std::ffi::c_void,
                key.len() as crate::generated::apr_ssize_t,
            );
            if val.is_null() {
                None
            } else {
                Some(&*(val as *const V))
            }
        }
    }

    pub fn set(&mut self, key: K, val: V) {
        let key = key.into();
        unsafe {
            crate::generated::apr_hash_set(
                self.hash,
                key.as_ptr() as *mut std::ffi::c_void,
                key.len() as crate::generated::apr_ssize_t,
                &val as *const V as *mut V as *mut std::ffi::c_void,
            );
        }
    }

    pub fn iter(&self) {
        unimplemented!();
    }
}
