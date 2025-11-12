//! APR tables and arrays implementation.
//!
//! This module provides APR's table (string key-value pairs) and array
//! (dynamic arrays) data structures.

use crate::pool::Pool;
pub use apr_sys::{apr_array_header_t, apr_table_t};
use std::ffi::{c_char, c_void, CStr, CString};
use std::marker::PhantomData;

/// A dynamic array that stores raw data.
///
/// This is a direct wrapper around APR's array_header implementation.
/// Arrays store fixed-size elements contiguously in memory.
pub struct Array<'pool> {
    ptr: *mut apr_array_header_t,
    _phantom: PhantomData<&'pool Pool<'pool>>,
}

impl<'pool> Array<'pool> {
    /// Create a new array.
    ///
    /// # Arguments
    /// * `pool` - Memory pool for allocation
    /// * `nelts` - Initial number of elements to allocate
    /// * `elt_size` - Size of each element in bytes
    pub fn new(pool: &'pool Pool<'pool>, nelts: i32, elt_size: i32) -> Self {
        let ptr = unsafe { apr_sys::apr_array_make(pool.as_mut_ptr(), nelts, elt_size) };
        Self {
            ptr,
            _phantom: PhantomData,
        }
    }

    /// Create an array from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be valid and point to an APR array header.
    pub unsafe fn from_ptr(ptr: *mut apr_array_header_t) -> Self {
        Self {
            ptr,
            _phantom: PhantomData,
        }
    }

    /// Push raw bytes onto the array.
    ///
    /// # Safety
    /// The bytes must be exactly `elt_size` bytes (as specified in `new`).
    pub unsafe fn push_raw(&mut self, data: &[u8]) {
        let dst = apr_sys::apr_array_push(self.ptr);
        std::ptr::copy_nonoverlapping(data.as_ptr(), dst as *mut u8, data.len());
    }

    /// Get a pointer to an element at the given index.
    ///
    /// # Safety
    /// The index must be valid and the caller must know the element type.
    pub unsafe fn get_raw(&self, index: usize) -> *mut c_void {
        let header = &*self.ptr;
        if index >= header.nelts as usize {
            panic!("Array index out of bounds");
        }
        let elts = header.elts as *mut u8;
        elts.add(index * header.elt_size as usize) as *mut c_void
    }

    /// Get the number of elements in the array.
    pub fn len(&self) -> usize {
        unsafe { (*self.ptr).nelts as usize }
    }

    /// Check if the array is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all elements from the array.
    pub fn clear(&mut self) {
        unsafe {
            apr_sys::apr_array_clear(self.ptr);
        }
    }

    /// Get the raw pointer to the array header.
    ///
    /// # Safety
    /// The caller must ensure proper usage of the raw pointer.
    pub unsafe fn as_ptr(&self) -> *const apr_array_header_t {
        self.ptr
    }

    /// Get a mutable raw pointer to the array header.
    ///
    /// # Safety
    /// The caller must ensure proper usage of the raw pointer.
    pub unsafe fn as_mut_ptr(&mut self) -> *mut apr_array_header_t {
        self.ptr
    }
}

/// A type-safe wrapper for arrays of a specific type.
pub struct TypedArray<'pool, T: Copy> {
    inner: Array<'pool>,
    _phantom: PhantomData<T>,
}

impl<'pool, T: Copy> TypedArray<'pool, T> {
    /// Create a new typed array.
    pub fn new(pool: &'pool Pool<'pool>, initial_size: i32) -> Self {
        Self {
            inner: Array::new(pool, initial_size, std::mem::size_of::<T>() as i32),
            _phantom: PhantomData,
        }
    }

    /// Create a typed array from an existing raw APR array pointer.
    ///
    /// # Safety
    /// The caller must ensure:
    /// - The pointer is valid and points to an APR array
    /// - The array contains elements of type T with correct size
    /// - The array outlives 'pool
    pub unsafe fn from_ptr(ptr: *mut apr_array_header_t) -> Self {
        Self {
            inner: Array::from_ptr(ptr),
            _phantom: PhantomData,
        }
    }

    /// Push a value onto the array.
    pub fn push(&mut self, value: T) {
        unsafe {
            let bytes = std::slice::from_raw_parts(
                &value as *const T as *const u8,
                std::mem::size_of::<T>(),
            );
            self.inner.push_raw(bytes);
        }
    }

    /// Get a value at the given index.
    pub fn get(&self, index: usize) -> Option<T> {
        if index >= self.len() {
            return None;
        }
        unsafe {
            let ptr = self.inner.get_raw(index) as *const T;
            Some(*ptr)
        }
    }

