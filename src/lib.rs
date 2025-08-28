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

#[ctor::ctor]
fn init() {
    unsafe {
        apr_sys::apr_initialize();
    }
}
