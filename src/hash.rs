//! Hash map implementation.
pub use crate::generated::apr_hash_t;
use crate::pool::{Pool, Pooled};
use std::marker::PhantomData;

/// A hash map.
pub struct Hash<'pool, K: IntoHashKey<'pool>, V>(
    Pooled<'pool, apr_hash_t>,
    PhantomData<(K, &'pool V)>,
);

/// A trait for types that can be used as keys in a hash map.
pub trait IntoHashKey<'pool> {
    /// Convert the value into a byte slice that can be used as a key in a hash map.
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
    /// Create a new hash map in the current pool.
    pub fn new(pool: &'pool Pool) -> Self {
        Self::from_ptr(Pooled::from_ptr(unsafe {
            crate::generated::apr_hash_make(pool.as_mut_ptr())
        }))
    }

    /// Create a new hash map from a raw pointer.
    pub fn from_ptr(raw: Pooled<'pool, apr_hash_t>) -> Self {
        Self(raw, PhantomData)
    }

    pub fn copy<'newpool, NK: IntoHashKey<'newpool>>(
        &self,
        pool: &'newpool Pool,
    ) -> Result<Hash<'newpool, NK, V>, crate::Status> {
        unsafe {
            let data = crate::generated::apr_hash_copy(pool.as_mut_ptr(), self.as_ptr());
            Ok(Hash(Pooled::from_ptr(data), PhantomData))
        }
    }

    /// Create a new hash map in the given pool.
    pub fn in_pool(pool: &std::rc::Rc<Pool>) -> Self {
        unsafe {
            let mut pool = pool.clone();
            let data = crate::generated::apr_hash_make(
                std::rc::Rc::get_mut(&mut pool).unwrap().as_mut_ptr(),
            );
            Self(Pooled::from_ptr(data), PhantomData)
        }
    }

    /// Returns the number of elements in the hash map.
    pub fn len(&mut self) -> usize {
        unsafe { crate::generated::apr_hash_count(&mut *self.0) as usize }
    }

    /// Returns true if the hash map contains no elements.
    pub fn is_empty(&mut self) -> bool {
        self.len() == 0
    }

    /// Clear the contents of the hash map.
    pub fn clear(&mut self) {
        unsafe { crate::generated::apr_hash_clear(&mut *self.0) }
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&mut self, key: K) -> Option<&'pool V> {
        let key = key.into_hash_key();
        unsafe {
            let val = crate::generated::apr_hash_get(
                &mut *self.0,
                key.as_ptr() as *mut std::ffi::c_void,
                key.len() as crate::generated::apr_ssize_t,
            );
            if val.is_null() {
                None
            } else {
                Some((val as *const V).as_ref().unwrap())
            }
        }
    }

    /// Inserts a key-value pair into the hash map.
    pub fn set(&mut self, key: K, val: &V) {
        let key = key.into_hash_key();
        unsafe {
            crate::generated::apr_hash_set(
                &mut *self.0,
                key.as_ptr() as *mut std::ffi::c_void,
                key.len() as crate::generated::apr_ssize_t,
                val as *const V as *mut V as *mut std::ffi::c_void,
            );
        }
    }

    /// Returns an iterator over the key-value pairs of the hash map.
    pub fn iter<'newpool>(&mut self, pool: &'newpool Pool) -> Iter<'newpool, V> {
        let first =
            unsafe { crate::generated::apr_hash_first(pool.as_mut_ptr(), self.0.as_mut_ptr()) };
        Iter(Pooled::option_from_ptr(first), PhantomData)
    }

    /// Returns an iterator over the keys of the hash map.
    pub fn keys<'newpool>(&mut self, pool: &'newpool Pool) -> Keys<'newpool> {
        let first =
            unsafe { crate::generated::apr_hash_first(pool.as_mut_ptr(), self.0.as_mut_ptr()) };
        Keys(Pooled::option_from_ptr(first), PhantomData)
    }

    /// Return a pointer to the underlying apr_hash_t.
    pub unsafe fn as_ptr(&self) -> *const apr_hash_t {
        &*self.0
    }

    /// Return a mutable pointer to the underlying apr_hash_t.
    pub unsafe fn as_mut_ptr(&mut self) -> *mut apr_hash_t {
        &mut *self.0
    }
}