    /// Get the number of elements.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the array is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clear all elements.
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    /// Create an iterator over the array elements.
    pub fn iter(&self) -> TypedArrayIter<'_, 'pool, T> {
        TypedArrayIter {
            array: self,
            index: 0,
        }
    }

    /// Get the raw pointer to the array header.
    ///
    /// # Safety
    /// The caller must ensure proper usage of the raw pointer.
    pub unsafe fn as_ptr(&self) -> *const apr_array_header_t {
        self.inner.as_ptr()
    }

    /// Get a mutable raw pointer to the array header.
    ///
    /// # Safety
    /// The caller must ensure proper usage of the raw pointer.
    pub unsafe fn as_mut_ptr(&mut self) -> *mut apr_array_header_t {
        self.inner.as_mut_ptr()
    }
}

/// Iterator for TypedArray.
pub struct TypedArrayIter<'a, 'pool, T: Copy> {
    array: &'a TypedArray<'pool, T>,
    index: usize,
}

impl<'a, 'pool, T: Copy> Iterator for TypedArrayIter<'a, 'pool, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.array.len() {
            let value = self.array.get(self.index);
            self.index += 1;
            value
        } else {
            None
        }
    }
}

impl<'pool, T: Copy> TypedArray<'pool, T> {
    /// Create a typed array from an iterator of values.
    pub fn from_iter<I>(pool: &'pool Pool<'pool>, iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let iter = iter.into_iter();
        let mut array = Self::new(pool, iter.len() as i32);
        for value in iter {
            array.push(value);
        }
        array
    }
}

impl<'pool, T: Copy> Extend<T> for TypedArray<'pool, T> {
    /// Extend the array with values from an iterator.
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.push(value);
        }
    }
}

/// A table that maps C strings to C strings.
///
/// This is a direct wrapper around APR's table implementation.
/// Tables are case-insensitive for keys and can have multiple values per key.
pub struct Table<'pool> {
    ptr: *mut apr_table_t,
    _phantom: PhantomData<&'pool Pool<'pool>>,
}

impl<'pool> Table<'pool> {
    /// Create a new table.
    pub fn new(pool: &'pool Pool<'pool>, nelts: i32) -> Self {
        let ptr = unsafe { apr_sys::apr_table_make(pool.as_mut_ptr(), nelts) };
        Self {
            ptr,
            _phantom: PhantomData,
        }
    }

    /// Create a table from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be valid and point to an APR table.
    pub unsafe fn from_ptr(ptr: *mut apr_table_t) -> Self {
        Self {
            ptr,
            _phantom: PhantomData,
        }
    }

    /// Set a key-value pair in the table.
    ///
    /// # Safety
    /// The key and value must be valid C strings.
    pub unsafe fn set_raw(&mut self, key: *const c_char, val: *const c_char) {
        apr_sys::apr_table_set(self.ptr, key, val);
    }

    /// Get a value from the table.
    ///
    /// # Safety
    /// The key must be a valid C string.
    pub unsafe fn get_raw(&self, key: *const c_char) -> *const c_char {
        apr_sys::apr_table_get(self.ptr, key)
    }

    /// Add a key-value pair (allows duplicates).
    ///
    /// # Safety
    /// The key and value must be valid C strings.
    pub unsafe fn add_raw(&mut self, key: *const c_char, val: *const c_char) {
        apr_sys::apr_table_add(self.ptr, key, val);
    }

    /// Remove entries with the given key.
    ///
    /// # Safety
    /// The key must be a valid C string.
    pub unsafe fn unset_raw(&mut self, key: *const c_char) {
        apr_sys::apr_table_unset(self.ptr, key);
    }

    /// Get the number of entries in the table.
    pub fn len(&self) -> usize {
        unsafe {
            let entries = apr_sys::apr_table_elts(self.ptr);
            (*entries).nelts as usize
        }
    }

    /// Check if the table is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all entries from the table.
    pub fn clear(&mut self) {
        unsafe {
            apr_sys::apr_table_clear(self.ptr);
        }
    }

    /// Get the raw pointer to the table.
    ///
    /// # Safety
    /// The caller must ensure proper usage of the raw pointer.
    pub unsafe fn as_ptr(&self) -> *const apr_table_t {
        self.ptr
    }

    /// Get a mutable raw pointer to the table.
    ///
    /// # Safety
    /// The caller must ensure proper usage of the raw pointer.
    pub unsafe fn as_mut_ptr(&mut self) -> *mut apr_table_t {
        self.ptr
    }
}

/// A type-safe wrapper for tables with string values.
pub struct StringTable<'pool> {
    inner: Table<'pool>,
}

