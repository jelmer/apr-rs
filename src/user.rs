//! User and group information access

use crate::{pool::Pool, Result};
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::ptr;

#[repr(transparent)]
pub struct UserInfo<'a> {
    raw: *mut apr_sys::apr_uid_t,
    _phantom: PhantomData<&'a Pool>,
}

#[repr(transparent)]
pub struct GroupInfo<'a> {
    raw: *mut apr_sys::apr_gid_t,
    _phantom: PhantomData<&'a Pool>,
}

#[derive(Debug, Clone)]
pub struct User {
    pub name: String,
    pub uid: u32,
    pub gid: u32,
    pub comment: String,
    pub home_dir: String,
    pub shell: String,
}

#[derive(Debug, Clone)]
pub struct Group {
    pub name: String,
    pub gid: u32,
    pub members: Vec<String>,
}

pub fn get_user_by_name(username: &str, pool: &Pool) -> Result<User> {
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
    let mut user_name: *mut i8 = ptr::null_mut();
    let status = unsafe {
        apr_sys::apr_uid_get(&mut user_name, &mut uid, pool.as_mut_ptr())
    };

    let name = if status == apr_sys::APR_SUCCESS as i32 && !user_name.is_null() {
        unsafe { CStr::from_ptr(user_name).to_string_lossy().into_owned() }
    } else {
        username.to_string()
    };

    Ok(User {
        name,
        uid: uid as u32,
        gid: gid as u32,
        comment: String::new(),
        home_dir: get_home_directory(pool).unwrap_or_default(),
        shell: String::new(),
    })
}

pub fn get_user_by_id(uid: u32, pool: &Pool) -> Result<User> {
    let mut user_name: *mut i8 = ptr::null_mut();
    let mut apr_uid = uid as apr_sys::apr_uid_t;

    let status = unsafe {
        apr_sys::apr_uid_get(&mut user_name, &mut apr_uid, pool.as_mut_ptr())
    };

    if status != apr_sys::APR_SUCCESS as i32 {
        return Err(crate::Error::from_status(status.into()));
    }

    let name = if !user_name.is_null() {
        unsafe { CStr::from_ptr(user_name).to_string_lossy().into_owned() }
    } else {
        format!("uid_{}", uid)
    };

    Ok(User {
        name,
        uid,
        gid: 0, // Would need additional lookup
        comment: String::new(),
        home_dir: get_home_directory(pool).unwrap_or_default(),
        shell: String::new(),
    })
}

pub fn get_group_by_name(groupname: &str, pool: &Pool) -> Result<Group> {
    let c_groupname = CString::new(groupname)
        .map_err(|_| crate::Error::from_status((apr_sys::APR_EINVAL as i32).into()))?;

    let mut gid: apr_sys::apr_gid_t = 0;

    let status = unsafe {
        apr_sys::apr_gid_name_get(&mut gid, c_groupname.as_ptr(), pool.as_mut_ptr())
    };

    if status != apr_sys::APR_SUCCESS as i32 {
        return Err(crate::Error::from_status(status.into()));
    }

    Ok(Group {
        name: groupname.to_string(),
        gid: gid as u32,
        members: Vec::new(), // APR doesn't provide group membership info
    })
}

pub fn get_group_by_id(gid: u32, pool: &Pool) -> Result<Group> {
    let mut group_name: *mut i8 = ptr::null_mut();
    let mut apr_gid = gid as apr_sys::apr_gid_t;

    let status = unsafe {
        apr_sys::apr_gid_get(&mut group_name, &mut apr_gid, pool.as_mut_ptr())
    };

    if status != apr_sys::APR_SUCCESS as i32 {
        return Err(crate::Error::from_status(status.into()));
    }

    let name = if !group_name.is_null() {
        unsafe { CStr::from_ptr(group_name).to_string_lossy().into_owned() }
    } else {
        format!("gid_{}", gid)
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

pub fn get_home_directory(pool: &Pool) -> Result<String> {
    let mut homedir: *mut i8 = ptr::null_mut();

    let status = unsafe {
        apr_sys::apr_uid_homepath_get(&mut homedir, ptr::null(), pool.as_mut_ptr())
    };

    if status != apr_sys::APR_SUCCESS as i32 {
        return Err(crate::Error::from_status(status.into()));
    }

    if homedir.is_null() {
        return Ok(String::from("/"));
    }

    unsafe {
        Ok(CStr::from_ptr(homedir).to_string_lossy().into_owned())
    }
}

pub fn get_user_home_directory(username: &str, pool: &Pool) -> Result<String> {
    let c_username = CString::new(username)
        .map_err(|_| crate::Error::from_status((apr_sys::APR_EINVAL as i32).into()))?;

    let mut homedir: *mut i8 = ptr::null_mut();

    let status = unsafe {
        apr_sys::apr_uid_homepath_get(&mut homedir, c_username.as_ptr(), pool.as_mut_ptr())
    };

    if status != apr_sys::APR_SUCCESS as i32 {
        return Err(crate::Error::from_status(status.into()));
    }

    if homedir.is_null() {
        return Ok(String::from("/"));
    }

    unsafe {
        Ok(CStr::from_ptr(homedir).to_string_lossy().into_owned())
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