//! Hash table support.

use crate::pool::Pool;
pub use apr_sys::apr_hash_t;
use std::ffi::c_void;
use std::marker::PhantomData;

/// A hash table that stores byte slices as keys and raw pointers as values.
///
/// This is a direct wrapper around APR's hash table implementation.
/// Keys are arbitrary byte sequences with an associated length.
/// Values are raw pointers that the hash table does not manage.
pub struct Hash<'pool> {
    ptr: *mut apr_hash_t,
    _phantom: PhantomData<&'pool Pool<'pool>>,
}

impl<'pool> Hash<'pool> {
    /// Create a new hash table in the given pool.
    pub fn new(pool: &'pool Pool<'pool>) -> Self {
        Self {
            ptr: unsafe { apr_sys::apr_hash_make(pool.as_mut_ptr()) },
            _phantom: PhantomData,
        }
    }

    /// Create a hash table from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be valid and point to an APR hash table.
    pub unsafe fn from_ptr(ptr: *mut apr_hash_t) -> Self {
        Self {
            ptr,
            _phantom: PhantomData,
        }
    }

    /// Insert a key-value pair into the hash table.
    ///
    /// The key is copied by APR, but the value is stored as-is.
    ///
    /// # Safety
    /// The caller must ensure the value pointer remains valid for the lifetime of the hash table,
    /// or until the key is removed/replaced.
    pub unsafe fn insert(&mut self, key: &[u8], value: *mut c_void) {
        apr_sys::apr_hash_set(
            self.ptr,
            key.as_ptr() as *const c_void,
            key.len() as apr_sys::apr_ssize_t,
            value,
        );
    }

    /// Get the value associated with a key.
    pub fn get(&self, key: &[u8]) -> Option<*mut c_void> {
        unsafe {
            let ptr = apr_sys::apr_hash_get(
                self.ptr,
                key.as_ptr() as *const c_void,
                key.len() as apr_sys::apr_ssize_t,
            );
            if ptr.is_null() {
                None
            } else {
                Some(ptr)
            }
        }
    }

    /// Remove a key from the hash table.
    pub fn remove(&mut self, key: &[u8]) {
        unsafe {
            apr_sys::apr_hash_set(
                self.ptr,
                key.as_ptr() as *const c_void,
                key.len() as apr_sys::apr_ssize_t,
                std::ptr::null_mut(),
            );
        }
    }

    /// Get the number of key-value pairs in the hash table.
    pub fn len(&self) -> usize {
        unsafe { apr_sys::apr_hash_count(self.ptr) as usize }
    }

    /// Check if the hash table is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all entries from the hash table.
    pub fn clear(&mut self) {
        unsafe {
            apr_sys::apr_hash_clear(self.ptr);
        }
    }

    /// Create an iterator over the hash table entries.
    pub fn iter(&self) -> HashIter<'pool> {
        HashIter {
            index: unsafe { apr_sys::apr_hash_first(std::ptr::null_mut(), self.ptr) },
            _phantom: PhantomData,
        }
    }

    /// Get the raw pointer to the APR hash table.
    ///
    /// # Safety
    /// The caller must ensure proper usage of the raw pointer.
    pub unsafe fn as_ptr(&self) -> *const apr_hash_t {
        self.ptr
    }

    /// Get a mutable raw pointer to the APR hash table.
    ///
    /// # Safety
    /// The caller must ensure proper usage of the raw pointer.
    pub unsafe fn as_mut_ptr(&mut self) -> *mut apr_hash_t {
        self.ptr
    }
}

/// Iterator over hash table entries.
pub struct HashIter<'pool> {
    index: *mut apr_sys::apr_hash_index_t,
    _phantom: PhantomData<&'pool ()>,
}

impl<'pool> Iterator for HashIter<'pool> {
    type Item = (&'pool [u8], *mut c_void);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index.is_null() {
            return None;
        }

        let mut key = std::ptr::null();
        let mut key_len: apr_sys::apr_ssize_t = 0;
        let mut val: *mut c_void = std::ptr::null_mut();

