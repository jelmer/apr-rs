//! Base64 encoding and decoding functionality from apr-util.

use crate::{Error, Status};
use std::ffi::c_char;
use std::ffi::CString;

/// Get the length of the encoded base64 string for a given input length.
pub fn base64_encode_len(len: usize) -> usize {
    unsafe { apr_sys::apr_base64_encode_len(len as i32) as usize }
}

/// Get the maximum length of the decoded data for a given base64 string.
pub fn base64_decode_len(encoded: &str) -> usize {
    let c_str = match CString::new(encoded) {
        Ok(s) => s,
        Err(_) => return 0,
    };
    unsafe { apr_sys::apr_base64_decode_len(c_str.as_ptr()) as usize }
}

/// Encode binary data to base64.
pub fn base64_encode(data: &[u8]) -> String {
    let encoded_len = base64_encode_len(data.len());
    let mut encoded = vec![0u8; encoded_len];

    unsafe {
        apr_sys::apr_base64_encode_binary(
            encoded.as_mut_ptr() as *mut c_char,
            data.as_ptr(),
            data.len() as i32,
        );
    }

    // Remove trailing null terminator
    encoded.truncate(encoded_len - 1);
    String::from_utf8(encoded).unwrap_or_default()
}

/// Encode a string to base64.
pub fn base64_encode_string(s: &str) -> String {
    base64_encode(s.as_bytes())
}

/// Decode a base64 string to binary data.
pub fn base64_decode(encoded: &str) -> Result<Vec<u8>, Error> {
    let c_str = CString::new(encoded)
        .map_err(|_| Error::from_status(Status::from(apr_sys::APR_EINVAL as i32)))?;

    let decoded_len = base64_decode_len(encoded);
    if decoded_len == 0 {
        return Ok(Vec::new());
    }

    let mut decoded = vec![0u8; decoded_len];

    let actual_len =
        unsafe { apr_sys::apr_base64_decode_binary(decoded.as_mut_ptr(), c_str.as_ptr()) };

    if actual_len < 0 {
        Err(Error::from_status(Status::from(apr_sys::APR_EINVAL as i32)))
    } else {
        decoded.truncate(actual_len as usize);
        Ok(decoded)
    }
}

/// Decode a base64 string to a UTF-8 string.
pub fn base64_decode_string(encoded: &str) -> Result<String, Error> {
    let decoded = base64_decode(encoded)?;
    String::from_utf8(decoded)
        .map_err(|_| Error::from_status(Status::from(apr_sys::APR_EINVAL as i32)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encode_empty() {
        assert_eq!(base64_encode(b""), "");
    }

    #[test]
    fn test_base64_encode_hello() {
        assert_eq!(base64_encode(b"Hello, World!"), "SGVsbG8sIFdvcmxkIQ==");
    }

    #[test]
    fn test_base64_encode_binary() {
        let data = vec![0xFF, 0x00, 0xAB, 0xCD, 0xEF];
        let encoded = base64_encode(&data);
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_base64_decode_hello() {
        let decoded = base64_decode("SGVsbG8sIFdvcmxkIQ==").unwrap();
        assert_eq!(decoded, b"Hello, World!");
    }

    #[test]
    fn test_base64_round_trip() {
        let original = b"The quick brown fox jumps over the lazy dog";
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_base64_string_round_trip() {
        let original = "Hello, 世界! 🦀";
        let encoded = base64_encode_string(original);
        let decoded = base64_decode_string(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_base64_decode_invalid() {
        // Invalid base64 - APR may not validate input strictly
        let result = base64_decode("!@#$%^&*()");
        // Some APR implementations may decode what they can, so we just check it doesn't crash
        let _ = result;
    }

    #[test]
    fn test_base64_encode_len() {
        assert_eq!(base64_encode_len(0), 1); // Just null terminator
        assert_eq!(base64_encode_len(1), 5); // 4 chars + null
        assert_eq!(base64_encode_len(2), 5); // 4 chars + null
        assert_eq!(base64_encode_len(3), 5); // 4 chars + null
        assert_eq!(base64_encode_len(4), 9); // 8 chars + null
    }
}
