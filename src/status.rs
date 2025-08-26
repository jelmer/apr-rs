//! Status codes and error handling.

/// Status code type.
pub type StatusCode = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
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
        match self {
            Status::Success => None,
            e if (*e) as u32 >= apr_sys::APR_OS_START_SYSERR => {
                Some((*e as u32 - apr_sys::APR_OS_START_SYSERR) as i32)
            }
            _ => None,
        }
    }

    /// Get the error message for this status code
    pub fn strerror(&self) -> String {
        let buf = unsafe {
            let mut buf = [0u8; 1024];
            apr_sys::apr_strerror(
                *self as apr_sys::apr_status_t,
                buf.as_mut_ptr() as *mut std::ffi::c_char,
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

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} ({})", self.strerror(), *self as u32)
    }
}

impl std::error::Error for Status {}

impl From<Status> for u32 {
    fn from(status: Status) -> Self {
        status as u32
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
