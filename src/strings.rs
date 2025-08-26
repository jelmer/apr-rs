//! String utilities for safe C string handling.
use crate::pool::Pool;
use std::ffi::{c_char, CStr, CString};
use std::marker::PhantomData;

/// Borrowed byte string backed by pool memory
///
/// This represents bytes from C strings, potentially containing non-UTF-8 data.
/// Use this when you need zero-copy access to C string data.
#[derive(Debug)]
pub struct BStr<'a> {
    data: &'a [u8],
    _pool: PhantomData<&'a Pool>,
}

impl<'a> BStr<'a> {
    /// Create a BStr from a C string pointer (unsafe)
    ///
    /// # Safety
    /// - ptr must be valid for the lifetime 'a
    /// - ptr must point to null-terminated string
    /// - The underlying pool must remain alive for 'a
    pub unsafe fn from_ptr(ptr: *const c_char) -> Self {
        if ptr.is_null() {
            BStr {
                data: &[],
                _pool: PhantomData,
            }
        } else {
            let cstr = CStr::from_ptr(ptr);
            BStr {
                data: cstr.to_bytes(),
                _pool: PhantomData,
            }
        }
    }

    /// Get the bytes as a slice
    pub fn as_bytes(&self) -> &[u8] {
        self.data
    }

    /// Try to convert to UTF-8 string
    pub fn to_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self.data)
    }

    /// Convert to UTF-8 string with lossy conversion
    pub fn to_string_lossy(&self) -> std::borrow::Cow<str> {
        String::from_utf8_lossy(self.data)
    }

    /// Check if the string is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the length in bytes
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl<'a> AsRef<[u8]> for BStr<'a> {
    fn as_ref(&self) -> &[u8] {
        self.data
    }
}

/// UTF-8 validated borrowed string backed by pool memory
#[derive(Debug)]
pub struct BStrUtf8<'a> {
    data: &'a str,
    _pool: PhantomData<&'a Pool>,
}

impl<'a> BStrUtf8<'a> {
    /// Create a BStrUtf8 from a C string pointer, validating UTF-8
    ///
    /// # Safety  
    /// - ptr must be valid for the lifetime 'a
    /// - ptr must point to null-terminated string
    /// - The underlying pool must remain alive for 'a
    pub unsafe fn from_ptr(ptr: *const c_char) -> Result<Self, std::str::Utf8Error> {
        if ptr.is_null() {
            Ok(BStrUtf8 {
                data: "",
                _pool: PhantomData,
            })
        } else {
            let cstr = CStr::from_ptr(ptr);
            let s = cstr.to_str()?;
            Ok(BStrUtf8 {
                data: s,
                _pool: PhantomData,
            })
        }
    }

    /// Get the string slice
    pub fn as_str(&self) -> &str {
        self.data
    }

    /// Check if the string is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the length in bytes
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl<'a> AsRef<str> for BStrUtf8<'a> {
    fn as_ref(&self) -> &str {
        self.data
    }
}

impl<'a> std::fmt::Display for BStrUtf8<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

/// Safe wrapper for pool-allocated C strings
pub struct PoolString<'a> {
    ptr: *const c_char,
    _marker: PhantomData<&'a Pool>,
}

impl<'a> PoolString<'a> {
    /// Get the raw C string pointer (for FFI calls)
    pub fn as_ptr(&self) -> *const c_char {
        self.ptr
    }

    /// Get as a BStr (borrowed bytes)
    pub fn as_bstr(&self) -> BStr<'a> {
        unsafe { BStr::from_ptr(self.ptr) }
    }

    /// Try to get as UTF-8 string
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        unsafe {
            let cstr = CStr::from_ptr(self.ptr);
            cstr.to_str()
        }
    }

    /// Get as bytes
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let cstr = CStr::from_ptr(self.ptr);
            cstr.to_bytes()
        }
    }

    /// Get length in bytes
    pub fn len(&self) -> usize {
        self.as_bstr().len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a> std::fmt::Display for PoolString<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_str() {
            Ok(s) => write!(f, "{}", s),
            Err(_) => write!(f, "{:?}", self.as_bytes()),
        }
    }
}

