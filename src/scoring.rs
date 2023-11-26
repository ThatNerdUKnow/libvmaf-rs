use libvmaf_sys::VmafPoolingMethod;
use ptrplus::AsPtr;
use thiserror::Error;

use crate::vmaf::{GetScores, ReadFrames, Vmaf2};

pub mod config;
pub mod error;
pub mod model;

/// This trait represents loading a Model or ModelCollection into the VMAF context and getting the score out of the context
pub trait VmafScoring {
    fn load(&self, vmaf_context: &mut Vmaf2<ReadFrames>) -> Result<(), VmafScoringError>;

    fn get_score_pooled(
        &self,
        vmaf_context: &Vmaf2<GetScores>,
        pool_method: VmafPoolingMethod,
        index_low: u32,
        index_high: u32,
    ) -> Result<f64, VmafScoringError>;

    fn get_score_at_index(
        &self,
        vmaf_context: &Vmaf2<GetScores>,
        index: u32,
    ) -> Result<f64, VmafScoringError>;
}

#[derive(Error, Debug)]
pub enum VmafScoringError {
    #[error("Could not load model {0}")]
    Load(String),
    #[error("Could not get VMAF score for model {0}")]
    GetScore(String),
    #[error("Could not get VMAF score for model {0} at index {1}")]
    GetScoreIndex(String, u32),
}
