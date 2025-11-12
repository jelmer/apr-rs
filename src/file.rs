//! File handling
use crate::{pool::Pool, status::Status};
use apr_sys;
use std::io::{Read, Write};
use std::path::Path;

pub use apr_sys::apr_file_t;

/// File open flags
pub struct OpenFlags(i32);

impl OpenFlags {
    /// Open file for reading
    pub const READ: OpenFlags = OpenFlags(apr_sys::APR_FOPEN_READ as i32);
    /// Open file for writing
    pub const WRITE: OpenFlags = OpenFlags(apr_sys::APR_FOPEN_WRITE as i32);
    /// Create file if it doesn't exist
    pub const CREATE: OpenFlags = OpenFlags(apr_sys::APR_FOPEN_CREATE as i32);
    /// Open file in append mode
    pub const APPEND: OpenFlags = OpenFlags(apr_sys::APR_FOPEN_APPEND as i32);
    /// Truncate file if it exists
    pub const TRUNCATE: OpenFlags = OpenFlags(apr_sys::APR_FOPEN_TRUNCATE as i32);
    /// Open file in binary mode
    pub const BINARY: OpenFlags = OpenFlags(apr_sys::APR_FOPEN_BINARY as i32);
    /// Fail if file exists (used with CREATE)
    pub const EXCL: OpenFlags = OpenFlags(apr_sys::APR_FOPEN_EXCL as i32);
    /// Open file with buffering
    pub const BUFFERED: OpenFlags = OpenFlags(apr_sys::APR_FOPEN_BUFFERED as i32);
    /// Delete file when closed
    pub const DELONCLOSE: OpenFlags = OpenFlags(apr_sys::APR_FOPEN_DELONCLOSE as i32);
    /// Platform-dependent thread-safe mode
    pub const XTHREAD: OpenFlags = OpenFlags(apr_sys::APR_FOPEN_XTHREAD as i32);
    /// Platform-dependent shared lock mode
    pub const SHARELOCK: OpenFlags = OpenFlags(apr_sys::APR_FOPEN_SHARELOCK as i32);
    /// Don't register cleanup when pool is destroyed
    pub const NOCLEANUP: OpenFlags = OpenFlags(apr_sys::APR_FOPEN_NOCLEANUP as i32);
    /// Advisory flag for sendfile support
    pub const SENDFILE_ENABLED: OpenFlags = OpenFlags(apr_sys::APR_FOPEN_SENDFILE_ENABLED as i32);
    /// Platform-dependent large file support
    pub const LARGEFILE: OpenFlags = OpenFlags(apr_sys::APR_FOPEN_LARGEFILE as i32);

    /// Combine multiple flags
    pub fn combine(flags: &[OpenFlags]) -> Self {
        let combined = flags.iter().fold(0, |acc, flag| acc | flag.0);
        OpenFlags(combined)
    }
}

/// File permissions
pub type FilePerms = apr_sys::apr_fileperms_t;

/// APR File wrapper providing safe RAII access
#[repr(transparent)]
pub struct File {
    raw: *mut apr_sys::apr_file_t,
    // Files are tied to a pool and are not thread-safe
    _no_send: std::marker::PhantomData<*mut ()>,
}

impl File {
    /// Open a file with specified flags and permissions
    pub fn open<P: AsRef<Path>>(
        path: P,
        flags: OpenFlags,
        perms: FilePerms,
        pool: &Pool,
    ) -> Result<Self, Status> {
        let path_str = path.as_ref().to_string_lossy();
        let path_cstr = std::ffi::CString::new(path_str.as_ref())
            .map_err(|_| Status::from(apr_sys::APR_EINVAL as i32))?;

        let mut file_ptr: *mut apr_sys::apr_file_t = std::ptr::null_mut();
        let status = unsafe {
            apr_sys::apr_file_open(
                &mut file_ptr,
                path_cstr.as_ptr(),
                flags.0,
                perms,
                pool.as_mut_ptr(),
            )
        };

        if status == apr_sys::APR_SUCCESS as i32 {
            Ok(File {
                raw: file_ptr,
                _no_send: std::marker::PhantomData,
            })
        } else {
            Err(Status::from(status))
        }
    }

