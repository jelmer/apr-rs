//! Version information for the Apache Portable Runtime (APR) and Apache Portable Utility (APU) libraries.

/// The major version of the Apache Portable Runtime (APR) library.
pub const APU_MAJOR_VERSION: u32 = apr_sys::APU_MAJOR_VERSION;

/// The minor version of the Apache Portable Runtime (APR) library.
pub const APU_MINOR_VERSION: u32 = apr_sys::APU_MINOR_VERSION;

/// The patch version of the Apache Portable Runtime (APR) library.
pub const APU_PATCH_VERSION: u32 = apr_sys::APU_PATCH_VERSION;

/// The version string of the Apache Portable Utility (APU) library.
pub fn apu_version_string() -> &'static str {
    unsafe {
        let ptr = apr_sys::apu_version_string();
        core::ffi::CStr::from_ptr(ptr).to_str().unwrap()
    }
}

/// The major version of the Apache Portable Runtime (APR) library.
pub const APR_MAJOR_VERSION: u32 = apr_sys::APR_MAJOR_VERSION;

/// The minor version of the Apache Portable Runtime (APR) library.
pub const APR_MINOR_VERSION: u32 = apr_sys::APR_MINOR_VERSION;

/// The patch version of the Apache Portable Runtime (APR) library.
pub const APR_PATCH_VERSION: u32 = apr_sys::APR_PATCH_VERSION;

/// The version string of the Apache Portable Runtime (APR) library.
pub fn apr_version_string() -> &'static str {
    unsafe {
        let ptr = apr_sys::apr_version_string();
        core::ffi::CStr::from_ptr(ptr).to_str().unwrap()
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
