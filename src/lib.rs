//! Rust bindings for the Apache Portable Runtime (APR) library.
//!
//! This crate provides a safe interface to the APR library. It is intended to be used by other
//! crates that need to interface with APR.

pub mod base64;
pub mod callbacks;
pub mod crypto;
pub mod date;
pub mod error;
pub mod file;
pub mod getopt;
pub mod hash;
pub mod md5;
pub mod mmap;
pub mod network;
pub mod paths;
pub mod pool;
pub mod sha1;
pub mod status;
pub mod strings;
pub mod strmatch;
pub mod tables;
pub mod time;
pub mod uri;
pub mod uuid;
pub mod versions;
pub mod xlate;
pub mod xml;

pub use error::{Error, ErrorContext, Result};
pub use pool::Pool;
pub use status::Status;

// Only re-export types that are commonly needed
pub use apr_sys::apr_status_t;
pub use apr_sys::apr_time_t;

/// Create an APR array with initial values.
///
/// # Examples
/// ```
/// # use apr::{Pool, apr_array};
/// let pool = Pool::new();
/// let arr = apr_array![&pool; 1, 2, 3, 4];
/// assert_eq!(arr.len(), 4);
/// ```
#[macro_export]
macro_rules! apr_array {
    ($pool:expr; $($item:expr),* $(,)?) => {{
        let mut array = $crate::tables::ArrayHeader::new($pool);
        $(
            array.push($item);
        )*
        array
    }};
}

/// Create an APR table with initial key-value pairs.
///
/// # Examples  
/// ```
/// # use apr::{Pool, apr_table};
/// let pool = Pool::new();
/// let table = apr_table![&pool; "key1" => "value1", "key2" => "value2"];
/// assert_eq!(table.get("key1"), Some("value1"));
/// ```
#[macro_export]
macro_rules! apr_table {
    ($pool:expr; $($key:expr => $value:expr),* $(,)?) => {{
        let mut table = $crate::tables::Table::new($pool);
        $(
            table.insert($key, $value);
        )*
        table
    }};
}

/// Create an APR hash with initial key-value pairs.
///
/// # Examples
/// ```
/// # use apr::{Pool, apr_hash};
/// let pool = Pool::new();
/// let hash = apr_hash![&pool; "key1" => &"value1", "key2" => &"value2"];
/// assert_eq!(hash.get("key1"), Some(&"value1"));
/// ```
#[macro_export]
macro_rules! apr_hash {
    ($pool:expr; $($key:expr => $value:expr),* $(,)?) => {{
        let mut hash = $crate::hash::Hash::new($pool);
        $(
            hash.insert($key, $value);
        )*
        hash
    }};
}

#[ctor::ctor]
fn init() {
    unsafe {
        apr_sys::apr_initialize();
    }
}
