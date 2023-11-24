 use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Couldn't load model {0}")]
    Load(String),
    #[error("Couldn't load model from path {0}")]
    Path(Box<PathBuf>),
}