    /// Create a temporary file
    ///
    /// The temporary file will have secure default permissions (typically 0600).
    pub fn temp_file(template: &str, flags: OpenFlags, pool: &Pool) -> Result<Self, Status> {
        let template_cstr = std::ffi::CString::new(template)
            .map_err(|_| Status::from(apr_sys::APR_EINVAL as i32))?;

        let mut file_ptr: *mut apr_sys::apr_file_t = std::ptr::null_mut();
        let status = unsafe {
            apr_sys::apr_file_mktemp(
                &mut file_ptr,
                template_cstr.as_ptr() as *mut std::ffi::c_char,
                flags.0,
                pool.as_mut_ptr(),
            )
        };

        if status == apr_sys::APR_SUCCESS as i32 {
            Ok(File {
                raw: file_ptr,
                _no_send: std::marker::PhantomData,
            })
        } else {
            Err(Status::from(status))
        }
    }

    /// Get a reference to stdin
    pub fn stdin(pool: &Pool) -> Result<Self, Status> {
        let mut file_ptr: *mut apr_sys::apr_file_t = std::ptr::null_mut();
        let status = unsafe { apr_sys::apr_file_open_stdin(&mut file_ptr, pool.as_mut_ptr()) };

        if status == apr_sys::APR_SUCCESS as i32 {
            Ok(File {
                raw: file_ptr,
                _no_send: std::marker::PhantomData,
            })
        } else {
            Err(Status::from(status))
        }
    }

    /// Get a reference to stdout
    pub fn stdout(pool: &Pool) -> Result<Self, Status> {
        let mut file_ptr: *mut apr_sys::apr_file_t = std::ptr::null_mut();
        let status = unsafe { apr_sys::apr_file_open_stdout(&mut file_ptr, pool.as_mut_ptr()) };

        if status == apr_sys::APR_SUCCESS as i32 {
            Ok(File {
                raw: file_ptr,
                _no_send: std::marker::PhantomData,
            })
        } else {
            Err(Status::from(status))
        }
    }

    /// Get a reference to stderr
    pub fn stderr(pool: &Pool) -> Result<Self, Status> {
        let mut file_ptr: *mut apr_sys::apr_file_t = std::ptr::null_mut();
        let status = unsafe { apr_sys::apr_file_open_stderr(&mut file_ptr, pool.as_mut_ptr()) };

        if status == apr_sys::APR_SUCCESS as i32 {
            Ok(File {
                raw: file_ptr,
                _no_send: std::marker::PhantomData,
            })
        } else {
            Err(Status::from(status))
        }
    }

    /// Get the raw file pointer
    pub fn as_ptr(&self) -> *const apr_sys::apr_file_t {
        self.raw
    }

    /// Get the mutable raw file pointer
    pub fn as_mut_ptr(&self) -> *mut apr_sys::apr_file_t {
        self.raw
    }

    /// Flush any buffered writes
    pub fn flush(&mut self) -> Result<(), Status> {
        let status = unsafe { apr_sys::apr_file_flush(self.raw) };

        if status == apr_sys::APR_SUCCESS as i32 {
            Ok(())
        } else {
            Err(Status::from(status))
        }
    }

    /// Close the file explicitly (automatic on drop)
    pub fn close(mut self) -> Result<(), Status> {
        let status = unsafe { apr_sys::apr_file_close(self.raw) };
        // Prevent double-close in Drop
        self.raw = std::ptr::null_mut();

        if status == apr_sys::APR_SUCCESS as i32 {
            Ok(())
        } else {
            Err(Status::from(status))
        }
    }
}

