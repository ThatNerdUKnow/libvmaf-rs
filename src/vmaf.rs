use std::{ops::Deref, ptr};

use libvmaf_sys::{
    vmaf_close, vmaf_init, vmaf_read_pictures, VmafConfiguration, VmafContext, VmafLogLevel,
    VmafPicture, VmafPoolingMethod, vmaf_use_features_from_model,
};
use ptrplus::AsPtr;

use crate::{
    error::FFIError,
    picture::{resolution::GetResolution, Picture, ValidRef},
    scoring::model::Model,
    vmaf::error::VmafError,
};

pub mod error;

pub struct Vmaf2 {
    context: *mut VmafContext
}

impl Vmaf2 {
    pub fn new(
        log_level: VmafLogLevel,
        n_threads: u32,
        n_subsample: u32,
        cpumask: u64,
    ) -> Result<Vmaf2, VmafError> {
        let config = VmafConfiguration {
            log_level,
            n_threads,
            n_subsample,
            cpumask,
        };

        let ctx: *mut libvmaf_sys::VmafContext = std::ptr::null_mut();

        debug_assert!(ctx.is_null());

        let mut vmaf: Vmaf2 = Vmaf2 {
            context: ctx
        };

        let err = unsafe { vmaf_init(&mut vmaf.context, config) };

        FFIError::check_err(err).map_err(|e| VmafError::Construct)?;

        Ok(vmaf)
    }

    pub fn use_features_from_model(&mut self,model:&mut Model) ->Result<(),VmafError>{
        let error = unsafe {
            vmaf_use_features_from_model(self.context, **model)
        };
        FFIError::check_err(error)?;
        Ok(())
    }

    pub fn read_framepair(
        &mut self,
        reference: Picture<ValidRef>,
        distorted: Picture<ValidRef>,
        index: u32,
    ) -> Result<(), VmafError> {
        let ref_resolution = reference.get_resolution();
        let dist_resolution = distorted.get_resolution();

        if ref_resolution != dist_resolution {
            return Err(VmafError::MismatchedResolution(
                ref_resolution.clone(),
                dist_resolution.clone(),
            ));
        }

        let err = unsafe {
            vmaf_read_pictures(
                **self,
                reference.as_ptr() as *mut VmafPicture,
                distorted.as_ptr() as *mut VmafPicture,
                index,
            )
        };

        reference.consume();
        distorted.consume();
        FFIError::check_err(err)?;
        Ok(())
    }

    pub fn flush_framebuffers(&mut self) -> Result<(), VmafError> {
        let null: *mut VmafPicture = ptr::null_mut();
        let err = unsafe { vmaf_read_pictures(**self, null.clone(), null.clone(), 0) };
        FFIError::check_err(err)?;

        Ok(())
    }

    pub fn get_score_at_index(&self,model:&mut Model, index: u32) -> Result<f64, VmafError> {
        let mut score:f64 = f64::default();

        let error = unsafe {
            libvmaf_sys::vmaf_score_at_index(self.context, **model, &mut score, index)
        };

        FFIError::check_err(error)?;

        Ok(score)
    }

    pub fn get_score_pooled(
        &self,
        model: &mut Model,
        pool_method: VmafPoolingMethod,
        index_low: u32,
        index_high: u32,
    ) -> Result<f64, VmafError> {
        let mut score: f64 = f64::default();

        let error = unsafe {
            libvmaf_sys::vmaf_score_pooled(self.context, **model, pool_method, &mut score, index_low, index_high)
        };

        FFIError::check_err(error);
        
        Ok(score)
    }
}

impl Deref for Vmaf2 {
    type Target = *mut VmafContext;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl Drop for Vmaf2 {
    fn drop(&mut self) {
        unsafe {
            assert!(!self.context.is_null());
            let err = vmaf_close(**self);
            FFIError::check_err(err).unwrap();
        }
    }
}
