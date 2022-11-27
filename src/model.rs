use std::{ffi::c_char, mem};

use errno::Errno;
pub use libvmaf_sys::VmafModelConfig;
use libvmaf_sys::{vmaf_model_destroy, vmaf_model_load, VmafModel};

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

impl Drop for Model {
    fn drop(&mut self) {
        unsafe {
            vmaf_model_destroy(self.0);
            assert!(self.0.is_null())
        }
    }
}