impl<'pool> StringTable<'pool> {
    /// Create a new string table.
    pub fn new(pool: &'pool Pool, initial_size: i32) -> Self {
        Self {
            inner: Table::new(pool, initial_size),
        }
    }

    /// Create a string table from an existing raw APR table pointer.
    ///
    /// # Safety
    /// The caller must ensure:
    /// - The pointer is valid and points to an APR table
    /// - The table contains valid C strings
    /// - The table outlives 'pool
    pub unsafe fn from_ptr(ptr: *mut apr_table_t, _pool: &'pool Pool) -> Self {
        Self {
            inner: Table::from_ptr(ptr),
        }
    }

    /// Set a key-value pair.
    pub fn set(&mut self, key: &str, value: &str) {
        let key_cstr = CString::new(key).expect("Invalid key");
        let val_cstr = CString::new(value).expect("Invalid value");

        // APR tables copy the strings, so we can use temporary CStrings
        unsafe {
            self.inner.set_raw(key_cstr.as_ptr(), val_cstr.as_ptr());
        }
    }

    /// Get a value from the table.
    pub fn get(&self, key: &str) -> Option<&str> {
        let key_cstr = CString::new(key).ok()?;

        unsafe {
            let val_ptr = self.inner.get_raw(key_cstr.as_ptr());
            if val_ptr.is_null() {
                None
            } else {
                let val_cstr = CStr::from_ptr(val_ptr);
                val_cstr.to_str().ok()
            }
        }
    }

    /// Add a key-value pair (allows duplicates).
    pub fn add(&mut self, key: &str, value: &str) {
        let key_cstr = CString::new(key).expect("Invalid key");
        let val_cstr = CString::new(value).expect("Invalid value");

        unsafe {
            self.inner.add_raw(key_cstr.as_ptr(), val_cstr.as_ptr());
        }
    }

    /// Remove all entries with the given key.
    pub fn unset(&mut self, key: &str) {
        if let Ok(key_cstr) = CString::new(key) {
            unsafe {
                self.inner.unset_raw(key_cstr.as_ptr());
            }
        }
    }

    /// Get the number of entries.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the table is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    /// Create an iterator over table entries.
    pub fn iter(&self) -> StringTableIter<'_, 'pool> {
        StringTableIter {
            table: &self.inner,
            index: 0,
            _phantom: PhantomData,
        }
    }
}

/// Iterator for StringTable that returns references to the strings.
pub struct StringTableIter<'a, 'pool> {
    table: &'a Table<'pool>,
    index: usize,
    _phantom: PhantomData<&'pool ()>,
}

impl<'a, 'pool> Iterator for StringTableIter<'a, 'pool> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let elts = apr_sys::apr_table_elts(self.table.ptr);
            let header = &*elts;

            if self.index >= header.nelts as usize {
                return None;
            }

            // APR table entries are stored as pairs of char* pointers
            // Each entry has: key, val, key_checksum (but we only need key and val)
            let entry_size = std::mem::size_of::<(*const c_char, *const c_char, u32)>();
            let entry_ptr = (header.elts as *const u8).add(self.index * entry_size);

            let key_ptr = *(entry_ptr as *const *const c_char);
            let val_ptr =
                *(entry_ptr.add(std::mem::size_of::<*const c_char>()) as *const *const c_char);

            self.index += 1;

            if key_ptr.is_null() {
                return self.next();
            }

            let key = CStr::from_ptr(key_ptr).to_str().ok()?;
            let val = if val_ptr.is_null() {
                ""
            } else {
                CStr::from_ptr(val_ptr).to_str().ok()?
            };

            Some((key, val))
        }
    }
}

impl<'pool> StringTable<'pool> {
    /// Create a string table from an iterator of key-value pairs.
    pub fn from_iter<'a, I>(pool: &'pool Pool, iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        let mut table = Self::new(pool, 16);
        for (key, value) in iter {
            table.set(key, value);
        }
        table
    }
}

impl<'pool> Extend<(String, String)> for StringTable<'pool> {
    /// Extend the table with key-value pairs from an iterator.
    fn extend<I: IntoIterator<Item = (String, String)>>(&mut self, iter: I) {
        for (key, value) in iter {
            self.set(&key, &value);
        }
    }
}