        unsafe {
            apr_sys::apr_hash_this(
                self.index,
                &mut key,
                &mut key_len,
                &mut val as *mut *mut c_void,
            );
        }

        self.index = unsafe { apr_sys::apr_hash_next(self.index) };

        // Handle null keys or invalid lengths
        let key = if key.is_null() || key_len <= 0 {
            &[][..]
        } else {
            let len = key_len as usize;
            if len > isize::MAX as usize {
                panic!(
                    "Invalid key_len {} in APR hash - possible memory corruption",
                    len
                );
            }
            unsafe { std::slice::from_raw_parts(key as *const u8, len) }
        };

        Some((key, val))
    }
}

/// A type-safe wrapper around Hash that handles reference types.
///
/// This provides a safer interface for common use cases where you want
/// to store references to Rust values in the hash table.
pub struct TypedHash<'pool, V> {
    inner: Hash<'pool>,
    _phantom: PhantomData<V>,
}

impl<'pool, V> TypedHash<'pool, V> {
    /// Create a new typed hash table.
    pub fn new(pool: &'pool Pool) -> Self {
        Self {
            inner: Hash::new(pool),
            _phantom: PhantomData,
        }
    }

    /// Create a typed hash from an existing raw APR hash pointer.
    ///
    /// # Safety
    /// The caller must ensure:
    /// - The pointer is valid and points to an APR hash
    /// - The hash contains values of type V (or compatible pointers)
    /// - The hash outlives 'pool
    pub unsafe fn from_ptr(ptr: *mut apr_hash_t) -> Self {
        Self {
            inner: Hash::from_ptr(ptr),
            _phantom: PhantomData,
        }
    }

    /// Insert a reference to a value.
    ///
    /// The value must outlive the pool.
    pub fn insert_ref(&mut self, key: &str, value: &'pool V) {
        unsafe {
            self.inner
                .insert(key.as_bytes(), value as *const V as *mut c_void);
        }
    }

    /// Insert a reference with a byte slice key.
    pub fn insert_bytes_ref(&mut self, key: &[u8], value: &'pool V) {
        unsafe {
            self.inner.insert(key, value as *const V as *mut c_void);
        }
    }

    /// Get a reference to a value.
    ///
    /// Returns None if the key doesn't exist.
    /// Panics if the stored value is NULL (which shouldn't happen with insert_ref).
    pub fn get_ref(&self, key: &str) -> Option<&'pool V> {
        self.inner.get(key.as_bytes()).map(|ptr| {
            if ptr.is_null() {
                panic!("Unexpected NULL value in TypedHash");
            }
            unsafe { &*(ptr as *const V) }
        })
    }

    /// Get a reference with a byte slice key.
    pub fn get_bytes_ref(&self, key: &[u8]) -> Option<&'pool V> {
        self.inner.get(key).map(|ptr| {
            if ptr.is_null() {
                panic!("Unexpected NULL value in TypedHash");
            }
            unsafe { &*(ptr as *const V) }
        })
    }

    /// Remove a key from the hash table.
    pub fn remove(&mut self, key: &str) {
        self.inner.remove(key.as_bytes());
    }

    /// Remove with a byte slice key.
    pub fn remove_bytes(&mut self, key: &[u8]) {
        self.inner.remove(key);
    }

    /// Get the number of entries.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the hash table is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    /// Iterate over the entries.
    pub fn iter(&self) -> TypedHashIter<'pool, V> {
        TypedHashIter {
            inner: self.inner.iter(),
            _phantom: PhantomData,
        }
    }
}

/// Iterator for TypedHash.
pub struct TypedHashIter<'pool, V> {
    inner: HashIter<'pool>,
    _phantom: PhantomData<V>,
}

impl<'pool, V: 'pool> Iterator for TypedHashIter<'pool, V> {
    type Item = (&'pool [u8], &'pool V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(key, val)| {
            if val.is_null() {
                panic!("Unexpected NULL value in TypedHash iterator");
            }
            let value = unsafe { &*(val as *const V) };
            (key, value)
        })
    }
}

