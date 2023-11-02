pub use crate::generated::{apr_interval_time_t, apr_time_t};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time(apr_time_t);

impl Time {
    pub fn now() -> Self {
        let time = unsafe { crate::generated::apr_time_now() };
        Self(time)
    }

    pub fn ctime(&self) -> String {
        let mut buf: [u8; crate::generated::APR_CTIME_LEN as usize] =
            [0; crate::generated::APR_CTIME_LEN as usize];
        unsafe {
            crate::generated::apr_ctime(buf.as_mut_ptr() as *mut i8, self.0);
        }
        String::from_utf8_lossy(&buf[..])
            .trim_end_matches('\0')
            .to_string()
    }

    pub fn rfc822_date(&self) -> String {
        let mut buf: [u8; crate::generated::APR_RFC822_DATE_LEN as usize] =
            [0; crate::generated::APR_RFC822_DATE_LEN as usize];
        unsafe {
            crate::generated::apr_rfc822_date(buf.as_mut_ptr() as *mut i8, self.0);
        }
        String::from_utf8_lossy(&buf[..])
            .trim_end_matches('\0')
            .to_string()
    }
}

impl From<Time> for apr_time_t {
    fn from(time: Time) -> Self {
        time.0
    }
}

impl From<apr_time_t> for Time {
    fn from(time: apr_time_t) -> Self {
        Self(time)
    }
}

type Interval = apr_interval_time_t;

pub fn sleep(interval: Interval) {
    unsafe {
        crate::generated::apr_sleep(interval);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_now() {
        Time::now();
    }

    #[test]
    fn test_ctime() {
        let t = Time::from(784111777000000);
        assert_eq!(t.ctime(), "Sun Nov 06 08:49:37 1994");
    }

    #[test]
    fn test_rfc822_date() {
        let t = Time::from(784111777000000);
        assert_eq!(t.rfc822_date(), "Sun, 06 Nov 1994 08:49:37 GMT");
    }
}
