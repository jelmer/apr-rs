//! Rust bindings for the Apache Portable Runtime (APR) library.
//!
//! This crate provides a safe interface to the APR library. It is intended to be used by other
//! crates that need to interface with APR.

pub mod callbacks;
pub mod date;
pub mod error;
pub mod file;
pub mod getopt;
pub mod hash;
pub mod mmap;
pub mod network;
pub mod paths;
pub mod pool;
pub mod status;
pub mod strings;
pub mod tables;
pub mod time;
pub mod uri;
pub mod versions;

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
