//! Hash table support.

use crate::pool::Pool;
pub use apr_sys::apr_hash_t;
use std::marker::PhantomData;

/// Trait for types that can be constructed from potentially NULL pointers.
///
/// This allows us to handle NULL values differently depending on the target type:
/// - `&T` panics on NULL (since references can't be NULL)
/// - `Option<&T>` returns `None` on NULL
/// - Raw pointers pass through NULL as-is
pub trait FromNullablePtr<'a>: Sized {
    /// Convert from a potentially NULL pointer.
    ///
    /// # Safety
    /// If ptr is non-NULL, it must point to valid data of type T that lives at least as long as 'a.
    unsafe fn from_nullable_ptr<T>(ptr: *mut T) -> Self;
}

// Implementation for references - panics on NULL
// This handles the case where V = T and we return &T
impl<'a, T> FromNullablePtr<'a> for &'a T {
    unsafe fn from_nullable_ptr<U>(ptr: *mut U) -> Self {
        if ptr.is_null() {
            panic!("Cannot convert NULL pointer to reference. Use Option<&T> if NULL values are expected.");
        }
        &*(ptr as *const T)
    }
}

// Implementation for Option<&T> - gracefully handles NULL
// This handles the case where V = T and we return Option<&T>
impl<'a, T> FromNullablePtr<'a> for Option<&'a T> {
    unsafe fn from_nullable_ptr<U>(ptr: *mut U) -> Self {
        if ptr.is_null() {
            None
        } else {
            Some(&*(ptr as *const T))
        }
    }
}

// Implementation for raw pointers - passes through NULL
impl<'a, T> FromNullablePtr<'a> for *const T {
    unsafe fn from_nullable_ptr<U>(ptr: *mut U) -> Self {
        ptr as *const T
    }
}

impl<'a, T> FromNullablePtr<'a> for *mut T {
    unsafe fn from_nullable_ptr<U>(ptr: *mut U) -> Self {
        ptr as *mut T
    }
}

/// Trait for types that can be stored in the hash (converted to void pointers).
pub trait IntoStoredPointer {
    /// Convert this value into a void pointer for storage.
    fn into_stored_pointer(self) -> *mut std::ffi::c_void;
}

// For references - store pointer to the referenced data
impl<T> IntoStoredPointer for &T {
    fn into_stored_pointer(self) -> *mut std::ffi::c_void {
        self as *const T as *mut std::ffi::c_void
    }
}

// For raw pointers - store the pointer value directly
impl<T> IntoStoredPointer for *const T {
    fn into_stored_pointer(self) -> *mut std::ffi::c_void {
        self as *mut std::ffi::c_void
    }
}

impl<T> IntoStoredPointer for *mut T {
    fn into_stored_pointer(self) -> *mut std::ffi::c_void {
        self as *mut std::ffi::c_void
    }
}

// For Option<&T> - convert None to NULL
impl<T> IntoStoredPointer for Option<&T> {
    fn into_stored_pointer(self) -> *mut std::ffi::c_void {
        match self {
            Some(ptr) => ptr as *const T as *mut std::ffi::c_void,
            None => std::ptr::null_mut(),
        }
    }
}

// For Option<*const T> and Option<*mut T>
impl<T> IntoStoredPointer for Option<*const T> {
    fn into_stored_pointer(self) -> *mut std::ffi::c_void {
        match self {
            Some(ptr) => ptr as *mut std::ffi::c_void,
            None => std::ptr::null_mut(),
        }
    }
}

impl<T> IntoStoredPointer for Option<*mut T> {
    fn into_stored_pointer(self) -> *mut std::ffi::c_void {
        self.unwrap_or(std::ptr::null_mut()) as *mut std::ffi::c_void
    }
}

/// A hash map.
///
/// APR hash maps store pointers to values. This wrapper provides a safe interface
/// where values must outlive the hash map through lifetime constraints.
///
/// The value type `V` must be a reference type and determines how NULL pointers are handled:
/// - `&T`: Panics on NULL (references can't be NULL in Rust)
/// - `Option<&T>`: Returns `None` for NULL, `Some(&T)` for valid pointers
/// - `*const T`, `*mut T`: Raw pointers, passes NULL through unchanged
#[derive(Debug)]
pub struct Hash<'data, K, V> {
    ptr: *mut apr_hash_t,
    _phantom: PhantomData<(&'data K, V, &'data Pool)>,
}

/// Trait for types that can be used as keys in a hash map.
pub trait IntoHashKey {
    /// Convert the value into a byte slice that can be used as a key.
    fn into_hash_key(&self) -> &[u8];
}

