use thiserror::Error;

/// An error context for Vmaf Pictures
#[derive(Error, Debug)]
pub enum PictureError {
    /// There was a problem constructing a Picture struct
    #[error("Encountered a problem when trying to construct Picture")]
    Construct,
    /// There was a problem decoding a picture
    #[error("Encountered a problem when trying to decode video")]
    Decode,
}