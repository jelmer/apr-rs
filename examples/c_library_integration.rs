//! Example demonstrating how to use apr-rs when creating bindings for C libraries that use APR.
//!
//! This example shows the typical pattern for integrating with APR-based C libraries:
//! 1. Creating APR memory pools
//! 2. Passing APR types to C functions
//! 3. Handling APR status codes
//! 4. Managing memory lifecycle

use apr::{Pool, Result, Status};
use std::ffi::CString;

// Example: Simulating bindings for a hypothetical C library that uses APR
// In a real scenario, these would be actual FFI declarations to your C library

#[allow(dead_code)]
mod ffi {
    use std::os::raw::{c_char, c_int};

    // Simulated C function signatures that would come from your C library
    // In reality, these would be:
    // extern "C" {
    //     pub fn library_init(pool: *mut apr_sys::apr_pool_t) -> apr_sys::apr_status_t;
    //     pub fn library_process_data(
    //         data: *const c_char,
    //         len: c_int,
    //         pool: *mut apr_sys::apr_pool_t
    //     ) -> apr_sys::apr_status_t;
    //     pub fn library_cleanup(pool: *mut apr_sys::apr_pool_t) -> apr_sys::apr_status_t;
    // }

    // For this example, we'll simulate these functions
    pub unsafe fn library_init(_pool: *mut apr_sys::apr_pool_t) -> apr_sys::apr_status_t {
        0 // APR_SUCCESS
    }

    pub unsafe fn library_process_data(
        _data: *const c_char,
        _len: c_int,
        _pool: *mut apr_sys::apr_pool_t,
    ) -> apr_sys::apr_status_t {
        0 // APR_SUCCESS
    }

    pub unsafe fn library_cleanup(_pool: *mut apr_sys::apr_pool_t) -> apr_sys::apr_status_t {
        0 // APR_SUCCESS
    }
}

/// Wrapper struct for our C library
pub struct CLibraryWrapper {
    pool: Pool,
}

impl CLibraryWrapper {
    /// Initialize the C library with its own memory pool
    pub fn new() -> Result<Self> {
        // Create a dedicated pool for this library instance
        let pool = Pool::new();

        // Initialize the C library
        let status = unsafe { Status::from(ffi::library_init(pool.as_mut_ptr())) };

        if !status.is_success() {
            return Err(status.into());
        }

        Ok(CLibraryWrapper { pool })
    }

    /// Process data using the C library
    pub fn process_data(&self, data: &str) -> Result<()> {
        // Convert Rust string to C string
        let c_data = CString::new(data)
            .map_err(|_| apr::Error::from(Status::from(apr_sys::APR_EINVAL as i32)))?;

        // Call the C function
        let status = unsafe {
            Status::from(ffi::library_process_data(
                c_data.as_ptr(),
                data.len() as i32,
                self.pool.as_mut_ptr(),
            ))
        };

        if status.is_success() {
            Ok(())
        } else {
            Err(status.into())
        }
    }

    /// Create a sub-operation with its own memory scope
    pub fn scoped_operation<F, R>(&self, operation: F) -> Result<R>
    where
        F: FnOnce(&Pool) -> Result<R>,
    {
        // Create a subpool for this operation
        let subpool = Pool::new();

        // Execute the operation with the subpool
        // Subpool is automatically cleaned up when dropped
        operation(&subpool)
    }
}

impl Drop for CLibraryWrapper {
    fn drop(&mut self) {
        // Clean up the C library
        unsafe {
            let _ = ffi::library_cleanup(self.pool.as_mut_ptr());
        }
        // Pool is automatically cleaned up when dropped
    }
}

fn main() -> Result<()> {
    println!("Demonstrating C library integration with APR...\n");

    // Initialize our wrapped C library
    let library = CLibraryWrapper::new()?;
    println!("✓ C library initialized with APR pool");

    // Process some data
    library.process_data("Hello from Rust!")?;
    println!("✓ Processed data through C library");

    // Demonstrate scoped memory management
    library.scoped_operation(|subpool| {
        println!("✓ Created subpool for scoped operation");

        // In a real scenario, you might pass this subpool to C functions
        // that need temporary memory
        let _subpool_ptr = subpool.as_mut_ptr();

        // Simulate some work...
        println!("  Performing work with subpool...");

        Ok(())
    })?;
    println!("✓ Subpool automatically cleaned up");

    // Demonstrate error handling
    match library.process_data("Another message") {
        Ok(()) => println!("✓ Successfully processed second message"),
        Err(e) => eprintln!("✗ Error processing message: {}", e),
    }

    println!("\n✓ Library will be cleaned up when dropped");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_lifecycle() {
        // Test that we can create and destroy the library wrapper
        let library = CLibraryWrapper::new().expect("Failed to create library");
        drop(library);
    }

    #[test]
    fn test_scoped_operations() {
        let library = CLibraryWrapper::new().expect("Failed to create library");

        let result = library.scoped_operation(|_pool| {
            // Simulate some operation
            Ok(42)
        });

        assert_eq!(result.unwrap(), 42);
    }
}
