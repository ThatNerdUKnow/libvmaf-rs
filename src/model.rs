use errno::Errno;
pub use libvmaf_sys::VmafModelConfig;
use libvmaf_sys::{vmaf_model_destroy, vmaf_model_load, VmafModel, VmafModelFlags};
use std::{
    ffi::{c_char, CString},
    mem,
    ops::{Deref, DerefMut},
};

pub struct Model(*mut VmafModel);

impl Model {
    pub fn new(config: VmafModelConfig, version: String) -> Result<Model, Errno> {
        let mut ptr: *mut VmafModel =
            unsafe { libc::malloc(mem::size_of::<VmafModel>()) as *mut VmafModel };
        let mut config = config.clone();

        let version_cstring: CString = CString::new(version).unwrap();
        let version_ptr: *const c_char = version_cstring.as_ptr() as *const c_char;
        let err = unsafe { vmaf_model_load(&mut ptr, &mut config, version_ptr) };

        match err {
            0 => Ok(Model(ptr)),
            _ => Err(Errno(-err)),
        }
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
            vmaf_model_destroy(self.0);
            self.0 = std::ptr::null_mut();
            assert!(self.0.is_null());
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
