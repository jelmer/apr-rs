pub use crate::generated::{apr_array_header_t, apr_table_t};

pub struct ArrayHeader(*mut apr_array_header_t);

impl ArrayHeader {
    pub fn is_empty(&self) -> bool {
        unsafe { crate::generated::apr_is_empty_array(self.0) != 0 }
    }

    pub fn len(&self) -> usize {
        unsafe { (*self.0).nelts as usize }
    }

    pub fn allocated(&self) -> usize {
        unsafe { (*self.0).nalloc as usize }
    }

    pub fn make(pool: &mut crate::Pool, nelts: i32, elt_size: i32) -> Self {
        unsafe {
            Self(crate::generated::apr_array_make(
                pool.into(),
                nelts,
                elt_size,
            ))
        }
    }

    pub fn nth(&self, index: usize) -> *mut std::ffi::c_void {
        unsafe {
            let ptr = (*self.0).elts as *mut *mut std::ffi::c_void;
            *ptr.add(index)
        }
    }

    pub fn elt_size(&self) -> usize {
        unsafe { (*self.0).elt_size as usize }
    }

    pub fn push(&mut self, item: *mut std::ffi::c_void) {
        unsafe {
            crate::generated::apr_array_push(self.0);
            let ptr = (*self.0).elts as *mut *mut std::ffi::c_void;
            *ptr.add((*self.0).nelts as usize - 1) = item;
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            crate::generated::apr_array_clear(self.0);
        }
    }

    pub fn cat(&mut self, other: &ArrayHeader) {
        unsafe {
            crate::generated::apr_array_cat(self.0, other.0);
        }
    }

    pub fn append(pool: &mut crate::Pool, first: &ArrayHeader, second: &ArrayHeader) -> Self {
        unsafe {
            Self(crate::generated::apr_array_append(
                pool.into(),
                first.0,
                second.0,
            ))
        }
    }

    pub fn copy(&self, pool: &mut crate::Pool) -> Self {
        unsafe { Self(crate::generated::apr_array_copy(pool.into(), self.0)) }
    }

    pub fn iter(&self) -> ArrayHeaderIterator {
        ArrayHeaderIterator::new(self)
    }
}

impl From<ArrayHeader> for *mut apr_array_header_t {
    fn from(array: ArrayHeader) -> Self {
        array.0
    }
}

impl From<ArrayHeader> for *const apr_array_header_t {
    fn from(array: ArrayHeader) -> Self {
        array.0
    }
}

impl From<*mut apr_array_header_t> for ArrayHeader {
    fn from(array: *mut apr_array_header_t) -> Self {
        Self(array)
    }
}

pub struct ArrayHeaderIterator<'a> {
    array: &'a ArrayHeader,
    index: usize,
}

impl<'a> ArrayHeaderIterator<'a> {
    pub fn new(array: &'a ArrayHeader) -> Self {
        Self { array, index: 0 }
    }
}

impl<'a> Iterator for ArrayHeaderIterator<'a> {
    type Item = *mut std::ffi::c_void;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.array.len() {
            let item = self.array.nth(self.index);
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

pub struct Table(*mut apr_table_t);

impl Table {
    pub fn is_empty(&self) -> bool {
        unsafe { crate::generated::apr_is_empty_table(self.0) != 0 }
    }

    pub fn copy(&self, pool: &mut crate::Pool) -> Self {
        unsafe { Self(crate::generated::apr_table_copy(pool.into(), self.0)) }
    }

    pub fn clear(&mut self) {
        unsafe {
            crate::generated::apr_table_clear(self.0);
        }
    }

    pub fn make(pool: &mut crate::Pool, nelts: usize) -> Self {
        unsafe { Self(crate::generated::apr_table_make(pool.into(), nelts as i32)) }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        let key = std::ffi::CString::new(key).unwrap();
        unsafe {
            let value = crate::generated::apr_table_get(self.0, key.as_ptr());
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
            let value = crate::generated::apr_table_getm(pool.into(), self.0, key.as_ptr());
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
            crate::generated::apr_table_set(self.0, key.as_ptr(), value.as_ptr());
        }
    }

    pub fn setn(&mut self, key: &str, value: &str) {
        let key = std::ffi::CString::new(key).unwrap();
        let value = std::ffi::CString::new(value).unwrap();
        unsafe {
            crate::generated::apr_table_setn(self.0, key.as_ptr(), value.as_ptr());
        }
    }

    pub fn unset(&mut self, key: &str) {
        let key = std::ffi::CString::new(key).unwrap();
        unsafe {
            crate::generated::apr_table_unset(self.0, key.as_ptr());
        }
    }

    pub fn merge(&mut self, key: &str, value: &str) {
        let key = std::ffi::CString::new(key).unwrap();
        let value = std::ffi::CString::new(value).unwrap();
        unsafe {
            crate::generated::apr_table_merge(self.0, key.as_ptr(), value.as_ptr());
        }
    }

    pub fn mergen(&mut self, key: &str, value: &str) {
        let key = std::ffi::CString::new(key).unwrap();
        let value = std::ffi::CString::new(value).unwrap();
        unsafe {
            crate::generated::apr_table_mergen(self.0, key.as_ptr(), value.as_ptr());
        }
    }

    pub fn add(&mut self, key: &str, value: &str) {
        let key = std::ffi::CString::new(key).unwrap();
        let value = std::ffi::CString::new(value).unwrap();
        unsafe {
            crate::generated::apr_table_add(self.0, key.as_ptr(), value.as_ptr());
        }
    }

    pub fn overlay(pool: &mut crate::Pool, overlay: &Table, base: &Table) -> Self {
        unsafe {
            Self(crate::generated::apr_table_overlay(
                pool.into(),
                overlay.0,
                base.0,
            ))
        }
    }
}
