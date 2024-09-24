pub use crate::generated::{apr_array_header_t, apr_table_t};
use crate::pool::PooledPtr;

/// A wrapper around an `apr_array_header_t`.
pub struct ArrayHeader<T: Sized>(PooledPtr<apr_array_header_t>, std::marker::PhantomData<T>);

impl<T: Sized + Copy> ArrayHeader<T> {
    /// Returns true if the array is empty.
    pub fn is_empty(&self) -> bool {
        unsafe { crate::generated::apr_is_empty_array(&*self.0) != 0 }
    }

    /// Returns the number of elements in the array.
    pub fn len(&self) -> usize {
        self.0.nelts as usize
    }

    /// Returns the number of elements that can be stored in the array without reallocating.
    pub fn allocated(&self) -> usize {
        self.0.nalloc as usize
    }

    /// Create an empty ArrayHeader.
    pub fn new() -> Self {
        Self::new_with_capacity(0)
    }

    pub fn new_with_capacity(nelts: usize) -> Self {
        Self(
            crate::pool::PooledPtr::initialize(|pool| unsafe {
                Ok::<_, crate::Status>(crate::generated::apr_array_make(
                    pool.as_mut_ptr(),
                    nelts as i32,
                    std::mem::size_of::<T>() as i32,
                ))
            })
            .unwrap(),
            std::marker::PhantomData,
        )
    }

    /// Create an ArrayHeader from a pool and a number of elements.
    pub fn in_pool(pool: &std::rc::Rc<crate::Pool>, nelts: usize) -> Self {
        unsafe {
            let mut pool = pool.clone();
            let hdr = crate::generated::apr_array_make(
                std::rc::Rc::get_mut(&mut pool).unwrap().as_mut_ptr(),
                nelts as i32,
                std::mem::size_of::<T>() as i32,
            );

            Self(
                crate::pool::PooledPtr::in_pool(pool, hdr),
                std::marker::PhantomData,
            )
        }
    }

    /// Create an ArrayHeader from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the pointer is valid and that the memory is owned by the pool.
    pub unsafe fn from_raw_parts(
        pool: &std::rc::Rc<crate::Pool>,
        raw: *mut apr_array_header_t,
    ) -> Self {
        Self(
            crate::pool::PooledPtr::in_pool(pool.clone(), raw),
            std::marker::PhantomData,
        )
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
        unsafe { self.0.elts.add(index * self.0.elt_size as usize) as *mut T }
    }

    /// Return the size of each element in the array.
    pub fn element_size(&self) -> usize {
        self.0.elt_size as usize
    }

    /// Push an element onto the end of the array.
    pub fn push(&mut self, item: T) {
        unsafe {
            let ptr = crate::generated::apr_array_push(&mut *self.0);
            // copy item to the memory at ptr
            std::ptr::copy_nonoverlapping(&item, ptr as *mut T, 1);
        }
    }

    /// Clear the array
    pub fn clear(&mut self) {
        unsafe {
            crate::generated::apr_array_clear(&mut *self.0);
        }
    }

    /// Concatenate two arrays.
    pub fn cat(&mut self, other: &ArrayHeader<T>) {
        unsafe {
            crate::generated::apr_array_cat(&mut *self.0, &*other.0);
        }
    }

    /// Append two arrays.
    pub fn append(first: &ArrayHeader<T>, second: &ArrayHeader<T>) -> Self {
        unsafe {
            Self(
                PooledPtr::initialize(|pool| {
                    Ok::<_, crate::Status>(crate::generated::apr_array_append(
                        pool.as_mut_ptr(),
                        &*first.0,
                        &*second.0,
                    ))
                })
                .unwrap(),
                std::marker::PhantomData,
            )
        }
    }

    /// Copy the array.
    pub fn copy(&self) -> Self {
        unsafe {
            Self(
                PooledPtr::initialize(|pool| {
                    Ok::<_, crate::Status>(crate::generated::apr_array_copy(
                        pool.as_mut_ptr(),
                        &*self.0,
                    ))
                })
                .unwrap(),
                std::marker::PhantomData,
            )
        }
    }

    /// Iterate over the entries in an array
    pub fn iter(&self) -> ArrayHeaderIterator<T> {
        ArrayHeaderIterator::new(self)
    }

    /// Return a pointer to the underlying `apr_array_header_t`.
    pub fn as_ptr(&self) -> *const crate::generated::apr_array_header_t {
        self.0.as_ref()
    }
}

impl<T: Sized + Copy> Default for ArrayHeader<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Sized + Copy> std::ops::Index<usize> for ArrayHeader<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.nth_unchecked(index) }
    }
}

