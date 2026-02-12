//! String utilities for safe C string handling.
use crate::pool::Pool;
use alloc::ffi::CString;
use alloc::string::String;
use core::ffi::{c_char, CStr};
use core::marker::PhantomData;

/// Borrowed byte string backed by pool memory
///
/// This represents bytes from C strings, potentially containing non-UTF-8 data.
/// Use this when you need zero-copy access to C string data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BStr<'a> {
    data: &'a [u8],
    _pool: PhantomData<&'a Pool<'a>>,
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
    pub fn to_str(&self) -> Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(self.data)
    }

    /// Convert to UTF-8 string with lossy conversion
    pub fn to_string_lossy(&self) -> alloc::borrow::Cow<'_, str> {
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

impl<'a> core::ops::Deref for BStr<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a> core::fmt::Display for BStr<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(self.data))
    }
}

impl<'a> From<&'a [u8]> for BStr<'a> {
    fn from(data: &'a [u8]) -> Self {
        BStr {
            data,
            _pool: PhantomData,
        }
    }
}

impl<'a> From<&'a str> for BStr<'a> {
    fn from(s: &'a str) -> Self {
        BStr {
            data: s.as_bytes(),
            _pool: PhantomData,
        }
    }
}

impl<'a> core::borrow::Borrow<[u8]> for BStr<'a> {
    fn borrow(&self) -> &[u8] {
        self.data
    }
}

impl<'a> PartialEq<&str> for BStr<'a> {
    fn eq(&self, other: &&str) -> bool {
        self.data == other.as_bytes()
    }
}

impl<'a> PartialEq<str> for BStr<'a> {
    fn eq(&self, other: &str) -> bool {
        self.data == other.as_bytes()
    }
}

impl<'a> PartialEq<&[u8]> for BStr<'a> {
    fn eq(&self, other: &&[u8]) -> bool {
        self.data == *other
    }
}

/// UTF-8 validated borrowed string backed by pool memory
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BStrUtf8<'a> {
    data: &'a str,
    _pool: PhantomData<&'a Pool<'a>>,
}

impl<'a> BStrUtf8<'a> {
    /// Create a BStrUtf8 from a C string pointer, validating UTF-8
    ///
    /// # Safety  
    /// - ptr must be valid for the lifetime 'a
    /// - ptr must point to null-terminated string
    /// - The underlying pool must remain alive for 'a
    pub unsafe fn from_ptr(ptr: *const c_char) -> Result<Self, core::str::Utf8Error> {
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

impl<'a> core::ops::Deref for BStrUtf8<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a> core::fmt::Display for BStrUtf8<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl<'a> From<&'a str> for BStrUtf8<'a> {
    fn from(data: &'a str) -> Self {
        BStrUtf8 {
            data,
            _pool: PhantomData,
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for BStrUtf8<'a> {
    type Error = core::str::Utf8Error;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        let s = core::str::from_utf8(data)?;
        Ok(BStrUtf8 {
            data: s,
            _pool: PhantomData,
        })
    }
}

impl<'a> core::borrow::Borrow<str> for BStrUtf8<'a> {
    fn borrow(&self) -> &str {
        self.data
    }
}

impl<'a> PartialEq<&str> for BStrUtf8<'a> {
    fn eq(&self, other: &&str) -> bool {
        self.data == *other
    }
}

impl<'a> PartialEq<String> for BStrUtf8<'a> {
    fn eq(&self, other: &String) -> bool {
        self.data == other.as_str()
    }
}

/// Safe wrapper for pool-allocated C strings
pub struct PoolString<'a> {
    ptr: *const c_char,
    _marker: PhantomData<&'a Pool<'a>>,
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
    pub fn as_str(&self) -> Result<&str, core::str::Utf8Error> {
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

impl<'a> core::fmt::Display for PoolString<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.as_str() {
            Ok(s) => write!(f, "{}", s),
            Err(_) => write!(f, "{:?}", self.as_bytes()),
        }
    }
}

impl<'a> core::fmt::Debug for PoolString<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.as_str() {
            Ok(s) => write!(f, "PoolString({:?})", s),
            Err(_) => write!(f, "PoolString({:?})", self.as_bytes()),
        }
    }
}

/// Duplicate a Rust string into pool-allocated memory as a C string
pub fn pstrdup<'a>(s: &str, pool: &'a Pool) -> Result<PoolString<'a>, alloc::ffi::NulError> {
    let cstring = CString::new(s)?;
    let ptr = unsafe { apr_sys::apr_pstrdup(pool.as_mut_ptr(), cstring.as_ptr()) };
    Ok(PoolString {
        ptr,
        _marker: PhantomData,
    })
}

/// Get raw pointer version (for advanced users)
pub fn pstrdup_raw(s: &str, pool: &Pool<'_>) -> Result<*const c_char, alloc::ffi::NulError> {
    Ok(pstrdup(s, pool)?.as_ptr())
}

