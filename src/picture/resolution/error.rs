use std::num::TryFromIntError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResolutionError {
    #[error("Couldn't convert u32 to usize")]
    IntConversion(#[from] TryFromIntError),
    #[error("Invalid resolution: {width}x{height}")]
    InvalidResolution { width: u32, height: u32 },
}
