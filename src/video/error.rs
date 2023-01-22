use std::path::PathBuf;
use thiserror::Error;



#[derive(Error, Debug)]
pub enum VideoError {
    #[error("Encountered an error when creating video context {0}")]
    Construct(PathBuf),
}