/// Generate a hash for a key using APR's default hash function.
pub fn hash_default(key: &[u8]) -> u32 {
    unsafe {
        let mut len = key.len() as apr_sys::apr_ssize_t;
        apr_sys::apr_hashfunc_default(key.as_ptr() as *const std::ffi::c_char, &mut len)
    }
}

impl<'pool> Hash<'pool> {
    /// Create a hash table from an iterator of key-value pairs.
    pub fn from_iter<'a, I>(pool: &'pool Pool, iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a [u8], *mut c_void)>,
    {
        let mut hash = Self::new(pool);
        for (key, value) in iter {
            unsafe {
                hash.insert(key, value);
            }
        }
        hash
    }
}

impl<'pool, 'a> Extend<(&'a [u8], *mut c_void)> for Hash<'pool> {
    /// Extend the hash table with key-value pairs from an iterator.
    /// Keys must be borrowed byte slices with stable addresses.
    fn extend<T: IntoIterator<Item = (&'a [u8], *mut c_void)>>(&mut self, iter: T) {
        for (key, value) in iter {
            unsafe {
                self.insert(key, value);
            }
        }
    }
}

impl<'pool, V: 'pool> TypedHash<'pool, V> {
    /// Create a typed hash from an iterator of key-value pairs.
    pub fn from_iter<'a, I>(pool: &'pool Pool, iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'pool V)>,
        'pool: 'a,
    {
        let mut hash = Self::new(pool);
        for (key, value) in iter {
            hash.insert_ref(key, value);
        }
        hash
    }

    /// Create a typed hash from an iterator of byte key-value pairs.
    pub fn from_bytes_iter<'a, I>(pool: &'pool Pool, iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a [u8], &'pool V)>,
        'pool: 'a,
    {
        let mut hash = Self::new(pool);
        for (key, value) in iter {
            hash.insert_bytes_ref(key, value);
        }
        hash
    }
}

impl<'pool, 'a, V: 'pool> Extend<(&'a str, &'pool V)> for TypedHash<'pool, V> {
    /// Extend the typed hash with key-value pairs from an iterator.
    /// Keys must be borrowed strings with stable addresses.
    fn extend<T: IntoIterator<Item = (&'a str, &'pool V)>>(&mut self, iter: T) {
        for (key, value) in iter {
            self.insert_ref(key, value);
        }
    }
}

