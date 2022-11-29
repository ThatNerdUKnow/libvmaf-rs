use std::{
    ffi::c_char,
    mem,
    ops::{Deref, DerefMut},
};

use errno::Errno;
pub use libvmaf_sys::VmafModelConfig;
use libvmaf_sys::{vmaf_model_destroy, vmaf_model_load, VmafModel, VmafModelFlags_VMAF_MODEL_FLAGS_DEFAULT, VmafModelFlags_VMAF_MODEL_FLAG_DISABLE_CLIP, VmafModelFlags_VMAF_MODEL_FLAG_ENABLE_TRANSFORM, VmafModelFlags_VMAF_MODEL_FLAG_DISABLE_TRANSFORM};

pub struct Model(*mut VmafModel);

impl Model {
    pub fn new(config: VmafModelConfig, version: c_char) -> Result<Model, Errno> {
        let mut ptr: *mut VmafModel =
            unsafe { libc::malloc(mem::size_of::<VmafModel>()) as *mut VmafModel };
        let mut config = config.clone();

        let err = unsafe { vmaf_model_load(&mut ptr, &mut config, &version) };

        match err {
            0 => Ok(Model(ptr)),
            _ => Err(Errno(-err)),
        }
    }
}

pub enum ModelFlags{
    Default = VmafModelFlags_VMAF_MODEL_FLAGS_DEFAULT as isize,
    DisableClip = VmafModelFlags_VMAF_MODEL_FLAG_DISABLE_CLIP as isize,
    EnableTransform = VmafModelFlags_VMAF_MODEL_FLAG_ENABLE_TRANSFORM as isize,
    DisableTransform = VmafModelFlags_VMAF_MODEL_FLAG_DISABLE_TRANSFORM as isize
}

pub struct ModelConfig(VmafModelConfig);

impl ModelConfig{
    pub fn new(name: *const c_char,flags: ModelFlags){
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
            vmaf_model_destroy(self.0);
            assert!(self.0.is_null())
        }
    }
}

#[cfg(test)]
mod test{
    use libvmaf_sys::VmafModelConfig;

    use super::Model;

    #[test]
    fn construct(){
        
        todo!()
    }
}