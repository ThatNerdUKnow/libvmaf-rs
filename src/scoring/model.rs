use libvmaf_sys::{
    vmaf_model_destroy, vmaf_model_load, vmaf_model_load_from_path, vmaf_score_at_index,
    vmaf_score_pooled, vmaf_use_features_from_model, VmafModel, VmafPoolingMethod,
};
use ptrplus::{AsPtr, IntoRaw};
use std::{
    ffi::{c_char, CString},
    fmt::Display,
    path::Path,
};

use crate::{error::FFIError, vmaf::Vmaf2};

use super::{config::ModelConfig, error::ModelError, VmafScoring, VmafScoringError};

#[derive(Debug)]
pub struct Model(*mut VmafModel, String);

impl Model {
    pub fn new(config: ModelConfig, version: String) -> Result<Model, ModelError> {
        let mut ptr: *mut VmafModel = std::ptr::null_mut();

        let mut config = config.as_ref().to_owned();

        let version_cstring: CString = CString::new(version.clone()).unwrap();
        let version_ptr: *const c_char = version_cstring.as_ptr() as *const c_char;
        let err = unsafe { vmaf_model_load(&mut ptr, &mut config, version_ptr) };

        FFIError::check_err(err).map_err(|e| ModelError::Load(version.clone()));

        Ok(Model(ptr, version))
    }

    pub fn version(&self) -> String {
        self.1.clone()
    }

    pub fn load_model(
        config: ModelConfig,
        path: impl AsRef<Path>,
    ) -> core::result::Result<Model, ModelError> {
        let mut ptr: *mut VmafModel = std::ptr::null_mut();

        let mut config = config.as_ref().to_owned();

        let path: &Path = path.as_ref();
        let path_ptr = CString::new(path.as_os_str().to_string_lossy().as_bytes())
            .map_err(|e| ModelError::Path(Box::new(path.to_path_buf())))?;

        let err = unsafe { vmaf_model_load_from_path(&mut ptr, &mut config, path_ptr.as_ptr()) };

        FFIError::check_err(err).map_err(|e| ModelError::Load(format!("{e}")))?;

        Ok(Model(ptr, path.to_string_lossy().into()))
    }
}

impl Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.1)
    }
}

impl IntoRaw for Model {
    type Raw = VmafModel;

    fn into_raw(self) -> *mut Self::Raw {
        self.0
    }
}

impl AsPtr for Model {
    type Raw = VmafModel;

    fn as_ptr(&self) -> *const Self::Raw {
        self.0
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        unsafe {
            assert!(!self.0.is_null());
            vmaf_model_destroy(self.0);
            self.0 = std::ptr::null_mut();
        }
    }
}

/*
impl TryFrom<Box<dyn AsRef<Path>>> for Model {
    type Error = ModelError;

    fn try_from(path: Box<dyn AsRef<Path>>) -> core::result::Result<Self, Self::Error> {
        Model::load_model(Default::default(), *path)
    }
}*/

impl VmafScoring for Model {
    fn load(&self, vmaf_context: &mut Vmaf2) -> Result<(), VmafScoringError> {
        let error = unsafe { vmaf_use_features_from_model(**vmaf_context, self.0) };

        FFIError::check_err(error).map_err(|e| VmafScoringError::Load(self.1.clone()))?;
        Ok(())
    }

    fn get_score_pooled(
        &self,
        vmaf_context: &Vmaf2,
        pool_method: VmafPoolingMethod,
        index_low: u32,
        index_high: u32,
    ) -> Result<f64, VmafScoringError> {
        let mut score: f64 = f64::default();

        let error = unsafe {
            vmaf_score_pooled(
                **vmaf_context,
                self.0,
                pool_method,
                &mut score,
                index_low,
                index_high,
            )
        };

        FFIError::check_err(error).map_err(|e| VmafScoringError::GetScore(self.1.clone()))?;

        Ok(score)
    }

    fn get_score_at_index(
        &self,
        vmaf_context: &Vmaf2,
        index: u32,
    ) -> Result<f64, VmafScoringError> {
        let mut score: f64 = f64::default();

        let error = unsafe { vmaf_score_at_index(**vmaf_context, self.0, &mut score, index) };

        FFIError::check_err(error)
            .map_err(|e| VmafScoringError::GetScoreIndex(self.1.clone(), index))?;

        Ok(score)
    }
}

#[cfg(test)]
mod test {

    use super::Model;
    use crate::scoring::config::ModelConfig;

    #[test]
    fn construct() {
        let config = ModelConfig::default();
        let _model: Model = Model::new(config, "vmaf_v0.6.1".to_string()).unwrap();
    }
}
