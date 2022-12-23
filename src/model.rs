use error_stack::{Result, ResultExt};
pub use libvmaf_sys::VmafModelConfig;
use libvmaf_sys::{vmaf_model_destroy, vmaf_model_load, VmafModel, VmafModelFlags};
use std::{
    ffi::{c_char, CString},
    fmt::Display,
    ops::{Deref, DerefMut},
};
use thiserror::Error;

use crate::error::VMAFError;

#[derive(Debug)]
pub struct Model(*mut VmafModel, String);

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Couldn't load model {0}")]
    Load(String),
}

impl Model {
    pub fn new(config: VmafModelConfig, version: String) -> Result<Model, ModelError> {
        let mut ptr: *mut VmafModel = std::ptr::null_mut();
        let mut config = config.clone();

        let version_cstring: CString = CString::new(version.clone()).unwrap();
        let version_ptr: *const c_char = version_cstring.as_ptr() as *const c_char;
        let err = unsafe { vmaf_model_load(&mut ptr, &mut config, version_ptr) };

        VMAFError::check_err(err).change_context(ModelError::Load(version.clone()))?;

        Ok(Model(ptr, version))
    }

    pub fn version(&self) -> String {
        self.1.clone()
    }
}

impl Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.1)
    }
}

pub struct ModelConfig(VmafModelConfig);

impl ModelConfig {
    pub fn new(name: *const c_char, flags: VmafModelFlags) {
        todo!()
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

impl Drop for Model {
    fn drop(&mut self) {
        unsafe {
            assert!(!self.0.is_null());
            vmaf_model_destroy(self.0);
            self.0 = std::ptr::null_mut();
        }
    }
}

#[cfg(test)]
mod test {
    use libvmaf_sys::{VmafModelConfig, VmafModelFlags};

    use super::Model;

    #[test]
    fn construct() {
        let config: VmafModelConfig = VmafModelConfig {
            name: std::ptr::null(),
            flags: VmafModelFlags::VMAF_MODEL_FLAGS_DEFAULT as u64,
        };
        let _model: Model = Model::new(config, "vmaf_v0.6.1".to_string()).unwrap();
    }
}
