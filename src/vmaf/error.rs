use errno::Errno;
use thiserror::Error;

use crate::video::resolution::Resolution;

/// An enum of every possible error calculating a Vmaf Score
#[derive(Error, Debug)]
pub enum VmafError {
    /// There was a problem reading frame data
    #[error("Couldn't read VmafPicture {0:?}")]
    ReadFrame(Errno),
    /// There was a problem clearing the buffers of feature extractors
    #[error("Couldn't clear feature extractor buffers")]
    ClearFrame,
    /// There was a problem getting a score for a given frame
    #[error("Couldn't get score for frame #{0}")]
    GetScore(u32),
    /// There was a problem constructing a Vmaf Context
    #[error("Couldn't construct a vmafcontext")]
    Construct,
    /// There was a problem using the feature extractors required by a model
    #[error("Couldn't use features from model {0:?}")]
    Feature(Option<String>),
    /// The two `Video`'s provided to `Vmaf::get_vmaf_scores()` had mismatching frame counts
    #[error("Mismatched frame counts: Reference: {0} Distorted: {1}")]
    FrameCount(usize, usize),
    /// The two `Video`'s provided had mismatching resolutions
    #[error("Mismatched resolutions: Reference: {0} Distorted: {1}")]
    Resolution(Resolution, Resolution),
    /// Something else went wrong when computing VMAF scores
    #[error("Couldn't run VMAF")]
    Other,
}
