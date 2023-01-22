use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Couldn't load model {0}")]
    Load(String),
}