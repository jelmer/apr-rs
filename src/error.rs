//! Improved error handling for APR operations
use crate::status::Status;
#[cfg(feature = "std")]
use alloc::boxed::Box;
use alloc::string::String;
use core::fmt;

/// High-level error type that wraps Status with additional context
#[derive(Debug)]
pub struct Error {
    status: Status,
    context: Option<String>,
    #[cfg(feature = "std")]
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl Error {
    /// Create a new Error from a Status
    pub fn from_status(status: Status) -> Self {
        Error {
            status,
            context: None,
            #[cfg(feature = "std")]
            source: None,
        }
    }

    /// Add context to the error
    pub fn context<S: Into<String>>(mut self, ctx: S) -> Self {
        self.context = Some(ctx.into());
        self
    }

    /// Add a source error
    #[cfg(feature = "std")]
    pub fn with_source<E: std::error::Error + Send + Sync + 'static>(mut self, source: E) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// Get the underlying Status
    pub fn status(&self) -> Status {
        self.status
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(context) = &self.context {
            write!(f, "{}: {}", context, self.status)
        } else {
            write!(f, "{}", self.status)
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

impl From<Status> for Error {
    fn from(status: Status) -> Self {
        Error::from_status(status)
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::from_status(Status::General).with_source(err)
    }
}

impl From<core::ffi::c_str::FromBytesWithNulError> for Error {
    fn from(_err: core::ffi::c_str::FromBytesWithNulError) -> Self {
        Error::from_status(Status::BadArgument)
    }
}

impl From<core::str::Utf8Error> for Error {
    fn from(_err: core::str::Utf8Error) -> Self {
        Error::from_status(Status::BadArgument)
    }
}

/// Result type using the improved Error
pub type Result<T> = core::result::Result<T, Error>;

/// Extension trait to add context to Results
pub trait ErrorContext<T> {
    /// Add context to an error result
    fn context<S: Into<String>>(self, ctx: S) -> Result<T>;

    /// Add context using a closure (lazy evaluation)
    fn with_context<F, S>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> S,
        S: Into<String>;
}

impl<T, E> ErrorContext<T> for core::result::Result<T, E>
where
    E: Into<Error>,
{
    fn context<S: Into<String>>(self, ctx: S) -> Result<T> {
        self.map_err(|e| e.into().context(ctx))
    }

    fn with_context<F, S>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> S,
        S: Into<String>,
    {
        self.map_err(|e| e.into().context(f()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

    #[test]
    fn test_error_from_status() {
        let err = Error::from_status(Status::NotFound);
        assert_eq!(err.status(), Status::NotFound);
        assert!(err.context.is_none());
    }

    #[test]
    fn test_error_with_context() {
        let err = Error::from_status(Status::NotFound).context("Failed to find config file");

        assert_eq!(err.status(), Status::NotFound);
        assert!(err.context.is_some());
        assert!(format!("{}", err).contains("Failed to find config file"));
    }

    #[test]
    fn test_error_context_trait() {
        let result: core::result::Result<(), Status> = Err(Status::NotFound);
        let err = result.context("File operation failed").unwrap_err();

        assert!(format!("{}", err).contains("File operation failed"));
    }
}
