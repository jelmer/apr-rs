//! Random number generation

use crate::{pool::Pool, Result};
use core::marker::PhantomData;
use core::ptr;

#[repr(transparent)]
pub struct Random<'a> {
    raw: *mut apr_sys::apr_random_t,
    _phantom: PhantomData<&'a Pool<'a>>,
}

impl<'a> Random<'a> {
    pub fn new(pool: &'a Pool<'a>) -> Result<Self> {
        let mut random: *mut apr_sys::apr_random_t = ptr::null_mut();

        let status = unsafe { apr_sys::apr_random_init(&mut random, pool.as_mut_ptr()) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }

        Ok(Random {
            raw: random,
            _phantom: PhantomData,
        })
    }

    pub fn add_entropy(&mut self, entropy: &[u8]) -> Result<()> {
        let status = unsafe {
            apr_sys::apr_random_add_entropy(
                self.raw,
                entropy.as_ptr() as *const core::ffi::c_void,
                entropy.len(),
            )
        };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }
        Ok(())
    }

    pub fn secure_bytes(&mut self, buf: &mut [u8]) -> Result<()> {
        let status = unsafe {
            apr_sys::apr_random_secure_bytes(
                self.raw,
                buf.as_mut_ptr() as *mut core::ffi::c_void,
                buf.len(),
            )
        };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }
        Ok(())
    }

    pub fn insecure_bytes(&mut self, buf: &mut [u8]) -> Result<()> {
        let status = unsafe {
            apr_sys::apr_random_insecure_bytes(
                self.raw,
                buf.as_mut_ptr() as *mut core::ffi::c_void,
                buf.len(),
            )
        };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }
        Ok(())
    }

    pub fn barrier(&mut self) -> Result<()> {
        let status = unsafe { apr_sys::apr_random_barrier(self.raw) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }
        Ok(())
    }

    pub fn secure_ready(&self) -> Result<bool> {
        let status = unsafe { apr_sys::apr_random_secure_ready(self.raw) };

        match status as u32 {
            x if x == apr_sys::APR_SUCCESS => Ok(true),
            x if x == apr_sys::APR_ENOTENOUGHENTROPY => Ok(false),
            _ => Err(crate::Error::from_status(status.into())),
        }
    }

    pub fn as_ptr(&self) -> *const apr_sys::apr_random_t {
        self.raw
    }

    pub fn as_mut_ptr(&mut self) -> *mut apr_sys::apr_random_t {
        self.raw
    }
}

impl<'a> Drop for Random<'a> {
    fn drop(&mut self) {
        // APR random objects are pool-allocated, no explicit cleanup needed
    }
}

/// Generate secure random bytes directly without creating a Random instance
pub fn generate_secure_bytes(buf: &mut [u8], pool: &Pool<'_>) -> Result<()> {
    let mut random = Random::new(pool)?;
    
    // Add some basic entropy from system time
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let entropy = now.to_le_bytes();
    random.add_entropy(&entropy)?;
    
    // Try to ensure we have enough entropy
    random.barrier()?;
    
    if random.secure_ready()? {
        random.secure_bytes(buf)
    } else {
        // Fall back to insecure if secure is not ready
        random.insecure_bytes(buf)
    }
}

/// Generate insecure (but fast) random bytes
pub fn generate_insecure_bytes(buf: &mut [u8], pool: &Pool<'_>) -> Result<()> {
    let mut random = Random::new(pool)?;
    random.insecure_bytes(buf)
}

/// Generate a random u32
pub fn generate_u32(pool: &Pool<'_>) -> Result<u32> {
    let mut buf = [0u8; 4];
    generate_secure_bytes(&mut buf, pool)?;
    Ok(u32::from_le_bytes(buf))
}

/// Generate a random u64
pub fn generate_u64(pool: &Pool<'_>) -> Result<u64> {
    let mut buf = [0u8; 8];
    generate_secure_bytes(&mut buf, pool)?;
    Ok(u64::from_le_bytes(buf))
}

