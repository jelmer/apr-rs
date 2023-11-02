//! Rust bindings for the Apache Portable Runtime (APR) library.
//!
//! This crate provides a safe interface to the APR library. It is intended to be used by other
//! crates that need to interface with APR.

mod generated;

pub mod date;
pub mod file;
pub mod hash;
pub mod pool;
pub mod status;
pub mod tables;
pub mod time;
pub mod uri;

pub use pool::Pool;
pub use status::Status;

pub use generated::apr_byte_t;
pub use generated::apr_file_t;
pub use generated::apr_getopt_option_t;
pub use generated::apr_getopt_t;
pub use generated::apr_int16_t;
pub use generated::apr_int32_t;
pub use generated::apr_int64_t;
pub use generated::apr_off_t;
pub use generated::apr_pool_t;
pub use generated::apr_size_t;
pub use generated::apr_status_t;
pub use generated::apr_time_t;
pub use generated::apr_uint16_t;
pub use generated::apr_uint32_t;
pub use generated::apr_uint64_t;

#[ctor::ctor]
fn initialize_apr() {
    unsafe {
        assert!(generated::apr_initialize() == generated::APR_SUCCESS as i32);
    }
}

pub const APU_MAJOR_VERSION: u32 = crate::generated::APU_MAJOR_VERSION;
pub const APU_MINOR_VERSION: u32 = crate::generated::APU_MINOR_VERSION;
pub const APU_PATCH_VERSION: u32 = crate::generated::APU_PATCH_VERSION;

pub fn apu_version_string() -> &'static str {
    unsafe {
        let ptr = crate::generated::apu_version_string();
        std::ffi::CStr::from_ptr(ptr).to_str().unwrap()
    }
}

pub const APR_MAJOR_VERSION: u32 = crate::generated::APR_MAJOR_VERSION;
pub const APR_MINOR_VERSION: u32 = crate::generated::APR_MINOR_VERSION;
pub const APR_PATCH_VERSION: u32 = crate::generated::APR_PATCH_VERSION;

pub fn apr_version_string() -> &'static str {
    unsafe {
        let ptr = crate::generated::apr_version_string();
        std::ffi::CStr::from_ptr(ptr).to_str().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apu_version_string() {
        apu_version_string();
    }

    #[test]
    fn test_apr_version_string() {
        apr_version_string();
    }
}
