//! Path handling utilities for cross-platform operations.
//!
//! This module provides utilities for handling paths in a cross-platform way,
//! particularly for converting between Rust Path types and C string representations
//! that APR and related libraries expect.

use crate::pool::Pool;
use crate::status::{apr_result, Status};
use crate::strings::{pstrdup, BStr, PoolString};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Convert a Rust Path to a pool-allocated C string suitable for APR functions
///
/// This handles platform-specific path encoding:
/// - On Unix: paths are typically UTF-8 bytes
/// - On Windows: converts from UTF-16 to the appropriate byte encoding
pub fn path_to_cstring<P: AsRef<Path>>(path: P, pool: &Pool) -> Result<PoolString, Status> {
    let path = path.as_ref();

    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let bytes = path.as_os_str().as_bytes();
        let path_str = String::from_utf8_lossy(bytes);
        pstrdup(&path_str, pool).map_err(|_| Status::BadArgument)
    }

    #[cfg(windows)]
    {
        let path_str = path.to_string_lossy();
        pstrdup(&path_str, pool).map_err(|_| Status::BadArgument)
    }
}

/// Convert a C string back to a Rust PathBuf
///
/// # Safety
/// The ptr must be a valid null-terminated C string
pub unsafe fn cstring_to_pathbuf(ptr: *const std::ffi::c_char) -> PathBuf {
    if ptr.is_null() {
        return PathBuf::new();
    }

    let bstr = BStr::from_ptr(ptr);

    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let os_str = OsStr::from_bytes(bstr.as_bytes());
        PathBuf::from(os_str)
    }

    #[cfg(windows)]
    {
        let path_str = bstr.to_string_lossy();
        PathBuf::from(path_str.as_ref())
    }
}

/// Normalize a path using APR's path normalization
pub fn normalize_path<P: AsRef<Path>>(path: P, pool: &Pool) -> Result<PathBuf, Status> {
    let path_cstr = path_to_cstring(path, pool)?;

    unsafe {
        let mut normalized_ptr: *const std::ffi::c_char = std::ptr::null();
        let status = apr_sys::apr_filepath_merge(
            &mut normalized_ptr as *mut _ as *mut *mut std::ffi::c_char,
            std::ptr::null(), // No root path
            path_cstr.as_ptr(),
            apr_sys::APR_FILEPATH_SECUREROOT as i32,
            pool.as_mut_ptr(),
        );

        apr_result(status)?;
        Ok(cstring_to_pathbuf(normalized_ptr))
    }
}

/// Check if a path is absolute using APR's path checking
pub fn is_absolute<P: AsRef<Path>>(_path: P, pool: &Pool) -> Result<bool, Status> {
    // Note: apr_filepath_root has a different signature than expected
    // For now, use Rust's built-in path checking
    // TODO: Investigate proper APR usage for this
    let _pool = pool; // Suppress warning
    Ok(_path.as_ref().is_absolute())
}

// Note: apr_filepath_name_get doesn't exist in APR
// Libraries using APR will need to implement their own basename/dirname

/// Join two paths
pub fn join_paths<P1: AsRef<Path>, P2: AsRef<Path>>(
    base: P1,
    path: P2,
    pool: &Pool,
) -> Result<PathBuf, Status> {
    let base_cstr = path_to_cstring(base, pool)?;
    let path_cstr = path_to_cstring(path, pool)?;

    unsafe {
        let mut joined_ptr: *const std::ffi::c_char = std::ptr::null();
        let status = apr_sys::apr_filepath_merge(
            &mut joined_ptr as *mut _ as *mut *mut std::ffi::c_char,
            base_cstr.as_ptr(),
            path_cstr.as_ptr(),
            0, // No special flags
            pool.as_mut_ptr(),
        );

        apr_result(status)?;
        Ok(cstring_to_pathbuf(joined_ptr))
    }
}

/// Get the current working directory
pub fn get_cwd(pool: &Pool) -> Result<PathBuf, Status> {
    unsafe {
        let mut cwd_ptr: *mut std::ffi::c_char = std::ptr::null_mut();
        let status = apr_sys::apr_filepath_get(
            &mut cwd_ptr,
            0, // No special flags
            pool.as_mut_ptr(),
        );

        apr_result(status)?;
        Ok(cstring_to_pathbuf(cwd_ptr))
    }
}

/// Set the current working directory
pub fn set_cwd<P: AsRef<Path>>(path: P, pool: &Pool) -> Result<(), Status> {
    let path_cstr = path_to_cstring(path, pool)?;

    unsafe {
        let status = apr_sys::apr_filepath_set(path_cstr.as_ptr(), pool.as_mut_ptr());

        apr_result(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_path_conversion() {
        let pool = Pool::new();
        let path = Path::new("/tmp/test/file.txt");

        let pool_string = path_to_cstring(path, &pool).unwrap();

        // Should contain the path
        assert!(pool_string.as_str().unwrap().contains("tmp"));
        assert!(pool_string.as_str().unwrap().contains("file.txt"));
    }

    #[test]
    fn test_get_cwd() {
        let pool = Pool::new();
        let cwd = get_cwd(&pool).unwrap();

        // Should get some path
        assert!(!cwd.as_os_str().is_empty());
    }
}