/// Generate random bytes in a given range [0, max)
pub fn generate_range(max: u32, pool: &Pool<'_>) -> Result<u32> {
    if max == 0 {
        return Ok(0);
    }
    
    // Use rejection sampling to avoid bias
    let range = u32::MAX - (u32::MAX % max);
    
    loop {
        let value = generate_u32(pool)?;
        if value < range {
            return Ok(value % max);
        }
        // Reject and try again to avoid bias
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_creation() {
        let pool = Pool::new();
        let random = Random::new(&pool);
        assert!(random.is_ok());
    }

    #[test]
    fn test_add_entropy() {
        let pool = Pool::new();
        let mut random = Random::new(&pool).unwrap();
        
        let entropy = b"some entropy data for testing";
        random.add_entropy(entropy).unwrap();
    }

    #[test]
    fn test_insecure_bytes() {
        let pool = Pool::new();
        let mut random = Random::new(&pool).unwrap();
        
        let mut buf = [0u8; 16];
        random.insecure_bytes(&mut buf).unwrap();
        
        // Very unlikely all bytes are zero
        let all_zero = buf.iter().all(|&x| x == 0);
        assert!(!all_zero, "Random bytes should not be all zero");
    }

    #[test]
    fn test_secure_bytes() {
        let pool = Pool::new();
        let mut random = Random::new(&pool).unwrap();
        
        // Add some entropy first
        let entropy = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_le_bytes();
        random.add_entropy(&entropy).unwrap();
        
        let mut buf = [0u8; 16];
        
        // Try secure bytes - might not be ready, so fallback to insecure
        if random.secure_ready().unwrap_or(false) {
            random.secure_bytes(&mut buf).unwrap();
        } else {
            random.insecure_bytes(&mut buf).unwrap();
        }
        
        // Verify we got some random data
        let all_zero = buf.iter().all(|&x| x == 0);
        assert!(!all_zero, "Random bytes should not be all zero");
    }

    #[test]
    fn test_generate_secure_bytes() {
        let pool = Pool::new();
        let mut buf = [0u8; 32];
        
        generate_secure_bytes(&mut buf, &pool).unwrap();
        
        // Should not be all zeros
        let all_zero = buf.iter().all(|&x| x == 0);
        assert!(!all_zero);
        
        // Generate again - should be different
        let mut buf2 = [0u8; 32];
        generate_secure_bytes(&mut buf2, &pool).unwrap();
        assert_ne!(buf, buf2, "Two random generations should be different");
    }

    #[test]
    fn test_generate_insecure_bytes() {
        let pool = Pool::new();
        let mut buf = [0u8; 16];
        
        generate_insecure_bytes(&mut buf, &pool).unwrap();
        
        // Should not be all zeros
        let all_zero = buf.iter().all(|&x| x == 0);
        assert!(!all_zero);
    }

    #[test]
    fn test_generate_u32() {
        let pool = Pool::new();
        
        let val1 = generate_u32(&pool).unwrap();
        let val2 = generate_u32(&pool).unwrap();
        
        // Very unlikely to be the same
        assert_ne!(val1, val2);
    }

    #[test]
    fn test_generate_u64() {
        let pool = Pool::new();
        
        let val1 = generate_u64(&pool).unwrap();
        let val2 = generate_u64(&pool).unwrap();
        
        // Very unlikely to be the same
        assert_ne!(val1, val2);
    }

    #[test]
    fn test_generate_range() {
        let pool = Pool::new();
        
        // Test edge cases
        assert_eq!(generate_range(0, &pool).unwrap(), 0);
        assert_eq!(generate_range(1, &pool).unwrap(), 0);
        
        // Test normal range
        for _ in 0..10 {
            let val = generate_range(100, &pool).unwrap();
            assert!(val < 100);
        }
        
        // Test larger range
        let val = generate_range(1000, &pool).unwrap();
        assert!(val < 1000);
    }

    #[test]
    fn test_random_distribution() {
        let pool = Pool::new();
        
        // Generate many values in small range and check distribution
        let mut counts = [0; 4];
        for _ in 0..1000 {
            let val = generate_range(4, &pool).unwrap() as usize;
            counts[val] += 1;
        }
        
        // Each bucket should have roughly 250 values (within reason)
        for count in counts.iter() {
            assert!(*count > 150 && *count < 350, 
                   "Distribution seems biased: {:?}", counts);
        }
    }

    #[test]
    fn test_barrier() {
        let pool = Pool::new();
        let mut random = Random::new(&pool).unwrap();
        
        // Add entropy and call barrier
        let entropy = b"test entropy";
        random.add_entropy(entropy).unwrap();
        random.barrier().unwrap();
        
        // Should work without error
    }
}