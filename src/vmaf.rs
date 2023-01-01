use crate::{error::FFIError, picture::PictureError, video::FrameNum};
use errno::Errno;
use error_stack::{bail, Report, Result, ResultExt};
use libvmaf_sys::{
    vmaf_close, vmaf_init, vmaf_read_pictures, vmaf_score_at_index, vmaf_use_features_from_model,
    VmafConfiguration, VmafContext, VmafPicture,
};
/// Re-export of Vmaf Log levels from `libvmaf-sys`
pub use libvmaf_sys::{VmafLogLevel, VmafModel};
use ptrplus::AsPtr;
use std::{
    ops::{Deref, DerefMut},
    ptr,
};
use thiserror::Error;

use crate::{model::Model, picture::Picture};

/// Safe wrapper around `*mut VmafContext`
/// This is the main struct you should be concerned with
/// if you want to calculate Vmaf scores
pub struct Vmaf(*mut VmafContext);

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
    #[error("Couldn't use features from model {0}")]
    Feature(String),
    /// The two `Video`'s provided to `Vmaf::get_vmaf_scores()` had mismatching frame counts
    #[error("Mismatched frame counts: Reference: {0} Distorted: {1}")]
    FrameCount(i64, i64),
    /// Something else went wrong when computing VMAF scores
    #[error("Couldn't run VMAF")]
    Other,
}

/// This struct represents the status of VMAF calculation  
///
/// For every frame pair decoded, a `Decode` variant is emitted to the callback function provided to `Vmaf::get_vmaf_scores()`
/// After all frames are decoded, the `GetScore` variants are emitted to `Vmaf::get_vmaf_scores()`
/// 
/// ### Important!
/// Given that the two `Video` structs passed to `Vmaf::get_vmaf_scores()` have the same number of frames,
///  the number of times each variant is emitted from `Vmaf::get_vmaf_scores()` is equal to the number of frame pairs.
/// In this way, you may calculate the progress of Vmaf score calculation in this manner:
/// `(# of times a variant has been emitted)/(number of frame pairs)`.
/// One may intuit that the progress of vmaf score calculation occurs in two stages,
/// Decoding, and Retrieving the score. Ideally this should be represented in two seperate progress bars
#[derive(Debug)]
pub enum VmafStatus {
    /// update on the decoding of a video framepair.
    /// Every time a frame pair is decoded and processed, this variant is emitted
    /// to the callback function provided to `Vmaf::get_vmaf_scores()`
    Decode,
    /// this variant is an update on the retrieval of a Vmaf Score after all
    /// frames are decoded and processed.
    /// After all frames are decoded, this variant is emitted to the callback function provided to 
    ///`Vmaf::get_vmaf_scores()`
    GetScore,
}

impl Vmaf {
    pub fn new(
        log_level: VmafLogLevel,
        n_threads: u32,
        n_subsample: u32,
        cpumask: u64,
    ) -> Result<Vmaf, VmafError> {
        // Build configuration type
        let config = VmafConfiguration {
            log_level,
            n_threads,
            n_subsample,
            cpumask,
        };
        // Allocate enough memmory for VmafContext
        let ctx: *mut libvmaf_sys::VmafContext = std::ptr::null_mut();

        // Our first pointer should be non-null
        assert!(ctx.is_null());

        let mut vmaf: Vmaf = Vmaf(ctx);
        // Let vmaf do its thing with our pointer
        let err = unsafe { vmaf_init(&mut *vmaf, config) };

        // ctx should no longer be null at this point
        assert!(!(*vmaf).is_null());

        // Return an error if vmaf_init returned an error code
        FFIError::check_err(err).change_context(VmafError::Construct)?;

        Ok(vmaf)
    }

