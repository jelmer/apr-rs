//! Hash map implementation.
pub use apr_sys::apr_hash_t;
use crate::pool::Pool;
use std::marker::PhantomData;

/// A hash map.
pub struct Hash<'pool, K: IntoHashKey<'pool>, V> {
    ptr: *mut apr_hash_t,
    _phantom: PhantomData<(K, &'pool V, &'pool Pool)>,
}

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
        Self {
            ptr: unsafe { apr_sys::apr_hash_make(pool.as_mut_ptr()) },
            _phantom: PhantomData,
        }
    }

    /// Create a new hash map from a raw pointer.
    pub fn from_ptr(ptr: *mut apr_hash_t) -> Self {
        Self {
            ptr,
            _phantom: PhantomData,
        }
    }

    pub fn copy<'newpool, NK: IntoHashKey<'newpool>>(
        &self,
        pool: &'newpool Pool,
    ) -> Result<Hash<'newpool, NK, V>, crate::Status> {
        unsafe {
            let data = apr_sys::apr_hash_copy(pool.as_mut_ptr(), self.as_ptr());
            Ok(Hash {
                ptr: data,
                _phantom: PhantomData,
            })
        }
    }

    /// Create a new hash map in the given pool.
    pub fn in_pool(pool: &std::rc::Rc<Pool>) -> Self {
        unsafe {
            let mut pool = pool.clone();
            let data = apr_sys::apr_hash_make(
                std::rc::Rc::get_mut(&mut pool).unwrap().as_mut_ptr(),
            );
            Self {
                ptr: data,
                _phantom: PhantomData,
            }
        }
    }

    /// Returns the number of elements in the hash map.
    pub fn len(&mut self) -> usize {
        unsafe { apr_sys::apr_hash_count(self.ptr) as usize }
    }

    /// Returns true if the hash map contains no elements.
    pub fn is_empty(&mut self) -> bool {
        self.len() == 0
    }

    /// Clear the contents of the hash map.
    pub fn clear(&mut self) {
        unsafe { apr_sys::apr_hash_clear(self.ptr) }
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&mut self, key: K) -> Option<&'pool V> {
        let key = key.into_hash_key();
        unsafe {
            let val = apr_sys::apr_hash_get(
                self.ptr,
                key.as_ptr() as *mut std::ffi::c_void,
                key.len() as apr_sys::apr_ssize_t,
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
            apr_sys::apr_hash_set(
                self.ptr,
                key.as_ptr() as *mut std::ffi::c_void,
                key.len() as apr_sys::apr_ssize_t,
                val as *const V as *mut V as *mut std::ffi::c_void,
            );
        }
    }

    /// Returns an iterator over the key-value pairs of the hash map.
    pub fn iter<'newpool>(&mut self, pool: &'newpool Pool) -> Iter<'newpool, V> {
        let first = unsafe { apr_sys::apr_hash_first(pool.as_mut_ptr(), self.ptr) };
        Iter {
            ptr: first,
            _phantom: PhantomData,
        }
    }

    /// Returns an iterator over the keys of the hash map.
    pub fn keys<'newpool>(&mut self, pool: &'newpool Pool) -> Keys<'newpool> {
        let first = unsafe { apr_sys::apr_hash_first(pool.as_mut_ptr(), self.ptr) };
        Keys {
            ptr: first,
            _phantom: PhantomData,
        }
    }

    /// Return a pointer to the underlying apr_hash_t.
    pub unsafe fn as_ptr(&self) -> *const apr_hash_t {
        self.ptr
    }

    /// Return a mutable pointer to the underlying apr_hash_t.
    pub unsafe fn as_mut_ptr(&mut self) -> *mut apr_hash_t {
        self.ptr
    }
}

/// An iterator over the key-value pairs of a hash map.
pub struct Iter<'pool, V> {
    ptr: *mut apr_sys::apr_hash_index_t,
    _phantom: PhantomData<&'pool V>,
}

impl<'pool, V> Iterator for Iter<'pool, V>
where
    V: 'pool,
{
    type Item = (&'pool [u8], &'pool V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr.is_null() {
            return None;
        }
        let mut key = std::ptr::null();
        let mut key_len = 0;
        let mut val = std::ptr::null_mut();
        unsafe {
            apr_sys::apr_hash_this(
                self.ptr,
                &mut key,
                &mut key_len,
                &mut val as *mut *mut std::ffi::c_void,
            );
        }

        self.ptr = unsafe { apr_sys::apr_hash_next(self.ptr) };

        let key = unsafe { std::slice::from_raw_parts(key as *const u8, key_len as usize) };

        Some((key, unsafe { &*(val as *const V) }))
    }
}

/// An iterator over the keys of a hash map.
pub struct Keys<'pool> {
    ptr: *mut apr_sys::apr_hash_index_t,
    _phantom: PhantomData<&'pool [u8]>,
}

impl<'pool> Iterator for Keys<'pool> {
    type Item = &'pool [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr.is_null() {
            return None;
        }
        let key = unsafe { apr_sys::apr_hash_this_key(self.ptr) };
        let key_len = unsafe { apr_sys::apr_hash_this_key_len(self.ptr) };

        self.ptr = unsafe { apr_sys::apr_hash_next(self.ptr) };

        let key = unsafe { std::slice::from_raw_parts(key as *const u8, key_len as usize) };

        Some(key)
    }
}

/// Generate a hash for a key with the default hash function.
pub fn hash_default(key: &[u8]) -> u32 {
    unsafe {
        let mut len = key.len() as apr_sys::apr_ssize_t;
        apr_sys::apr_hashfunc_default(key.as_ptr() as *const std::ffi::c_char, &mut len)
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
