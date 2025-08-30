//! Safe Rust bindings for the Apache Portable Runtime (APR) library.
//!
//! This crate provides safe Rust abstractions over the Apache Portable Runtime (APR) and 
//! APR-Util C libraries. APR is a portability layer that provides a predictable and 
//! consistent interface to underlying platform-specific implementations.
//!
//! # Primary Use Case
//!
//! **This crate is primarily useful when developing Rust bindings for C libraries that 
//! depend on APR.** Many Apache projects and other C libraries use APR for cross-platform 
//! compatibility and memory management. If you're creating Rust bindings for such libraries, 
//! this crate provides the necessary APR functionality with a safe Rust interface.
//!
//! # Core Concepts
//!
//! ## Memory Pools
//!
//! APR uses a hierarchical memory pool system for all memory allocation. This is fundamental
//! to how APR and APR-based libraries work:
//!
//! ```no_run
//! use apr::Pool;
//!
//! // Create a root pool
//! let pool = Pool::new();
//! 
//! // Create a subpool for scoped allocations
//! let subpool = pool.create_subpool().unwrap();
//! // Memory in subpool is freed when subpool is dropped
//! ```
//!
//! ## Error Handling
//!
//! APR functions return status codes that this crate converts to Rust `Result` types:
//!
//! ```no_run
//! use apr::{Pool, file::File};
//!
//! let pool = Pool::new();
//! match File::open("example.txt", apr::file::Flag::READ, 0, &pool) {
//!     Ok(file) => { /* use file */ },
//!     Err(e) => eprintln!("Failed to open file: {}", e),
//! }
//! ```
//!
//! # Interfacing with C Libraries
//!
//! When working with C libraries that use APR, you'll often need to pass raw APR pointers:
//!
//! ```no_run
//! use apr::{Pool, Status};
//!
//! extern "C" {
//!     fn some_apr_function(pool: *mut apr_sys::apr_pool_t) -> apr_sys::apr_status_t;
//! }
//!
//! let pool = Pool::new();
//! let status = unsafe {
//!     Status::from(some_apr_function(pool.as_mut_ptr()))
//! };
//! ```
//!
//! # Module Overview
//!
//! - [`pool`] - Memory pool management (fundamental to APR)
//! - [`error`] - Error types and status code handling
//! - [`file`] - File I/O operations
//! - [`network`] - Network I/O and socket operations
//! - [`hash`] - Hash table implementation
//! - [`tables`] - Ordered key-value pairs
//! - [`strings`] - String manipulation utilities
//! - [`time`] - Time handling and formatting
//! - [`crypto`] - Cryptographic functions (MD5, SHA1)
//! - [`base64`] - Base64 encoding/decoding
//! - [`uri`] - URI parsing and manipulation
//! - [`uuid`] - UUID generation
//! - [`xml`] - XML parsing utilities
//!
//! # Safety
//!
//! This crate aims to provide safe abstractions, but when interfacing with C:
//! - Some operations require `unsafe` blocks for raw pointer handling
//! - APR initialization is handled automatically via Rust's runtime
//! - Memory pools ensure proper cleanup when dropped
//! - The crate leverages Rust's ownership system for resource management

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
pub mod queue;
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