/// An iterator over the key-value pairs of a hash map.
pub struct Iter<'pool, V>(
    Option<Pooled<'pool, crate::generated::apr_hash_index_t>>,
    PhantomData<&'pool V>,
);

impl<'pool, V> Iterator for Iter<'pool, V>
where
    V: 'pool,
{
    type Item = (&'pool [u8], &'pool V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_ref()?;
        let mut key = std::ptr::null();
        let mut key_len = 0;
        let mut val = std::ptr::null_mut();
        unsafe {
            crate::generated::apr_hash_this(
                self.0.as_mut().unwrap().as_mut_ptr(),
                &mut key,
                &mut key_len,
                &mut val as *mut *mut std::ffi::c_void,
            );
        }

        self.0 = Pooled::option_from_ptr(unsafe {
            crate::generated::apr_hash_next(self.0.as_mut().unwrap().as_mut_ptr())
        });

        let key = unsafe { std::slice::from_raw_parts(key as *const u8, key_len as usize) };

        Some((key, unsafe { &*(val as *const V) }))
    }
}

/// An iterator over the keys of a hash map.
pub struct Keys<'pool>(
    Option<Pooled<'pool, crate::generated::apr_hash_index_t>>,
    PhantomData<&'pool [u8]>,
);

impl<'pool> Iterator for Keys<'pool> {
    type Item = &'pool [u8];

    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_ref()?;
        let key =
            unsafe { crate::generated::apr_hash_this_key(self.0.as_mut().unwrap().as_mut_ptr()) };
        let key_len = unsafe {
            crate::generated::apr_hash_this_key_len(self.0.as_mut().unwrap().as_mut_ptr())
        };

        self.0 = Pooled::option_from_ptr(unsafe {
            crate::generated::apr_hash_next(self.0.as_mut().unwrap().as_mut_ptr())
        });

        let key = unsafe { std::slice::from_raw_parts(key as *const u8, key_len as usize) };

        Some(key)
    }
}

/// Generate a hash for a key with the default hash function.
pub fn hash_default(key: &[u8]) -> u32 {
    unsafe {
        let mut len = key.len() as crate::generated::apr_ssize_t;
        crate::generated::apr_hashfunc_default(key.as_ptr() as *const std::ffi::c_char, &mut len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hash_default() {
        assert_eq!(super::hash_default(b"foo"), super::hash_default(b"foo"));
        assert_ne!(super::hash_default(b"foo"), super::hash_default(b"bar"));
    }

    #[test]
    fn test_hash() {
        let pool = Pool::new();
        let mut hash = super::Hash::new(&pool);
        assert!(hash.is_empty());
        assert!(hash.get("foo").is_none());
        hash.set("foo", &"bar");
        assert!(!hash.is_empty());
        assert_eq!(hash.get("foo"), Some(&"bar"));
        let items = hash.iter(&pool).collect::<Vec<_>>();
        assert_eq!(items.len(), 1);
        assert_eq!(hash.len(), 1);
        assert_eq!(items[0], (&b"foo"[..], &"bar"));
        assert_eq!(hash.keys(&pool).collect::<Vec<_>>(), vec![&b"foo"[..]]);
        hash.clear();
        assert!(hash.is_empty());
    }

    #[test]
    fn test_clone() {
        let pool = Pool::new();
        let mut hash = super::Hash::new(&pool);
        hash.set("foo", &"bar");
        let mut hash2 = hash.copy(&pool).unwrap();
        assert_eq!(hash2.get("foo"), Some(&"bar"));
    }
}
