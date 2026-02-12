//! Status codes and error handling.

use alloc::string::{String, ToString};

/// Status code type.
pub type StatusCode = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// APR status codes that can be returned from various operations.
pub enum Status {
    /// Operation completed successfully.
    Success, // APR_SUCCESS
    /// Could not perform a stat on the file.
    NoStat, // APR_ENOSTAT
    /// Could not create a new pool.
    NoPool, // APR_ENOPOOL
    /// An invalid date has been provided.
    BadDate, // APR_EBADDATE
    /// An invalid socket was specified.
    InvalidSocket, // APR_EINVALSOCK
    /// No process was specified.
    NoProcess, // APR_ENOPROC
    /// No time was specified.
    NoTime, // APR_ENOTIME
    /// No directory was specified.
    NoDirectory, // APR_ENODIR
    /// No lock was specified.
    NoLock, // APR_ENOLOCK
    /// No poll was specified.
    NoPoll, // APR_ENOPOLL
    /// No socket was specified.
    NoSocket, // APR_ENOSOCKET
    /// No thread was specified.
    NoThread, // APR_ENOTHREAD
    /// No thread key was specified.
    NoThreadKey, // APR_ENOTHDKEY
    /// There is no shared memory available.
    NoSharedMemoryAvailable, // APR_ENOSHMAVAIL
    /// A DSO loading error occurred.
    DSOOpen, // APR_EDSOOPEN
    /// A general failure, not covered elsewhere.
    General, // APR_EGENERAL
    /// An invalid IP address was specified.
    BadIpAddress, // APR_EBADIP
    /// An invalid mask was specified.
    BadMask, // APR_EBADMASK
    /// Could not find the requested symbol.
    SymbolNotFound, // APR_ESYMNOTFOUND
    /// Not enough entropy to complete the operation.
    NotEnoughEntropy, // APR_ENOTENOUGHENTROPY

    /// Program is currently executing in the child.
    InChild, // APR_INCHILD
    /// Program is currently executing in the parent.
    InParent, // APR_INPARENT
    /// The thread is detached.
    Detach, // APR_DETACH
    /// The thread is not detached.
    NotDetach, // APR_NOTDETACH
    /// The child has finished executing.
    ChildDone, // APR_CHILD_DONE
    /// The child has not finished executing.
    ChildNotDone, // APR_CHILD_NOTDONE
    /// The operation did not finish before the timeout.
    TimeUp, // APR_TIMEUP
    /// The operation was incomplete.
    Incomplete, // APR_INCOMPLETE
    /// An invalid character was specified.
    BadCh, // APR_BADCH
    /// An invalid argument was passed to a function.
    BadArgument, // APR_BADARG
    /// The end of file was reached.
    Eof, // APR_EOF
    /// Could not find the requested resource.
    NotFound, // APR_NOTFOUND
    /// This is an anonymous operation.
    Anonymous, // APR_ANONYMOUS
    /// This is a file based operation.
    FileBased, // APR_FILEBASED
    /// This is a key based operation.
    KeyBased, // APR_KEYBASED
    /// There was a problem during initialization.
    Initializer, // APR_EINIT
    /// The feature has not been implemented.
    NotImplemented, // APR_ENOTIMPL
    /// Two parameters were not compatible.
    Mismatch, // APR_EMISMATCH
    /// The given path was absolute.
    Absolute, // APR_EABSOLUTE
    /// The given path was relative.
    Relative, // APR_ERELATIVE
    /// The given path was neither relative nor absolute.
    IncompleteError, // APR_EINCOMPLETE
    /// The given path was above the root path.
    AboveRoot, // APR_EABOVEROOT
    /// The given resource is busy.
    Busy, // APR_EBUSY
    /// The process is not recognized by the system.
    ProcessUnknown, // APR_EPROC_UNKNOWN
}

impl Status {
    /// Check if this status represents a success
    pub fn is_success(&self) -> bool {
        matches!(self, Status::Success)
    }

    /// Check if this status represents an error
    pub fn is_error(&self) -> bool {
        !self.is_success()
    }

    /// Get the raw OS error code, if available
    pub fn raw_os_error(&self) -> Option<i32> {
        let status_code: u32 = (*self).into();
        match self {
            Status::Success => None,
            _ if status_code >= apr_sys::APR_OS_START_SYSERR => {
                Some((status_code - apr_sys::APR_OS_START_SYSERR) as i32)
            }
            _ => None,
        }
    }

    /// Get the error message for this status code
    pub fn strerror(&self) -> String {
        let status_code: u32 = (*self).into();
        let buf = unsafe {
            let mut buf = [0u8; 1024];
            apr_sys::apr_strerror(
                status_code as apr_sys::apr_status_t,
                buf.as_mut_ptr() as *mut core::ffi::c_char,
                buf.len(),
            );
            buf
        };
        String::from_utf8_lossy(&buf).to_string()
    }
}

