//! URI parsing and manipulation.
pub use crate::generated::apr_uri_t;
use crate::pool::{Pool, Pooled};
use std::ffi::CStr;

/// A structure to represent a URI.
pub struct Uri<'pool>(Pooled<'pool, apr_uri_t>);

impl<'pool> Uri<'pool> {
    /// Return the scheme of the URI.
    pub fn scheme(&self) -> Option<&str> {
        unsafe {
            if self.0.scheme.is_null() {
                None
            } else {
                Some(CStr::from_ptr(self.0.scheme).to_str().unwrap())
            }
        }
    }

    /// Return the hostinfo of the URI.
    pub fn hostinfo(&self) -> Option<&str> {
        unsafe {
            if self.0.hostinfo.is_null() {
                None
            } else {
                Some(CStr::from_ptr(self.0.hostinfo).to_str().unwrap())
            }
        }
    }

    /// Return the username of the URI.
    pub fn user(&self) -> Option<&str> {
        unsafe {
            if self.0.user.is_null() {
                None
            } else {
                Some(CStr::from_ptr(self.0.user).to_str().unwrap())
            }
        }
    }

    /// Return the password of the URI.
    pub fn password(&self) -> Option<&str> {
        unsafe {
            if self.0.password.is_null() {
                None
            } else {
                Some(CStr::from_ptr(self.0.password).to_str().unwrap())
            }
        }
    }

    /// Return the hostname of the URI.
    pub fn hostname(&self) -> Option<&str> {
        unsafe {
            if self.0.hostname.is_null() {
                None
            } else {
                Some(CStr::from_ptr(self.0.hostname).to_str().unwrap())
            }
        }
    }

    /// Return the port of the URI.
    pub fn port(&self) -> u16 {
        self.0.port
    }

    /// Return the path of the URI.
    pub fn path(&self) -> Option<&str> {
        unsafe {
            if self.0.path.is_null() {
                None
            } else {
                Some(CStr::from_ptr(self.0.path).to_str().unwrap())
            }
        }
    }

    /// Return the query of the URI.
    pub fn query(&self) -> Option<&str> {
        unsafe {
            if self.0.query.is_null() {
                None
            } else {
                Some(CStr::from_ptr(self.0.query).to_str().unwrap())
            }
        }
    }

    /// Return the fragment of the URI.
    pub fn fragment(&self) -> Option<&str> {
        unsafe {
            if self.0.fragment.is_null() {
                None
            } else {
                Some(CStr::from_ptr(self.0.fragment).to_str().unwrap())
            }
        }
    }

    /// Return the port as a string
    pub fn port_str(&self) -> Option<&str> {
        unsafe {
            if self.0.port_str.is_null() {
                None
            } else {
                Some(CStr::from_ptr(self.0.port_str).to_str().unwrap())
            }
        }
    }

    /// Return whether the URI has been initialized.
    pub fn is_initialized(&self) -> bool {
        self.0.is_initialized() != 0
    }

    /// Return whether the DNS has been looked up.
    pub fn dns_looked_up(&self) -> bool {
        self.0.dns_looked_up() != 0
    }

    /// Return whether the DNS has been resolved.
    pub fn dns_resolved(&self) -> bool {
        self.0.dns_resolved() != 0
    }

    /// Unparse the URI, returning a string.
    pub fn unparse(&self, flags: u32) -> String {
        let pool = crate::Pool::new();
        unsafe {
            CStr::from_ptr(crate::generated::apr_uri_unparse(
                pool.as_mut_ptr(),
                &*self.0,
                flags,
            ))
            .to_str()
            .unwrap()
        }
        .to_string()
    }

    /// Parse a hostinfo string.
    pub fn parse_hostinfo(pool: &'pool Pool, hostinfo: &str) -> Result<Self, crate::Status> {
        let mut uri = pool.calloc::<apr_uri_t>();
        let hostinfo = std::ffi::CString::new(hostinfo).unwrap();
        let status = unsafe {
            crate::generated::apr_uri_parse_hostinfo(
                pool.as_mut_ptr(),
                hostinfo.as_ptr() as *const std::ffi::c_char,
                uri.as_mut_ptr(),
            )
        };
        let status = crate::Status::from(status);
        if status.is_success() {
            Ok(Uri(uri))
        } else {
            Err(status)
        }
    }

    /// Parse a URI string.
    pub fn parse(pool: &'pool Pool, url: &str) -> Result<Self, crate::Status> {
        let mut uri = pool.calloc::<apr_uri_t>();
        let url = std::ffi::CString::new(url).unwrap();
        let status = unsafe {
            crate::generated::apr_uri_parse(
                pool.as_mut_ptr(),
                url.as_ptr() as *const std::ffi::c_char,
                uri.as_mut_ptr(),
            )
        };
        let status = crate::Status::from(status);
        if status.is_success() {
            Ok(Uri(uri))
        } else {
            Err(status)
        }
    }
}

/// Return the default port for a given scheme.
pub fn port_of_scheme(scheme: &str) -> u16 {
    let scheme = std::ffi::CString::new(scheme).unwrap();
    unsafe { crate::generated::apr_uri_port_of_scheme(scheme.as_ptr() as *const std::ffi::c_char) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_port_of_scheme() {
        assert_eq!(80, super::port_of_scheme("http"));
        assert_eq!(443, super::port_of_scheme("https"));
        assert_eq!(0, super::port_of_scheme("unknown"));
    }

    #[test]
    fn test_parse() {
        let pool = Pool::new();
        let uri = super::Uri::parse(&pool, "http://example.com:8080/").unwrap();
        assert_eq!("http", uri.scheme().unwrap());
        assert_eq!(Some("example.com:8080"), uri.hostinfo());
        assert_eq!(Some("example.com"), uri.hostname());
        assert_eq!(8080, uri.port());
        assert_eq!(Some("/"), uri.path());
        assert_eq!(None, uri.query());
        assert_eq!(None, uri.fragment());
        assert_eq!(Some("8080"), uri.port_str());
        assert!(uri.is_initialized());
        assert!(!uri.dns_looked_up());
        assert!(!uri.dns_resolved());
    }

    #[test]
    fn test_parse_hostinfo() {
        let pool = Pool::new();
        let uri = super::Uri::parse_hostinfo(&pool, "example.com:8080").unwrap();
        assert_eq!(None, uri.scheme());
        assert_eq!(Some("example.com:8080"), uri.hostinfo());
        assert_eq!(Some("example.com"), uri.hostname());
        assert_eq!(8080, uri.port());
        assert_eq!(None, uri.path());
        assert_eq!(None, uri.query());
        assert_eq!(None, uri.fragment());
        assert_eq!(Some("8080"), uri.port_str());
        assert!(uri.is_initialized());
        assert!(!uri.dns_looked_up());
        assert!(!uri.dns_resolved());
    }
}

// TODO(jelmer): Rather than serializing/deserializing, we should be able to just copy the fields
// over.
#[cfg(feature = "url")]
impl From<url::Url> for Uri {
    fn from(url: url::Url) -> Self {
        let s = url.as_str();
        Self::parse(s).unwrap()
    }
}

#[cfg(feature = "url")]
impl From<Uri> for url::Url {
    fn from(uri: Uri) -> Self {
        let s = uri.unparse(0);
        url::Url::parse(&s).unwrap()
    }
}
