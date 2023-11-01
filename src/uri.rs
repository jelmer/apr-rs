pub use crate::generated::apr_uri_t;
use std::ffi::CStr;

pub struct Uri(*mut apr_uri_t);

impl Uri {
    pub fn scheme(&self) -> &str {
        unsafe { CStr::from_ptr((*self.0).scheme).to_str().unwrap() }
    }

    pub fn hostinfo(&self) -> &str {
        unsafe { CStr::from_ptr((*self.0).hostinfo).to_str().unwrap() }
    }

    pub fn user(&self) -> &str {
        unsafe { CStr::from_ptr((*self.0).user).to_str().unwrap() }
    }

    pub fn password(&self) -> &str {
        unsafe { CStr::from_ptr((*self.0).password).to_str().unwrap() }
    }

    pub fn hostname(&self) -> &str {
        unsafe { CStr::from_ptr((*self.0).hostname).to_str().unwrap() }
    }

    pub fn port(&self) -> u16 {
        unsafe { (*self.0).port }
    }

    pub fn path(&self) -> &str {
        unsafe { CStr::from_ptr((*self.0).path).to_str().unwrap() }
    }

    pub fn query(&self) -> &str {
        unsafe { CStr::from_ptr((*self.0).query).to_str().unwrap() }
    }

    pub fn fragment(&self) -> &str {
        unsafe { CStr::from_ptr((*self.0).fragment).to_str().unwrap() }
    }

    pub fn port_str(&self) -> &str {
        unsafe { CStr::from_ptr((*self.0).port_str).to_str().unwrap() }
    }

    pub fn is_initialized(&self) -> bool {
        unsafe { (*self.0).is_initialized() != 0 }
    }

    pub fn dns_looked_up(&self) -> bool {
        unsafe { (*self.0).dns_looked_up() != 0 }
    }

    pub fn dns_resolved(&self) -> bool {
        unsafe { (*self.0).dns_resolved() != 0 }
    }

    pub fn unparse(&self, flags: u32) -> String {
        let mut pool = crate::Pool::new();
        unsafe {
            CStr::from_ptr(crate::generated::apr_uri_unparse(
                (&mut pool).into(),
                self.0,
                flags,
            ))
            .to_str()
            .unwrap()
        }
        .to_string()
    }

    pub fn parse_hostinfo(pool: &mut crate::Pool, hostinfo: &str) -> Result<Self, crate::Status> {
        let mut uri = pool.alloc::<apr_uri_t>();
        unsafe {
            let hostinfo = std::ffi::CStr::from_ptr(hostinfo.as_ptr() as *const i8);
            let status = crate::generated::apr_uri_parse_hostinfo(
                pool.into(),
                hostinfo.as_ptr() as *const i8,
                &mut uri as *mut _ as *mut _,
            );
            let status = crate::Status::from(status);
            if status.is_success() {
                Ok(Uri(uri as *mut _))
            } else {
                Err(status)
            }
        }
    }

    pub fn parse(pool: &mut crate::Pool, url: &str) -> Result<Self, crate::Status> {
        let mut uri = pool.alloc::<apr_uri_t>();
        unsafe {
            let url = std::ffi::CStr::from_ptr(url.as_ptr() as *const i8);
            let status = crate::generated::apr_uri_parse(
                pool.into(),
                url.as_ptr() as *const i8,
                &mut uri as *mut _ as *mut _,
            );
            let status = crate::Status::from(status);
            if status.is_success() {
                Ok(Uri(uri as *mut _))
            } else {
                Err(status)
            }
        }
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
}
