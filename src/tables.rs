use crate::pool::Pool;
pub use apr_sys::{apr_array_header_t, apr_table_t};
use std::marker::PhantomData;

/// Trait for types that can be constructed from potentially NULL C string pointers.
///
/// APR tables store C strings, so this handles the NULL conversion for different target types:
/// - `&CStr` panics on NULL (references can't be NULL)
/// - `Option<&CStr>` returns `None` on NULL
/// - Raw pointers pass through NULL as-is
pub trait FromNullableCStr<'a>: Sized {
    /// Convert from a potentially NULL C string pointer.
    ///
    /// # Safety
    /// If ptr is non-NULL, it must point to a valid null-terminated C string
    /// that lives at least as long as 'a.
    unsafe fn from_nullable_cstr(ptr: *const std::ffi::c_char) -> Self;
}

// Implementation for &CStr - panics on NULL
impl<'a> FromNullableCStr<'a> for &'a std::ffi::CStr {
    unsafe fn from_nullable_cstr(ptr: *const std::ffi::c_char) -> Self {
        if ptr.is_null() {
            panic!("Cannot convert NULL pointer to &CStr. Use Option<&CStr> if NULL values are expected.");
        }
        std::ffi::CStr::from_ptr(ptr)
    }
}

// Implementation for Option<&CStr> - gracefully handles NULL
impl<'a> FromNullableCStr<'a> for Option<&'a std::ffi::CStr> {
    unsafe fn from_nullable_cstr(ptr: *const std::ffi::c_char) -> Self {
        if ptr.is_null() {
            None
        } else {
            Some(std::ffi::CStr::from_ptr(ptr))
        }
    }
}

// Implementation for raw pointers - passes through NULL
impl<'a> FromNullableCStr<'a> for *const std::ffi::c_char {
    unsafe fn from_nullable_cstr(ptr: *const std::ffi::c_char) -> Self {
        ptr
    }
}

// Implementation for CString - owned, panics on NULL
impl<'a> FromNullableCStr<'a> for std::ffi::CString {
    unsafe fn from_nullable_cstr(ptr: *const std::ffi::c_char) -> Self {
        if ptr.is_null() {
            panic!("Cannot convert NULL pointer to CString. Use Option<CString> if NULL values are expected.");
        }
        std::ffi::CStr::from_ptr(ptr).to_owned()
    }
}

// Implementation for Option<CString> - gracefully handles NULL
impl<'a> FromNullableCStr<'a> for Option<std::ffi::CString> {
    unsafe fn from_nullable_cstr(ptr: *const std::ffi::c_char) -> Self {
        if ptr.is_null() {
            None
        } else {
            Some(std::ffi::CStr::from_ptr(ptr).to_owned())
        }
    }
}

/// Trait for types that can be stored in APR tables (converted to C string pointers).
pub trait IntoStoredCStr {
    /// Convert this value into a C string pointer for storage.
    fn into_stored_cstr(&self) -> *const std::ffi::c_char;
}

// For C string types - store the pointer directly
impl IntoStoredCStr for &std::ffi::CStr {
    fn into_stored_cstr(&self) -> *const std::ffi::c_char {
        self.as_ptr()
    }
}

impl IntoStoredCStr for std::ffi::CString {
    fn into_stored_cstr(&self) -> *const std::ffi::c_char {
        self.as_ptr()
    }
}

impl IntoStoredCStr for &str {
    fn into_stored_cstr(&self) -> *const std::ffi::c_char {
        // This is unsafe but commonly needed - user must ensure the CString outlives usage
        // Better to use CStr/CString directly for safety
        std::ffi::CString::new(*self).unwrap().into_raw() as *const _
    }
}

// For Option types - convert None to NULL
impl IntoStoredCStr for Option<&std::ffi::CStr> {
    fn into_stored_cstr(&self) -> *const std::ffi::c_char {
        match self {
            Some(s) => s.as_ptr(),
            None => std::ptr::null(),
        }
    }
}

