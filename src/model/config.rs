use std::ffi::{NulError, CString};

use libvmaf_sys::{VmafModelConfig, VmafModelFlags};

pub struct ModelConfig(VmafModelConfig);

impl ModelConfig {
    pub fn new<'a, N: AsRef<&'a str>>(
        name: N,
        flags: VmafModelFlags,
    ) -> Result<ModelConfig, NulError> {
        let name = CString::new(*name.as_ref())?.into_raw();
        Ok(ModelConfig(VmafModelConfig {
            name,
            flags: flags as u64,
        }))
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        ModelConfig(VmafModelConfig {
            name: std::ptr::null(),
            flags: VmafModelFlags::VMAF_MODEL_FLAGS_DEFAULT as u64,
        })
    }
}