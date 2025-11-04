//! UUID generation functionality from apr-util.

use crate::{Error, Status};
use std::ffi::c_char;
use std::ffi::CStr;
use std::fmt;

/// A universally unique identifier (UUID).
#[derive(Clone, Copy)]
pub struct Uuid {
    uuid: apr_sys::apr_uuid_t,
}

impl Uuid {
    /// Generate a new random UUID.
    pub fn new() -> Self {
        let mut uuid = apr_sys::apr_uuid_t { data: [0; 16] };
        unsafe {
            apr_sys::apr_uuid_get(&mut uuid);
        }
        Uuid { uuid }
    }

    /// Parse a UUID from a string representation.
    ///
    /// The string should be in the standard format:
    /// "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
    pub fn parse(s: &str) -> Result<Self, Error> {
        let c_str = std::ffi::CString::new(s)
            .map_err(|_| Error::from_status(Status::from(apr_sys::APR_EINVAL as i32)))?;

        let mut uuid = apr_sys::apr_uuid_t { data: [0; 16] };

        let status = unsafe { apr_sys::apr_uuid_parse(&mut uuid, c_str.as_ptr()) };

        if status == apr_sys::APR_SUCCESS as i32 {
            Ok(Uuid { uuid })
        } else {
            Err(Error::from_status(Status::from(status)))
        }
    }

    /// Format the UUID as a standard string representation.
    pub fn format(&self) -> String {
        let mut buffer = vec![0u8; 37]; // 36 chars + null terminator
        unsafe {
            apr_sys::apr_uuid_format(buffer.as_mut_ptr() as *mut c_char, &self.uuid);
        }
        let c_str = unsafe { CStr::from_ptr(buffer.as_ptr() as *const c_char) };
        c_str.to_string_lossy().into_owned()
    }

    /// Get the raw bytes of the UUID.
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.uuid.data
    }

    /// Create a UUID from raw bytes.
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Uuid {
            uuid: apr_sys::apr_uuid_t { data: bytes },
        }
    }
}

impl PartialEq for Uuid {
    fn eq(&self, other: &Self) -> bool {
        self.uuid.data == other.uuid.data
    }
}

impl Eq for Uuid {}

impl std::hash::Hash for Uuid {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.data.hash(state);
    }
}

impl Default for Uuid {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

impl fmt::Debug for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Uuid({})", self.format())
    }
}

impl std::str::FromStr for Uuid {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_uuid_generate() {
        let uuid1 = Uuid::new();
        let uuid2 = Uuid::new();

        // UUIDs should be unique
        assert_ne!(uuid1, uuid2);
    }

    #[test]
    fn test_uuid_format() {
        let uuid = Uuid::new();
        let formatted = uuid.format();

        // Check format: 8-4-4-4-12 hex digits with dashes
        assert_eq!(formatted.len(), 36);
        assert_eq!(formatted.chars().nth(8), Some('-'));
        assert_eq!(formatted.chars().nth(13), Some('-'));
        assert_eq!(formatted.chars().nth(18), Some('-'));
        assert_eq!(formatted.chars().nth(23), Some('-'));
    }

    #[test]
    fn test_uuid_parse_valid() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let uuid = Uuid::parse(uuid_str).unwrap();
        assert_eq!(uuid.format().to_lowercase(), uuid_str.to_lowercase());
    }

    #[test]
    fn test_uuid_parse_invalid() {
        assert!(Uuid::parse("not-a-uuid").is_err());
        assert!(Uuid::parse("550e8400-e29b-41d4-a716").is_err()); // Too short
        assert!(Uuid::parse("550e8400-e29b-41d4-a716-446655440000-extra").is_err());
        // Too long
    }

    #[test]
    fn test_uuid_round_trip() {
        let uuid1 = Uuid::new();
        let formatted = uuid1.format();
        let uuid2 = Uuid::parse(&formatted).unwrap();
        assert_eq!(uuid1, uuid2);
    }

    #[test]
    fn test_uuid_from_str() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let uuid: Uuid = uuid_str.parse().unwrap();
        assert_eq!(uuid.to_string().to_lowercase(), uuid_str.to_lowercase());
    }

    #[test]
    fn test_uuid_bytes() {
        let bytes = [
            0x55, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4, 0xa7, 0x16, 0x44, 0x66, 0x55, 0x44,
            0x00, 0x00,
        ];
        let uuid = Uuid::from_bytes(bytes);
        assert_eq!(uuid.as_bytes(), &bytes);
    }

    #[test]
    fn test_uuid_uniqueness() {
        let mut uuids = HashSet::new();
        for _ in 0..100 {
            let uuid = Uuid::new();
            assert!(uuids.insert(uuid.format()));
        }
        assert_eq!(uuids.len(), 100);
    }

    #[test]
    fn test_uuid_display_debug() {
        let uuid = Uuid::new();
        let display = format!("{}", uuid);
        let debug = format!("{:?}", uuid);

        assert_eq!(display, uuid.format());
        assert!(debug.starts_with("Uuid("));
        assert!(debug.ends_with(")"));
    }
}
