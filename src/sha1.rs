//! SHA1 hashing functionality from apr-util.

use crate::pool::Pool;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

/// SHA1 context for incremental hashing.
pub struct Sha1Context<'pool> {
    ctx: apr_sys::apr_sha1_ctx_t,
    _pool: PhantomData<&'pool Pool<'pool>>,
}

impl<'pool> Sha1Context<'pool> {
    /// Create a new SHA1 context.
    pub fn new(_pool: &'pool Pool<'pool>) -> Self {
        let mut ctx = MaybeUninit::uninit();
        unsafe {
            apr_sys::apr_sha1_init(ctx.as_mut_ptr());
        }

        Sha1Context {
            ctx: unsafe { ctx.assume_init() },
            _pool: PhantomData,
        }
    }

    /// Update the SHA1 context with more data.
    pub fn update(&mut self, data: &[u8]) {
        unsafe {
            apr_sys::apr_sha1_update(
                &mut self.ctx,
                data.as_ptr() as *const std::os::raw::c_char,
                data.len() as std::os::raw::c_uint,
            );
        }
    }

    /// Update the SHA1 context with more data (binary-safe version).
    pub fn update_binary(&mut self, data: &[u8]) {
        unsafe {
            apr_sys::apr_sha1_update_binary(
                &mut self.ctx,
                data.as_ptr() as *const std::os::raw::c_uchar,
                data.len() as std::os::raw::c_uint,
            );
        }
    }

    /// Finalize the SHA1 context and return the digest.
    pub fn finalize(mut self) -> [u8; APR_SHA1_DIGESTSIZE] {
        let mut digest = [0u8; APR_SHA1_DIGESTSIZE];
        unsafe {
            apr_sys::apr_sha1_final(digest.as_mut_ptr(), &mut self.ctx);
        }
        digest
    }
}

/// Size of a SHA1 digest in bytes.
pub const APR_SHA1_DIGESTSIZE: usize = 20;

/// Compute the SHA1 digest of data (pool-less API).
pub fn hash(data: &[u8]) -> [u8; APR_SHA1_DIGESTSIZE] {
    crate::pool::with_tmp_pool(|pool| sha1(data, pool))
}

/// Encode data as a SHA1 hash in hex format (pool-less API).
pub fn hash_hex(data: &[u8]) -> String {
    crate::pool::with_tmp_pool(|pool| sha1_encode(data, pool))
}

/// Compute the SHA1 digest of data in one shot (pool-exposed API).
pub fn sha1(data: &[u8], pool: &Pool<'_>) -> [u8; APR_SHA1_DIGESTSIZE] {
    let mut ctx = Sha1Context::new(pool);
    ctx.update_binary(data);
    ctx.finalize()
}

/// Encode data as a SHA1 hash in hex format (pool-exposed API).
pub fn sha1_encode(data: &[u8], pool: &Pool<'_>) -> String {
    let digest = sha1(data, pool);
    let mut result = String::with_capacity(APR_SHA1_DIGESTSIZE * 2);
    for byte in digest.iter() {
        result.push_str(&format!("{:02x}", byte));
    }
    result
}

/// Encode data as a SHA1 hash in base64 format.
pub fn sha1_base64(data: &[u8], pool: &Pool<'_>) -> String {
    let digest = sha1(data, pool);
    base64_encode(&digest)
}

fn base64_encode(data: &[u8]) -> String {
    const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    let mut i = 0;
    while i < data.len() {
        let b1 = data[i];
        let b2 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b3 = if i + 2 < data.len() { data[i + 2] } else { 0 };

        result.push(BASE64_CHARS[(b1 >> 2) as usize] as char);
        result.push(BASE64_CHARS[(((b1 & 0x03) << 4) | (b2 >> 4)) as usize] as char);

        if i + 1 < data.len() {
            result.push(BASE64_CHARS[(((b2 & 0x0f) << 2) | (b3 >> 6)) as usize] as char);
        } else {
            result.push('=');
        }

        if i + 2 < data.len() {
            result.push(BASE64_CHARS[(b3 & 0x3f) as usize] as char);
        } else {
            result.push('=');
        }

        i += 3;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha1_empty() {
        let pool = Pool::new();
        let hex = sha1_encode(b"", &pool);
        assert_eq!(hex, "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    }

    #[test]
    fn test_sha1_hello_world() {
        let pool = Pool::new();
        let hex = sha1_encode(b"Hello, World!", &pool);
        assert_eq!(hex, "0a0a9f2a6772942557ab5355d76af442f8f65e01");
    }

    #[test]
    fn test_sha1_incremental() {
        let pool = Pool::new();
        let mut ctx = Sha1Context::new(&pool);
        ctx.update_binary(b"Hello, ");
        ctx.update_binary(b"World!");
        let digest = ctx.finalize();

        let expected = sha1(b"Hello, World!", &pool);
        assert_eq!(digest, expected);
    }

    #[test]
    fn test_sha1_the_quick_brown_fox() {
        let pool = Pool::new();
        let hex = sha1_encode(b"The quick brown fox jumps over the lazy dog", &pool);
        assert_eq!(hex, "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12");
    }
}
