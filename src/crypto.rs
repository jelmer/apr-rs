//! Cryptographic functionality from apr-util.
//!
//! Provides symmetric encryption and decryption using various crypto backends
//! (OpenSSL, NSS, CommonCrypto, etc.).

use crate::pool::Pool;
use crate::{Error, Status};
use std::ffi::CString;
use std::marker::PhantomData;
use std::ptr;

/// Crypto driver/factory handle.
pub struct CryptoDriver<'pool> {
    driver: *const apr_sys::apr_crypto_driver_t,
    _pool: PhantomData<&'pool Pool>,
}

/// Crypto context handle.
pub struct Crypto<'pool> {
    factory: *mut apr_sys::apr_crypto_t,
    _pool: PhantomData<&'pool Pool>,
}

/// Encryption/decryption block handle.
pub struct CryptoBlock<'pool> {
    block: *mut apr_sys::apr_crypto_block_t,
    _pool: PhantomData<&'pool Pool>,
}

/// Key for encryption/decryption.
pub struct CryptoKey<'pool> {
    key: *mut apr_sys::apr_crypto_key_t,
    _pool: PhantomData<&'pool Pool>,
}

/// Block cipher mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockCipherMode {
    /// Electronic Codebook mode
    ECB,
    /// Cipher Block Chaining mode
    CBC,
}

impl From<BlockCipherMode> for apr_sys::apr_crypto_block_key_mode_e {
    fn from(mode: BlockCipherMode) -> Self {
        match mode {
            BlockCipherMode::ECB => apr_sys::apr_crypto_block_key_mode_e_APR_MODE_ECB,
            BlockCipherMode::CBC => apr_sys::apr_crypto_block_key_mode_e_APR_MODE_CBC,
        }
    }
}

/// Block cipher algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockCipherAlgorithm {
    /// AES with 128-bit key
    AES128,
    /// AES with 192-bit key
    AES192,
    /// AES with 256-bit key
    AES256,
    /// Triple DES
    DES3,
}

impl From<BlockCipherAlgorithm> for apr_sys::apr_crypto_block_key_type_e {
    fn from(algo: BlockCipherAlgorithm) -> Self {
        match algo {
            BlockCipherAlgorithm::AES128 => apr_sys::apr_crypto_block_key_type_e_APR_KEY_AES_128,
            BlockCipherAlgorithm::AES192 => apr_sys::apr_crypto_block_key_type_e_APR_KEY_AES_192,
            BlockCipherAlgorithm::AES256 => apr_sys::apr_crypto_block_key_type_e_APR_KEY_AES_256,
            BlockCipherAlgorithm::DES3 => apr_sys::apr_crypto_block_key_type_e_APR_KEY_3DES_192,
        }
    }
}

/// Initialize the crypto library (pool-less API).
pub fn init() -> Result<(), Error> {
    crate::pool::with_tmp_pool(|pool| {
        let status = unsafe { apr_sys::apr_crypto_init(pool.as_ptr() as *mut apr_sys::apr_pool_t) };

        if status == apr_sys::APR_SUCCESS as i32 {
            Ok(())
        } else {
            Err(Error::from_status(Status::from(status)))
        }
    })
}

/// Encrypt data using a simple API (pool-less).
pub fn encrypt_aes256(key: &[u8], data: &[u8], iv: Option<&[u8]>) -> Result<Vec<u8>, Error> {
    crate::pool::with_tmp_pool(|pool| {
        let driver = get_driver("openssl", pool)?;
        let crypto = driver.make_crypto(pool)?;
        let crypto_key = crypto.make_key(
            BlockCipherAlgorithm::AES256,
            BlockCipherMode::CBC,
            key,
            pool,
        )?;
        crypto.encrypt(&crypto_key, data, iv, pool)
    })
}

/// Decrypt data using a simple API (pool-less).
pub fn decrypt_aes256(key: &[u8], data: &[u8], iv: Option<&[u8]>) -> Result<Vec<u8>, Error> {
    crate::pool::with_tmp_pool(|pool| {
        let driver = get_driver("openssl", pool)?;
        let crypto = driver.make_crypto(pool)?;
        let crypto_key = crypto.make_key(
            BlockCipherAlgorithm::AES256,
            BlockCipherMode::CBC,
            key,
            pool,
        )?;
        crypto.decrypt(&crypto_key, data, iv, pool)
    })
}

