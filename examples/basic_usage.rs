//! Basic usage examples for the apr-rs crate.
//!
//! This example demonstrates fundamental APR concepts that are essential
//! when working with APR-based C libraries.

use apr::{Pool, Result, Status};
use apr::hash::Hash;
use apr::tables::Table;

fn main() -> Result<()> {
    // Memory Pools - The foundation of APR memory management
    let root_pool = Pool::new();
    let subpool = Pool::new();
    let pool_ptr = root_pool.as_mut_ptr(); // Raw pointer for C interop
    drop(subpool); // Subpool cleaned up
    
    // Hash Tables
    let mut hash = Hash::<&str, String>::new(&root_pool);
    hash.set("config_path", &"/etc/myapp.conf".to_string());
    hash.set("log_level", &"debug".to_string());
    hash.set("port", &"8080".to_string());
    
    if let Some(path) = hash.get("config_path") {
        println!("Config path: {}", path);
    }
    
    // APR Tables (allows duplicate keys)
    let mut table = Table::new(&root_pool);
    table.set("header", "Content-Type: text/html");
    table.add("header", "Cache-Control: no-cache");
    
    // Error Handling
    let error_status = Status::from(apr_sys::APR_ENOENT as i32);
    if !error_status.is_success() {
        println!("Error: {}", apr::Error::from(error_status));
    }
    
    Ok(())
}