use libvmaf_sys::{VmafModel, vmaf_model_load, vmaf_model_load_from_path, vmaf_model_destroy};
use ptrplus::{AsPtr, IntoRaw};
use std::{
    ffi::{c_char, CString},
    fmt::Display,
    ops::{Deref, DerefMut},
    path::Path,
};

use crate::error::FFIError;

use self::{config::ModelConfig, error::ModelError};

pub mod config;
pub mod error;

#[derive(Debug)]
pub struct Model(*mut VmafModel, String);

impl Model {
    pub fn new(config: ModelConfig, version: String) -> Result<Model, ModelError> {
        let mut ptr: *mut VmafModel = std::ptr::null_mut();

        let mut config = config.as_ref().to_owned();

        let version_cstring: CString = CString::new(version.clone()).unwrap();
        let version_ptr: *const c_char = version_cstring.as_ptr() as *const c_char;
        let err = unsafe { vmaf_model_load(&mut ptr, &mut config, version_ptr) };

        FFIError::check_err(err).map_err(|_e| ModelError::Load(version.clone()))?;

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
            .map_err(|_e| ModelError::Path(Box::new(path.to_path_buf())))?;

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

impl Deref for Model {
    type Target = *mut VmafModel;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Model {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod test {

    use super::{config::ModelConfig, Model};

    #[test]
    fn construct() {
        let config = ModelConfig::default();
        let _model: Model = Model::new(config, "vmaf_v0.6.1".to_string()).unwrap();
    }
}