/// Create a pool-allocated C string from a Rust string
///
/// This is a convenience function that allocates a null-terminated C string
/// in the given pool's memory and returns it as a `CStr` reference.
///
/// # Example
/// ```
/// use apr::pool::Pool;
/// use apr::strings::make_cstring;
///
/// let pool = Pool::new();
/// let cstr = make_cstring("hello", &pool).unwrap();
/// assert_eq!(cstr.to_str().unwrap(), "hello");
/// ```
pub fn make_cstring<'a>(s: &str, pool: &'a Pool) -> Result<&'a CStr, alloc::ffi::NulError> {
    let ptr = pstrdup_raw(s, pool)?;
    Ok(unsafe { CStr::from_ptr(ptr) })
}

/// Duplicate a limited portion of a Rust string into pool-allocated memory
pub fn pstrndup<'a>(
    s: &str,
    n: usize,
    pool: &'a Pool,
) -> Result<PoolString<'a>, alloc::ffi::NulError> {
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
            data.as_ptr() as *const core::ffi::c_void,
            data.len(),
        ) as *const u8;
        core::slice::from_raw_parts(ptr, data.len())
    }
}

// Note: apr_pstrcat is a varargs function which is hard to call from Rust.
// If needed, concatenate strings manually and use pstrdup.

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;
    use alloc::string::ToString;

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
    fn test_make_cstring() {
        let pool = Pool::new();

        // Test basic functionality
        let cstr = make_cstring("hello world", &pool).unwrap();
        assert_eq!(cstr.to_str().unwrap(), "hello world");
        assert_eq!(cstr.to_bytes(), b"hello world");

        // Test with empty string
        let empty_cstr = make_cstring("", &pool).unwrap();
        assert_eq!(empty_cstr.to_str().unwrap(), "");
        assert_eq!(empty_cstr.to_bytes(), b"");

        // Test with Unicode
        let unicode_cstr = make_cstring("Hello, 世界!", &pool).unwrap();
        assert_eq!(unicode_cstr.to_str().unwrap(), "Hello, 世界!");

        // Test that null bytes in the middle cause an error
        let result = make_cstring("hello\0world", &pool);
        assert!(result.is_err());
    }

    #[test]
    fn test_pool_string_display() {
        let pool = Pool::new();
        let pooled = pstrdup("hello", &pool).unwrap();
        assert_eq!(format!("{}", pooled), "hello");
        assert_eq!(format!("{:?}", pooled), "PoolString(\"hello\")");
    }

    #[test]
    fn test_bstr_traits() {
        let data = b"hello world";
        let bstr = BStr::from(&data[..]);

        // Test Clone, Copy
        let bstr2 = bstr;
        // Intentionally testing that Clone works even though Copy is implemented
        #[allow(clippy::clone_on_copy)]
        let bstr3 = bstr.clone();
        assert_eq!(bstr, bstr2);
        assert_eq!(bstr2, bstr3);

        // Test Display and Deref
        assert_eq!(format!("{}", bstr), "hello world");
        assert_eq!(bstr.len(), 11);
        assert_eq!(&bstr[0..5], b"hello");

        // Test From conversions
        let from_str = BStr::from("test");
        assert_eq!(from_str.as_bytes(), b"test");
    }

    #[test]
    fn test_bstr_utf8_traits() {
        let s = "hello 🦀";
        let bstr_utf8 = BStrUtf8::from(s);

        // Test Clone, Copy, PartialEq
        let bstr2 = bstr_utf8;
        assert_eq!(bstr_utf8, bstr2);

        // Test Display and Deref
        assert_eq!(format!("{}", bstr_utf8), "hello 🦀");
        assert_eq!(bstr_utf8.len(), 10); // UTF-8 bytes

        // Test TryFrom
        let from_bytes = BStrUtf8::try_from("hello".as_bytes()).unwrap();
        assert_eq!(from_bytes.as_str(), "hello");

        // Test invalid UTF-8
        let invalid = BStrUtf8::try_from(&[0xFF, 0xFF][..]);
        assert!(invalid.is_err());
    }

    #[test]
    fn test_advanced_string_traits() {
        // Test BStr with various PartialEq implementations
        let bstr = BStr::from("hello");
        assert_eq!(bstr, "hello");
        assert_eq!(bstr, "hello");
        assert_eq!(bstr, &b"hello"[..]);

        // Test Borrow trait
        let borrowed: &[u8] = core::borrow::Borrow::borrow(&bstr);
        assert_eq!(borrowed, b"hello");

        // Test BStrUtf8 PartialEq implementations
        let bstr_utf8 = BStrUtf8::from("hello");
        assert_eq!(bstr_utf8, "hello");
        assert_eq!(bstr_utf8, String::from("hello"));

        // Test Borrow trait for BStrUtf8
        let borrowed: &str = core::borrow::Borrow::borrow(&bstr_utf8);
        assert_eq!(borrowed, "hello");
    }
}
