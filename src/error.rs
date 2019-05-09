//! Result and Error types.

use std::ffi::CStr;

/// Error type for this crate.
#[derive(Debug, Clone)]
pub enum Error {
    /// A generic error, with a description string.
    Generic(String),
}

impl Error {
    /// Construct an `Error` from a malloc-allocated C string, then free the C string.
    pub(crate) fn take(x: *const libc::c_char) -> Self {
        unsafe {
            let msg = CStr::from_ptr(x).to_string_lossy().into_owned();
            libc::free(x as *mut _);
            Error::Generic(msg)
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        use self::Error::*;
        match *self {
            Generic(ref x) => x,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use self::Error::*;
        match *self {
            Generic(ref x) => f.pad(x),
        }
    }
}

/// Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;
