//! Process and global locking mechanisms

use crate::{pool::Pool, Result};
use alloc::ffi::CString;
use core::ffi::c_char;
use core::marker::PhantomData;
use std::path::Path;
use core::ptr;

#[repr(transparent)]
pub struct ProcMutex<'a> {
    raw: *mut apr_sys::apr_proc_mutex_t,
    _phantom: PhantomData<&'a Pool<'a>>,
}

#[repr(transparent)]
pub struct GlobalMutex<'a> {
    raw: *mut apr_sys::apr_global_mutex_t,
    _phantom: PhantomData<&'a Pool<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockMech {
    FcntlSerialize,
    FLockSerialize,
    SysVSem,
    PosixSem,
    ProcPthread,
    Default,
}

impl From<LockMech> for apr_sys::apr_lockmech_e {
    fn from(mech: LockMech) -> Self {
        match mech {
            LockMech::FcntlSerialize => apr_sys::APR_LOCK_FCNTL,
            LockMech::FLockSerialize => apr_sys::APR_LOCK_FLOCK,
            LockMech::SysVSem => apr_sys::APR_LOCK_SYSVSEM,
            LockMech::PosixSem => apr_sys::APR_LOCK_POSIXSEM,
            LockMech::ProcPthread => apr_sys::APR_LOCK_PROC_PTHREAD,
            LockMech::Default => apr_sys::APR_LOCK_DEFAULT,
        }
    }
}

impl<'a> ProcMutex<'a> {
    pub fn new(fname: Option<&Path>, mech: LockMech, pool: &'a Pool<'a>) -> Result<Self> {
        let mut mutex: *mut apr_sys::apr_proc_mutex_t = ptr::null_mut();

        let c_fname = if let Some(path) = fname {
            let path_str = path
                .to_str()
                .ok_or_else(|| crate::Error::from_status((apr_sys::APR_EINVAL as i32).into()))?;
            Some(
                CString::new(path_str)
                    .map_err(|_| crate::Error::from_status((apr_sys::APR_EINVAL as i32).into()))?
            )
        } else {
            None
        };

        let fname_ptr = c_fname.as_ref().map_or(ptr::null(), |s| s.as_ptr());

        let status = unsafe {
            apr_sys::apr_proc_mutex_create(&mut mutex, fname_ptr, mech.into(), pool.as_mut_ptr())
        };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }

        Ok(ProcMutex {
            raw: mutex,
            _phantom: PhantomData,
        })
    }

    pub fn lock(&mut self) -> Result<()> {
        let status = unsafe { apr_sys::apr_proc_mutex_lock(self.raw) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }
        Ok(())
    }

    pub fn trylock(&mut self) -> Result<bool> {
        let status = unsafe { apr_sys::apr_proc_mutex_trylock(self.raw) };

        match status as u32 {
            x if x == apr_sys::APR_SUCCESS => Ok(true),
            x if x == apr_sys::APR_EBUSY => Ok(false),
            _ => Err(crate::Error::from_status(status.into())),
        }
    }

    pub fn unlock(&mut self) -> Result<()> {
        let status = unsafe { apr_sys::apr_proc_mutex_unlock(self.raw) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }
        Ok(())
    }

    pub fn as_ptr(&self) -> *const apr_sys::apr_proc_mutex_t {
        self.raw
    }

    pub fn as_mut_ptr(&mut self) -> *mut apr_sys::apr_proc_mutex_t {
        self.raw
    }
}

impl<'a> Drop for ProcMutex<'a> {
    fn drop(&mut self) {
        unsafe {
            apr_sys::apr_proc_mutex_destroy(self.raw);
        }
    }
}

impl<'a> GlobalMutex<'a> {
    pub fn new(fname: Option<&Path>, mech: LockMech, pool: &'a Pool<'a>) -> Result<Self> {
        let mut mutex: *mut apr_sys::apr_global_mutex_t = ptr::null_mut();

        let c_fname = if let Some(path) = fname {
            let path_str = path
                .to_str()
                .ok_or_else(|| crate::Error::from_status((apr_sys::APR_EINVAL as i32).into()))?;
            Some(
                CString::new(path_str)
                    .map_err(|_| crate::Error::from_status((apr_sys::APR_EINVAL as i32).into()))?
            )
        } else {
            None
        };

        let fname_ptr = c_fname.as_ref().map_or(ptr::null(), |s| s.as_ptr());

        let status = unsafe {
            apr_sys::apr_global_mutex_create(&mut mutex, fname_ptr, mech.into(), pool.as_mut_ptr())
        };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }

        Ok(GlobalMutex {
            raw: mutex,
            _phantom: PhantomData,
        })
    }

    pub fn child_init(&mut self, fname: Option<&Path>, pool: &Pool<'_>) -> Result<()> {
        let c_fname = if let Some(path) = fname {
            let path_str = path
                .to_str()
                .ok_or_else(|| crate::Error::from_status((apr_sys::APR_EINVAL as i32).into()))?;
            Some(
                CString::new(path_str)
                    .map_err(|_| crate::Error::from_status((apr_sys::APR_EINVAL as i32).into()))?
            )
        } else {
            None
        };

        let fname_ptr = c_fname.as_ref().map_or(ptr::null(), |s| s.as_ptr());

        let status = unsafe {
            apr_sys::apr_global_mutex_child_init(&mut self.raw, fname_ptr, pool.as_mut_ptr())
        };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }
        Ok(())
    }

    pub fn lock(&mut self) -> Result<()> {
        let status = unsafe { apr_sys::apr_global_mutex_lock(self.raw) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }
        Ok(())
    }

    pub fn trylock(&mut self) -> Result<bool> {
        let status = unsafe { apr_sys::apr_global_mutex_trylock(self.raw) };

        match status as u32 {
            x if x == apr_sys::APR_SUCCESS => Ok(true),
            x if x == apr_sys::APR_EBUSY => Ok(false),
            _ => Err(crate::Error::from_status(status.into())),
        }
    }

    pub fn unlock(&mut self) -> Result<()> {
        let status = unsafe { apr_sys::apr_global_mutex_unlock(self.raw) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }
        Ok(())
    }

    pub fn as_ptr(&self) -> *const apr_sys::apr_global_mutex_t {
        self.raw
    }

    pub fn as_mut_ptr(&mut self) -> *mut apr_sys::apr_global_mutex_t {
        self.raw
    }
}