    /// Use this function to get a vector of vmaf scores.
    /// 
    /// To implement `TryInto` for Picture, you may dereference `Picture` to get a `*mut VmafPicture`.
    /// Fill the data property of the VmafPicture raw pointer with pixel data. View `impl TryFrom<VideoFrame> for Picture`
    /// 
    /// for reference. If you don't need a custom type for this, just use `Video`; given a path and a resolution it will
    /// decode and scale the video you want to load for you
    pub fn get_vmaf_scores<
        I: FrameNum + Iterator<Item = impl TryInto<Picture, Error = Report<PictureError>>>,
    >(
        mut self,
        reference: I,
        distorted: I,
        model: Model,
        callback: Option<impl Fn(VmafStatus) -> ()>,
    ) -> Result<Vec<f64>, VmafError> {
        self.use_features_from_model(&model)
            .change_context(VmafError::Feature(model.version()))?;

        let ref_frames = reference.get_num_frames();
        let dist_frames = distorted.get_num_frames();

        if ref_frames != dist_frames {
            return Err(Report::new(VmafError::FrameCount(ref_frames, dist_frames)));
        }

        let framepair = reference
            .zip(distorted)
            .map(
                |(reference, distorted)| -> Result<(Picture, Picture), PictureError> {
                    let reference_pic = TryInto::<Picture>::try_into(reference);
                    let distorted_pic = TryInto::<Picture>::try_into(distorted);

                    if let Some(callback) = &callback {
                        callback(VmafStatus::Decode)
                    }

                    match (reference_pic, distorted_pic) {
                        (Ok(reference), Ok(distorted)) => Ok((reference, distorted)),
                        (Ok(_), Err(distortederr)) => Err(distortederr),
                        (Err(referenceerr), Ok(_)) => Err(referenceerr),
                        (Err(referr), Err(_)) => Err(referr),
                    }
                },
            )
            .enumerate()
            .map(|(index, result)| match result {
                Ok((reference, distorted)) => {
                    match self.read_pictures(reference, distorted, index.try_into().unwrap()) {
                        Ok(()) => Ok(index),
                        Err(e) => Err(e).change_context(VmafError::Other),
                    }
                }
                Err(error) => Err(error.change_context(VmafError::Other)),
            })
            .collect::<Vec<Result<usize, VmafError>>>();

        self.finish_reading_pictures()
            .change_context(VmafError::ClearFrame)?;

        let mut scores: Vec<f64> = vec![];

        for pairindex in framepair {
            if let Some(callback) = &callback {
                callback(VmafStatus::GetScore)
            }
            match pairindex {
                Ok(index) => {
                    let score = self
                        .get_score_at_index(&model, index.try_into().unwrap())
                        .change_context(VmafError::GetScore(index.try_into().unwrap()))?;
                    scores.push(score);
                }
                Err(e) => bail!(e),
            }
        }

        Ok(scores)
    }

    fn use_features_from_model(&mut self, model: &Model) -> Result<(), FFIError> {
        let err = unsafe { vmaf_use_features_from_model(self.0, model.as_ptr() as *mut VmafModel) };

        FFIError::check_err(err)
    }
    fn read_pictures(
        &mut self,
        reference: Picture,
        distorted: Picture,
        index: u32,
    ) -> Result<(), FFIError> {
        let err = unsafe { vmaf_read_pictures(self.0, *reference, *distorted, index) };

        FFIError::check_err(err)
    }

    fn finish_reading_pictures(&mut self) -> Result<(), FFIError> {
        let null: *mut VmafPicture = ptr::null_mut();
        let err = unsafe { vmaf_read_pictures(self.0, null.clone(), null.clone(), 0) };

        FFIError::check_err(err)
    }

    fn get_score_at_index(&mut self, model: &Model, index: u32) -> Result<f64, FFIError> {
        let mut score: f64 = 0.0;

        let err = unsafe {
            vmaf_score_at_index(
                self.0,
                model.as_ptr() as *mut VmafModel,
                &mut score as *mut f64,
                index,
            )
        };

        FFIError::check_err(err)?;

        Ok(score)
    }
}

impl Drop for Vmaf {
    fn drop(&mut self) {
        unsafe {
            assert!(!self.0.is_null());
            let err = vmaf_close(self.0);
            FFIError::check_err(err)
                .attach_printable("Encountered error when dropping VmafContext")
                .unwrap();
        }
    }
}

impl Deref for Vmaf {
    type Target = *mut VmafContext;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Vmaf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
#[cfg(test)]
mod test {
    use crate::{model::Model, video::Video};

    use super::{Vmaf, VmafStatus};
    use libvmaf_sys::{VmafLogLevel, VmafModelConfig, VmafModelFlags};

    #[test]
    fn construct() {
        let _vmaf = Vmaf::new(VmafLogLevel::VMAF_LOG_LEVEL_DEBUG, 1, 0, 0)
            .expect("Recieved error code from constructor");
        drop(_vmaf)
    }

    #[test]
    fn get_vmaf_scores() {
        let _vmaf = Vmaf::new(
            VmafLogLevel::VMAF_LOG_LEVEL_DEBUG,
            num_cpus::get().try_into().unwrap(),
            0,
            0,
        )
        .expect("Recieved error code from constructor");

        let reference: Video = Video::new(&"./video/Big Buck Bunny 720P.m4v", 1920, 1080).unwrap();
        let distorted: Video = Video::new(&"./video/Big Buck Bunny 720P.m4v", 1920, 1080).unwrap();
        let config: VmafModelConfig = VmafModelConfig {
            name: std::ptr::null(),
            flags: VmafModelFlags::VMAF_MODEL_FLAGS_DEFAULT as u64,
        };
        let _model: Model = Model::new(config, "vmaf_v0.6.1".to_string()).unwrap();

        let x = |x: VmafStatus| match x {
            VmafStatus::Decode => println!("{x:?}"),
            VmafStatus::GetScore => println!("{x:?}"),
        };

        _vmaf
            .get_vmaf_scores(reference, distorted, _model, Some(x))
            .unwrap();
    }
}
