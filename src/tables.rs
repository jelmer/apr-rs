use crate::pool::Pool;
pub use apr_sys::{apr_array_header_t, apr_table_t};
use std::marker::PhantomData;

/// A wrapper around an `apr_array_header_t`.
#[derive(Debug)]
pub struct ArrayHeader<'pool, T: Sized> {
    ptr: *mut apr_array_header_t,
    _phantom: PhantomData<(T, &'pool Pool)>,
}

impl<'pool, T: Sized + Copy> ArrayHeader<'pool, T> {
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
    pub fn new(pool: &'pool Pool) -> Self {
        Self::new_with_capacity(pool, 0)
    }

    /// Create a new array with a given capacity
    pub fn new_with_capacity(pool: &'pool Pool, nelts: usize) -> Self {
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

    /// Builder pattern: create array, reserve capacity, and populate
    pub fn with_items<I: IntoIterator<Item = T>>(pool: &'pool Pool, items: I) -> Self {
        let items: Vec<T> = items.into_iter().collect();
        let mut array = Self::new_with_capacity(pool, items.len());
        for item in items {
            array.push(item);
        }
        array
    }

    /// Create an array from a raw APR array pointer
    pub fn from_ptr(ptr: *mut apr_array_header_t) -> Self {
        Self {
            ptr,
            _phantom: PhantomData,
        }
    }

    /// Return the element at the given index.
    pub fn nth(&self, index: usize) -> Option<T> {
        if index < self.len() {
            Some(unsafe { *self.nth_unchecked(index) })
        } else {
            None
        }
    }

    /// Return a pointer to the element at the given index.
    unsafe fn nth_unchecked(&self, index: usize) -> *mut T {
        unsafe { (*self.ptr).elts.add(index * (*self.ptr).elt_size as usize) as *mut T }
    }

    /// Return the size of each element in the array.
    pub fn element_size(&self) -> usize {
        unsafe { (*self.ptr).elt_size as usize }
    }

    /// Push an element onto the end of the array.
    pub fn push(&mut self, item: T) {
        unsafe {
            let ptr = apr_sys::apr_array_push(self.ptr);
            // copy item to the memory at ptr
            std::ptr::copy_nonoverlapping(&item, ptr as *mut T, 1);
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

    /// Clear the array
    pub fn clear(&mut self) {
        unsafe {
            apr_sys::apr_array_clear(self.ptr);
        }
    }

    /// Concatenate two arrays.
    pub fn cat(&mut self, other: &ArrayHeader<T>) {
        unsafe {
            apr_sys::apr_array_cat(self.ptr, other.ptr);
        }
    }

    /// Append two arrays.
    pub fn append<'newpool>(
        pool: &'newpool Pool,
        first: &ArrayHeader<T>,
        second: &ArrayHeader<T>,
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

    /// Iterate over the entries in an array
    pub fn iter(&self) -> ArrayHeaderIterator<T> {
        ArrayHeaderIterator::new(self)
    }

    /// Create an enumerating iterator over the array (index, value).
    pub fn iter_enumerated(&self) -> impl Iterator<Item = (usize, T)> + '_ {
        (0..self.len()).map(move |i| (i, self.nth(i).unwrap()))
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

impl<'pool, T: Sized + Copy> std::ops::Index<usize> for ArrayHeader<'pool, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.nth_unchecked(index) }
    }
}

/// An iterator over the elements of an `ArrayHeader`.
#[derive(Debug)]
pub struct ArrayHeaderIterator<'pool, T: Sized> {
    array: &'pool ArrayHeader<'pool, T>,
    index: usize,
}

impl<'pool, T: Sized> ArrayHeaderIterator<'pool, T> {
    fn new(array: &'pool ArrayHeader<T>) -> Self {
        Self { array, index: 0 }
    }
}

impl<'a, T: Sized + Copy> Iterator for ArrayHeaderIterator<'a, T> {
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
impl<'pool, T: Sized + Copy> IntoIterator for &'pool ArrayHeader<'pool, T> {
    type Item = T;
    type IntoIter = ArrayHeaderIterator<'pool, T>;

    fn into_iter(self) -> Self::IntoIter {
        ArrayHeaderIterator::new(self)
    }
}

// Implement FromIterator for ArrayHeader
impl<'pool, T: Sized + Copy> ArrayHeader<'pool, T> {
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
pub struct Table<'pool> {
    ptr: *mut apr_table_t,
    _phantom: PhantomData<&'pool Pool>,
}

impl<'pool> Table<'pool> {
    /// Check if the table is empty.
    pub fn is_empty(&self) -> bool {
        unsafe { apr_sys::apr_is_empty_table(self.ptr) != 0 }
    }