impl<'pool, 'a> Extend<(&'a str, &'a str)> for StringTable<'pool> {
    /// Extend the table with key-value pairs from an iterator.
    fn extend<I: IntoIterator<Item = (&'a str, &'a str)>>(&mut self, iter: I) {
        for (key, value) in iter {
            self.set(key, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array_basic() {
        let pool = Pool::new();
        let mut array = Array::new(&pool, 10, std::mem::size_of::<i32>() as i32);

        assert!(array.is_empty());

        // Push some values
        let value1 = 42i32;
        let value2 = 84i32;

        unsafe {
            array.push_raw(&value1.to_ne_bytes());
            array.push_raw(&value2.to_ne_bytes());
        }

        assert_eq!(array.len(), 2);

        // Get values back
        unsafe {
            let ptr1 = array.get_raw(0) as *const i32;
            let ptr2 = array.get_raw(1) as *const i32;
            assert_eq!(*ptr1, 42);
            assert_eq!(*ptr2, 84);
        }
    }

    #[test]
    fn test_typed_array() {
        let pool = Pool::new();
        let mut array = TypedArray::<i32>::new(&pool, 10);

        assert!(array.is_empty());

        array.push(42);
        array.push(84);
        array.push(126);

        assert_eq!(array.len(), 3);

        assert_eq!(array.get(0), Some(42));
        assert_eq!(array.get(1), Some(84));
        assert_eq!(array.get(2), Some(126));
        assert_eq!(array.get(3), None);

        // Test iteration
        let values: Vec<_> = array.iter().collect();
        assert_eq!(values, vec![42, 84, 126]);
    }

    #[test]
    fn test_table_basic() {
        let pool = Pool::new();
        let mut table = Table::new(&pool, 10);

        assert!(table.is_empty());

        // Set some values
        let key1 = CString::new("key1").unwrap();
        let val1 = CString::new("value1").unwrap();
        let key2 = CString::new("key2").unwrap();
        let val2 = CString::new("value2").unwrap();

        unsafe {
            table.set_raw(key1.as_ptr(), val1.as_ptr());
            table.set_raw(key2.as_ptr(), val2.as_ptr());
        }

        assert_eq!(table.len(), 2);

        // Get values back
        unsafe {
            let result = table.get_raw(key1.as_ptr());
            assert!(!result.is_null());
            assert_eq!(CStr::from_ptr(result).to_str().unwrap(), "value1");
        }
    }

    #[test]
    fn test_string_table() {
        let pool = Pool::new();
        let mut table = StringTable::new(&pool, 10);

        assert!(table.is_empty());

        table.set("key1", "value1");
        table.set("key2", "value2");

        assert_eq!(table.len(), 2);

        assert_eq!(table.get("key1"), Some("value1"));
        assert_eq!(table.get("key2"), Some("value2"));
        assert_eq!(table.get("key3"), None);

        // Test add (duplicates)
        table.add("key1", "another_value1");
        assert!(table.len() > 2); // Should have more entries now

        // Test unset
        table.unset("key1");
        assert_eq!(table.get("key1"), None);
    }

    #[test]
    fn test_typed_array_from_iter() {
        let pool = Pool::new();

        let data = vec![1, 2, 3, 4, 5];
        let array = TypedArray::<i32>::from_iter(&pool, data.clone());

        assert_eq!(array.len(), 5);
        for (i, &val) in data.iter().enumerate() {
            assert_eq!(array.get(i), Some(val));
        }
    }

    #[test]
    fn test_typed_array_extend() {
        let pool = Pool::new();
        let mut array = TypedArray::<i32>::new(&pool, 10);

        array.push(1);
        array.push(2);
        assert_eq!(array.len(), 2);

        array.extend(vec![3, 4, 5]);
        assert_eq!(array.len(), 5);

        assert_eq!(array.get(0), Some(1));
        assert_eq!(array.get(4), Some(5));
    }

    #[test]
    fn test_string_table_from_iter() {
        let pool = Pool::new();

        let data = vec![("key1", "value1"), ("key2", "value2"), ("key3", "value3")];

        let table = StringTable::from_iter(&pool, data);

        assert_eq!(table.len(), 3);
        assert_eq!(table.get("key1"), Some("value1"));
        assert_eq!(table.get("key2"), Some("value2"));
        assert_eq!(table.get("key3"), Some("value3"));
    }

    #[test]
    fn test_string_table_extend() {
        let pool = Pool::new();
        let mut table = StringTable::new(&pool, 10);

        table.set("a", "1");
        assert_eq!(table.len(), 1);

        table.extend(vec![("b", "2"), ("c", "3")]);
        assert_eq!(table.len(), 3);

        assert_eq!(table.get("a"), Some("1"));
        assert_eq!(table.get("b"), Some("2"));
        assert_eq!(table.get("c"), Some("3"));

        // Test extend with String pairs
        table.extend(vec![
            ("d".to_string(), "4".to_string()),
            ("e".to_string(), "5".to_string()),
        ]);
        assert_eq!(table.len(), 5);
    }
}
