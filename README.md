# apr-rs

Rust bindings for the Apache Portable Runtime (APR) library and the associated APR-Util library.

[![Crates.io](https://img.shields.io/crates/v/apr.svg)](https://crates.io/crates/apr)
[![Documentation](https://docs.rs/apr/badge.svg)](https://docs.rs/apr)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

## Overview

This crate provides safe Rust bindings to the Apache Portable Runtime (APR), a C library that forms the foundation of the Apache HTTP Server and other Apache projects. APR provides a predictable and consistent interface to underlying platform-specific implementations for:

- Memory management and pool allocation
- File and network I/O
- Process and thread management
- Time handling
- String manipulation
- Data structures (hash tables, arrays, etc.)
- Cryptographic functions

## Primary Use Case: C Library Interoperability

**This crate is primarily useful when developing Rust bindings for C libraries that depend on APR.** Many Apache projects and other C libraries use APR for cross-platform compatibility and memory management. If you're creating Rust bindings for such libraries, this crate provides the necessary APR functionality with a safe Rust interface.

### Examples of C libraries that use APR:
- Apache HTTP Server modules
- [Subversion (SVN) libraries](https://github.com/jelmer/subversion-rs)
- Apache Serf
- Any custom C library built on top of APR

## Features

- **Safe Rust API**: Wraps APR's C API with safe Rust abstractions
- **Memory pools**: APR's hierarchical memory management system
- **Cross-platform**: Inherits APR's platform abstraction layer
- **Comprehensive coverage**: Bindings for most commonly-used APR and APR-Util functionality

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
apr = "0.3"
```

### Prerequisites

You need to have APR and APR-Util installed on your system:

#### Ubuntu/Debian
```bash
sudo apt-get install libapr1-dev libaprutil1-dev
```

#### Fedora/RHEL/CentOS
```bash
sudo dnf install apr-devel apr-util-devel
```

#### macOS (using Homebrew)
```bash
brew install apr apr-util
```

#### Building from source
If you need to build APR from source, download it from the [Apache APR website](https://apr.apache.org/).

## Usage Examples

### Basic Pool Usage

APR uses memory pools for all memory allocation. This is fundamental when working with APR-based C libraries:

```rust
use apr::Pool;

fn main() -> apr::Result<()> {
    // Create a root memory pool
    let pool = Pool::new();
    
    // Pools can have child pools for hierarchical memory management
    let subpool = pool.create_subpool()?;
    
    // Memory allocated from pools is automatically freed when the pool is dropped
    Ok(())
}
```

### Working with APR-based C Libraries

When creating bindings for C libraries that use APR, you'll typically need to:

1. Initialize APR (handled automatically by this crate)
2. Create memory pools for the C library to use
3. Pass APR types between Rust and C

Example of integrating with a hypothetical APR-based C library:

```rust
use apr::{Pool, Status};
use std::ptr;

// Hypothetical C library that uses APR
extern "C" {
    fn some_c_function(pool: *mut apr_sys::apr_pool_t) -> apr_sys::apr_status_t;
}

fn main() -> apr::Result<()> {
    let pool = Pool::new();
    
    // Get raw APR pool pointer to pass to C functions
    let status = unsafe { 
        Status::from(some_c_function(pool.as_mut_ptr()))
    };
    
    if status.is_success() {
        println!("C function succeeded!");
    } else {
        return Err(status.into());
    }
    
    Ok(())
}
```

### File Operations

```rust
use apr::{Pool, file::File};

fn main() -> apr::Result<()> {
    let pool = Pool::new();
    
    // Open a file using APR
    let file = File::open("example.txt", apr::file::Flag::READ, 0, &pool)?;
    
    // Read file contents
    let mut buffer = vec![0u8; 1024];
    let bytes_read = file.read(&mut buffer)?;
    
    println!("Read {} bytes", bytes_read);
    Ok(())
}
```

### Hash Tables

```rust
use apr::{Pool, hash::Hash};

fn main() -> apr::Result<()> {
    let pool = Pool::new();
    
    // Create a hash table
    let mut hash = Hash::<String>::new(&pool);
    
    // Insert key-value pairs
    hash.set("key1", "value1".to_string());
    hash.set("key2", "value2".to_string());
    
    // Retrieve values
    if let Some(value) = hash.get("key1") {
        println!("Found: {}", value);
    }
    
    Ok(())
}
```

## Module Organization

The crate is organized into modules that mirror APR's structure:

- `pool` - Memory pool management
- `file` - File I/O operations
- `network` - Network I/O and socket operations
- `hash` - Hash table implementation
- `tables` - APR table (ordered key-value pairs)
- `strings` - String manipulation utilities
- `time` - Time handling functions
- `error` - Error handling and status codes
- `crypto` - Cryptographic functions (MD5, SHA1, etc.)
- `base64` - Base64 encoding/decoding
- `uri` - URI parsing and manipulation
- `uuid` - UUID generation
- `xml` - XML parsing utilities

## Safety

This crate aims to provide safe Rust abstractions over APR's C API. However, when interfacing with C libraries:

- Some operations require `unsafe` blocks when dealing with raw pointers
- The crate handles APR initialization automatically using Rust's standard library features
- Memory management through pools helps prevent memory leaks
- Rust's ownership system is leveraged to ensure proper resource cleanup

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.

When contributing, please:
- Add tests for new functionality
- Update documentation as needed
- Follow Rust naming conventions and idioms
- Ensure all tests pass with `cargo test`

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Related Projects

- [apr-sys](https://crates.io/crates/apr-sys) - Low-level FFI bindings to APR (used by this crate)
- [Apache APR](https://apr.apache.org/) - The underlying C library

## Support

For questions and discussions, please use the GitHub issues tracker.