impl IntoHashKey for &[u8] {
    fn into_hash_key(&self) -> &[u8] {
        self
    }
}

impl IntoHashKey for &str {
    fn into_hash_key(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl IntoHashKey for String {
    fn into_hash_key(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<'data, K: IntoHashKey, V: 'data> Hash<'data, K, V> {
    /// Create a new hash map in the given pool.
    pub fn new(pool: &Pool) -> Self {
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

    /// Insert a key-value pair into the hash map.
    ///
    /// For reference types (&T), stores a pointer to the referenced data.
    /// For raw pointer types, stores the pointer value directly.
    /// For Option types, converts None to NULL pointer.
    pub fn insert(&mut self, key: K, value: V)
    where
        V: IntoStoredPointer,
    {
        let key_bytes = key.into_hash_key();
        unsafe {
            apr_sys::apr_hash_set(
                self.ptr,
                key_bytes.as_ptr() as *const std::ffi::c_void,
                key_bytes.len() as apr_sys::apr_ssize_t,
                value.into_stored_pointer(),
            );
        }
    }

    /// Remove a key from the hash map.
    pub fn remove(&mut self, key: K) {
        let key_bytes = key.into_hash_key();
        unsafe {
            apr_sys::apr_hash_set(
                self.ptr,
                key_bytes.as_ptr() as *const std::ffi::c_void,
                key_bytes.len() as apr_sys::apr_ssize_t,
                std::ptr::null_mut(),
            );
        }
    }

    /// Get the value for the given key.
    ///
    /// The behavior depends on the value type V:
    /// - For `&T`: Returns `None` if key doesn't exist, panics if stored value is NULL
    /// - For `Option<&T>`: Returns `None` if key doesn't exist or stored value is NULL
    /// - For raw pointers: Returns `None` if key doesn't exist, passes through NULL pointers
    pub fn get(&self, key: K) -> Option<V>
    where
        V: FromNullablePtr<'data>,
    {
        let key_bytes = key.into_hash_key();
        unsafe {
            let ptr = apr_sys::apr_hash_get(
                self.ptr,
                key_bytes.as_ptr() as *const std::ffi::c_void,
                key_bytes.len() as apr_sys::apr_ssize_t,
            );
            if ptr.is_null() {
                None // Key doesn't exist
            } else {
                Some(V::from_nullable_ptr(ptr))
            }
        }
    }

    /// Check if the hash map contains the given key.
    pub fn contains_key(&self, key: K) -> bool
    where
        V: FromNullablePtr<'data>,
    {
        self.get(key).is_some()
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

    /// Returns an iterator over the key-value pairs of the hash map.
    ///
    /// Note: Requires a pool for APR's internal iterator allocation.
    pub fn iter(&self, pool: &Pool) -> Iter<'data, V>
    where
        V: FromNullablePtr<'data>,
    {
        let first = unsafe { apr_sys::apr_hash_first(pool.as_mut_ptr() as *mut _, self.ptr) };
        Iter {
            ptr: first,
            _phantom: PhantomData,
        }
    }

    /// Returns an iterator over the keys of the hash map.
    pub fn keys(&self, pool: &Pool) -> Keys {
        let first = unsafe { apr_sys::apr_hash_first(pool.as_mut_ptr() as *mut _, self.ptr) };
        Keys {
            ptr: first,
            _phantom: PhantomData,
        }
    }

    /// Convert into an iterator over key-value pairs.
    /// This consumes the hash and requires a pool for iteration.
    pub fn into_iter(self, pool: &Pool) -> Iter<'data, V>
    where
        V: FromNullablePtr<'data>,
    {
        let first = unsafe { apr_sys::apr_hash_first(pool.as_mut_ptr() as *mut _, self.ptr) };
        Iter {
            ptr: first,
            _phantom: PhantomData,
        }
    }

    /// Copy this hash table to a new pool
    pub fn copy(&self, pool: &Pool) -> Hash<'data, K, V> {
        Hash {
            ptr: unsafe { apr_sys::apr_hash_copy(pool.as_mut_ptr(), self.ptr) },
            _phantom: PhantomData,
        }
    }

    /// Create a Hash from an iterator of key-value pairs.
    pub fn from_iter<I>(pool: &Pool, iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        V: IntoStoredPointer,
    {
        let mut hash = Self::new(pool);
        for (key, value) in iter {
            hash.insert(key, value);
        }
        hash
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
pub struct Iter<'data, V> {
    ptr: *mut apr_sys::apr_hash_index_t,
    _phantom: PhantomData<&'data V>,
}

impl<'data, V> Iterator for Iter<'data, V>
where
    V: FromNullablePtr<'data>,
{
    type Item = (&'data [u8], V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr.is_null() {
            return None;
        }
        let mut key = std::ptr::null();
        let mut key_len = 0;
        let mut val: *mut std::ffi::c_void = std::ptr::null_mut();
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
        let value = unsafe { V::from_nullable_ptr(val) };

        Some((key, value))
    }
}

/// An iterator over the keys of a hash map.
pub struct Keys {
    ptr: *mut apr_sys::apr_hash_index_t,
    _phantom: PhantomData<()>,
}

impl Iterator for Keys {
    type Item = &'static [u8];

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
    fn test_hash_basic_operations() {
        let pool = Pool::new();

        // Create some data that will outlive the hash
        let value1 = String::from("hello");
        let value2 = String::from("world");

        let mut hash: Hash<&str, &String> = Hash::new(&pool);
        assert!(hash.is_empty());
        assert!(hash.get("foo").is_none());

        // Insert values - pass references to the data
        hash.insert("key1", &value1);
        hash.insert("key2", &value2);

        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 2);

        // Get values - returns Option<&String> since V is &String
        assert_eq!(hash.get("key1"), Some(&value1));
        assert_eq!(hash.get("key2"), Some(&value2));
        assert!(hash.get("nonexistent").is_none());

        // Check contains_key
        assert!(hash.contains_key("key1"));
        assert!(hash.contains_key("key2"));
        assert!(!hash.contains_key("nonexistent"));

        // Remove a key
        hash.remove("key1");
        assert_eq!(hash.len(), 1);
        assert!(!hash.contains_key("key1"));
        assert!(hash.get("key1").is_none());

        // Clear all
        hash.clear();
        assert!(hash.is_empty());
        assert_eq!(hash.len(), 0);
    }

    #[test]
    fn test_hash_with_option_values() {
        let pool = Pool::new();

        let value = String::from("test");

        let mut hash: Hash<&str, Option<&String>> = Hash::new(&pool);

        // Insert a value (Some wraps the reference)
        hash.insert("key1", Some(&value));

        // Insert a NULL value
        hash.insert("key2", None::<&String>);

        // Get should return Some(Some(&value)) for valid data
        assert_eq!(hash.get("key1"), Some(Some(&value)));

        // APR hash treats NULL values the same as missing keys
        // So key2 with NULL value appears as if it doesn't exist
        assert_eq!(hash.get("key2"), None);

        // Non-existent key should return None
        assert_eq!(hash.get("nonexistent"), None);

        // This demonstrates that Option<&T> can handle NULL gracefully
        // while &T would panic on NULL
    }

    #[test]
    fn test_hash_with_raw_pointers() {
        let pool = Pool::new();

        let value = 42i32;
        let ptr = &value as *const i32;

        let mut hash: Hash<&str, *const i32> = Hash::new(&pool);
        hash.insert("key", ptr);

        assert_eq!(hash.get("key"), Some(ptr));
        assert_eq!(hash.get("nonexistent"), None);
    }

    #[test]
    fn test_hash_iteration() {
        let pool = Pool::new();

        let value1 = 42i32;
        let value2 = 84i32;

        let mut hash: Hash<&str, &i32> = Hash::new(&pool);
        hash.insert("key1", &value1);
        hash.insert("key2", &value2);

        // Test iterator
        let items: Vec<_> = hash.iter(&pool).collect();
        assert_eq!(items.len(), 2);

        // Verify we can find both values
        let values: Vec<i32> = items.iter().map(|(_, v)| **v).collect();
        assert!(values.contains(&42));
        assert!(values.contains(&84));

        // Test keys iterator
        let keys: Vec<_> = hash.keys(&pool).collect();
        assert_eq!(keys.len(), 2);

        // Test into_iter
        let items: Vec<_> = hash.into_iter(&pool).collect();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_hash_from_iter() {
        let pool = Pool::new();

        let value1 = "hello".to_string();
        let value2 = "world".to_string();

        let data = vec![("key1", &value1), ("key2", &value2)];
        let hash: Hash<&str, &String> = Hash::from_iter(&pool, data);

        assert_eq!(hash.len(), 2);
        assert_eq!(hash.get("key1"), Some(&value1));
        assert_eq!(hash.get("key2"), Some(&value2));
    }

    #[test]
    fn test_hash_copy() {
        let pool1 = Pool::new();
        let pool2 = Pool::new();

        let value = "test".to_string();

        let mut hash1: Hash<&str, &String> = Hash::new(&pool1);
        hash1.insert("key", &value);

        let hash2 = hash1.copy(&pool2);
        assert_eq!(hash2.get("key"), Some(&value));
        assert_eq!(hash1.len(), hash2.len());
    }
}
