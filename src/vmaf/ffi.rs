use std::ptr;

use crate::{error::FFIError, picture::Picture};
use libvmaf_sys::{vmaf_read_pictures, VmafPicture};
use ptrplus::AsPtr;

use super::Vmaf;

impl Vmaf {
    /*
    pub(super) fn use_features_from_model(&mut self, model: &Model) -> Result<(), FFIError> {
        let err = unsafe { vmaf_use_features_from_model(self.0, model.as_ptr() as *mut VmafModel) };

        FFIError::check_err(err)
    }
    */

    pub(super) fn read_pictures(
        &mut self,
        reference: Picture,
        distorted: Picture,
        index: u32,
    ) -> Result<(), FFIError> {
        let err = unsafe {
            vmaf_read_pictures(
                self.0,
                reference.as_ptr() as *mut VmafPicture,
                distorted.as_ptr() as *mut VmafPicture,
                index,
            )
        };

        reference.consume();
        distorted.consume();

        //mem::forget(reference);
        //mem::forget(distorted);

        FFIError::check_err(err)
    }

    pub(super) fn finish_reading_pictures(&mut self) -> Result<(), FFIError> {
        let null: *mut VmafPicture = ptr::null_mut();
        let err = unsafe { vmaf_read_pictures(self.0, null.clone(), null.clone(), 0) };

        FFIError::check_err(err)
    }

    /*
    pub(super) fn get_score_at_index(&mut self, model: &Model, index: u32) -> Result<f64, FFIError> {
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
    }*/
}
