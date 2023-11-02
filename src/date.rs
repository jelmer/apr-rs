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

#[test]
fn test_parse_http() {
    let expected = Time::from(784111777000000);
    assert_eq!(parse_http("WTAF"), None);
    assert_eq!(parse_http("Sun, 06 Nov 1994 08:49:37 GMT"), Some(expected));
    assert_eq!(parse_http("Sunday, 06-Nov-94 08:49:37 GMT"), Some(expected));
    assert_eq!(parse_http("Sun Nov  6 08:49:37 1994"), Some(expected));
}

#[test]
fn test_parse_rfc() {
    let expected = Time::from(784111777000000);
    assert_eq!(parse_rfc("Sun, 06 Nov 1994 08:49:37 GMT"), Some(expected)); //  RFC 822, updated by RFC 1123
    assert_eq!(parse_rfc("Sunday, 06-Nov-94 08:49:37 GMT"), Some(expected)); // RFC 850, obsoleted by RFC 1036
    assert_eq!(parse_rfc("Sun Nov  6 08:49:37 1994"), Some(expected)); // ANSI C's asctime() format
    assert_eq!(parse_rfc("Sun, 6 Nov 1994 08:49:37 GMT"), Some(expected)); // RFC 822, updated by RFC 1123
    assert_eq!(parse_rfc("Sun, 06 Nov 94 08:49:37 GMT"), Some(expected)); // RFC 822
    assert_eq!(parse_rfc("Sun, 6 Nov 94 08:49:37 GMT"), Some(expected)); // RFC 822
    let expected_without_secs = Time::from(784111740000000);
    assert_eq!(
        parse_rfc("Sun, 06 Nov 94 08:49 GMT"),
        Some(expected_without_secs)
    ); // Unknown [drtr\@ast.cam.ac.uk]
    assert_eq!(
        parse_rfc("Sun, 6 Nov 94 08:49 GMT"),
        Some(expected_without_secs)
    ); // Unknown [drtr\@ast.cam.ac.uk]
    assert_eq!(parse_rfc("Sun, 06 Nov 94 8:49:37 GMT"), Some(expected)); // Unknown [Elm 70.85]
    assert_eq!(parse_rfc("Sun, 6 Nov 94 8:49:37 GMT"), Some(expected)); // Unknown [Elm 70.85]
}