    /// Copy the table to a new pool
    pub fn copy<'newpool>(&self, pool: &'newpool Pool) -> Table<'newpool> {
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

    /// Return the item with the given key.
    pub fn get(&self, key: &str) -> Option<&str> {
        let key = std::ffi::CString::new(key).unwrap();
        unsafe {
            let value = apr_sys::apr_table_get(self.ptr, key.as_ptr());
            if value.is_null() {
                None
            } else {
                Some(std::ffi::CStr::from_ptr(value).to_str().unwrap())
            }
        }
    }

    /// Get a value from the table using a memory pool for allocation
    pub fn getm(&self, pool: &mut crate::Pool, key: &str) -> Option<&str> {
        let key = std::ffi::CString::new(key).unwrap();
        unsafe {
            let value = apr_sys::apr_table_getm(pool.as_mut_ptr(), self.ptr, key.as_ptr());
            if value.is_null() {
                None
            } else {
                Some(std::ffi::CStr::from_ptr(value).to_str().unwrap())
            }
        }
    }

    /// Set the value of a key.
    pub fn set(&mut self, key: &str, value: &str) {
        let key = std::ffi::CString::new(key).unwrap();
        let value = std::ffi::CString::new(value).unwrap();
        unsafe {
            apr_sys::apr_table_set(self.ptr, key.as_ptr(), value.as_ptr());
        }
    }

    /// Set a value in the table without copying the strings
    pub fn setn(&mut self, key: &str, value: &str) {
        let key = std::ffi::CString::new(key).unwrap();
        let value = std::ffi::CString::new(value).unwrap();
        unsafe {
            apr_sys::apr_table_setn(self.ptr, key.as_ptr(), value.as_ptr());
        }
    }

    /// Remove all entries with the given key
    pub fn unset(&mut self, key: &str) {
        let key = std::ffi::CString::new(key).unwrap();
        unsafe {
            apr_sys::apr_table_unset(self.ptr, key.as_ptr());
        }
    }

    /// Merge a value with existing values for the key
    pub fn merge(&mut self, key: &str, value: &str) {
        let key = std::ffi::CString::new(key).unwrap();
        let value = std::ffi::CString::new(value).unwrap();
        unsafe {
            apr_sys::apr_table_merge(self.ptr, key.as_ptr(), value.as_ptr());
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
        overlay: &Table,
        base: &Table,
    ) -> Table<'newpool> {
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
    pub fn iter(&self) -> TableIterator {
        TableIterator::new(self)
    }

    /// Return a pointer to the underlying apr_table_t.
    pub fn as_ptr(&self) -> *const apr_table_t {
        self.ptr
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
pub struct TableIterator<'a> {
    table: &'a Table<'a>,
    entries: Vec<(String, String)>,
    index: usize,
}

impl<'a> TableIterator<'a> {
    fn new(table: &'a Table<'a>) -> Self {
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

impl<'a> Iterator for TableIterator<'a> {
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
impl<'pool> IntoIterator for &'pool Table<'pool> {
    type Item = (String, String);
    type IntoIter = TableIterator<'pool>;

    fn into_iter(self) -> Self::IntoIter {
        TableIterator::new(self)
    }
}

// Add FromIterator-like functionality for Table
impl<'pool> Table<'pool> {
    /// Create a Table from an iterator of key-value pairs.
    pub fn from_iter<I: IntoIterator<Item = (String, String)>>(pool: &'pool Pool, iter: I) -> Self {
        let mut table = Self::new_with_capacity(pool, 0);
        for (key, value) in iter {
            table.set(&key, &value);
        }
        table
    }

    /// Create a new empty table (convenience method).
    pub fn new(pool: &'pool Pool) -> Self {
        Self::new_with_capacity(pool, 0)
    }

    /// Insert or update a key-value pair (more HashMap-like).
    pub fn insert(&mut self, key: &str, value: &str) {
        self.set(key, value);
    }

    /// Insert a key-value pair and return self for chaining.
    pub fn with_insert(mut self, key: &str, value: &str) -> Self {
        self.insert(key, value);
        self
    }

    /// Check if the table contains a key.
    pub fn contains_key(&self, key: &str) -> bool {
        self.get(key).is_some()
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

        assert_eq!(array[1], 2);

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

        assert_eq!(array[1], "2");
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

        table.set("key1", "value1");
        table.set("key2", "value2");
        table.set("key3", "value3");

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
        let table = super::Table::new_with_capacity(&pool, 5);

        let items: Vec<(String, String)> = table.iter().collect();
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_into_iterator() {
        let pool = crate::pool::Pool::new();
        let mut table = super::Table::new_with_capacity(&pool, 3);
        table.set("a", "1");
        table.set("b", "2");

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
        assert_eq!(array[0], 1);
        assert_eq!(array[3], 4);
    }

    #[test]
    fn test_table_from_iter() {
        let pool = crate::pool::Pool::new();
        let data = vec![
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
        ];
        let table = super::Table::from_iter(&pool, data);

        assert_eq!(table.get("key1"), Some("value1"));
        assert_eq!(table.get("key2"), Some("value2"));
    }

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
        assert_eq!(array2[1], 20);
    }

    #[test]
    fn test_table_hashmap_like_api() {
        let pool = crate::pool::Pool::new();
        let mut table = super::Table::new(&pool);

        table.insert("key1", "value1");
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
        let table = super::Table::new(&pool)
            .with_insert("key1", "value1")
            .with_insert("key2", "value2")
            .with_remove("key2");
        assert!(table.contains_key("key1"));
        assert!(!table.contains_key("key2"));
    }

    #[test]
    fn test_advanced_iterators() {
        let pool = crate::pool::Pool::new();
        let array = super::ArrayHeader::with_items(&pool, vec![10, 20, 30]);

        // Test enumerated iterator
        let enumerated: Vec<_> = array.iter_enumerated().collect();
        assert_eq!(enumerated, vec![(0, 10), (1, 20), (2, 30)]);

        // Test indices iterator
        let indices: Vec<_> = array.indices().collect();
        assert_eq!(indices, vec![0, 1, 2]);
    }
}
