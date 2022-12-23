use std::fmt::Display;

use errno::Errno;
use error_stack::{Result, Report};
use thiserror::Error;

#[derive(Debug, Error)]
pub struct VMAFError {
    errno: Errno,
}

impl VMAFError {
    fn new(err: i32) -> VMAFError {
        VMAFError { errno: Errno(-err) }
    }

    pub fn check_err(err:i32)->Result<(),VMAFError>{
        match err{
            0 => Ok(()),
            _ => Err(Report::new(VMAFError::new(err)))
        }
    }
}

impl Display for VMAFError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.errno)
    }
}
