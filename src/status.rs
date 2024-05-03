pub type StatusCode = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        matches!(self, Status::Success)
    }

    pub fn is_error(&self) -> bool {
        !self.is_success()
    }

    pub fn raw_os_error(&self) -> Option<i32> {
        match self {
            Status::Success => None,
            e if (*e) as u32 >= crate::generated::APR_OS_START_SYSERR => {
                Some((*e as u32 - crate::generated::APR_OS_START_SYSERR) as i32)
            }
            _ => None,
        }
    }

    pub fn strerror(&self) -> String {
        let buf = unsafe {
            let mut buf = [0u8; 1024];
            crate::generated::apr_strerror(
                *self as crate::generated::apr_status_t,
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