/// Get a crypto driver by name (pool-exposed API).
pub fn get_driver<'pool>(name: &str, pool: &'pool Pool) -> Result<CryptoDriver<'pool>, Error> {
    let name_cstr = CString::new(name)
        .map_err(|_| Error::from_status(Status::from(apr_sys::APR_EINVAL as i32)))?;

    let mut driver: *const apr_sys::apr_crypto_driver_t = ptr::null();
    let params_ptr: *const i8 = ptr::null();
    let mut error_ptr: *const apr_sys::apu_err_t = ptr::null();

    let status = unsafe {
        apr_sys::apr_crypto_get_driver(
            &mut driver,
            name_cstr.as_ptr(),
            params_ptr,
            &mut error_ptr,
            pool.as_ptr() as *mut apr_sys::apr_pool_t,
        )
    };

    if status == apr_sys::APR_SUCCESS as i32 {
        Ok(CryptoDriver {
            driver,
            _pool: PhantomData,
        })
    } else {
        Err(Error::from_status(Status::from(status)))
    }
}

impl Crypto<'_> {
    /// Initialize the crypto library (pool-exposed API).
    pub fn init(pool: &Pool) -> Result<(), Error> {
        let status = unsafe { apr_sys::apr_crypto_init(pool.as_ptr() as *mut apr_sys::apr_pool_t) };

        if status == apr_sys::APR_SUCCESS as i32 {
            Ok(())
        } else {
            Err(Error::from_status(Status::from(status)))
        }
    }
}

impl<'pool> CryptoDriver<'pool> {
    /// Create a crypto factory from this driver.
    pub fn make_crypto(&self, pool: &'pool Pool) -> Result<Crypto<'pool>, Error> {
        let mut factory: *mut apr_sys::apr_crypto_t = ptr::null_mut();
        let params_ptr: *const i8 = ptr::null();

        let status = unsafe {
            apr_sys::apr_crypto_make(
                &mut factory,
                self.driver,
                params_ptr,
                pool.as_ptr() as *mut apr_sys::apr_pool_t,
            )
        };

        if status == apr_sys::APR_SUCCESS as i32 {
            Ok(Crypto {
                factory,
                _pool: PhantomData,
            })
        } else {
            Err(Error::from_status(Status::from(status)))
        }
    }
}

impl<'pool> Crypto<'pool> {
    /// Create a key for encryption/decryption.
    pub fn make_key(
        &self,
        algorithm: BlockCipherAlgorithm,
        mode: BlockCipherMode,
        key_data: &[u8],
        pool: &'pool Pool,
    ) -> Result<CryptoKey<'pool>, Error> {
        let mut key: *mut apr_sys::apr_crypto_key_t = ptr::null_mut();
        let mut iv_size: apr_sys::apr_size_t = 0;

        let status = unsafe {
            apr_sys::apr_crypto_passphrase(
                &mut key,
                &mut iv_size,
                key_data.as_ptr() as *const i8,
                key_data.len() as apr_sys::apr_size_t,
                ptr::null(), // salt
                0,           // saltLen
                algorithm.into(),
                mode.into(),
                1,    // doPad
                4096, // iterations
                self.factory,
                pool.as_ptr() as *mut apr_sys::apr_pool_t,
            )
        };

        if status == apr_sys::APR_SUCCESS as i32 {
            Ok(CryptoKey {
                key,
                _pool: PhantomData,
            })
        } else {
            Err(Error::from_status(Status::from(status)))
        }
    }

    /// Encrypt data.
    pub fn encrypt(
        &self,
        key: &CryptoKey,
        plaintext: &[u8],
        iv: Option<&[u8]>,
        pool: &Pool,
    ) -> Result<Vec<u8>, Error> {
        let mut block: *mut apr_sys::apr_crypto_block_t = ptr::null_mut();
        let mut block_size: apr_sys::apr_size_t = 0;

        let mut iv_ptr = iv.map(|v| v.as_ptr()).unwrap_or(ptr::null());

        // Initialize encryption
        let status = unsafe {
            apr_sys::apr_crypto_block_encrypt_init(
                &mut block,
                &mut iv_ptr,
                key.key,
                &mut block_size,
                pool.as_ptr() as *mut apr_sys::apr_pool_t,
            )
        };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(Error::from_status(Status::from(status)));
        }

        // Get block size
        unsafe {
            apr_sys::apr_crypto_block_encrypt(
                &mut ptr::null_mut(),
                &mut block_size,
                ptr::null(),
                0,
                block,
            );
        }

        // Allocate output buffer
        let mut ciphertext = vec![0u8; plaintext.len() + block_size as usize];
        let mut out_ptr = ciphertext.as_mut_ptr();
        let mut out_len = ciphertext.len() as apr_sys::apr_size_t;