impl<'a> std::fmt::Debug for PoolString<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_str() {
            Ok(s) => write!(f, "PoolString({:?})", s),
            Err(_) => write!(f, "PoolString({:?})", self.as_bytes()),
        }
    }
}

/// Duplicate a Rust string into pool-allocated memory as a C string
pub fn pstrdup<'a>(s: &str, pool: &'a Pool) -> Result<PoolString<'a>, std::ffi::NulError> {
    let cstring = CString::new(s)?;
    let ptr = unsafe { apr_sys::apr_pstrdup(pool.as_mut_ptr(), cstring.as_ptr()) };
    Ok(PoolString {
        ptr,
        _marker: PhantomData,
    })
}

/// Get raw pointer version (for advanced users)
pub fn pstrdup_raw(s: &str, pool: &Pool) -> Result<*const c_char, std::ffi::NulError> {
    Ok(pstrdup(s, pool)?.as_ptr())
}

/// Duplicate a limited portion of a Rust string into pool-allocated memory
pub fn pstrndup<'a>(
    s: &str,
    n: usize,
    pool: &'a Pool,
) -> Result<PoolString<'a>, std::ffi::NulError> {
    let cstring = CString::new(s)?;
    let ptr = unsafe { apr_sys::apr_pstrndup(pool.as_mut_ptr(), cstring.as_ptr(), n) };
    Ok(PoolString {
        ptr,
        _marker: PhantomData,
    })
}

/// Copy bytes into pool-allocated memory (not null-terminated)
///
/// Returns an immutable slice since Pool is borrowed immutably
pub fn pmemdup<'a>(data: &[u8], pool: &'a Pool) -> &'a [u8] {
    unsafe {
        let ptr = apr_sys::apr_pmemdup(
            pool.as_mut_ptr(),
            data.as_ptr() as *const std::ffi::c_void,
            data.len(),
        ) as *const u8;
        std::slice::from_raw_parts(ptr, data.len())
    }
}

// Note: apr_pstrcat is a varargs function which is hard to call from Rust.
// If needed, concatenate strings manually and use pstrdup.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bstr() {
        let test_str = "Hello, world!";
        let cstring = CString::new(test_str).unwrap();

        unsafe {
            let bstr = BStr::from_ptr(cstring.as_ptr());
            assert_eq!(bstr.as_bytes(), test_str.as_bytes());
            assert_eq!(bstr.to_str().unwrap(), test_str);
            assert!(!bstr.is_empty());
            assert_eq!(bstr.len(), test_str.len());
        }
    }

    #[test]
    fn test_bstr_utf8() {
        let test_str = "Hello, 世界!";
        let cstring = CString::new(test_str).unwrap();

        unsafe {
            let bstr_utf8 = BStrUtf8::from_ptr(cstring.as_ptr()).unwrap();
            assert_eq!(bstr_utf8.as_str(), test_str);
            assert!(!bstr_utf8.is_empty());
            assert_eq!(bstr_utf8.len(), test_str.len());
        }
    }

    #[test]
    fn test_pool_string_operations() {
        let pool = Pool::new();

        let pooled = pstrdup("test string", &pool).unwrap();
        assert_eq!(pooled.as_str().unwrap(), "test string");
        assert_eq!(pooled.len(), 11);
        assert!(!pooled.is_empty());

        let bstr = pooled.as_bstr();
        assert_eq!(bstr.to_str().unwrap(), "test string");

        // Test pmemdup
        let data = b"binary data";
        let copied = pmemdup(data, &pool);
        assert_eq!(copied, data);
    }

    #[test]
    fn test_pool_string_display() {
        let pool = Pool::new();
        let pooled = pstrdup("hello", &pool).unwrap();
        assert_eq!(format!("{}", pooled), "hello");
        assert_eq!(format!("{:?}", pooled), "PoolString(\"hello\")");
    }
}