impl IntoStoredCStr for Option<&str> {
    fn into_stored_cstr(&self) -> *const std::ffi::c_char {
        match self {
            Some(s) => std::ffi::CString::new(*s).unwrap().into_raw() as *const _,
            None => std::ptr::null(),
        }
    }
}

/// Trait for types that can be retrieved from APR arrays.
pub trait FromAprArrayElement<'array>: Sized {
    /// Convert from array element pointer to this type.
    ///
    /// # Safety
    /// The caller must ensure:
    /// - The pointer is valid and points to an element of the correct type
    /// - The data lives at least as long as 'array lifetime
    unsafe fn from_apr_array_element(ptr: *const std::ffi::c_void) -> Self;
}

/// Trait for types that can be stored in APR arrays.
pub trait IntoAprArrayElement: Sized {
    /// Convert this value into bytes for storage in an APR array.
    fn into_apr_array_element(&self) -> Vec<u8>;
}

// Basic implementations for Copy types
impl<'array, T: Copy> FromAprArrayElement<'array> for T {
    unsafe fn from_apr_array_element(ptr: *const std::ffi::c_void) -> Self {
        *(ptr as *const T)
    }
}

impl<T: Copy> IntoAprArrayElement for T {
    fn into_apr_array_element(&self) -> Vec<u8> {
        let size = std::mem::size_of::<T>();
        let mut bytes = Vec::with_capacity(size);
        unsafe {
            let ptr = self as *const T as *const u8;
            bytes.extend_from_slice(std::slice::from_raw_parts(ptr, size));
        }
        bytes
    }
}

/// A wrapper around an `apr_array_header_t`.
#[derive(Debug)]
pub struct ArrayHeader<'pool, T> {
    ptr: *mut apr_array_header_t,
    _phantom: PhantomData<(T, &'pool Pool)>,
}

impl<'pool, T> ArrayHeader<'pool, T> {
    /// Returns true if the array is empty.
    pub fn is_empty(&self) -> bool {
        unsafe { apr_sys::apr_is_empty_array(self.ptr) != 0 }
    }

    /// Returns the number of elements in the array.
    pub fn len(&self) -> usize {
        unsafe { (*self.ptr).nelts as usize }
    }

    /// Returns the number of elements that can be stored in the array without reallocating.
    pub fn allocated(&self) -> usize {
        unsafe { (*self.ptr).nalloc as usize }
    }

    /// Create an empty ArrayHeader.
    pub fn new(pool: &'pool Pool) -> Self
    where
        T: Sized,
    {
        Self::new_with_capacity(pool, 0)
    }

    /// Create a new array with a given capacity
    pub fn new_with_capacity(pool: &'pool Pool, nelts: usize) -> Self
    where
        T: Sized,
    {
        let array = unsafe {
            apr_sys::apr_array_make(
                pool.as_mut_ptr(),
                nelts as i32,
                std::mem::size_of::<T>() as i32,
            )
        };

        Self {
            ptr: array,
            _phantom: PhantomData,
        }
    }

    /// Create an array from a raw APR array pointer
    pub fn from_ptr(ptr: *mut apr_array_header_t) -> Self {
        Self {
            ptr,
            _phantom: PhantomData,
        }
    }

    /// Return a pointer to the element at the given index.
    unsafe fn nth_ptr(&self, index: usize) -> *const std::ffi::c_void {
        (*self.ptr).elts.add(index * (*self.ptr).elt_size as usize) as *const std::ffi::c_void
    }

    /// Return the size of each element in the array.
    pub fn element_size(&self) -> usize {
        unsafe { (*self.ptr).elt_size as usize }
    }

    /// Clear the array
    pub fn clear(&mut self) {
        unsafe {
            apr_sys::apr_array_clear(self.ptr);
        }
    }

    /// Create an iterator over just the indices.
    pub fn indices(&self) -> impl Iterator<Item = usize> + '_ {
        0..self.len()
    }

    /// Return a pointer to the underlying `apr_array_header_t`.
    pub fn as_ptr(&self) -> *const apr_sys::apr_array_header_t {
        self.ptr
    }

    /// Get a mutable raw pointer to the underlying APR array header
    pub unsafe fn as_mut_ptr(&mut self) -> *mut apr_sys::apr_array_header_t {
        self.ptr
    }
}

// Methods that require FromAprArrayElement for retrieving values
impl<'pool, T: FromAprArrayElement<'pool>> ArrayHeader<'pool, T> {
    /// Return the element at the given index.
    pub fn nth(&self, index: usize) -> Option<T> {
        if index < self.len() {
            Some(unsafe { T::from_apr_array_element(self.nth_ptr(index)) })
        } else {
            None
        }
    }

