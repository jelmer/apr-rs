//! Rust bindings for the Apache Portable Runtime (APR) library.
//!
//! This crate provides a safe interface to the APR library. It is intended to be used by other
//! crates that need to interface with APR.

mod generated;

pub mod date;
pub mod file;
pub mod getopt;
pub mod hash;
pub mod pool;
pub mod status;
pub mod tables;
pub mod time;
pub mod uri;
pub mod versions;

pub use pool::Pool;
pub use status::Status;

pub use generated::apr_byte_t;
pub use generated::apr_dir_t;
pub use generated::apr_exit_why_e;
pub use generated::apr_file_t;
pub use generated::apr_fileperms_t;
pub use generated::apr_finfo_t;
pub use generated::apr_getopt_option_t;
pub use generated::apr_getopt_t;
pub use generated::apr_int16_t;
pub use generated::apr_int32_t;
pub use generated::apr_int64_t;
pub use generated::apr_off_t;
pub use generated::apr_pool_t;
pub use generated::apr_proc_t;
pub use generated::apr_seek_where_t;
pub use generated::apr_size_t;
pub use generated::apr_status_t;
pub use generated::apr_time_t;
pub use generated::apr_uint16_t;
pub use generated::apr_uint32_t;
pub use generated::apr_uint64_t;

#[ctor::ctor]
fn initialize_apr() {
    unsafe {
        assert!(generated::apr_initialize() == generated::APR_SUCCESS as i32);
    }
}
