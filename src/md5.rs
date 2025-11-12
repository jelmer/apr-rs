//! MD5 hashing functionality from apr-util.

use crate::pool::Pool;
use crate::{Error, Status};
use std::ffi::c_char;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

/// MD5 context for incremental hashing.
pub struct Md5Context<'pool> {
    ctx: apr_sys::apr_md5_ctx_t,
    _pool: PhantomData<&'pool Pool<'pool>>,
}

impl<'pool> Md5Context<'pool> {
    /// Create a new MD5 context.
    pub fn new(_pool: &'pool Pool<'pool>) -> Result<Self, Error> {
        let mut ctx = MaybeUninit::uninit();
        let status = unsafe { apr_sys::apr_md5_init(ctx.as_mut_ptr()) };

        if status == apr_sys::APR_SUCCESS as i32 {
            Ok(Md5Context {
                ctx: unsafe { ctx.assume_init() },
                _pool: PhantomData,
            })
        } else {
            Err(Error::from_status(Status::from(status)))
        }
    }

    /// Update the MD5 context with more data.
    pub fn update(&mut self, data: &[u8]) -> Result<(), Error> {
        let status = unsafe {
            apr_sys::apr_md5_update(
                &mut self.ctx,
                data.as_ptr() as *const std::os::raw::c_void,
                data.len() as apr_sys::apr_size_t,
            )
        };

        if status == apr_sys::APR_SUCCESS as i32 {
            Ok(())
        } else {
            Err(Error::from_status(Status::from(status)))
        }
    }

    /// Finalize the MD5 context and return the digest.
    pub fn finalize(mut self) -> [u8; APR_MD5_DIGESTSIZE] {
        let mut digest = [0u8; APR_MD5_DIGESTSIZE];
        unsafe {
            apr_sys::apr_md5_final(digest.as_mut_ptr(), &mut self.ctx);
        }
        digest
    }
}

/// Size of an MD5 digest in bytes.
pub const APR_MD5_DIGESTSIZE: usize = 16;

/// Compute the MD5 digest of data (pool-less API).
pub fn hash(data: &[u8]) -> Result<[u8; APR_MD5_DIGESTSIZE], Error> {
    crate::pool::with_tmp_pool(|pool| md5(data, pool))
}

/// Encode data as an MD5 hash in hex format (pool-less API).
pub fn hash_hex(data: &[u8]) -> Result<String, Error> {
    crate::pool::with_tmp_pool(|pool| md5_encode(data, pool))
}

/// Compute the MD5 digest of data in one shot (pool-exposed API).
pub fn md5(data: &[u8], pool: &Pool<'_>) -> Result<[u8; APR_MD5_DIGESTSIZE], Error> {
    let mut ctx = Md5Context::new(pool)?;
    ctx.update(data)?;
    Ok(ctx.finalize())
}

/// Encode data as an MD5 hash in hex format (pool-exposed API).
pub fn md5_encode(data: &[u8], pool: &Pool<'_>) -> Result<String, Error> {
    let digest = md5(data, pool)?;
    let mut result = String::with_capacity(APR_MD5_DIGESTSIZE * 2);
    for byte in digest.iter() {
        result.push_str(&format!("{:02x}", byte));
    }
    Ok(result)
}

/// Encode a password using the Apache MD5 algorithm (for .htpasswd files).
pub fn md5_encode_password(password: &str, salt: &str) -> Result<String, Error> {
    let password_cstr = std::ffi::CString::new(password)
        .map_err(|_| Error::from_status(Status::from(apr_sys::APR_EINVAL as i32)))?;
    let salt_cstr = std::ffi::CString::new(salt)
        .map_err(|_| Error::from_status(Status::from(apr_sys::APR_EINVAL as i32)))?;

    let mut result_buf = vec![0u8; 120]; // Apache MD5 passwords are at most 120 chars

    let status = unsafe {
        apr_sys::apr_md5_encode(
            password_cstr.as_ptr(),
            salt_cstr.as_ptr(),
            result_buf.as_mut_ptr() as *mut c_char,
            result_buf.len() as apr_sys::apr_size_t,
        )
    };

    if status == apr_sys::APR_SUCCESS as i32 {
        let cstr = unsafe { CStr::from_ptr(result_buf.as_ptr() as *const c_char) };
        Ok(cstr.to_string_lossy().into_owned())
    } else {
        Err(Error::from_status(Status::from(status)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md5_empty() {
        let pool = Pool::new();
        let _digest = md5(b"", &pool).unwrap();
        let hex = md5_encode(b"", &pool).unwrap();
        assert_eq!(hex, "d41d8cd98f00b204e9800998ecf8427e");
    }

    #[test]
    fn test_md5_hello_world() {
        let pool = Pool::new();
        let _digest = md5(b"Hello, World!", &pool).unwrap();
        let hex = md5_encode(b"Hello, World!", &pool).unwrap();
        assert_eq!(hex, "65a8e27d8879283831b664bd8b7f0ad4");
    }

    #[test]
    fn test_md5_incremental() {
        let pool = Pool::new();
        let mut ctx = Md5Context::new(&pool).unwrap();
        ctx.update(b"Hello, ").unwrap();
        ctx.update(b"World!").unwrap();
        let digest = ctx.finalize();

        let expected = md5(b"Hello, World!", &pool).unwrap();
        assert_eq!(digest, expected);
    }

    #[test]
    fn test_md5_password_encoding() {
        let encoded = md5_encode_password("password", "12345678").unwrap();
        // Apache MD5 passwords start with $apr1$
        assert!(encoded.starts_with("$apr1$"));
    }
}
