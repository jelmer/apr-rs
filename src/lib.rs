#![deny(missing_docs)]

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

/// Base64 encoding and decoding
pub mod base64;
/// Callback function types and utilities
pub mod callbacks;
/// Cryptographic operations (encryption, decryption)
pub mod crypto;
/// Date parsing and formatting utilities
pub mod date;
/// Error types and result handling
pub mod error;
/// File I/O operations
pub mod file;
/// Command-line option parsing
pub mod getopt;
/// Hash table data structure
pub mod hash;
/// MD5 hashing functions
pub mod md5;
/// Memory-mapped file support
pub mod mmap;
/// Network I/O and socket operations
pub mod network;
/// File path manipulation utilities
pub mod paths;
/// Memory pool management
pub mod pool;
/// Thread-safe queue data structure
pub mod queue;
/// SHA1 hashing functions
pub mod sha1;
/// APR status codes
pub mod status;
/// String manipulation utilities
pub mod strings;
/// String pattern matching
pub mod strmatch;
/// APR table data structure (ordered key-value pairs)
pub mod tables;
/// Time handling and conversion
pub mod time;
/// URI parsing and manipulation
pub mod uri;
/// UUID generation
pub mod uuid;
/// Version information
pub mod versions;
/// Character set translation
pub mod xlate;
/// XML parsing utilities
pub mod xml;

pub use error::{Error, ErrorContext, Result};
pub use pool::{Pool, PoolHandle};
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

// APR initialization via ctor (runs before any threads are created).
//
// APR requires apr_initialize() to be called in a single-threaded context.
// The ctor runs at library load time, before test threads are created, satisfying this requirement.
//
// We intentionally do NOT call apr_terminate() in a dtor because:
//
// 1. In multi-threaded applications, dtors can run while threads still exist or are being torn down,
//    leading to SIGSEGV crashes if apr_terminate() is called while APR pools are in use.
//
// 2. The most problematic case is `cargo test`, where the destructor can be called while test
//    threads are still running (observed crashes on macOS with ctor 0.6.0).
//
// 3. However, the same issue can occur in production applications with:
//    - Detached threads or thread pools
//    - Signal handlers (SIGTERM, SIGINT)
//    - Panic unwinding
//    - Other global destructors running in undefined order
//
// 4. APR documentation itself recommends using `atexit(apr_terminate)` rather than destructor
//    mechanisms, acknowledging the cleanup timing challenges.
//
// 5. For short-running programs, the OS will reclaim all memory on process exit anyway, making
//    explicit apr_terminate() unnecessary. For long-running programs, the risk of crashes
//    outweighs the benefit of explicit cleanup.
//
// Related issues:
// - https://github.com/mmastrac/rust-ctor/issues/8
// - https://github.com/rust-lang/cargo/issues/5438
// - https://dev.apr.apache.narkive.com/cHRvpf93/thread-safety-of-apr-initialize
#[ctor::ctor]
fn init() {
    unsafe {
        apr_sys::apr_initialize();
    }
}

// No dtor: intentionally allow OS to clean up on process exit to avoid crashes
// in multi-threaded scenarios (both tests and production applications).

/// Initialize the APR library.
///
/// # Safety
///
/// This function is automatically called via `#[ctor::ctor]` at library load time, before any
/// threads are created. **You should not need to call this function manually** in normal usage.
///
/// However, if you need explicit control over APR initialization (for example, in testing
/// scenarios or when dynamically loading/unloading APR), you can call this function.
///
/// ## Safety Requirements
///
/// 1. **Single-threaded context**: APR requires `apr_initialize()` to be called in a
///    single-threaded context before any threads are created. Calling this from a
///    multi-threaded context will lead to undefined behavior.
///
/// 2. **Multiple calls**: It is safe to call this function multiple times, but each call
///    must be balanced with a corresponding call to [`terminate()`]. APR maintains an
///    internal reference count.
///
/// 3. **Already initialized**: Since this crate automatically initializes APR at load time,
///    calling this function will increment APR's internal initialization count, requiring
///    an additional [`terminate()`] call to fully shut down APR.
///
/// # Examples
///
/// ```no_run
/// use apr;
///
/// // Only call from single-threaded context before any threads exist
/// unsafe {
///     apr::initialize();
/// }
///
/// // ... use APR ...
///
/// // Must balance with terminate if you called initialize
/// unsafe {
///     apr::terminate();
/// }
/// ```
pub unsafe fn initialize() {
    unsafe {
        apr_sys::apr_initialize();
    }
}

/// Terminate the APR library and clean up all internal data structures.
///
/// # Safety
///
/// This function is **extremely dangerous** and should be used with great caution:
///
/// 1. **Multi-threading hazard**: If any thread is using APR when this is called, the program
///    will crash with SIGSEGV. This includes background threads, thread pools, or any thread
///    that might be accessing APR pools.
///
/// 2. **Pool destruction**: All APR pools will be destroyed. Any subsequent use of Pool objects
///    or data allocated from pools will result in undefined behavior.
///
/// 3. **Global state**: APR maintains global state. After calling this function, creating new
///    pools or using any APR functionality will fail or cause undefined behavior.
///
/// # When to Use This
///
/// In most cases, **you should not call this function**. The operating system will clean up
/// all APR memory when the process exits. This function is only useful in very specific
/// scenarios:
///
/// - Long-running processes that load/unload APR dynamically
/// - Testing scenarios where you need to verify resource cleanup
/// - Applications that explicitly manage APR lifecycle and can guarantee all threads have
///   stopped using APR
///
/// # Alternative
///
/// APR documentation recommends using `std::process::exit()` or allowing normal program
/// termination rather than explicitly calling `apr_terminate()`.
///
/// # Examples
///
/// ```no_run
/// use apr;
///
/// // Ensure all threads have finished and no APR objects are in use
/// // ... join all threads, drop all pools, etc ...
///
/// // Only then is it safe to call terminate
/// unsafe {
///     apr::terminate();
/// }
/// // After this point, APR cannot be used
/// ```
pub unsafe fn terminate() {
    unsafe {
        apr_sys::apr_terminate();
    }
}
