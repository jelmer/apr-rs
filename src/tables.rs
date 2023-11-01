pub use crate::generated::{apr_array_header_t, apr_table_t};

pub struct ArrayHeader<'pool, T: Sized>(
    *mut apr_array_header_t,
    std::marker::PhantomData<&'pool ()>,
    std::marker::PhantomData<T>,
);

impl<'pool, T: Sized> ArrayHeader<'pool, T> {
    pub fn is_empty(&self) -> bool {
        unsafe { crate::generated::apr_is_empty_array(self.0) != 0 }
    }

    pub fn len(&self) -> usize {
        unsafe { (*self.0).nelts as usize }
    }

    pub fn allocated(&self) -> usize {
        unsafe { (*self.0).nalloc as usize }
    }

    pub fn new(pool: &'pool mut crate::Pool, nelts: usize) -> Self {
        unsafe {
            Self(
                crate::generated::apr_array_make(
                    pool.into(),
                    nelts as i32,
                    std::mem::size_of::<T>() as i32,
                ),
                std::marker::PhantomData,
                std::marker::PhantomData,
            )
        }
    }

    pub fn nth(&self, index: usize) -> T {
        unsafe {
            let ptr = (*self.0).elts as *mut *mut std::ffi::c_void;
            std::mem::transmute_copy(&*ptr.add(index))
        }
    }

    pub fn elt_size(&self) -> usize {
        unsafe { (*self.0).elt_size as usize }
    }

    pub fn push(&mut self, item: T) {
        unsafe {
            let ptr = crate::generated::apr_array_push(self.0);
            // copy item to the memory at ptr
            std::ptr::copy_nonoverlapping(&item, ptr as *mut T, 1);
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            crate::generated::apr_array_clear(self.0);
        }
    }

    pub fn cat(&mut self, other: &ArrayHeader<T>) {
        unsafe {
            crate::generated::apr_array_cat(self.0, other.0);
        }
    }

    pub fn append(
        pool: &'pool mut crate::Pool,
        first: &ArrayHeader<T>,
        second: &ArrayHeader<T>,
    ) -> Self {
        unsafe {
            Self(
                crate::generated::apr_array_append(pool.into(), first.0, second.0),
                std::marker::PhantomData,
                std::marker::PhantomData,
            )
        }
    }

    pub fn copy(&self, pool: &mut crate::Pool) -> Self {
        unsafe {
            Self(
                crate::generated::apr_array_copy(pool.into(), self.0),
                std::marker::PhantomData,
                std::marker::PhantomData,
            )
        }
    }

    pub fn iter(&self) -> ArrayHeaderIterator<T> {
        ArrayHeaderIterator::new(self)
    }
}

impl<'pool, T: Sized> From<ArrayHeader<'pool, T>> for *mut apr_array_header_t {
    fn from(array: ArrayHeader<T>) -> Self {
        array.0
    }
}

impl<'pool, T: Sized> From<ArrayHeader<'pool, T>> for *const apr_array_header_t {
    fn from(array: ArrayHeader<T>) -> Self {
        array.0
    }
}

impl<'pool, T: Sized> From<*mut apr_array_header_t> for ArrayHeader<'pool, T> {
    fn from(array: *mut apr_array_header_t) -> Self {
        Self(array, std::marker::PhantomData, std::marker::PhantomData)
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

impl<'a, T: Sized> Iterator for ArrayHeaderIterator<'a, T> {
    type Item = T;

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

#[cfg(test)]
mod tests {
    #[test]
    fn test_array_header() {
        let mut pool = crate::Pool::new();
        let mut array = super::ArrayHeader::new(&mut pool, 3);
        array.push(std::ffi::CString::new("test1").unwrap().as_ptr());
        array.push(std::ffi::CString::new("test2").unwrap().as_ptr());
        array.push(std::ffi::CString::new("test3").unwrap().as_ptr());
    }
}
