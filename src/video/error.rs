use crate::picture::resolution::error::ResolutionError;
use ffmpeg_next::format::Pixel;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VideoError {
   // #[error("Encountered an error when creating video context {0}")]
   // Construct(PathBuf),
    #[error("Cannot create a scaler given resolution {0:?}")]
    Resolution(#[from] ResolutionError),
    #[error("Cannot create a scaler given Pixel format {0:?}")]
    Format(Pixel),
    #[error("FFMPEG Error")]
    FFMPEG(#[from] ffmpeg_next::util::error::Error),
}
