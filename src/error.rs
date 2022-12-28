use std::fmt::Display;

use errno::Errno;
use error_stack::{Report, Result};
use thiserror::Error;

#[derive(Debug, Error)]
pub struct FFIError {
    errno: Errno,
}

impl FFIError {
    fn new(err: i32) -> FFIError {
        FFIError { errno: Errno(-err) }
    }

    pub fn check_err(err: i32) -> Result<(), FFIError> {
        match err {
            0 => Ok(()),
            _ => Err(Report::new(FFIError::new(err))),
        }
    }
}

impl Display for FFIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OS Code {}, {}", self.errno.0, self.errno)
    }
}
