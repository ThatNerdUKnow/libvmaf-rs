use anyhow::{bail, Context, Error};
use errno::Errno;
pub use libvmaf_sys::VmafLogLevel;
use libvmaf_sys::{
    vmaf_close, vmaf_init, vmaf_read_pictures, vmaf_score_at_index, vmaf_use_features_from_model,
    VmafConfiguration, VmafContext, VmafPicture,
};
use std::{
    ops::{Deref, DerefMut},
    ptr,
};

use crate::{model::Model, picture::Picture};
pub struct Vmaf(*mut VmafContext);

enum VmafError {
    ReadFrame,
    ClearFrame,
}

impl Vmaf {
    pub fn new(
        log_level: VmafLogLevel,
        n_threads: u32,
        n_subsample: u32,
        cpumask: u64,
    ) -> Result<Vmaf, Errno> {
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
        match err {
            0 => Ok(vmaf),
            _ => Err(Errno(-err)),
        }
    }

    /// Use this function to get a vector of vmaf scores.
    /// To implement `TryInto` for Picture, you may dereference `Picture` to get a `*mut VmafPicture`.
    /// Fill the data property of the VmafPicture raw pointer with pixel data. View `impl TryFrom<VideoFrame> for Picture`
    /// for reference. If you don't need a custom type for this, just use `Video`; given a path and a resolution it will
    /// decode and scale the video you want to load for you
    pub fn get_vmaf_scores(
        mut self,
        reference: impl Iterator<Item = impl TryInto<Picture, Error = anyhow::Error>>,
        distorted: impl Iterator<Item = impl TryInto<Picture, Error = anyhow::Error>>,
        model: Model,
    ) -> Result<Vec<f64>, anyhow::Error> {
        self.use_features_from_model(&model)?;

        let framepair = reference
            .zip(distorted)
            .map(|(reference, distorted)| {
                let reference_pic = TryInto::<Picture>::try_into(reference);
                let distorted_pic = TryInto::<Picture>::try_into(distorted);

                match (reference_pic, distorted_pic) {
                    (Ok(reference), Ok(distorted)) => Ok((reference, distorted)),
                    (Ok(_), Err(distortederr)) => Err(distortederr),
                    (Err(referenceerr), Ok(_)) => Err(referenceerr),
                    (Err(referr), Err(disterr)) => Err(referr).context(format!("{disterr}")),
                }
            })
            .enumerate()
            .map(|(index, result)| -> anyhow::Result<usize> {
                match result {
                    Ok((reference, distorted)) => {
                        match self.read_pictures(reference, distorted, index.try_into().unwrap()) {
                            Ok(()) => Ok(index),
                            Err(e) => Err(Error::new(e)),
                        }
                    }
                    Err(error) => Err(error),
                }
            })
            .collect::<Vec<anyhow::Result<usize>>>();

        self.finish_reading_pictures()?;

        let mut scores: Vec<f64> = vec![];

        for pairindex in framepair {
            match pairindex {
                Ok(index) => {
                    let score = self.get_score_at_index(&model, index.try_into().unwrap())?;
                    scores.push(score);
                }
                Err(e) => bail!(e),
            }
        }

        Ok(scores)
    }

    fn use_features_from_model(&mut self, model: &Model) -> Result<(), Errno> {
        let err = unsafe { vmaf_use_features_from_model(self.0, **model) };

        match err {
            0 => Ok(()),
            _ => Err(Errno(-err)),
        }
    }
    fn read_pictures(
        &mut self,
        reference: Picture,
        distorted: Picture,
        index: u32,
    ) -> Result<(), Errno> {
        let err = unsafe { vmaf_read_pictures(self.0, *reference, *distorted, index) };

        match err {
            0 => Ok(()),
            _ => Err(Errno(-err)),
        }
    }

    fn finish_reading_pictures(&mut self) -> Result<(), Errno> {
        let null: *mut VmafPicture = ptr::null_mut();
        let err = unsafe { vmaf_read_pictures(self.0, null.clone(), null.clone(), 0) };

        match err {
            0 => Ok(()),
            _ => Err(Errno(-err)),
        }
    }

    fn get_score_at_index(&mut self, model: &Model, index: u32) -> Result<f64, Errno> {
        let mut score: *mut f64 = ptr::null_mut();
        let err = unsafe { vmaf_score_at_index(self.0, **model, score, index) };

        match err {
            0 => unsafe { Ok(*score) },
            _ => Err(Errno(-err)),
        }
    }
}

impl Drop for Vmaf {
    fn drop(&mut self) {
        unsafe {
            assert!(!self.0.is_null());
            let err = vmaf_close(self.0);
            self.0 = std::ptr::null_mut();
            if err < 0 {
                panic!("Got Error: {:?} when dropping Vmaf Context", Errno(-err));
            };
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
    use crate::{video::Video, model::Model};

    use super::Vmaf;
    use libvmaf_sys::{VmafLogLevel, VmafModelConfig, VmafModelFlags};

    #[test]
    fn construct() {
        let _vmaf = Vmaf::new(VmafLogLevel::VMAF_LOG_LEVEL_DEBUG, 1, 0, 0)
            .expect("Recieved error code from constructor");
        drop(_vmaf)
    }

    #[test]
    fn get_vmaf_scores() {
        let _vmaf = Vmaf::new(VmafLogLevel::VMAF_LOG_LEVEL_DEBUG, 1, 0, 0)
            .expect("Recieved error code from constructor");

        let reference: Video = Video::new(&"./video/Big Buck Bunny 720P.m4v", 1920, 1080).unwrap();
        let distorted: Video = Video::new(&"./video/Big Buck Bunny 720P.m4v", 1920, 1080).unwrap();
        let config: VmafModelConfig = VmafModelConfig {
            name: std::ptr::null(),
            flags: VmafModelFlags::VMAF_MODEL_FLAGS_DEFAULT as u64,
        };
        let _model: Model = Model::new(config, "vmaf_v0.6.1".to_string()).unwrap();

        _vmaf.get_vmaf_scores(reference, distorted, _model).unwrap();
    }
}
