pub use crate::generated::apr_hash_t;
use std::marker::PhantomData;

pub struct Hash<'pool, K: IntoHashKey<'pool>, V> {
    hash: *mut apr_hash_t,
    _marker: PhantomData<&'pool ()>,
    _marker2: PhantomData<(K, V)>,
}

pub trait IntoHashKey<'pool> {
    fn into_hash_key(self) -> &'pool [u8];
}

impl<'pool> IntoHashKey<'pool> for &'pool [u8] {
    fn into_hash_key(self) -> &'pool [u8] {
        self
    }
}

impl<'pool> IntoHashKey<'pool> for &'pool str {
    fn into_hash_key(self) -> &'pool [u8] {
        self.as_bytes()
    }
}

impl<'pool, K: IntoHashKey<'pool>, V> Hash<'pool, K, V> {
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

    pub fn get(&self, key: K) -> Option<&'pool V> {
        let key = key.into_hash_key();
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
        let key = key.into_hash_key();
        unsafe {
            crate::generated::apr_hash_set(
                self.hash,
                key.as_ptr() as *mut std::ffi::c_void,
                key.len() as crate::generated::apr_ssize_t,
                &val as *const V as *mut V as *mut std::ffi::c_void,
            );
        }
    }

    pub fn iter(&self) -> Iter<'pool, K, V> {
        Iter {
            index: unsafe { crate::generated::apr_hash_first(std::ptr::null_mut(), self.hash) },
            _marker: PhantomData,
            _marker2: PhantomData,
        }
    }

    pub fn keys(&self) -> Keys<'pool, K> {
        Keys {
            index: unsafe { crate::generated::apr_hash_first(std::ptr::null_mut(), self.hash) },
            _marker: PhantomData,
            _marker2: PhantomData,
        }
    }
}

pub struct Iter<'pool, K: IntoHashKey<'pool>, V> {
    index: *mut crate::generated::apr_hash_index_t,
    _marker: PhantomData<&'pool ()>,
    _marker2: PhantomData<(K, V)>,
}

impl<'pool, K: IntoHashKey<'pool>, V> Iterator for Iter<'pool, K, V>
where
    K: 'pool,
    V: 'pool,
{
    type Item = (&'pool K, &'pool V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index.is_null() {
            return None;
        }
        let mut key = std::ptr::null();
        let mut key_len = 0;
        let mut val = std::ptr::null_mut();
        unsafe {
            crate::generated::apr_hash_this(
                self.index,
                &mut key,
                &mut key_len,
                &mut val as *mut *mut std::ffi::c_void,
            );
        }

        self.index = unsafe { crate::generated::apr_hash_next(self.index) };

        Some((unsafe { &*(key as *const K) }, unsafe {
            &*(val as *const V)
        }))
    }
}

pub struct Keys<'pool, K: IntoHashKey<'pool>> {
    index: *mut crate::generated::apr_hash_index_t,
    _marker: PhantomData<&'pool ()>,
    _marker2: PhantomData<K>,
}

impl<'pool, K: IntoHashKey<'pool>> Iterator for Keys<'pool, K>
where
    K: 'pool,
{
    type Item = &'pool K;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index.is_null() {
            return None;
        }
        let key = unsafe { crate::generated::apr_hash_this_key(self.index) };

        self.index = unsafe { crate::generated::apr_hash_next(self.index) };

        Some(unsafe { &*(key as *const K) })
    }
}

impl<'pool, K: IntoHashKey<'pool>, V> From<Hash<'pool, K, V>> for *mut apr_hash_t {
    fn from(hash: Hash<'pool, K, V>) -> Self {
        hash.hash
    }
}

impl<'pool, K: IntoHashKey<'pool>, V> From<Hash<'pool, K, V>> for *const apr_hash_t {
    fn from(hash: Hash<'pool, K, V>) -> Self {
        hash.hash
    }
}

pub fn hash_default(key: &[u8]) -> u32 {
    unsafe {
        let mut len = key.len() as crate::generated::apr_ssize_t;
        crate::generated::apr_hashfunc_default(key.as_ptr() as *const i8, &mut len)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_hash_default() {
        assert_eq!(super::hash_default(b"foo"), super::hash_default(b"foo"));
        assert_ne!(super::hash_default(b"foo"), super::hash_default(b"bar"));
    }

    #[test]
    fn test_hash() {
        let mut pool = crate::Pool::new();
        let mut hash = super::Hash::new(&mut pool);
        assert!(hash.is_empty());
        hash.set("foo", "bar");
        return;
        assert!(!hash.is_empty());
        let items = hash.iter().collect::<Vec<_>>();
        assert_eq!(items.len(), 1);
        assert_eq!(hash.len(), 1);
        assert_eq!(hash.get("foo"), Some(&"bar"));
        hash.clear();
        assert!(hash.is_empty());
    }
}