impl<'pool, 'a, V: 'pool> Extend<(&'a [u8], &'pool V)> for TypedHash<'pool, V> {
    /// Extend the typed hash with byte key-value pairs from an iterator.
    /// Keys must be borrowed byte slices with stable addresses.
    fn extend<T: IntoIterator<Item = (&'a [u8], &'pool V)>>(&mut self, iter: T) {
        for (key, value) in iter {
            self.insert_bytes_ref(key, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_basic_operations() {
        let pool = Pool::new();
        let mut hash = Hash::new(&pool);

        assert!(hash.is_empty());
        assert_eq!(hash.len(), 0);

        // Test with raw pointers
        let value1 = 42i32;
        let value2 = 84i32;

        unsafe {
            hash.insert(b"key1", &value1 as *const i32 as *mut c_void);
            hash.insert(b"key2", &value2 as *const i32 as *mut c_void);
        }

        assert_eq!(hash.len(), 2);
        assert!(!hash.is_empty());

        // Get values back
        let ptr1 = hash.get(b"key1").unwrap();
        let ptr2 = hash.get(b"key2").unwrap();

        unsafe {
            assert_eq!(*(ptr1 as *const i32), 42);
            assert_eq!(*(ptr2 as *const i32), 84);
        }

        // Remove a key
        hash.remove(b"key1");
        assert_eq!(hash.len(), 1);
        assert!(hash.get(b"key1").is_none());
    }

    #[test]
    fn test_typed_hash() {
        let pool = Pool::new();
        let mut hash = TypedHash::<String>::new(&pool);

        let value1 = "hello".to_string();
        let value2 = "world".to_string();

        hash.insert_ref("key1", &value1);
        hash.insert_ref("key2", &value2);

        assert_eq!(hash.len(), 2);

        assert_eq!(hash.get_ref("key1"), Some(&value1));
        assert_eq!(hash.get_ref("key2"), Some(&value2));
        assert_eq!(hash.get_ref("key3"), None);

        // Test iteration
        let items: Vec<_> = hash.iter().collect();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_hash_iteration() {
        let pool = Pool::new();
        let mut hash = Hash::new(&pool);

        let value1 = 1;
        let value2 = 2;

        unsafe {
            hash.insert(b"a", &value1 as *const i32 as *mut c_void);
            hash.insert(b"b", &value2 as *const i32 as *mut c_void);
        }

        let items: Vec<_> = hash.iter().collect();
        assert_eq!(items.len(), 2);

        for (key, val) in items {
            unsafe {
                let value = *(val as *const i32);
                if key == b"a" {
                    assert_eq!(value, 1);
                } else if key == b"b" {
                    assert_eq!(value, 2);
                } else {
                    panic!("Unexpected key");
                }
            }
        }
    }

    #[test]
    fn test_hash_clear() {
        let pool = Pool::new();
        let mut hash = TypedHash::<i32>::new(&pool);

        let value = 42;
        hash.insert_ref("key", &value);
        assert_eq!(hash.len(), 1);

        hash.clear();
        assert_eq!(hash.len(), 0);
        assert!(hash.is_empty());
    }

    #[test]
    fn test_hash_default_function() {
        // Test that the hash function produces consistent results
        assert_eq!(hash_default(b"foo"), hash_default(b"foo"));
        assert_ne!(hash_default(b"foo"), hash_default(b"bar"));
    }

    #[test]
    fn test_hash_with_empty_keys() {
        let pool = Pool::new();
        let mut hash = TypedHash::<i32>::new(&pool);

        let val1 = 42;
        let val2 = 84;

        // Insert with empty key
        hash.insert_bytes_ref(b"", &val1);
        hash.insert_ref("non-empty", &val2);

        assert_eq!(hash.len(), 2);

        // Check we can retrieve both
        assert_eq!(hash.get_bytes_ref(b""), Some(&val1));
        assert_eq!(hash.get_ref("non-empty"), Some(&val2));
    }

    #[test]
    fn test_hash_large_keys() {
        let pool = Pool::new();
        let mut hash = TypedHash::<String>::new(&pool);

        let large_key = vec![b'x'; 10000];
        let value = "large".to_string();

        hash.insert_bytes_ref(&large_key, &value);
        assert_eq!(hash.get_bytes_ref(&large_key), Some(&value));
    }

    #[test]
    fn test_typed_hash_from_iter() {
        let pool = Pool::new();

        let val1 = "value1".to_string();
        let val2 = "value2".to_string();
        let val3 = "value3".to_string();

        let data = vec![("key1", &val1), ("key2", &val2), ("key3", &val3)];

        let hash = TypedHash::from_iter(&pool, data);
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get_ref("key1"), Some(&val1));
        assert_eq!(hash.get_ref("key2"), Some(&val2));
        assert_eq!(hash.get_ref("key3"), Some(&val3));
    }

    #[test]
    fn test_typed_hash_extend() {
        let pool = Pool::new();

        // Create values that live for the whole test function scope
        let val1 = "v1".to_string();
        let val2 = "v2".to_string();
        let val3 = "v3".to_string();

        let mut hash = TypedHash::<String>::new(&pool);

        hash.insert_ref("a", &val1);
        assert_eq!(hash.len(), 1);
        assert_eq!(hash.get_ref("a"), Some(&val1));

        // Test extend with &[u8] keys (borrowed data with stable addresses)
        hash.extend(vec![(b"b".as_slice(), &val2)]);
        assert_eq!(hash.len(), 2);
        assert_eq!(hash.get_bytes_ref(b"b"), Some(&val2));

        // Test extend with &str keys
        hash.extend(vec![("c", &val3)]);
        assert_eq!(hash.len(), 3);
        assert_eq!(hash.get_ref("c"), Some(&val3));
    }
}
