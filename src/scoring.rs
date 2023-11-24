use std::path::Path;

use libvmaf_sys::VmafPoolingMethod;
use thiserror::Error;

use crate::vmaf::Vmaf;

pub mod config;
pub mod error;
pub mod model;

/// This trait represents loading a Model or ModelCollection into the VMAF context and getting the score out of the context
pub trait VmafScoring: TryFrom<Box<dyn AsRef<Path>>> {
    fn load(&self, vmaf_context: &mut Vmaf) -> Result<(), VmafScoringError>;

    fn get_score_pooled(
        &self,
        vmaf_context: &Vmaf,
        pool_method: VmafPoolingMethod,
        index_low: u32,
        index_high: u32,
    ) -> Result<f64, VmafScoringError>;

    fn get_score_at_index(&self, vmaf_context: &Vmaf, index: u32) -> Result<f64, VmafScoringError>;
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
