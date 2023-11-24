use std::fmt::Display;

use errno::Errno;
use thiserror::Error;

/// This is a wrapper type around errors we get back from `libvmaf`
///
/// Libvmaf uses the c calling convention of using an i32 as the return type for most functions.
/// If an operation fails, libvmaf functions will return negative numbers. These negative numbers correspond to OS error codes, or `Errnos`.
#[derive(Debug, Error)]
pub struct FFIError {
    errno: Errno,
}

impl FFIError {
    fn new(err: i32) -> FFIError {
        FFIError { errno: Errno(-err) }
    }

    /// This function will determine if a given i32 should be considered an error.
    /// If we do get an erroneous i32, it should be interpreted as an OS error code.
    /// since libvmaf returns the negative of these OS codes,
    /// automatically flip the sign so that we can correctly determine the
    /// OS code to display in case of error
    pub fn check_err(err: i32) -> Result<(), FFIError> {
        match err {
            0 => Ok(()),
            _ => Err(FFIError::new(err)),
        }
    }
}

impl Display for FFIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OS Code {}, {}", self.errno.0, self.errno)
    }
}