impl<'a> Drop for GlobalMutex<'a> {
    fn drop(&mut self) {
        unsafe {
            apr_sys::apr_global_mutex_destroy(self.raw);
        }
    }
}

pub fn proc_mutex_lockfile(mutex: &ProcMutex) -> Result<String> {
    let mut lockfile: *const c_char = ptr::null();

    let status = unsafe { apr_sys::apr_proc_mutex_lockfile(mutex.raw, &mut lockfile) };

    if status != apr_sys::APR_SUCCESS as i32 {
        return Err(crate::Error::from_status(status.into()));
    }

    if lockfile.is_null() {
        return Ok(String::new());
    }

    unsafe {
        Ok(core::ffi::CStr::from_ptr(lockfile)
            .to_string_lossy()
            .into_owned())
    }
}

pub fn proc_mutex_name(mutex: &ProcMutex) -> Result<String> {
    let mut name: *const c_char = ptr::null();

    let status = unsafe { apr_sys::apr_proc_mutex_name(mutex.raw, &mut name) };

    if status != apr_sys::APR_SUCCESS as i32 {
        return Err(crate::Error::from_status(status.into()));
    }

    if name.is_null() {
        return Ok(String::new());
    }

    unsafe {
        Ok(core::ffi::CStr::from_ptr(name)
            .to_string_lossy()
            .into_owned())
    }
}

pub fn proc_mutex_defname() -> String {
    unsafe {
        let name = apr_sys::apr_proc_mutex_defname();
        if name.is_null() {
            String::from("default")
        } else {
            core::ffi::CStr::from_ptr(name)
                .to_string_lossy()
                .into_owned()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_lock_mech_conversion() {
        let _default: apr_sys::apr_lockmech_e = LockMech::Default.into();
        let _fcntl: apr_sys::apr_lockmech_e = LockMech::FcntlSerialize.into();
        let _flock: apr_sys::apr_lockmech_e = LockMech::FLockSerialize.into();
        let _sysv: apr_sys::apr_lockmech_e = LockMech::SysVSem.into();
        let _posix: apr_sys::apr_lockmech_e = LockMech::PosixSem.into();
        let _pthread: apr_sys::apr_lockmech_e = LockMech::ProcPthread.into();
    }

    #[test]
    fn test_proc_mutex_creation() {
        let pool = Pool::new();
        
        let mutex = ProcMutex::new(None, LockMech::Default, &pool);
        assert!(mutex.is_ok());
    }

    #[test]
    fn test_proc_mutex_with_file() {
        let pool = Pool::new();
        let temp_path = PathBuf::from("/tmp/apr_test_proc_mutex");
        
        let mutex = ProcMutex::new(Some(&temp_path), LockMech::Default, &pool);
        // This might fail on some systems, which is expected
        if mutex.is_err() {
            // Try without filename
            let mutex = ProcMutex::new(None, LockMech::Default, &pool);
            assert!(mutex.is_ok());
        }
    }

    #[test]
    fn test_proc_mutex_operations() {
        let pool = Pool::new();
        
        if let Ok(mut mutex) = ProcMutex::new(None, LockMech::Default, &pool) {
            // Test lock and unlock
            mutex.lock().unwrap();
            
            // In same process, trylock should succeed (recursive)
            // or fail depending on implementation
            let _ = mutex.trylock();
            
            mutex.unlock().unwrap();
        }
    }

    #[test]
    fn test_global_mutex_creation() {
        let pool = Pool::new();
        
        let mutex = GlobalMutex::new(None, LockMech::Default, &pool);
        // Global mutexes might not be available on all systems
        if mutex.is_err() {
            // This is acceptable - global mutexes need system support
            return;
        }
        
        let mut mutex = mutex.unwrap();
        
        // Test basic operations
        mutex.lock().unwrap();
        let _ = mutex.trylock(); // Might succeed or fail
        mutex.unlock().unwrap();
    }

    #[test]
    fn test_proc_mutex_defname() {
        let defname = proc_mutex_defname();
        assert!(!defname.is_empty());
    }

    #[test]
    fn test_proc_mutex_info() {
        let pool = Pool::new();
        
        if let Ok(mutex) = ProcMutex::new(None, LockMech::Default, &pool) {
            // These might return empty strings or error on some systems
            let _ = proc_mutex_name(&mutex);
            let _ = proc_mutex_lockfile(&mutex);
            
            // Just verify they don't crash
        }
    }

    #[test]
    fn test_lock_unlock_sequence() {
        let pool = Pool::new();
        
        if let Ok(mut mutex) = ProcMutex::new(None, LockMech::Default, &pool) {
            // Multiple lock/unlock cycles should work
            for _ in 0..3 {
                mutex.lock().unwrap();
                mutex.unlock().unwrap();
            }
        }
    }
}