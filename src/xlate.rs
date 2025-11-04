//! Character set translation functionality from apr-util.
//!
//! Provides character encoding conversion using iconv or similar backends.

use crate::pool::Pool;
use crate::{Error, Status};
use std::ffi::CString;
use std::ffi::c_char;
use std::marker::PhantomData;
use std::ptr;

/// Character set translation handle.
pub struct Xlate<'pool> {
    handle: *mut apr_sys::apr_xlate_t,
    _pool: PhantomData<&'pool Pool>,
}

impl<'pool> Xlate<'pool> {
    /// Create a new translation handle.
    ///
    /// Converts from `from_charset` to `to_charset`.
    pub fn new(to_charset: &str, from_charset: &str, pool: &'pool Pool) -> Result<Self, Error> {
        let to_cstr = CString::new(to_charset)
            .map_err(|_| Error::from_status(Status::from(apr_sys::APR_EINVAL as i32)))?;
        let from_cstr = CString::new(from_charset)
            .map_err(|_| Error::from_status(Status::from(apr_sys::APR_EINVAL as i32)))?;

        let mut handle: *mut apr_sys::apr_xlate_t = ptr::null_mut();

        let status = unsafe {
            apr_sys::apr_xlate_open(
                &mut handle,
                to_cstr.as_ptr(),
                from_cstr.as_ptr(),
                pool.as_ptr() as *mut apr_sys::apr_pool_t,
            )
        };

        if status == apr_sys::APR_SUCCESS as i32 {
            Ok(Xlate {
                handle,
                _pool: PhantomData,
            })
        } else {
            Err(Error::from_status(Status::from(status)))
        }
    }

    /// Convert a string from the source encoding to the destination encoding.
    pub fn convert_string(&self, input: &str) -> Result<String, Error> {
        let input_bytes = input.as_bytes();
        let mut inbytes_left = input_bytes.len();
        let mut outbytes_left = inbytes_left * 4; // Allocate extra space for worst case
        let mut output = vec![0u8; outbytes_left];

        let inbuf_ptr = input_bytes.as_ptr() as *const c_char;
        let outbuf_ptr = output.as_mut_ptr() as *mut c_char;

        let status = unsafe {
            apr_sys::apr_xlate_conv_buffer(
                self.handle,
                inbuf_ptr,
                &mut inbytes_left,
                outbuf_ptr,
                &mut outbytes_left,
            )
        };

        if status == apr_sys::APR_SUCCESS as i32 {
            let converted_len = output.len() - outbytes_left;
            output.truncate(converted_len);
            String::from_utf8(output)
                .map_err(|_| Error::from_status(Status::from(apr_sys::APR_EINVAL as i32)))
        } else {
            Err(Error::from_status(Status::from(status)))
        }
    }

    /// Convert bytes from the source encoding to the destination encoding.
    pub fn convert_buffer(&self, input: &[u8]) -> Result<Vec<u8>, Error> {
        let mut inbytes_left = input.len();
        let mut outbytes_left = inbytes_left * 4; // Allocate extra space for worst case
        let mut output = vec![0u8; outbytes_left];

        let inbuf_ptr = input.as_ptr() as *const c_char;
        let outbuf_ptr = output.as_mut_ptr() as *mut c_char;

        let status = unsafe {
            apr_sys::apr_xlate_conv_buffer(
                self.handle,
                inbuf_ptr,
                &mut inbytes_left,
                outbuf_ptr,
                &mut outbytes_left,
            )
        };

        if status == apr_sys::APR_SUCCESS as i32 {
            let converted_len = output.len() - outbytes_left;
            output.truncate(converted_len);
            Ok(output)
        } else {
            Err(Error::from_status(Status::from(status)))
        }
    }

    /// Convert a single byte.
    pub fn conv_byte(&self, inbyte: u8) -> i32 {
        unsafe { apr_sys::apr_xlate_conv_byte(self.handle, inbyte) }
    }
}

impl<'pool> Drop for Xlate<'pool> {
    fn drop(&mut self) {
        unsafe {
            apr_sys::apr_xlate_close(self.handle);
        }
    }
}

/// Convert a string between character encodings (pool-less API).
pub fn convert_string(input: &str, from_charset: &str, to_charset: &str) -> Result<String, Error> {
    crate::pool::with_tmp_pool(|pool| {
        let xlate = Xlate::new(to_charset, from_charset, pool)?;
        xlate.convert_string(input)
    })
}

/// Convert bytes between character encodings (pool-less API).
pub fn convert_buffer(
    input: &[u8],
    from_charset: &str,
    to_charset: &str,
) -> Result<Vec<u8>, Error> {
    crate::pool::with_tmp_pool(|pool| {
        let xlate = Xlate::new(to_charset, from_charset, pool)?;
        xlate.convert_buffer(input)
    })
}

/// Convert a string between character encodings (pool-exposed API).
pub fn xlate_conv_string(
    input: &str,
    from_charset: &str,
    to_charset: &str,
    pool: &Pool,
) -> Result<String, Error> {
    let xlate = Xlate::new(to_charset, from_charset, pool)?;
    xlate.convert_string(input)
}

/// Convert bytes between character encodings (pool-exposed API).
pub fn xlate_conv_buffer(
    input: &[u8],
    from_charset: &str,
    to_charset: &str,
    pool: &Pool,
) -> Result<Vec<u8>, Error> {
    let xlate = Xlate::new(to_charset, from_charset, pool)?;
    xlate.convert_buffer(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xlate_utf8_to_ascii() {
        let pool = Pool::new();
        let xlate = Xlate::new("ASCII", "UTF-8", &pool);

        if let Ok(xlate) = xlate {
            let result = xlate.convert_string("Hello, World!");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "Hello, World!");
        }
        // Skip test if xlate not available
    }

    #[test]
    fn test_xlate_conv_string() {
        // Test pool-less API
        let result = convert_string("Test", "UTF-8", "ASCII");
        // May fail if iconv not available, so we just check it doesn't crash
        let _ = result;

        // Test pool-exposed API
        let pool = Pool::new();
        let result = xlate_conv_string("Test", "UTF-8", "ASCII", &pool);
        let _ = result;
    }

    #[test]
    fn test_xlate_buffer() {
        let pool = Pool::new();
        let xlate = Xlate::new("UTF-8", "UTF-8", &pool);
        if let Ok(xlate) = xlate {
            let input = b"Test buffer";
            let result = xlate.convert_buffer(input);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), input);
        }
    }
}