impl From<u32> for Status {
    fn from(status: u32) -> Self {
        match status {
            apr_sys::APR_SUCCESS => Status::Success,
            apr_sys::APR_ENOSTAT => Status::NoStat,
            apr_sys::APR_ENOPOOL => Status::NoPool,
            apr_sys::APR_EBADDATE => Status::BadDate,
            apr_sys::APR_EINVALSOCK => Status::InvalidSocket,
            apr_sys::APR_ENOPROC => Status::NoProcess,
            apr_sys::APR_ENOTIME => Status::NoTime,
            apr_sys::APR_ENODIR => Status::NoDirectory,
            apr_sys::APR_ENOLOCK => Status::NoLock,
            apr_sys::APR_ENOPOLL => Status::NoPoll,
            apr_sys::APR_ENOSOCKET => Status::NoSocket,
            apr_sys::APR_ENOTHREAD => Status::NoThread,
            apr_sys::APR_ENOTHDKEY => Status::NoThreadKey,
            apr_sys::APR_ENOSHMAVAIL => Status::NoSharedMemoryAvailable,
            apr_sys::APR_EDSOOPEN => Status::DSOOpen,
            apr_sys::APR_EGENERAL => Status::General,
            apr_sys::APR_EBADIP => Status::BadIpAddress,
            apr_sys::APR_EBADMASK => Status::BadMask,
            apr_sys::APR_ESYMNOTFOUND => Status::SymbolNotFound,
            apr_sys::APR_ENOTENOUGHENTROPY => Status::NotEnoughEntropy,

            apr_sys::APR_INCHILD => Status::InChild,
            apr_sys::APR_INPARENT => Status::InParent,
            apr_sys::APR_DETACH => Status::Detach,
            apr_sys::APR_NOTDETACH => Status::NotDetach,
            apr_sys::APR_CHILD_DONE => Status::ChildDone,
            apr_sys::APR_CHILD_NOTDONE => Status::ChildNotDone,
            apr_sys::APR_TIMEUP => Status::TimeUp,
            apr_sys::APR_INCOMPLETE => Status::Incomplete,
            apr_sys::APR_BADCH => Status::BadCh,
            apr_sys::APR_BADARG => Status::BadArgument,
            apr_sys::APR_EOF => Status::Eof,
            apr_sys::APR_NOTFOUND => Status::NotFound,
            apr_sys::APR_ANONYMOUS => Status::Anonymous,
            apr_sys::APR_FILEBASED => Status::FileBased,
            apr_sys::APR_KEYBASED => Status::KeyBased,
            apr_sys::APR_EINIT => Status::Initializer,
            apr_sys::APR_ENOTIMPL => Status::NotImplemented,
            apr_sys::APR_EMISMATCH => Status::Mismatch,
            apr_sys::APR_EABSOLUTE => Status::Absolute,
            apr_sys::APR_ERELATIVE => Status::Relative,
            apr_sys::APR_EINCOMPLETE => Status::IncompleteError,
            apr_sys::APR_EABOVEROOT => Status::AboveRoot,
            apr_sys::APR_EBUSY => Status::Busy,
            apr_sys::APR_EPROC_UNKNOWN => Status::ProcessUnknown,

            // For unknown or OS-specific error codes, return a General error
            // APR maps OS errors into its status space
            _ => Status::General,
        }
    }
}