/// An iterator over the elements of an `ArrayHeader`.
pub struct ArrayHeaderIterator<'a, T: Sized> {
    array: &'a ArrayHeader<T>,
    index: usize,
}

impl<'a, T: Sized> ArrayHeaderIterator<'a, T> {
    fn new(array: &'a ArrayHeader<T>) -> Self {
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

impl<T: Sized + Copy> FromIterator<T> for ArrayHeader<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut array = ArrayHeader::new();
        for item in iter {
            array.push(item);
        }
        array
    }
}

pub struct Table<'pool>(PooledPtr<apr_table_t>, std::marker::PhantomData<&'pool ()>);

impl<'pool> Clone for Table<'pool> {
    fn clone(&self) -> Self {
        unsafe {
            Self(
                PooledPtr::initialize(|pool| {
                    Ok::<_, crate::Status>(crate::generated::apr_table_copy(
                        pool.as_mut_ptr(),
                        &*self.0,
                    ))
                })
                .unwrap(),
                std::marker::PhantomData,
            )
        }
    }
}

impl<'pool> Table<'pool> {
    /// Check if the table is empty.
    pub fn is_empty(&self) -> bool {
        unsafe { crate::generated::apr_is_empty_table(&*self.0) != 0 }
    }

    /// Clear the table.
    pub fn clear(&mut self) {
        unsafe {
            crate::generated::apr_table_clear(&mut *self.0);
        }
    }

    /// Create a new table, with space for nelts entries.
    pub fn new_with_capacity(nelts: usize) -> Self {
        unsafe {
            Self(
                PooledPtr::initialize(|pool| {
                    Ok::<_, crate::Status>(crate::generated::apr_table_make(
                        pool.as_mut_ptr(),
                        nelts as i32,
                    ))
                })
                .unwrap(),
                std::marker::PhantomData,
            )
        }
    }

    /// Return the item with the given key.
    pub fn get(&self, key: &str) -> Option<&str> {
        let key = std::ffi::CString::new(key).unwrap();
        unsafe {
            let value = crate::generated::apr_table_get(&*self.0, key.as_ptr());
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
            let value = crate::generated::apr_table_getm(pool.as_mut_ptr(), &*self.0, key.as_ptr());
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
            crate::generated::apr_table_set(&mut *self.0, key.as_ptr(), value.as_ptr());
        }
    }

    pub fn setn(&mut self, key: &str, value: &str) {
        let key = std::ffi::CString::new(key).unwrap();
        let value = std::ffi::CString::new(value).unwrap();
        unsafe {
            crate::generated::apr_table_setn(&mut *self.0, key.as_ptr(), value.as_ptr());
        }
    }

    pub fn unset(&mut self, key: &str) {
        let key = std::ffi::CString::new(key).unwrap();
        unsafe {
            crate::generated::apr_table_unset(&mut *self.0, key.as_ptr());
        }
    }

    pub fn merge(&mut self, key: &str, value: &str) {
        let key = std::ffi::CString::new(key).unwrap();
        let value = std::ffi::CString::new(value).unwrap();
        unsafe {
            crate::generated::apr_table_merge(&mut *self.0, key.as_ptr(), value.as_ptr());
        }
    }

    pub fn mergen(&mut self, key: &str, value: &str) {
        let key = std::ffi::CString::new(key).unwrap();
        let value = std::ffi::CString::new(value).unwrap();
        unsafe {
            crate::generated::apr_table_mergen(&mut *self.0, key.as_ptr(), value.as_ptr());
        }
    }

    /// Add a key/value pair to the table.
    pub fn add(&mut self, key: &str, value: &str) {
        let key = std::ffi::CString::new(key).unwrap();
        let value = std::ffi::CString::new(value).unwrap();
        unsafe {
            crate::generated::apr_table_add(&mut *self.0, key.as_ptr(), value.as_ptr());
        }
    }

    /// Overlay one table on top of another.
    pub fn overlay(overlay: &Table, base: &Table) -> Self {
        unsafe {
            Self(
                PooledPtr::initialize(|pool| {
                    Ok::<_, crate::Status>(crate::generated::apr_table_overlay(
                        pool.as_mut_ptr(),
                        &*overlay.0,
                        &*base.0,
                    ))
                })
                .unwrap(),
                std::marker::PhantomData,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ints() {
        let mut array = super::ArrayHeader::new();
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
        let mut array = super::ArrayHeader::new();
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
        let mut array = super::ArrayHeader::new();
        array.push("1");
        array.push("2");
        array.push("3");
        array.push("4");

        assert_eq!(array.iter().collect::<Vec<_>>(), vec!["1", "2", "3", "4"]);
    }
}