        // Encrypt data
        let status = unsafe {
            apr_sys::apr_crypto_block_encrypt(
                &mut out_ptr,
                &mut out_len,
                plaintext.as_ptr(),
                plaintext.len() as apr_sys::apr_size_t,
                block,
            )
        };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(Error::from_status(Status::from(status)));
        }

        // Finalize encryption
        let mut final_len = ciphertext.len() as apr_sys::apr_size_t - out_len;
        let status =
            unsafe { apr_sys::apr_crypto_block_encrypt_finish(out_ptr, &mut final_len, block) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(Error::from_status(Status::from(status)));
        }

        // Clean up block
        unsafe {
            apr_sys::apr_crypto_block_cleanup(block);
        }

        ciphertext.truncate((out_len + final_len) as usize);
        Ok(ciphertext)
    }

    /// Decrypt data.
    pub fn decrypt(
        &self,
        key: &CryptoKey,
        ciphertext: &[u8],
        iv: Option<&[u8]>,
        pool: &Pool,
    ) -> Result<Vec<u8>, Error> {
        let mut block: *mut apr_sys::apr_crypto_block_t = ptr::null_mut();
        let mut block_size: apr_sys::apr_size_t = 0;

        let iv_ptr = iv.map(|v| v.as_ptr()).unwrap_or(ptr::null());

        // Initialize decryption
        let status = unsafe {
            apr_sys::apr_crypto_block_decrypt_init(
                &mut block,
                &mut block_size,
                iv_ptr,
                key.key,
                pool.as_ptr() as *mut apr_sys::apr_pool_t,
            )
        };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(Error::from_status(Status::from(status)));
        }

        // Allocate output buffer
        let mut plaintext = vec![0u8; ciphertext.len()];
        let mut out_ptr = plaintext.as_mut_ptr();
        let mut out_len = plaintext.len() as apr_sys::apr_size_t;

        // Decrypt data
        let status = unsafe {
            apr_sys::apr_crypto_block_decrypt(
                &mut out_ptr,
                &mut out_len,
                ciphertext.as_ptr(),
                ciphertext.len() as apr_sys::apr_size_t,
                block,
            )
        };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(Error::from_status(Status::from(status)));
        }

        // Finalize decryption
        let mut final_len = plaintext.len() as apr_sys::apr_size_t - out_len;
        let status =
            unsafe { apr_sys::apr_crypto_block_decrypt_finish(out_ptr, &mut final_len, block) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(Error::from_status(Status::from(status)));
        }

        // Clean up block
        unsafe {
            apr_sys::apr_crypto_block_cleanup(block);
        }

        plaintext.truncate((out_len + final_len) as usize);
        Ok(plaintext)
    }
}

/// Get list of available crypto drivers.
pub fn crypto_drivers(pool: &Pool) -> Vec<String> {
    // Common driver names to try
    let drivers = ["openssl", "nss", "commoncrypto", "mscapi", "mscng"];
    let mut available = Vec::new();

    for name in &drivers {
        if get_driver(name, pool).is_ok() {
            available.push(name.to_string());
        }
    }

    available
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_init() {
        let pool = Pool::new();
        // Crypto init may fail if no drivers available
        let _ = Crypto::init(&pool);
    }

    #[test]
    fn test_crypto_drivers() {
        let pool = Pool::new();
        let _ = Crypto::init(&pool);
        let drivers = crypto_drivers(&pool);
        // May be empty if no drivers available
        println!("Available crypto drivers: {:?}", drivers);
    }

    #[test]
    fn test_encrypt_decrypt() {
        let pool = Pool::new();

        // Try to initialize crypto
        if Crypto::init(&pool).is_err() {
            return; // Skip if crypto not available
        }

        // Try to get a driver
        let driver = match get_driver("openssl", &pool)
            .or_else(|_| get_driver("nss", &pool))
            .or_else(|_| get_driver("commoncrypto", &pool))
        {
            Ok(d) => d,
            Err(_) => return, // No drivers available
        };

        let crypto = match driver.make_crypto(&pool) {
            Ok(c) => c,
            Err(_) => return,
        };

        let key_data = b"thisisasecretkey";
        let key = match crypto.make_key(
            BlockCipherAlgorithm::AES128,
            BlockCipherMode::CBC,
            key_data,
            &pool,
        ) {
            Ok(k) => k,
            Err(_) => return,
        };

        let plaintext = b"Hello, World! This is a test.";
        let iv = b"1234567890123456"; // 16 bytes for AES

        // Encrypt
        let ciphertext = match crypto.encrypt(&key, plaintext, Some(iv), &pool) {
            Ok(c) => c,
            Err(_) => return,
        };

        assert!(!ciphertext.is_empty());
        assert_ne!(&ciphertext[..], plaintext);

        // Decrypt
        let decrypted = match crypto.decrypt(&key, &ciphertext, Some(iv), &pool) {
            Ok(p) => p,
            Err(_) => return,
        };

        assert_eq!(&decrypted[..], plaintext);
    }
}
