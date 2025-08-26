//! Time handling.
pub use apr_sys::{apr_interval_time_t, apr_time_t};

/// Time in microseconds since the epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time(apr_time_t);

impl Time {
    /// Returns the current time.
    pub fn now() -> Self {
        let time = unsafe { apr_sys::apr_time_now() };
        Self(time)
    }

    /// Create a Time from microseconds since the Unix epoch.
    pub fn from_micros(micros: apr_time_t) -> Self {
        Self(micros)
    }

    /// Get the time as microseconds since the Unix epoch.
    pub fn as_micros(&self) -> apr_time_t {
        self.0
    }

    /// Returns the time as a string in the format `Sun Nov 06 08:49:37 1994`.
    pub fn ctime(&self) -> String {
        let mut buf: [u8; apr_sys::APR_CTIME_LEN as usize] =
            [0; apr_sys::APR_CTIME_LEN as usize];
        unsafe {
            apr_sys::apr_ctime(buf.as_mut_ptr() as *mut std::ffi::c_char, self.0);
        }
        String::from_utf8_lossy(&buf[..])
            .trim_end_matches('\0')
            .to_string()
    }

    /// Returns the time as a string in the format `Sun, 06 Nov 1994 08:49:37 GMT`.
    pub fn rfc822_date(&self) -> String {
        let mut buf: [u8; apr_sys::APR_RFC822_DATE_LEN as usize] =
            [0; apr_sys::APR_RFC822_DATE_LEN as usize];
        unsafe {
            apr_sys::apr_rfc822_date(buf.as_mut_ptr() as *mut std::ffi::c_char, self.0);
        }
        String::from_utf8_lossy(&buf[..])
            .trim_end_matches('\0')
            .to_string()
    }
}

/// Convert SystemTime to apr_time_t (microseconds since Unix epoch)
pub fn to_apr_time(system_time: std::time::SystemTime) -> apr_time_t {
    system_time
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as apr_time_t
}

/// Convert apr_time_t to SystemTime
pub fn to_system_time(apr_time: apr_time_t) -> std::time::SystemTime {
    std::time::UNIX_EPOCH + std::time::Duration::from_micros(apr_time as u64)
}

impl From<std::time::SystemTime> for Time {
    fn from(system_time: std::time::SystemTime) -> Self {
        Self(to_apr_time(system_time))
    }
}

impl From<Time> for std::time::SystemTime {
    fn from(time: Time) -> Self {
        to_system_time(time.0)
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

/// Sleep for the given interval.
pub fn sleep(interval: Interval) {
    unsafe {
        apr_sys::apr_sleep(interval);
    }
}

/// Trait for types that can be converted into a `Time`.
pub trait IntoTime {
    /// Converts the value into a `Time`.
    fn as_apr_time(&self) -> Time;
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

    #[test]
    fn test_system_time_conversion() {
        use std::time::{Duration, SystemTime};

        // Test converting from SystemTime to Time and back
        let system_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1234567890);
        let apr_time = Time::from(system_time);
        let converted_back: SystemTime = apr_time.into();

        // Should be equal (allowing for microsecond precision)
        let diff = converted_back
            .duration_since(system_time)
            .unwrap_or_else(|_| system_time.duration_since(converted_back).unwrap());
        assert!(diff < Duration::from_millis(1));
    }

    #[test]
    fn test_utility_functions() {
        use std::time::{Duration, SystemTime};

        let system_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
        let apr_time = to_apr_time(system_time);
        let converted_back = to_system_time(apr_time);

        let diff = converted_back
            .duration_since(system_time)
            .unwrap_or_else(|_| system_time.duration_since(converted_back).unwrap());
        assert!(diff < Duration::from_millis(1));
    }
}