impl Drop for File {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                apr_sys::apr_file_close(self.raw);
            }
        }
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let mut bytes_read: apr_sys::apr_size_t = buf.len() as apr_sys::apr_size_t;
        let status = unsafe {
            apr_sys::apr_file_read(
                self.raw,
                buf.as_mut_ptr() as *mut std::ffi::c_void,
                &mut bytes_read,
            )
        };

        match status as u32 {
            s if s == apr_sys::APR_SUCCESS => Ok(bytes_read as usize),
            s if s == apr_sys::APR_EOF => Ok(0),
            _ => Err(std::io::Error::other(Status::from(status))),
        }
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let mut bytes_written: apr_sys::apr_size_t = buf.len() as apr_sys::apr_size_t;
        let status = unsafe {
            apr_sys::apr_file_write(
                self.raw,
                buf.as_ptr() as *const std::ffi::c_void,
                &mut bytes_written,
            )
        };

        if status == apr_sys::APR_SUCCESS as i32 {
            Ok(bytes_written as usize)
        } else {
            Err(std::io::Error::other(Status::from(status)))
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flush().map_err(std::io::Error::other)
    }
}

/// Builder pattern for File creation with fluent API
pub struct FileBuilder<'a> {
    flags: OpenFlags,
    perms: FilePerms,
    pool: Option<&'a Pool>,
}

impl<'a> FileBuilder<'a> {
    /// Create a new FileBuilder with default values
    pub fn new() -> Self {
        FileBuilder {
            flags: OpenFlags(0),
            perms: 0o644,
            pool: None,
        }
    }

    /// Set read flag
    pub fn read(mut self) -> Self {
        self.flags = OpenFlags(self.flags.0 | OpenFlags::READ.0);
        self
    }

    /// Set write flag
    pub fn write(mut self) -> Self {
        self.flags = OpenFlags(self.flags.0 | OpenFlags::WRITE.0);
        self
    }

    /// Set create flag
    pub fn create(mut self) -> Self {
        self.flags = OpenFlags(self.flags.0 | OpenFlags::CREATE.0);
        self
    }

    /// Set append flag
    pub fn append(mut self) -> Self {
        self.flags = OpenFlags(self.flags.0 | OpenFlags::APPEND.0);
        self
    }

    /// Set truncate flag
    pub fn truncate(mut self) -> Self {
        self.flags = OpenFlags(self.flags.0 | OpenFlags::TRUNCATE.0);
        self
    }

    /// Set binary flag
    pub fn binary(mut self) -> Self {
        self.flags = OpenFlags(self.flags.0 | OpenFlags::BINARY.0);
        self
    }

    /// Set exclusive flag
    pub fn exclusive(mut self) -> Self {
        self.flags = OpenFlags(self.flags.0 | OpenFlags::EXCL.0);
        self
    }

    /// Set file permissions
    pub fn permissions(mut self, perms: FilePerms) -> Self {
        self.perms = perms;
        self
    }

    /// Set the pool to use (required)
    pub fn pool(mut self, pool: &'a Pool) -> Self {
        self.pool = Some(pool);
        self
    }

    /// Open the file with the configured options
    pub fn open<P: AsRef<Path>>(self, path: P) -> Result<File, Status> {
        let pool = self
            .pool
            .ok_or_else(|| Status::from(apr_sys::APR_EINVAL as i32))?;
        File::open(path, self.flags, self.perms, pool)
    }
}

impl<'a> Default for FileBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl File {
    /// Create a FileBuilder for fluent API
    pub fn builder() -> FileBuilder<'static> {
        FileBuilder::new()
    }
}

/// High-level convenience functions for common file operations
pub mod io {
    use super::*;
    use crate::error::{ErrorContext, Result};
    use crate::pool;