    /// Get the first element, if any.
    pub fn first(&self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            self.nth(0)
        }
    }

    /// Get the last element, if any.
    pub fn last(&self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            self.nth(self.len() - 1)
        }
    }

    /// Iterate over the entries in an array
    pub fn iter(&'pool self) -> ArrayHeaderIterator<'pool, T> {
        ArrayHeaderIterator::new(self)
    }
}

// Methods that require IntoAprArrayElement for storing values
impl<'pool, T: IntoAprArrayElement> ArrayHeader<'pool, T> {
    /// Push an element onto the end of the array.
    pub fn push(&mut self, item: T) {
        unsafe {
            let ptr = apr_sys::apr_array_push(self.ptr);
            let bytes = item.into_apr_array_element();
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, bytes.len());
        }
    }

    /// Push an element and return self for chaining.
    pub fn with_push(mut self, item: T) -> Self {
        self.push(item);
        self
    }

    /// Extend the array with items from an iterator (like Vec::extend).
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }

    /// Extend the array with items and return self for chaining.
    pub fn with_extend<I: IntoIterator<Item = T>>(mut self, iter: I) -> Self {
        self.extend(iter);
        self
    }

    /// Builder pattern: create array, reserve capacity, and populate
    pub fn with_items<I: IntoIterator<Item = T>>(pool: &'pool Pool, items: I) -> Self
    where
        T: Sized,
    {
        let items: Vec<T> = items.into_iter().collect();
        let mut array = Self::new_with_capacity(pool, items.len());
        for item in items {
            array.push(item);
        }
        array
    }
}

// Methods that require both traits
impl<'pool, T: FromAprArrayElement<'pool> + IntoAprArrayElement> ArrayHeader<'pool, T> {
    /// Concatenate two arrays.
    pub fn cat(&mut self, other: &ArrayHeader<'pool, T>) {
        unsafe {
            apr_sys::apr_array_cat(self.ptr, other.ptr);
        }
    }

    /// Append two arrays.
    pub fn append<'newpool>(
        pool: &'newpool Pool,
        first: &ArrayHeader<'pool, T>,
        second: &ArrayHeader<'pool, T>,
    ) -> ArrayHeader<'newpool, T> {
        ArrayHeader {
            ptr: unsafe { apr_sys::apr_array_append(pool.as_mut_ptr(), first.ptr, second.ptr) },
            _phantom: PhantomData,
        }
    }

    /// Copy the array.
    pub fn copy<'newpool>(&self, pool: &'newpool Pool) -> ArrayHeader<'newpool, T> {
        ArrayHeader {
            ptr: unsafe { apr_sys::apr_array_copy(pool.as_mut_ptr(), self.ptr) },
            _phantom: PhantomData,
        }
    }
}

/// An iterator over the elements of an `ArrayHeader`.
#[derive(Debug)]
pub struct ArrayHeaderIterator<'pool, T> {
    array: &'pool ArrayHeader<'pool, T>,
    index: usize,
}

impl<'pool, T> ArrayHeaderIterator<'pool, T> {
    fn new(array: &'pool ArrayHeader<'pool, T>) -> Self {
        Self { array, index: 0 }
    }
}

