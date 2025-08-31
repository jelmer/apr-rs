//! Basic usage examples for the apr-rs crate.
//!
//! This example demonstrates fundamental APR concepts that are essential
//! when working with APR-based C libraries.

use apr::hash::{Hash, TypedHash};
use apr::tables::StringTable;
use apr::{Pool, Result, Status};
use std::ffi::c_void;

fn main() -> Result<()> {
    // Memory Pools - The foundation of APR memory management
    let root_pool = Pool::new();
    let subpool = Pool::new();
    let _pool_ptr = root_pool.as_mut_ptr(); // Raw pointer for C interop
    drop(subpool); // Subpool cleaned up

    // Hash Tables - using TypedHash for string keys and values
    let config_path = "/etc/myapp.conf".to_string();
    let log_level = "debug".to_string();
    let port = "8080".to_string();

    let mut hash = TypedHash::new(&root_pool);
    hash.insert_ref("config_path", &config_path);
    hash.insert_ref("log_level", &log_level);
    hash.insert_ref("port", &port);

    if let Some(path) = hash.get_ref("config_path") {
        println!("Config path: {}", path);
    }

    // Raw Hash example - stores raw pointers
    let mut raw_hash = Hash::new(&root_pool);
    unsafe {
        raw_hash.insert(b"key1", &config_path as *const _ as *mut c_void);
        if let Some(ptr) = raw_hash.get(b"key1") {
            let value = &*(ptr as *const String);
            println!("Raw hash value: {}", value);
        }
    }

    // APR Tables (allows duplicate keys) - using StringTable for convenience
    let mut table = StringTable::new(&root_pool, 10);
    table.set("Content-Type", "text/html");
    table.add("Cache-Control", "no-cache");

    if let Some(content_type) = table.get("Content-Type") {
        println!("Content-Type: {}", content_type);
    }

    // Error Handling
    let error_status = Status::from(apr_sys::APR_ENOENT as i32);
    if !error_status.is_success() {
        println!("Error: {}", apr::Error::from(error_status));
    }

    Ok(())
}
