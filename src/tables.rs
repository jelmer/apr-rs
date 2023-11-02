pub use crate::generated::{apr_array_header_t, apr_table_t};
use crate::pool::PooledPtr;

pub struct ArrayHeader<'pool, T: Sized>(
    PooledPtr<'pool, apr_array_header_t>,
    std::marker::PhantomData<T>,
);

impl<'pool, T: Sized + Copy> ArrayHeader<'pool, T> {
    pub fn is_empty(&self) -> bool {
        unsafe { crate::generated::apr_is_empty_array(&*self.0) != 0 }
    }

    pub fn len(&self) -> usize {
        self.0.nelts as usize
    }

    pub fn allocated(&self) -> usize {
        self.0.nalloc as usize
    }

    pub fn new() -> Self {
        Self::new_with_capacity(0)
    }

    pub fn new_with_capacity(nelts: usize) -> Self {
        Self(
            crate::pool::PooledPtr::initialize(|pool| unsafe {
                Ok::<_, crate::Status>(crate::generated::apr_array_make(
                    pool.into(),
                    nelts as i32,
                    std::mem::size_of::<T>() as i32,
                ))
            })
            .unwrap(),
            std::marker::PhantomData,
        )
    }

    pub fn in_pool(pool: &std::rc::Rc<crate::Pool>, nelts: usize) -> Self {
        unsafe {
            let pool = pool.clone();
            let hdr = crate::generated::apr_array_make(
                pool.as_ref().into(),
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

    pub fn nth(&self, index: usize) -> Option<T> {
        if index < self.len() {
            Some(unsafe { *self.nth_unchecked(index) })
        } else {
            None
        }
    }

    unsafe fn nth_unchecked(&self, index: usize) -> *mut T {
        unsafe { self.0.elts.add(index * self.0.elt_size as usize) as *mut T }
    }

    pub fn element_size(&self) -> usize {
        self.0.elt_size as usize
    }

    pub fn push(&mut self, item: T) {
        unsafe {
            let ptr = crate::generated::apr_array_push(&mut *self.0);
            // copy item to the memory at ptr
            std::ptr::copy_nonoverlapping(&item, ptr as *mut T, 1);
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            crate::generated::apr_array_clear(&mut *self.0);
        }
    }

    pub fn cat(&mut self, other: &ArrayHeader<T>) {
        unsafe {
            crate::generated::apr_array_cat(&mut *self.0, &*other.0);
        }
    }

    pub fn append(first: &ArrayHeader<T>, second: &ArrayHeader<T>) -> Self {
        unsafe {
            Self(
                PooledPtr::initialize(|pool| {
                    Ok::<_, crate::Status>(crate::generated::apr_array_append(
                        pool.into(),
                        &*first.0,
                        &*second.0,
                    ))
                })
                .unwrap(),
                std::marker::PhantomData,
            )
        }
    }

    pub fn copy(&self) -> Self {
        unsafe {
            Self(
                PooledPtr::initialize(|pool| {
                    Ok::<_, crate::Status>(crate::generated::apr_array_copy(pool.into(), &*self.0))
                })
                .unwrap(),
                std::marker::PhantomData,
            )
        }
    }

    pub fn iter(&self) -> ArrayHeaderIterator<T> {
        ArrayHeaderIterator::new(self)
    }
}

impl<T: Sized + Copy> Default for ArrayHeader<'_, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'pool, T: Sized> From<ArrayHeader<'pool, T>> for *const apr_array_header_t {
    fn from(array: ArrayHeader<T>) -> Self {
        &*array.0
    }
}

impl<'pool, T: Sized> From<&ArrayHeader<'pool, T>> for *const apr_array_header_t {
    fn from(array: &ArrayHeader<T>) -> Self {
        &*array.0
    }
}

impl<T: Sized + Copy> std::ops::Index<usize> for ArrayHeader<'_, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.nth_unchecked(index) }
    }
}

pub struct ArrayHeaderIterator<'pool, T: Sized> {
    array: &'pool ArrayHeader<'pool, T>,
    index: usize,
}

impl<'a, T: Sized> ArrayHeaderIterator<'a, T> {
    pub fn new(array: &'a ArrayHeader<T>) -> Self {
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

impl<'pool, T: Sized + Copy> FromIterator<T> for ArrayHeader<'pool, T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut array = ArrayHeader::new();
        for item in iter {
            array.push(item);
        }
        array
    }
}

pub struct Table<'pool>(PooledPtr<'pool, apr_table_t>);

impl<'pool> Clone for Table<'pool> {
    fn clone(&self) -> Self {
        unsafe {
            Self(
                PooledPtr::initialize(|pool| {
                    Ok::<_, crate::Status>(crate::generated::apr_table_copy(pool.into(), &*self.0))
                })
                .unwrap(),
            )
        }
    }
}

impl<'pool> Table<'pool> {
    pub fn is_empty(&self) -> bool {
        unsafe { crate::generated::apr_is_empty_table(&*self.0) != 0 }
    }

    pub fn clear(&mut self) {
        unsafe {
            crate::generated::apr_table_clear(&mut *self.0);
        }
    }

    pub fn new_with_capacity(nelts: usize) -> Self {
        unsafe {
            Self(
                PooledPtr::initialize(|pool| {
                    Ok::<_, crate::Status>(crate::generated::apr_table_make(
                        pool.into(),
                        nelts as i32,
                    ))
                })
                .unwrap(),
            )
        }
    }

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
            let value = crate::generated::apr_table_getm(pool.into(), &*self.0, key.as_ptr());
            if value.is_null() {
                None
            } else {
                Some(std::ffi::CStr::from_ptr(value).to_str().unwrap())
            }
        }
    }

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

    pub fn add(&mut self, key: &str, value: &str) {
        let key = std::ffi::CString::new(key).unwrap();
        let value = std::ffi::CString::new(value).unwrap();
        unsafe {
            crate::generated::apr_table_add(&mut *self.0, key.as_ptr(), value.as_ptr());
        }
    }

    pub fn overlay(overlay: &Table, base: &Table) -> Self {
        unsafe {
            Self(
                PooledPtr::initialize(|pool| {
                    Ok::<_, crate::Status>(crate::generated::apr_table_overlay(
                        pool.into(),
                        &*overlay.0,
                        &*base.0,
                    ))
                })
                .unwrap(),
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
}