impl<'pool, T: FromAprArrayElement<'pool>> Iterator for ArrayHeaderIterator<'pool, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.array.nth(self.index) {
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

// Implement IntoIterator for ArrayHeader
impl<'pool, T: FromAprArrayElement<'pool>> IntoIterator for &'pool ArrayHeader<'pool, T> {
    type Item = T;
    type IntoIter = ArrayHeaderIterator<'pool, T>;

    fn into_iter(self) -> Self::IntoIter {
        ArrayHeaderIterator::new(self)
    }
}

// Implement FromIterator for ArrayHeader
impl<'pool, T: IntoAprArrayElement + Sized> ArrayHeader<'pool, T> {
    /// Create an ArrayHeader from an iterator.
    pub fn from_iter<I: IntoIterator<Item = T>>(pool: &'pool Pool, iter: I) -> Self {
        let mut array = Self::new(pool);
        for item in iter {
            array.push(item);
        }
        array
    }
}

/// APR table data structure (ordered key-value pairs, allows duplicates)
#[derive(Debug)]
pub struct Table<'pool, V> {
    ptr: *mut apr_table_t,
    _phantom: PhantomData<(V, &'pool Pool)>,
}

impl<'pool, V> Table<'pool, V> {
    /// Check if the table is empty.
    pub fn is_empty(&self) -> bool {
        unsafe { apr_sys::apr_is_empty_table(self.ptr) != 0 }
    }

    /// Copy the table to a new pool
    pub fn copy<'newpool>(&self, pool: &'newpool Pool) -> Table<'newpool, V> {
        let newtable = unsafe { apr_sys::apr_table_copy(pool.as_mut_ptr(), self.ptr) };
        Table {
            ptr: newtable,
            _phantom: PhantomData,
        }
    }

    /// Clear the table.
    pub fn clear(&mut self) {
        unsafe {
            apr_sys::apr_table_clear(self.ptr);
        }
    }

    /// Create a new table, with space for nelts entries.
    pub fn new_with_capacity(pool: &'pool Pool, nelts: usize) -> Self {
        let ret = unsafe { apr_sys::apr_table_make(pool.as_mut_ptr(), nelts as i32) };
        Self {
            ptr: ret,
            _phantom: PhantomData,
        }
    }

    /// Check if the table contains a key.
    pub fn contains_key(&self, key: &str) -> bool {
        let key = std::ffi::CString::new(key).unwrap();
        unsafe {
            let value = apr_sys::apr_table_get(self.ptr, key.as_ptr());
            !value.is_null()
        }
    }

    /// Remove all entries with the given key
    pub fn unset(&mut self, key: &str) {
        let key = std::ffi::CString::new(key).unwrap();
        unsafe {
            apr_sys::apr_table_unset(self.ptr, key.as_ptr());
        }
    }

    /// Create a new empty table (convenience method).
    pub fn new(pool: &'pool Pool) -> Self {
        Self::new_with_capacity(pool, 0)
    }

    /// Return a pointer to the underlying apr_table_t.
    pub fn as_ptr(&self) -> *const apr_table_t {
        self.ptr
    }
}

// Methods that require FromNullableCStr for retrieving values
impl<'pool, V: FromNullableCStr<'pool>> Table<'pool, V> {
    /// Return the item with the given key.
    pub fn get(&self, key: &str) -> Option<V> {
        let key = std::ffi::CString::new(key).unwrap();
        unsafe {
            let value = apr_sys::apr_table_get(self.ptr, key.as_ptr());
            if value.is_null() {
                None
            } else {
                Some(V::from_nullable_cstr(value))
            }
        }
    }

    /// Get a value from the table using a memory pool for allocation
    pub fn getm(&self, pool: &mut crate::Pool, key: &str) -> Option<V> {
        let key = std::ffi::CString::new(key).unwrap();
        unsafe {
            let value = apr_sys::apr_table_getm(pool.as_mut_ptr(), self.ptr, key.as_ptr());
            if value.is_null() {
                None
            } else {
                Some(V::from_nullable_cstr(value))
            }
        }
    }
}

