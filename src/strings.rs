//! String utilities.
use crate::pool::Pool;
use std::ffi::c_char;

/// Duplicate a string into memory allocated from a pool.
pub fn pstrdup(s: &str, pool: &mut Pool) -> *mut c_char {
    unsafe {
        let cs = std::ffi::CStr::from_ptr(s.as_ptr() as *const _);
        crate::generated::apr_pstrndup(pool.as_mut_ptr(), cs.as_ptr(), s.len())
    }
}

/// Duplicate a string into memory allocated from a pool.
pub fn pstrndup(s: &str, n: usize, pool: &mut Pool) -> *mut c_char {
    unsafe {
        let s = std::ffi::CStr::from_ptr(s.as_ptr() as *const _);
        crate::generated::apr_pstrndup(pool.as_mut_ptr(), s.as_ptr(), n)
    }
}
