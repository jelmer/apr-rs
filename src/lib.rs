//! Rust bindings for the Apache Portable Runtime (APR) library.
//!
//! This crate provides a safe interface to the APR library. It is intended to be used by other
//! crates that need to interface with APR.

mod generated;

pub mod date;
pub mod file;
pub mod hash;
pub mod pool;
pub mod tables;
pub mod time;
pub mod uri;

pub use pool::Pool;

pub use generated::apr_byte_t;
pub use generated::apr_file_t;
pub use generated::apr_getopt_option_t;
pub use generated::apr_getopt_t;
pub use generated::apr_int16_t;
pub use generated::apr_int32_t;
pub use generated::apr_int64_t;
pub use generated::apr_off_t;
pub use generated::apr_pool_t;
pub use generated::apr_size_t;
pub use generated::apr_status_t;
pub use generated::apr_time_t;
pub use generated::apr_uint16_t;
pub use generated::apr_uint32_t;
pub use generated::apr_uint64_t;

#[ctor::ctor]
fn initialize_apr() {
    unsafe {
        assert!(generated::apr_initialize() == generated::APR_SUCCESS as i32);
    }
}

pub type StatusCode = u32;

#[derive(Debug)]
pub enum Status {
    Success,                 // APR_SUCCESS
    NoStat,                  // APR_ENOSTAT
    NoPool,                  // APR_ENOPOOL
    BadDate,                 // APR_EBADDATE
    InvalidSocket,           // APR_EINVALSOCK
    NoProcess,               // APR_ENOPROC
    NoTime,                  // APR_ENOTIME
    NoDirectory,             // APR_ENODIR
    NoLock,                  // APR_ENOLOCK
    NoPoll,                  // APR_ENOPOLL
    NoSocket,                // APR_ENOSOCKET
    NoThread,                // APR_ENOTHREAD
    NoThreadKey,             // APR_ENOTHDKEY
    NoSharedMemoryAvailable, // APR_ENOSHMAVAIL
    DSOOpen,                 // APR_EDSOOPEN
    General,                 // APR_EGENERAL
    BadIpAddress,            // APR_EBADIP
    BadMask,                 // APR_EBADMASK
    SymbolNotFound,          // APR_ESYMNOTFOUND
    NotEnoughEntropy,        // APR_ENOTENOUGHENTROPY

    InChild,         // APR_INCHILD
    InParent,        // APR_INPARENT
    Detach,          // APR_DETACH
    NotDetach,       // APR_NOTDETACH
    ChildDone,       // APR_CHILD_DONE
    ChildNotDone,    // APR_CHILD_NOTDONE
    TimeUp,          // APR_TIMEUP
    Incomplete,      // APR_INCOMPLETE
    BadCh,           // APR_BADCH
    BadArgument,     // APR_BADARG
    Eof,             // APR_EOF
    NotFound,        // APR_NOTFOUND
    Anonymous,       // APR_ANONYMOUS
    FileBased,       // APR_FILEBASED
    KeyBased,        // APR_KEYBASED
    Initializer,     // APR_EINIT
    NotImplemented,  // APR_ENOTIMPL
    Mismatch,        // APR_EMISMATCH
    Absolute,        // APR_EABSOLUTE
    Relative,        // APR_ERELATIVE
    IncompleteError, // APR_EINCOMPLETE
    AboveRoot,       // APR_EABOVEROOT
    Busy,            // APR_EBUSY
    ProcessUnknown,  // APR_EPROC_UNKNOWN
}

impl Status {
    pub fn is_success(&self) -> bool {
        match self {
            Status::Success => true,
            _ => false,
        }
    }

    pub fn is_error(&self) -> bool {
        !self.is_success()
    }
}

