use ffmpeg_next::format::Pixel;
use std::path::PathBuf;
use thiserror::Error;

use super::resolution::Resolution;

#[derive(Error, Debug)]
pub enum VideoError {
    #[error("Encountered an error when creating video context {0}")]
    Construct(PathBuf),
    #[error("Cannot create a scaler given resolution {0}")]
    Resolution(Resolution),
    #[error("Cannot create a scaler given Pixel format {0:?}")]
    Format(Pixel),
}