// Methods that require IntoStoredCStr for storing values
impl<'pool, V: IntoStoredCStr> Table<'pool, V> {
    /// Set the value of a key.
    pub fn set(&mut self, key: &str, value: V) {
        let key = std::ffi::CString::new(key).unwrap();
        unsafe {
            apr_sys::apr_table_set(self.ptr, key.as_ptr(), value.into_stored_cstr());
        }
    }

    /// Set a value in the table without copying the strings
    pub fn setn(&mut self, key: &str, value: V) {
        let key = std::ffi::CString::new(key).unwrap();
        unsafe {
            apr_sys::apr_table_setn(self.ptr, key.as_ptr(), value.into_stored_cstr());
        }
    }

    /// Merge a value with existing values for the key
    pub fn merge(&mut self, key: &str, value: V) {
        let key = std::ffi::CString::new(key).unwrap();
        unsafe {
            apr_sys::apr_table_merge(self.ptr, key.as_ptr(), value.into_stored_cstr());
        }
    }

    /// Merge a value without copying the strings
    pub fn mergen(&mut self, key: &str, value: &str) {
        let key = std::ffi::CString::new(key).unwrap();
        let value = std::ffi::CString::new(value).unwrap();
        unsafe {
            apr_sys::apr_table_mergen(self.ptr, key.as_ptr(), value.as_ptr());
        }
    }

    /// Add a key/value pair to the table.
    pub fn add(&mut self, key: &str, value: &str) {
        let key = std::ffi::CString::new(key).unwrap();
        let value = std::ffi::CString::new(value).unwrap();
        unsafe {
            apr_sys::apr_table_add(self.ptr, key.as_ptr(), value.as_ptr());
        }
    }

    /// Overlay one table on top of another.
    pub fn overlay<'newpool>(
        pool: &'newpool Pool,
        overlay: &Table<V>,
        base: &Table<V>,
    ) -> Table<'newpool, V> {
        let new_table =
            unsafe { apr_sys::apr_table_overlay(pool.as_mut_ptr(), overlay.ptr, base.ptr) };
        Table {
            ptr: new_table,
            _phantom: PhantomData,
        }
    }

    /// Get an iterator over the table's key-value pairs.
    ///
    /// This returns owned String values for safety and to avoid lifetime issues.
    pub fn iter(&self) -> TableIterator<V> {
        TableIterator::new(self)
    }

    /// Return a mutable pointer to the underlying apr_table_t
    pub unsafe fn as_mut_ptr(&mut self) -> *mut apr_table_t {
        self.ptr
    }
}

/// Iterator over key-value pairs in an APR table.
///
/// This iterator yields owned (String, String) pairs for safety,
/// as APR table entries may be modified while iterating.
#[derive(Debug)]
pub struct TableIterator<'a, V> {
    table: &'a Table<'a, V>,
    entries: Vec<(String, String)>,
    index: usize,
}

impl<'a, V> TableIterator<'a, V> {
    fn new(table: &'a Table<'a, V>) -> Self {
        let mut entries = Vec::new();

        // Use apr_table_do to iterate over all entries
        extern "C" fn callback(
            rec: *mut std::ffi::c_void,
            key: *const std::ffi::c_char,
            value: *const std::ffi::c_char,
        ) -> std::ffi::c_int {
            let entries = unsafe { &mut *(rec as *mut Vec<(String, String)>) };
            let key = unsafe { std::ffi::CStr::from_ptr(key).to_string_lossy().into_owned() };
            let value = unsafe {
                std::ffi::CStr::from_ptr(value)
                    .to_string_lossy()
                    .into_owned()
            };
            entries.push((key, value));
            1 // Continue iteration
        }

        unsafe {
            apr_sys::apr_table_do(
                Some(callback),
                &mut entries as *mut Vec<(String, String)> as *mut std::ffi::c_void,
                table.as_ptr(),
                std::ptr::null::<std::ffi::c_char>(),
            );
        }

        TableIterator {
            table,
            entries,
            index: 0,
        }
    }
}

