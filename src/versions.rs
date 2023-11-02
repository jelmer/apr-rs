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