impl core::fmt::Display for Status {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let status_code: u32 = (*self).into();
        write!(f, "{} ({})", self.strerror(), status_code)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Status {}

impl From<Status> for u32 {
    fn from(status: Status) -> Self {
        match status {
            Status::Success => apr_sys::APR_SUCCESS,
            Status::NoStat => apr_sys::APR_ENOSTAT,
            Status::NoPool => apr_sys::APR_ENOPOOL,
            Status::BadDate => apr_sys::APR_EBADDATE,
            Status::InvalidSocket => apr_sys::APR_EINVALSOCK,
            Status::NoProcess => apr_sys::APR_ENOPROC,
            Status::NoTime => apr_sys::APR_ENOTIME,
            Status::NoDirectory => apr_sys::APR_ENODIR,
            Status::NoLock => apr_sys::APR_ENOLOCK,
            Status::NoPoll => apr_sys::APR_ENOPOLL,
            Status::NoSocket => apr_sys::APR_ENOSOCKET,
            Status::NoThread => apr_sys::APR_ENOTHREAD,
            Status::NoThreadKey => apr_sys::APR_ENOTHDKEY,
            Status::NoSharedMemoryAvailable => apr_sys::APR_ENOSHMAVAIL,
            Status::DSOOpen => apr_sys::APR_EDSOOPEN,
            Status::General => apr_sys::APR_EGENERAL,
            Status::BadIpAddress => apr_sys::APR_EBADIP,
            Status::BadMask => apr_sys::APR_EBADMASK,
            Status::SymbolNotFound => apr_sys::APR_ESYMNOTFOUND,
            Status::NotEnoughEntropy => apr_sys::APR_ENOTENOUGHENTROPY,
            Status::InChild => apr_sys::APR_INCHILD,
            Status::InParent => apr_sys::APR_INPARENT,
            Status::Detach => apr_sys::APR_DETACH,
            Status::NotDetach => apr_sys::APR_NOTDETACH,
            Status::ChildDone => apr_sys::APR_CHILD_DONE,
            Status::ChildNotDone => apr_sys::APR_CHILD_NOTDONE,
            Status::TimeUp => apr_sys::APR_TIMEUP,
            Status::Incomplete => apr_sys::APR_INCOMPLETE,
            Status::BadCh => apr_sys::APR_BADCH,
            Status::BadArgument => apr_sys::APR_BADARG,
            Status::Eof => apr_sys::APR_EOF,
            Status::NotFound => apr_sys::APR_NOTFOUND,
            Status::Anonymous => apr_sys::APR_ANONYMOUS,
            Status::FileBased => apr_sys::APR_FILEBASED,
            Status::KeyBased => apr_sys::APR_KEYBASED,
            Status::Initializer => apr_sys::APR_EINIT,
            Status::NotImplemented => apr_sys::APR_ENOTIMPL,
            Status::Mismatch => apr_sys::APR_EMISMATCH,
            Status::Absolute => apr_sys::APR_EABSOLUTE,
            Status::Relative => apr_sys::APR_ERELATIVE,
            Status::IncompleteError => apr_sys::APR_EINCOMPLETE,
            Status::AboveRoot => apr_sys::APR_EABOVEROOT,
            Status::Busy => apr_sys::APR_EBUSY,
            Status::ProcessUnknown => apr_sys::APR_EPROC_UNKNOWN,
        }
    }
}

impl From<i32> for Status {
    fn from(status: i32) -> Self {
        (status as u32).into()
    }
}

#[cfg(feature = "std")]
impl From<std::io::ErrorKind> for Status {
    fn from(kind: std::io::ErrorKind) -> Self {
        (kind as u32).into()
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Status {
    fn from(error: std::io::Error) -> Self {
        error.kind().into()
    }
}

#[cfg(feature = "std")]
impl From<Status> for std::io::Error {
    fn from(status: Status) -> Self {
        let kind = match status {
            Status::NotFound | Status::NoDirectory => std::io::ErrorKind::NotFound,
            Status::BadArgument | Status::InvalidSocket => std::io::ErrorKind::InvalidInput,
            Status::Eof => std::io::ErrorKind::UnexpectedEof,
            Status::Busy => std::io::ErrorKind::ResourceBusy,
            Status::TimeUp => std::io::ErrorKind::TimedOut,
            _ => return std::io::Error::other(status),
        };

        std::io::Error::new(kind, status)
    }
}

/// Generic helper to convert APR status codes to Results
///
/// This follows the common C pattern where 0/APR_SUCCESS means success
/// and non-zero means error. Used throughout APR and libraries built on it.
pub fn apr_result(status_code: i32) -> Result<(), Status> {
    if status_code == apr_sys::APR_SUCCESS as i32 {
        Ok(())
    } else {
        Err(Status::from(status_code))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

    #[test]
    fn test_status_to_u32_returns_apr_value() {
        // Test that converting Status to u32 returns the actual APR constant value,
        // not the enum discriminant
        let status = Status::Success;
        let code: u32 = status.into();
        assert_eq!(code, apr_sys::APR_SUCCESS);

        let status = Status::NotFound;
        let code: u32 = status.into();
        assert_eq!(code, apr_sys::APR_NOTFOUND);

        let status = Status::Eof;
        let code: u32 = status.into();
        assert_eq!(code, apr_sys::APR_EOF);
    }

    #[test]
    fn test_status_roundtrip() {
        // Test that we can convert from APR constant to Status and back
        let original = apr_sys::APR_ENOSTAT;
        let status = Status::from(original);
        let code: u32 = status.into();
        assert_eq!(code, original);

        let original = apr_sys::APR_BADARG;
        let status = Status::from(original);
        let code: u32 = status.into();
        assert_eq!(code, original);
    }

    #[test]
    fn test_status_display_uses_correct_value() {
        // Verify that Display shows the correct APR status code
        let status = Status::NotFound;
        let display = format!("{}", status);
        // Should contain the APR_NOTFOUND value, not the discriminant
        assert!(display.contains(&format!("({})", apr_sys::APR_NOTFOUND)));
    }
}