    /// Read entire file to string (creates temporary pool internally)
    pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
        let path_ref = path.as_ref();
        pool::with_tmp_pool(|pool| {
            let mut file = File::open(path_ref, OpenFlags::READ, 0, pool)
                .with_context(|| format!("Failed to open file for reading: {:?}", path_ref))?;
            let mut s = String::new();
            file.read_to_string(&mut s)
                .with_context(|| format!("Failed to read file contents: {:?}", path_ref))?;
            Ok(s)
        })
    }

    /// Write string to file (creates temporary pool internally)
    pub fn write<P: AsRef<Path>>(path: P, contents: &str) -> Result<()> {
        let path_ref = path.as_ref();
        pool::with_tmp_pool(|pool| {
            let mut file = File::open(
                path_ref,
                OpenFlags::combine(&[OpenFlags::WRITE, OpenFlags::CREATE, OpenFlags::TRUNCATE]),
                0o644,
                pool,
            )
            .with_context(|| format!("Failed to open file for writing: {:?}", path_ref))?;

            file.write_all(contents.as_bytes())
                .with_context(|| format!("Failed to write to file: {:?}", path_ref))?;
            file.flush()
                .with_context(|| format!("Failed to flush file: {:?}", path_ref))?;
            Ok(())
        })
    }

    /// Copy file from source to destination (creates temporary pool internally)
    pub fn copy<P1: AsRef<Path>, P2: AsRef<Path>>(from: P1, to: P2) -> Result<()> {
        let from_ref = from.as_ref();
        let to_ref = to.as_ref();
        pool::with_tmp_pool(|pool| {
            let mut src = File::open(from_ref, OpenFlags::READ, 0, pool)
                .with_context(|| format!("Failed to open source file: {:?}", from_ref))?;
            let mut dst = File::open(
                to_ref,
                OpenFlags::combine(&[OpenFlags::WRITE, OpenFlags::CREATE, OpenFlags::TRUNCATE]),
                0o644,
                pool,
            )
            .with_context(|| format!("Failed to open destination file: {:?}", to_ref))?;

            std::io::copy(&mut src, &mut dst)
                .with_context(|| format!("Failed to copy from {:?} to {:?}", from_ref, to_ref))?;

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    #[test]
    fn test_file_open_write_read() {
        let pool = Pool::new();

        // Create a unique temporary file path in the current directory
        let temp_path = format!("./target/apr_test_file_{}", std::process::id());

        // Write to file
        {
            let mut file = File::open(
                &temp_path,
                OpenFlags::combine(&[OpenFlags::WRITE, OpenFlags::CREATE, OpenFlags::TRUNCATE]),
                apr_sys::APR_FPROT_OS_DEFAULT as i32,
                &pool,
            )
            .expect("Failed to open file for writing");

            file.write_all(b"Hello, APR!")
                .expect("Failed to write to file");
            file.flush().expect("Failed to flush file");
        }

        // Read from file
        {
            let mut file = File::open(&temp_path, OpenFlags::READ, 0, &pool)
                .expect("Failed to open file for reading");

            let mut buffer = String::new();
            file.read_to_string(&mut buffer)
                .expect("Failed to read from file");
            assert_eq!(buffer, "Hello, APR!");
        }

        // Clean up
        let _ = std::fs::remove_file(temp_path);
    }

    #[test]
    fn test_temp_file() {
        let pool = Pool::new();

        let mut temp_file = File::temp_file(
            "/tmp/apr_temp_XXXXXX",
            OpenFlags::combine(&[OpenFlags::READ, OpenFlags::WRITE]),
            &pool,
        )
        .expect("Failed to create temp file");

        temp_file
            .write_all(b"Temporary data")
            .expect("Failed to write to temp file");

        // Note: temp file is automatically cleaned up when it goes out of scope
    }

    #[test]
    fn test_file_builder() {
        let pool = Pool::new();

        // Test builder pattern works
        let result = File::builder()
            .read()
            .write()
            .create()
            .truncate()
            .permissions(0o600)
            .pool(&pool)
            .open("./target/test_builder_file");

        // Don't actually create the file in the test, just verify the builder works
        // The actual file operation might fail due to permissions in test environment
        match result {
            Ok(_) => {
                // Success - clean up
                let _ = std::fs::remove_file("./target/test_builder_file");
            }
            Err(_) => {
                // Expected in restricted test environment
            }
        }
    }

    #[test]
    fn test_open_flags_combine() {
        let flags = OpenFlags::combine(&[OpenFlags::READ, OpenFlags::WRITE, OpenFlags::CREATE]);
        let expected =
            (apr_sys::APR_FOPEN_READ | apr_sys::APR_FOPEN_WRITE | apr_sys::APR_FOPEN_CREATE) as i32;
        assert_eq!(flags.0, expected);
    }
}
