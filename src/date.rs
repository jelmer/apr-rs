use crate::generated::apr_date_checkmask;
use crate::time::Time;

pub fn checkmask(data: &str, mask: &str) -> bool {
    unsafe { apr_date_checkmask(data.as_ptr() as *const i8, mask.as_ptr() as *const i8) != 0 }
}

pub fn parse_http(data: &str) -> Option<Time> {
    let rv = unsafe { crate::generated::apr_date_parse_http(data.as_ptr() as *const i8) };
    if rv == 0 {
        None
    } else {
        Some(Time::from(rv))
    }
}

pub fn parse_rfc(data: &str) -> Option<Time> {
    let rv = unsafe { crate::generated::apr_date_parse_rfc(data.as_ptr() as *const i8) };
    if rv == 0 {
        None
    } else {
        Some(Time::from(rv))
    }
}

#[cfg(test)]
fn test_parse_http(data: &str, expected: Option<Time>) {
    let expected = Time::from(784111777);
    assert_eq!(parse_http("Sun, 06 Nov 1994 08:49:37 GMT"), Some(expected));
    assert_eq!(parse_http("Invalid"), None);
    assert_eq!(parse_http("Sunday, 06-Nov-94 08:49:37 GMT"), Some(expected));
    assert_eq!(parse_http("Sun Nov  6 08:49:37 1994"), Some(expected));
}