impl<'a, V> Iterator for TableIterator<'a, V> {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.entries.len() {
            let item = self.entries[self.index].clone();
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

// Implement IntoIterator for Table
impl<'pool, V> IntoIterator for &'pool Table<'pool, V> {
    type Item = (String, String);
    type IntoIter = TableIterator<'pool, V>;

    fn into_iter(self) -> Self::IntoIter {
        TableIterator::new(self)
    }
}

// Add FromIterator-like functionality for Table
// Additional methods that work with the IntoStoredCStr constraint
impl<'pool, V: IntoStoredCStr> Table<'pool, V> {
    /// Insert or update a key-value pair (more HashMap-like).
    pub fn insert(&mut self, key: &str, value: V) {
        self.set(key, value);
    }

    /// Insert a key-value pair and return self for chaining.
    pub fn with_insert(mut self, key: &str, value: V) -> Self {
        self.insert(key, value);
        self
    }

    /// Remove a key and return true if it existed.
    pub fn remove(&mut self, key: &str) -> bool {
        let existed = self.contains_key(key);
        self.unset(key);
        existed
    }

    /// Remove a key and return self for chaining.
    pub fn with_remove(mut self, key: &str) -> Self {
        self.remove(key);
        self
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ints() {
        let pool = crate::pool::Pool::new();
        let mut array = super::ArrayHeader::new(&pool);
        array.push(1);
        array.push(2);
        array.push(3);
        array.push(4);

        assert_eq!(std::mem::size_of::<i32>(), 4);
        assert_eq!(array.element_size(), 4);

        assert_eq!(array.len(), 4);

        assert_eq!(array.iter().collect::<Vec<_>>(), vec![1, 2, 3, 4]);

        assert_eq!(array.nth(1), Some(2));

        assert_eq!(array.nth(2), Some(3));
        assert_eq!(array.nth(10), None);
    }

    #[test]
    fn test_strings() {
        let pool = crate::pool::Pool::new();
        let mut array = super::ArrayHeader::new(&pool);
        array.push("1");
        array.push("2");
        array.push("3");
        array.push("4");

        assert_eq!(array.element_size(), 16);

        assert_eq!(array.len(), 4);

        assert_eq!(array.iter().collect::<Vec<_>>(), vec!["1", "2", "3", "4"]);

        assert_eq!(array.nth(1), Some("2"));
    }

    #[test]
    fn test_convert() {
        let pool = crate::pool::Pool::new();
        let mut array = super::ArrayHeader::new(&pool);
        array.push("1");
        array.push("2");
        array.push("3");
        array.push("4");

        assert_eq!(array.iter().collect::<Vec<_>>(), vec!["1", "2", "3", "4"]);
    }

    #[test]
    fn test_table_iterator() {
        let pool = crate::pool::Pool::new();
        let mut table = super::Table::new_with_capacity(&pool, 5);

        let value1 = std::ffi::CString::new("value1").unwrap();
        let value2 = std::ffi::CString::new("value2").unwrap();
        let value3 = std::ffi::CString::new("value3").unwrap();

        table.set("key1", value1.as_c_str());
        table.set("key2", value2.as_c_str());
        table.set("key3", value3.as_c_str());

        let items: Vec<(String, String)> = table.iter().collect();
        assert_eq!(items.len(), 3);

        // Check that all key-value pairs are present (order may vary)
        assert!(items.contains(&("key1".to_string(), "value1".to_string())));
        assert!(items.contains(&("key2".to_string(), "value2".to_string())));
        assert!(items.contains(&("key3".to_string(), "value3".to_string())));
    }

    #[test]
    fn test_empty_table_iterator() {
        let pool = crate::pool::Pool::new();
        let table: super::Table<&std::ffi::CStr> = super::Table::new_with_capacity(&pool, 5);

        let items: Vec<(String, String)> = table.iter().collect();
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_into_iterator() {
        let pool = crate::pool::Pool::new();
        let v1 = std::ffi::CString::new("1").unwrap();
        let v2 = std::ffi::CString::new("2").unwrap();
        let mut table: super::Table<&std::ffi::CStr> = super::Table::new_with_capacity(&pool, 3);
        table.set("a", v1.as_c_str());
        table.set("b", v2.as_c_str());

        let mut items: Vec<_> = (&table).into_iter().collect();
        items.sort_by_key(|(k, _)| k.clone());

        assert_eq!(
            items,
            vec![
                ("a".to_string(), "1".to_string()),
                ("b".to_string(), "2".to_string())
            ]
        );
    }

    #[test]
    fn test_array_into_iterator() {
        let pool = crate::pool::Pool::new();
        let mut array = super::ArrayHeader::new(&pool);
        array.push(1);
        array.push(2);
        array.push(3);

        let items: Vec<_> = (&array).into_iter().collect();
        assert_eq!(items, vec![1, 2, 3]);
    }

    #[test]
    fn test_array_from_iter() {
        let pool = crate::pool::Pool::new();
        let data = vec![1, 2, 3, 4];
        let array = super::ArrayHeader::from_iter(&pool, data);

        assert_eq!(array.len(), 4);
        assert_eq!(array.nth(0), Some(1));
        assert_eq!(array.nth(3), Some(4));
    }

    // Note: test_table_from_iter removed as Table doesn't have from_iter method

    #[test]
    fn test_array_extend_and_accessors() {
        let pool = crate::pool::Pool::new();
        let mut array = super::ArrayHeader::new(&pool);

        array.extend(vec![1, 2, 3]);
        assert_eq!(array.len(), 3);
        assert_eq!(array.first(), Some(1));
        assert_eq!(array.last(), Some(3));

        let array2 = super::ArrayHeader::with_items(&pool, vec![10, 20, 30]);
        assert_eq!(array2.len(), 3);
        assert_eq!(array2.nth(1), Some(20));
    }

    #[test]
    fn test_table_hashmap_like_api() {
        let pool = crate::pool::Pool::new();
        let mut table: super::Table<&std::ffi::CStr> = super::Table::new(&pool);

        let value1 = std::ffi::CString::new("value1").unwrap();
        table.insert("key1", value1.as_c_str());
        assert!(table.contains_key("key1"));
        assert!(!table.contains_key("nonexistent"));

        assert!(table.remove("key1"));
        assert!(!table.remove("key1")); // Second remove returns false
        assert!(!table.contains_key("key1"));
    }

    #[test]
    fn test_fluent_apis() {
        let pool = crate::pool::Pool::new();

        // Array fluent API
        let array = super::ArrayHeader::new(&pool)
            .with_push(1)
            .with_push(2)
            .with_extend(vec![3, 4, 5]);
        assert_eq!(array.len(), 5);
        assert_eq!(array.last(), Some(5));

        // Table fluent API
        let value1 = std::ffi::CString::new("value1").unwrap();
        let value2 = std::ffi::CString::new("value2").unwrap();
        let table: super::Table<&std::ffi::CStr> = super::Table::new(&pool)
            .with_insert("key1", value1.as_c_str())
            .with_insert("key2", value2.as_c_str())
            .with_remove("key2");
        assert!(table.contains_key("key1"));
        assert!(!table.contains_key("key2"));
    }

    #[test]
    fn test_advanced_iterators() {
        let pool = crate::pool::Pool::new();
        let array = super::ArrayHeader::with_items(&pool, vec![10, 20, 30]);

        // Test enumerated iterator
        let enumerated: Vec<_> = array.iter().enumerate().collect();
        assert_eq!(enumerated, vec![(0, 10), (1, 20), (2, 30)]);

        // Test indices iterator
        let indices: Vec<_> = array.indices().collect();
        assert_eq!(indices, vec![0, 1, 2]);
    }
}
