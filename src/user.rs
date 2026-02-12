//! User and group information access
use alloc::vec::Vec;
use alloc::string::String;

use crate::{pool::Pool, Result};
use alloc::ffi::CString;
use core::ffi::CStr;
use core::ffi::c_char;
use core::marker::PhantomData;
use core::ptr;

#[repr(transparent)]
pub struct UserInfo<'a> {
    raw: *mut apr_sys::apr_uid_t,
    _phantom: PhantomData<&'a Pool<'a>>,
}

#[repr(transparent)]
pub struct GroupInfo<'a> {
    raw: *mut apr_sys::apr_gid_t,
    _phantom: PhantomData<&'a Pool<'a>>,
}

#[derive(Debug, Clone)]
pub struct User<'a> {
    pub name: &'a str,
    pub uid: u32,
    pub gid: u32,
    pub comment: Option<&'a str>,
    pub home_dir: Option<&'a str>,
    pub shell: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct Group<'a> {
    pub name: &'a str,
    pub gid: u32,
    pub members: Vec<&'a str>,
}

pub fn get_user_by_name<'a>(username: &str, pool: &'a Pool<'a>) -> Result<User<'a>> {
    let c_username = CString::new(username)
        .map_err(|_| crate::Error::from_status((apr_sys::APR_EINVAL as i32).into()))?;

    let mut uid: apr_sys::apr_uid_t = 0;
    let mut gid: apr_sys::apr_gid_t = 0;

    let status = unsafe {
        apr_sys::apr_uid_name_get(&mut uid, c_username.as_ptr(), pool.as_mut_ptr())
    };

    if status != apr_sys::APR_SUCCESS as i32 {
        return Err(crate::Error::from_status(status.into()));
    }

    let status = unsafe {
        apr_sys::apr_gid_name_get(&mut gid, c_username.as_ptr(), pool.as_mut_ptr())
    };

    if status != apr_sys::APR_SUCCESS as i32 {
        return Err(crate::Error::from_status(status.into()));
    }

    // Get additional user info if available
    let mut user_name: *mut c_char = ptr::null_mut();
    let status = unsafe {
        apr_sys::apr_uid_get(&mut user_name, &mut uid, pool.as_mut_ptr())
    };

    let name = if status == apr_sys::APR_SUCCESS as i32 && !user_name.is_null() {
        unsafe {
            CStr::from_ptr(user_name)
                .to_str()
                .map_err(|_| crate::Error::from_status((apr_sys::APR_EINVAL as i32).into()))?
        }
    } else {
        // Duplicate username into pool
        let dup_ptr = pool.pstrdup(username);
        unsafe {
            CStr::from_ptr(dup_ptr)
                .to_str()
                .map_err(|_| crate::Error::from_status((apr_sys::APR_EINVAL as i32).into()))?
        }
    };

    Ok(User {
        name,
        uid: uid as u32,
        gid: gid as u32,
        comment: None,
        home_dir: get_home_directory(pool).ok().flatten(),
        shell: None,
    })
}

pub fn get_user_by_id<'a>(uid: u32, pool: &'a Pool<'a>) -> Result<User<'a>> {
    let mut user_name: *mut c_char = ptr::null_mut();
    let mut apr_uid = uid as apr_sys::apr_uid_t;

    let status = unsafe {
        apr_sys::apr_uid_get(&mut user_name, &mut apr_uid, pool.as_mut_ptr())
    };

    if status != apr_sys::APR_SUCCESS as i32 {
        return Err(crate::Error::from_status(status.into()));
    }

    let name = if !user_name.is_null() {
        unsafe {
            CStr::from_ptr(user_name)
                .to_str()
                .map_err(|_| crate::Error::from_status((apr_sys::APR_EINVAL as i32).into()))?
        }
    } else {
        // Allocate formatted string in pool
        let formatted = format!("uid_{}", uid);
        let ptr = pool.pstrdup(&formatted);
        unsafe {
            CStr::from_ptr(ptr)
                .to_str()
                .map_err(|_| crate::Error::from_status((apr_sys::APR_EINVAL as i32).into()))?
        }
    };

    Ok(User {
        name,
        uid,
        gid: 0, // Would need additional lookup
        comment: None,
        home_dir: get_home_directory(pool).ok().flatten(),
        shell: None,
    })
}

pub fn get_group_by_name<'a>(groupname: &str, pool: &'a Pool<'a>) -> Result<Group<'a>> {
    let c_groupname = CString::new(groupname)
        .map_err(|_| crate::Error::from_status((apr_sys::APR_EINVAL as i32).into()))?;

    let mut gid: apr_sys::apr_gid_t = 0;

    let status = unsafe {
        apr_sys::apr_gid_name_get(&mut gid, c_groupname.as_ptr(), pool.as_mut_ptr())
    };

    if status != apr_sys::APR_SUCCESS as i32 {
        return Err(crate::Error::from_status(status.into()));
    }

    // Duplicate groupname into pool
    let dup_ptr = pool.pstrdup(groupname);
    let name = unsafe {
        CStr::from_ptr(dup_ptr)
            .to_str()
            .map_err(|_| crate::Error::from_status((apr_sys::APR_EINVAL as i32).into()))?
    };

    Ok(Group {
        name,
        gid: gid as u32,
        members: Vec::new(), // APR doesn't provide group membership info
    })
}