impl From<u32> for Status {
    fn from(status: u32) -> Self {
        match status {
            crate::generated::APR_SUCCESS => Status::Success,
            crate::generated::APR_ENOSTAT => Status::NoStat,
            crate::generated::APR_ENOPOOL => Status::NoPool,
            crate::generated::APR_EBADDATE => Status::BadDate,
            crate::generated::APR_EINVALSOCK => Status::InvalidSocket,
            crate::generated::APR_ENOPROC => Status::NoProcess,
            crate::generated::APR_ENOTIME => Status::NoTime,
            crate::generated::APR_ENODIR => Status::NoDirectory,
            crate::generated::APR_ENOLOCK => Status::NoLock,
            crate::generated::APR_ENOPOLL => Status::NoPoll,
            crate::generated::APR_ENOSOCKET => Status::NoSocket,
            crate::generated::APR_ENOTHREAD => Status::NoThread,
            crate::generated::APR_ENOTHDKEY => Status::NoThreadKey,
            crate::generated::APR_ENOSHMAVAIL => Status::NoSharedMemoryAvailable,
            crate::generated::APR_EDSOOPEN => Status::DSOOpen,
            crate::generated::APR_EGENERAL => Status::General,
            crate::generated::APR_EBADIP => Status::BadIpAddress,
            crate::generated::APR_EBADMASK => Status::BadMask,
            crate::generated::APR_ESYMNOTFOUND => Status::SymbolNotFound,
            crate::generated::APR_ENOTENOUGHENTROPY => Status::NotEnoughEntropy,

            crate::generated::APR_INCHILD => Status::InChild,
            crate::generated::APR_INPARENT => Status::InParent,
            crate::generated::APR_DETACH => Status::Detach,
            crate::generated::APR_NOTDETACH => Status::NotDetach,
            crate::generated::APR_CHILD_DONE => Status::ChildDone,
            crate::generated::APR_CHILD_NOTDONE => Status::ChildNotDone,
            crate::generated::APR_TIMEUP => Status::TimeUp,
            crate::generated::APR_INCOMPLETE => Status::Incomplete,
            crate::generated::APR_BADCH => Status::BadCh,
            crate::generated::APR_BADARG => Status::BadArgument,
            crate::generated::APR_EOF => Status::Eof,
            crate::generated::APR_NOTFOUND => Status::NotFound,
            crate::generated::APR_ANONYMOUS => Status::Anonymous,
            crate::generated::APR_FILEBASED => Status::FileBased,
            crate::generated::APR_KEYBASED => Status::KeyBased,
            crate::generated::APR_EINIT => Status::Initializer,
            crate::generated::APR_ENOTIMPL => Status::NotImplemented,
            crate::generated::APR_EMISMATCH => Status::Mismatch,
            crate::generated::APR_EABSOLUTE => Status::Absolute,
            crate::generated::APR_ERELATIVE => Status::Relative,
            crate::generated::APR_EINCOMPLETE => Status::IncompleteError,
            crate::generated::APR_EABOVEROOT => Status::AboveRoot,
            crate::generated::APR_EBUSY => Status::Busy,
            crate::generated::APR_EPROC_UNKNOWN => Status::ProcessUnknown,

            _ => panic!("Unknown status code: {}", status),
        }
    }
}

impl From<i32> for Status {
    fn from(status: i32) -> Self {
        (status as u32).into()
    }
}

impl From<std::io::ErrorKind> for Status {
    fn from(kind: std::io::ErrorKind) -> Self {
        (kind as u32).into()
    }
}

impl From<std::io::Error> for Status {
    fn from(error: std::io::Error) -> Self {
        error.kind().into()
    }
}

pub const APU_MAJOR_VERSION: u32 = crate::generated::APU_MAJOR_VERSION;
pub const APU_MINOR_VERSION: u32 = crate::generated::APU_MINOR_VERSION;
pub const APU_PATCH_VERSION: u32 = crate::generated::APU_PATCH_VERSION;

pub fn apu_version_string() -> &'static str {
    unsafe {
        let ptr = crate::generated::apu_version_string();
        std::ffi::CStr::from_ptr(ptr).to_str().unwrap()
    }
}

pub const APR_MAJOR_VERSION: u32 = crate::generated::APR_MAJOR_VERSION;
pub const APR_MINOR_VERSION: u32 = crate::generated::APR_MINOR_VERSION;
pub const APR_PATCH_VERSION: u32 = crate::generated::APR_PATCH_VERSION;

pub fn apr_version_string() -> &'static str {
    unsafe {
        let ptr = crate::generated::apr_version_string();
        std::ffi::CStr::from_ptr(ptr).to_str().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apu_version_string() {
        apu_version_string();
    }

    #[test]
    fn test_apr_version_string() {
        apr_version_string();
    }
}
