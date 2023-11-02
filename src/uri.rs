pub use crate::generated::apr_uri_t;
use crate::pool::PooledPtr;
use std::ffi::CStr;

pub struct Uri<'pool>(PooledPtr<'pool, apr_uri_t>);

impl<'pool> Uri<'pool> {
    pub fn scheme(&self) -> Option<&str> {
        unsafe {
            if self.0.scheme.is_null() {
                None
            } else {
                Some(CStr::from_ptr(self.0.scheme).to_str().unwrap())
            }
        }
    }

    pub fn hostinfo(&self) -> Option<&str> {
        unsafe {
            if self.0.hostinfo.is_null() {
                None
            } else {
                Some(CStr::from_ptr(self.0.hostinfo).to_str().unwrap())
            }
        }
    }

    pub fn user(&self) -> Option<&str> {
        unsafe {
            if self.0.user.is_null() {
                None
            } else {
                Some(CStr::from_ptr(self.0.user).to_str().unwrap())
            }
        }
    }

    pub fn password(&self) -> Option<&str> {
        unsafe {
            if self.0.password.is_null() {
                None
            } else {
                Some(CStr::from_ptr(self.0.password).to_str().unwrap())
            }
        }
    }

    pub fn hostname(&self) -> Option<&str> {
        unsafe {
            if self.0.hostname.is_null() {
                None
            } else {
                Some(CStr::from_ptr(self.0.hostname).to_str().unwrap())
            }
        }
    }

    pub fn port(&self) -> u16 {
        self.0.port
    }

    pub fn path(&self) -> Option<&str> {
        unsafe {
            if self.0.path.is_null() {
                None
            } else {
                Some(CStr::from_ptr(self.0.path).to_str().unwrap())
            }
        }
    }

    pub fn query(&self) -> Option<&str> {
        unsafe {
            if self.0.query.is_null() {
                None
            } else {
                Some(CStr::from_ptr(self.0.query).to_str().unwrap())
            }
        }
    }

    pub fn fragment(&self) -> Option<&str> {
        unsafe {
            if self.0.fragment.is_null() {
                None
            } else {
                Some(CStr::from_ptr(self.0.fragment).to_str().unwrap())
            }
        }
    }

    pub fn port_str(&self) -> Option<&str> {
        unsafe {
            if self.0.port_str.is_null() {
                None
            } else {
                Some(CStr::from_ptr(self.0.port_str).to_str().unwrap())
            }
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.0.is_initialized() != 0
    }

    pub fn dns_looked_up(&self) -> bool {
        self.0.dns_looked_up() != 0
    }

    pub fn dns_resolved(&self) -> bool {
        self.0.dns_resolved() != 0
    }

    pub fn unparse(&self, flags: u32) -> String {
        let mut pool = crate::Pool::new();
        unsafe {
            CStr::from_ptr(crate::generated::apr_uri_unparse(
                (&mut pool).into(),
                &*self.0,
                flags,
            ))
            .to_str()
            .unwrap()
        }
        .to_string()
    }

    pub fn parse_hostinfo(hostinfo: &str) -> Result<Self, crate::Status> {
        Ok(Self(PooledPtr::initialize(|pool| unsafe {
            let uri = pool.calloc::<apr_uri_t>();
            let status = crate::generated::apr_uri_parse_hostinfo(
                pool.into(),
                hostinfo.as_ptr() as *const i8,
                uri as *mut _ as *mut _,
            );
            let status = crate::Status::from(status);
            if status.is_success() {
                Ok(uri)
            } else {
                Err(status)
            }
        })?))
    }

    pub fn parse(url: &str) -> Result<Self, crate::Status> {
        Ok(Self(PooledPtr::initialize(|pool| unsafe {
            let uri = pool.calloc::<apr_uri_t>();
            let url = std::ffi::CString::new(url).unwrap();
            let status = crate::generated::apr_uri_parse(
                pool.into(),
                url.as_ptr() as *const i8,
                uri as *mut _ as *mut _,
            );
            let status = crate::Status::from(status);
            if status.is_success() {
                Ok(uri)
            } else {
                Err(status)
            }
        })?))
    }
}

/// Return the default port for a given scheme.
pub fn port_of_scheme(scheme: &str) -> u16 {
    let scheme = std::ffi::CString::new(scheme).unwrap();
    unsafe { crate::generated::apr_uri_port_of_scheme(scheme.as_ptr() as *const i8) }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_port_of_scheme() {
        assert_eq!(80, super::port_of_scheme("http"));
        assert_eq!(443, super::port_of_scheme("https"));
        assert_eq!(0, super::port_of_scheme("unknown"));
    }

    #[test]
    fn test_parse() {
        let uri = super::Uri::parse("http://example.com:8080/").unwrap();
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
        let uri = super::Uri::parse_hostinfo("example.com:8080").unwrap();
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