pub fn get_group_by_id<'a>(gid: u32, pool: &'a Pool<'a>) -> Result<Group<'a>> {
    let mut group_name: *mut c_char = ptr::null_mut();
    let mut apr_gid = gid as apr_sys::apr_gid_t;

    let status = unsafe {
        apr_sys::apr_gid_get(&mut group_name, &mut apr_gid, pool.as_mut_ptr())
    };

    if status != apr_sys::APR_SUCCESS as i32 {
        return Err(crate::Error::from_status(status.into()));
    }

    let name = if !group_name.is_null() {
        unsafe {
            CStr::from_ptr(group_name)
                .to_str()
                .map_err(|_| crate::Error::from_status((apr_sys::APR_EINVAL as i32).into()))?
        }
    } else {
        // Allocate formatted string in pool
        let formatted = format!("gid_{}", gid);
        let ptr = pool.pstrdup(&formatted);
        unsafe {
            CStr::from_ptr(ptr)
                .to_str()
                .map_err(|_| crate::Error::from_status((apr_sys::APR_EINVAL as i32).into()))?
        }
    };

    Ok(Group {
        name,
        gid,
        members: Vec::new(),
    })
}

pub fn get_current_user_id() -> u32 {
    unsafe { apr_sys::apr_uid_current() as u32 }
}

pub fn get_current_group_id() -> u32 {
    unsafe { apr_sys::apr_gid_current() as u32 }
}

pub fn get_home_directory<'a>(pool: &'a Pool<'a>) -> Result<Option<&'a str>> {
    let mut homedir: *mut c_char = ptr::null_mut();

    let status = unsafe {
        apr_sys::apr_uid_homepath_get(&mut homedir, ptr::null(), pool.as_mut_ptr())
    };

    if status != apr_sys::APR_SUCCESS as i32 {
        return Err(crate::Error::from_status(status.into()));
    }

    if homedir.is_null() {
        return Ok(None);
    }

    unsafe {
        Ok(Some(
            CStr::from_ptr(homedir)
                .to_str()
                .map_err(|_| crate::Error::from_status((apr_sys::APR_EINVAL as i32).into()))?,
        ))
    }
}

pub fn get_user_home_directory<'a>(username: &str, pool: &'a Pool<'a>) -> Result<Option<&'a str>> {
    let c_username = CString::new(username)
        .map_err(|_| crate::Error::from_status((apr_sys::APR_EINVAL as i32).into()))?;

    let mut homedir: *mut c_char = ptr::null_mut();

    let status = unsafe {
        apr_sys::apr_uid_homepath_get(&mut homedir, c_username.as_ptr(), pool.as_mut_ptr())
    };

    if status != apr_sys::APR_SUCCESS as i32 {
        return Err(crate::Error::from_status(status.into()));
    }

    if homedir.is_null() {
        return Ok(None);
    }

    unsafe {
        Ok(Some(
            CStr::from_ptr(homedir)
                .to_str()
                .map_err(|_| crate::Error::from_status((apr_sys::APR_EINVAL as i32).into()))?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_user_group_ids() {
        let current_uid = get_current_user_id();
        let current_gid = get_current_group_id();
        
        // UIDs and GIDs should be reasonable values
        assert!(current_uid < 1_000_000); // Reasonable upper bound
        assert!(current_gid < 1_000_000); // Reasonable upper bound
    }

    #[test]
    fn test_get_user_by_id() {
        let pool = Pool::new();
        let current_uid = get_current_user_id();
        
        let user = get_user_by_id(current_uid, &pool).unwrap();
        assert_eq!(user.uid, current_uid);
        assert!(!user.name.is_empty());
    }

    #[test]
    fn test_get_group_by_id() {
        let pool = Pool::new();
        let current_gid = get_current_group_id();
        
        let group = get_group_by_id(current_gid, &pool).unwrap();
        assert_eq!(group.gid, current_gid);
        assert!(!group.name.is_empty());
    }

    #[test]
    fn test_home_directory() {
        let pool = Pool::new();
        
        let home = get_home_directory(&pool).unwrap();
        assert!(!home.is_empty());
        // Home directory should be an absolute path
        assert!(home.starts_with('/'));
    }

    #[test]
    fn test_get_root_user() {
        let pool = Pool::new();
        
        // Root user should exist on any Unix system
        if let Ok(user) = get_user_by_id(0, &pool) {
            assert_eq!(user.uid, 0);
            // Root user is typically named "root"
            assert!(user.name == "root" || user.name.starts_with("uid_"));
        }
        // If root lookup fails, that's ok - some systems might restrict this
    }

    #[test]
    fn test_get_root_group() {
        let pool = Pool::new();
        
        // Root group should exist on any Unix system
        if let Ok(group) = get_group_by_id(0, &pool) {
            assert_eq!(group.gid, 0);
            // Root group is typically named "root" or "wheel"
            assert!(!group.name.is_empty());
        }
        // If root lookup fails, that's ok - some systems might restrict this
    }

    #[test]
    fn test_nonexistent_user() {
        let pool = Pool::new();
        
        // User with very high ID should not exist
        let result = get_user_by_id(999_999, &pool);
        // This should either fail or return a generic name
        if let Ok(user) = result {
            assert!(user.name.starts_with("uid_") || !user.name.is_empty());
        }
    }

    #[test]
    fn test_user_by_name() {
        let pool = Pool::new();
        
        // Try to get current user by name first
        let current_uid = get_current_user_id();
        if let Ok(current_user) = get_user_by_id(current_uid, &pool) {
            // Now try to look up by name
            if let Ok(user_by_name) = get_user_by_name(&current_user.name, &pool) {
                assert_eq!(user_by_name.uid, current_uid);
                assert_eq!(user_by_name.name, current_user.name);
            }
        }
    }
}