//! Hash map implementation.
use crate::pool::Pool;
pub use apr_sys::apr_hash_t;
use std::marker::PhantomData;

/// A hash map.
#[derive(Debug)]
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

    /// Copy this hash table to a new pool
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
            let data =
                apr_sys::apr_hash_make(std::rc::Rc::get_mut(&mut pool).unwrap().as_mut_ptr());
            Self {
                ptr: data,
                _phantom: PhantomData,
            }
        }
    }

    /// Returns the number of elements in the hash map.
    pub fn len(&self) -> usize {
        unsafe { apr_sys::apr_hash_count(self.ptr) as usize }
    }

    /// Returns true if the hash map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear the contents of the hash map.
    pub fn clear(&mut self) {
        unsafe { apr_sys::apr_hash_clear(self.ptr) }
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: K) -> Option<&'pool V> {
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
    pub fn insert(&mut self, key: K, val: &V) {
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

    /// Inserts a key-value pair into the hash map.
    ///
    /// # Deprecated
    /// Use `insert()` instead.
    #[deprecated(since = "0.4.0", note = "Use `insert()` instead")]
    pub fn set(&mut self, key: K, val: &V) {
        self.insert(key, val);
    }

    /// Returns an iterator over the key-value pairs of the hash map.
    pub fn iter<'newpool>(&self, pool: &'newpool Pool) -> Iter<'newpool, V> {
        let first = unsafe { apr_sys::apr_hash_first(pool.as_mut_ptr() as *mut _, self.ptr) };
        Iter {
            ptr: first,
            _phantom: PhantomData,
        }
    }

    /// Returns an iterator over the keys of the hash map.
    pub fn keys<'newpool>(&self, pool: &'newpool Pool) -> Keys<'newpool> {
        let first = unsafe { apr_sys::apr_hash_first(pool.as_mut_ptr() as *mut _, self.ptr) };
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

    /// Check if the hash contains a given key.
    pub fn contains_key(&self, key: K) -> bool {
        self.get(key).is_some()
    }

    /// Remove a key from the hash (by setting value to null).
    pub fn remove(&mut self, key: K) {
        let key = key.into_hash_key();
        unsafe {
            apr_sys::apr_hash_set(
                self.ptr,
                key.as_ptr() as *mut std::ffi::c_void,
                key.len() as apr_sys::apr_ssize_t,
                std::ptr::null_mut(),
            );
        }
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

// We can't implement Index for Hash because get() returns Option<&V>
// and Index must return &V directly. Users should use .get() method instead.

// Implement IntoIterator for Hash (note: this requires a pool for APR iteration)
// We can't implement the standard IntoIterator without a pool parameter
// So we'll add a convenience method instead
impl<'pool, K: IntoHashKey<'pool>, V> Hash<'pool, K, V> {
    /// Convert into an iterator over key-value pairs.
    /// This consumes the hash and requires a pool for iteration.
    pub fn into_iter<'newpool>(self, pool: &'newpool Pool) -> Iter<'newpool, V> {
        self.iter(pool)
    }

    /// Create a Hash from an iterator of key-value pairs.
    pub fn from_iter<I>(pool: &'pool Pool, iter: I) -> Self
    where
        I: IntoIterator<Item = (K, &'pool V)>,
    {
        let mut hash = Self::new(pool);
        for (key, value) in iter {
            hash.insert(key, value);
        }
        hash
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
        hash.insert("foo", &"bar");
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
        hash.insert("foo", &"bar");
        let hash2 = hash.copy(&pool).unwrap();
        assert_eq!(hash2.get("foo"), Some(&"bar"));
    }

    #[test]
    fn test_insert_method() {
        let pool = Pool::new();
        let mut hash = super::Hash::new(&pool);
        hash.insert("foo", &"bar");
        assert_eq!(hash.get("foo"), Some(&"bar"));

        // Test deprecated set() method still works
        #[allow(deprecated)]
        {
            hash.set("baz", &"qux");
            assert_eq!(hash.get("baz"), Some(&"qux"));
        }
    }

    #[test]
    fn test_into_iterator() {
        let pool = Pool::new();
        let mut hash = super::Hash::new(&pool);
        hash.insert("foo", &"bar");
        hash.insert("baz", &"qux");

        let mut items: Vec<_> = hash.into_iter(&pool).collect();
        items.sort_by_key(|(k, _)| *k);

        assert_eq!(items.len(), 2);
        assert!(items.contains(&(&b"foo"[..], &"bar")));
        assert!(items.contains(&(&b"baz"[..], &"qux")));
    }

    #[test]
    fn test_hash_convenience_methods() {
        let pool = Pool::new();
        let mut hash = super::Hash::new(&pool);

        // Test contains_key
        assert!(!hash.contains_key("key1"));
        hash.insert("key1", &"value1");
        assert!(hash.contains_key("key1"));

        // Test remove
        hash.remove("key1");
        assert!(!hash.contains_key("key1"));
        assert!(hash.get("key1").is_none());
    }
}
