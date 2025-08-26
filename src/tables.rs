pub use apr_sys::{apr_array_header_t, apr_table_t};
use crate::pool::Pool;
use std::marker::PhantomData;

/// A wrapper around an `apr_array_header_t`.
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
            ptr: unsafe {
                apr_sys::apr_array_append(pool.as_mut_ptr(), first.ptr, second.ptr)
            },
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

    /// Return a pointer to the underlying `apr_array_header_t`.
    pub fn as_ptr(&self) -> *const apr_sys::apr_array_header_t {
        self.ptr
    }

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

pub struct Table<'pool> {
    ptr: *mut apr_table_t,
    _phantom: PhantomData<&'pool Pool>,
}

impl<'pool> Table<'pool> {
    /// Check if the table is empty.
    pub fn is_empty(&self) -> bool {
        unsafe { apr_sys::apr_is_empty_table(self.ptr) != 0 }
    }

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

    pub fn setn(&mut self, key: &str, value: &str) {
        let key = std::ffi::CString::new(key).unwrap();
        let value = std::ffi::CString::new(value).unwrap();
        unsafe {
            apr_sys::apr_table_setn(self.ptr, key.as_ptr(), value.as_ptr());
        }
    }

    pub fn unset(&mut self, key: &str) {
        let key = std::ffi::CString::new(key).unwrap();
        unsafe {
            apr_sys::apr_table_unset(self.ptr, key.as_ptr());
        }
    }

    pub fn merge(&mut self, key: &str, value: &str) {
        let key = std::ffi::CString::new(key).unwrap();
        let value = std::ffi::CString::new(value).unwrap();
        unsafe {
            apr_sys::apr_table_merge(self.ptr, key.as_ptr(), value.as_ptr());
        }
    }

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
}
