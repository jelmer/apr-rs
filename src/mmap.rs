//! Memory-mapped file operations

use crate::{pool::Pool, Result};
use std::marker::PhantomData;
use std::ptr;
use std::slice;

/// Memory-mapped file
pub struct Mmap<'a> {
    raw: *mut apr_sys::apr_mmap_t,
    offset: i64,
    _phantom: PhantomData<&'a Pool>,
}

/// Memory map access flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MmapFlag {
    /// Read-only access
    Read,
    /// Write access
    Write,
}

impl From<MmapFlag> for i32 {
    fn from(flag: MmapFlag) -> Self {
        match flag {
            MmapFlag::Read => apr_sys::APR_MMAP_READ as i32,
            MmapFlag::Write => apr_sys::APR_MMAP_WRITE as i32,
        }
    }
}

impl<'a> Mmap<'a> {
    /// Create a new memory map.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the `file` pointer is valid and properly
    /// initialized for the lifetime of the memory map.
    pub unsafe fn create(
        file: *mut apr_sys::apr_file_t,
        offset: i64,
        size: usize,
        flag: MmapFlag,
        pool: &'a Pool,
    ) -> Result<Self> {
        let mut mmap: *mut apr_sys::apr_mmap_t = ptr::null_mut();

        let status = apr_sys::apr_mmap_create(
            &mut mmap,
            file,
            offset as apr_sys::apr_off_t,
            size,
            flag.into(),
            pool.as_mut_ptr(),
        );

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }

        Ok(Mmap {
            raw: mmap,
            offset,
            _phantom: PhantomData,
        })
    }

    /// Duplicate an existing memory map
    pub fn dup(other: &Self, pool: &'a Pool) -> Result<Self> {
        let mut new_mmap: *mut apr_sys::apr_mmap_t = ptr::null_mut();

        let status = unsafe { apr_sys::apr_mmap_dup(&mut new_mmap, other.raw, pool.as_mut_ptr()) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }

        Ok(Mmap {
            raw: new_mmap,
            offset: other.offset,
            _phantom: PhantomData,
        })
    }

    /// Get the memory map as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let ptr = (*self.raw).mm as *const u8;
            let size = (*self.raw).size;
            if ptr.is_null() || size == 0 {
                &[]
            } else {
                slice::from_raw_parts(ptr, size)
            }
        }
    }

    /// Get the memory map as a mutable byte slice
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            let ptr = (*self.raw).mm as *mut u8;
            let size = (*self.raw).size;
            if ptr.is_null() || size == 0 {
                &mut []
            } else {
                slice::from_raw_parts_mut(ptr, size)
            }
        }
    }

    /// Get the offset of the memory map within the file
    pub fn offset(&self) -> i64 {
        self.offset
    }

    /// Get the size of the memory map
    pub fn size(&self) -> usize {
        unsafe { (*self.raw).size }
    }

    /// Get a raw pointer to the underlying APR mmap structure
    pub fn as_ptr(&self) -> *const apr_sys::apr_mmap_t {
        self.raw
    }

    /// Get a mutable raw pointer to the underlying APR mmap structure
    pub fn as_mut_ptr(&mut self) -> *mut apr_sys::apr_mmap_t {
        self.raw
    }
}

impl<'a> Drop for Mmap<'a> {
    fn drop(&mut self) {
        unsafe {
            apr_sys::apr_mmap_delete(self.raw);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::{File, OpenFlags};
    use std::io::Write;

    #[test]
    fn test_mmap_read() {
        let pool = Pool::new();

        // Create a temporary file with known content
        let test_content = b"Hello, APR mmap test!";
        let temp_path = "/tmp/apr_mmap_test.txt";

        {
            let mut temp_file = std::fs::File::create(temp_path).unwrap();
            temp_file.write_all(test_content).unwrap();
            temp_file.sync_all().unwrap();
        }

        // Open file with APR
        let file =
            File::open(temp_path, OpenFlags::READ, apr_sys::APR_UREAD as i32, &pool).unwrap();

        // Create memory map
        let mmap = unsafe {
            Mmap::create(
                file.as_mut_ptr(),
                0,
                test_content.len(),
                MmapFlag::Read,
                &pool,
            )
        }
        .unwrap();

        // Verify mapped content
        let mapped_data = mmap.as_bytes();
        assert_eq!(mapped_data, test_content);
        assert_eq!(mmap.size(), test_content.len());
        assert_eq!(mmap.offset(), 0);

        drop(mmap);
        drop(file);

        // Clean up
        std::fs::remove_file(temp_path).unwrap();
    }

    #[test]
    fn test_mmap_dup() {
        let pool = Pool::new();

        // Create a temporary file
        let test_content = b"Duplicate test data";
        let temp_path = "/tmp/apr_mmap_dup_test.txt";

        {
            let mut temp_file = std::fs::File::create(temp_path).unwrap();
            temp_file.write_all(test_content).unwrap();
            temp_file.sync_all().unwrap();
        }

        // Open file with APR
        let file =
            File::open(temp_path, OpenFlags::READ, apr_sys::APR_UREAD as i32, &pool).unwrap();

        // Create original memory map
        let mmap1 = unsafe {
            Mmap::create(
                file.as_mut_ptr(),
                0,
                test_content.len(),
                MmapFlag::Read,
                &pool,
            )
        }
        .unwrap();

        // Duplicate the memory map
        let mmap2 = Mmap::dup(&mmap1, &pool).unwrap();

        // Both should have same content and properties
        assert_eq!(mmap1.as_bytes(), mmap2.as_bytes());
        assert_eq!(mmap1.size(), mmap2.size());
        assert_eq!(mmap1.offset(), mmap2.offset());
        assert_eq!(mmap1.as_bytes(), test_content);

        drop(mmap2);
        drop(mmap1);
        drop(file);

        // Clean up
        std::fs::remove_file(temp_path).unwrap();
    }

    #[test]
    fn test_mmap_flag_conversion() {
        assert_eq!(i32::from(MmapFlag::Read), apr_sys::APR_MMAP_READ as i32);
        assert_eq!(i32::from(MmapFlag::Write), apr_sys::APR_MMAP_WRITE as i32);
    }

    #[test]
    fn test_mmap_empty() {
        let pool = Pool::new();

        // Create file with minimal content
        let temp_path = "/tmp/apr_mmap_empty_test.txt";
        std::fs::write(temp_path, "x").unwrap();

        let file =
            File::open(temp_path, OpenFlags::READ, apr_sys::APR_UREAD as i32, &pool).unwrap();

        // Memory map small file
        let mmap =
            unsafe { Mmap::create(file.as_ptr() as *mut _, 0, 1, MmapFlag::Read, &pool) }.unwrap();

        assert_eq!(mmap.as_bytes(), b"x");
        assert_eq!(mmap.size(), 1);
        assert_eq!(mmap.offset(), 0);

        drop(mmap);
        drop(file);
        std::fs::remove_file(temp_path).unwrap();
    }
}
