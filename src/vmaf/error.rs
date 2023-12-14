use thiserror::Error;

use crate::{error::FFIError, picture::resolution::Resolution, scoring::VmafScoringError};

/// An enum of every possible error calculating a Vmaf Score
#[derive(Error, Debug)]
pub enum VmafError {
    #[error("There was a problem calling libvmaf")]
    FFI(#[from] FFIError),
    /// There was a problem clearing the buffers of feature extractors
    #[error("Couldn't clear feature extractor buffers")]
    ClearFrame,
    /// There was a problem getting a score for a given frame
    #[error("Couldn't get score for frame #{0}")]
    GetScore(u32),
    /// There was a problem constructing a Vmaf Context
    #[error("Couldn't construct a vmafcontext")]
    Construct(FFIError),
    /// The two `Video`'s provided to `Vmaf::get_vmaf_scores()` had mismatching frame counts
    #[error("Mismatched frame counts: Reference: {0} Distorted: {1}")]
    MismatchedFrameCount(usize, usize),
    /// The two `Video`'s provided had mismatching resolutions
    #[error("Mismatched resolutions: Reference: {0} Distorted: {1}")]
    MismatchedResolution(Resolution, Resolution),
    #[error("There was a problem with the VMAF Model")]
    Model(#[from] VmafScoringError),
}
