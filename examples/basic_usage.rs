//! Basic usage examples for the apr-rs crate.
//!
//! This example demonstrates fundamental APR concepts that are essential
//! when working with APR-based C libraries.

use apr::hash::Hash;
use apr::tables::Table;
use apr::{Pool, Result, Status};
use std::ffi::{CStr, CString};

fn main() -> Result<()> {
    // Memory Pools - The foundation of APR memory management
    let root_pool = Pool::new();
    let subpool = Pool::new();
    let _pool_ptr = root_pool.as_mut_ptr(); // Raw pointer for C interop
    drop(subpool); // Subpool cleaned up

    // Hash Tables - store references to data
    let config_path = "/etc/myapp.conf".to_string();
    let log_level = "debug".to_string();
    let port = "8080".to_string();

    let mut hash: Hash<&str, &String> = Hash::new(&root_pool);
    hash.insert("config_path", &config_path);
    hash.insert("log_level", &log_level);
    hash.insert("port", &port);

    if let Some(path) = hash.get("config_path") {
        println!("Config path: {}", path);
    }

    // APR Tables (allows duplicate keys) - tables store C strings
    let content_type = CString::new("Content-Type: text/html").unwrap();
    let cache_control = CString::new("Cache-Control: no-cache").unwrap();

    let mut table: Table<&CStr> = Table::new(&root_pool);
    table.set("header", content_type.as_c_str());
    table.add(
        "header",
        cache_control.as_c_str().to_string_lossy().as_ref(),
    );

    // Error Handling
    let error_status = Status::from(apr_sys::APR_ENOENT as i32);
    if !error_status.is_success() {
        println!("Error: {}", apr::Error::from(error_status));
    }

    Ok(())
}
