use std::{
    mem,
    ops::{Deref, DerefMut},
};

use errno::Errno;
pub use libvmaf_sys::VmafConfiguration;
use libvmaf_sys::{vmaf_close, vmaf_init, VmafContext};

struct Vmaf(*mut VmafContext);

impl Vmaf {
    pub fn new(config: VmafConfiguration) -> Result<Vmaf, Errno> {
        // Allocate enough memmory for VmafContext
        let mut ctx: *mut libvmaf_sys::VmafContext = std::ptr::null_mut();

        // Our first pointer should be non-null
        assert!(ctx.is_null());

        // Let vmaf do its thing with our pointer
        let err = unsafe { vmaf_init(&mut ctx, config) };

        // ctx should no longer be null at this point
        assert!(!ctx.is_null());

        // Return an error if vmaf_init returned an error code
        match err {
            0 => Ok(Vmaf(ctx)),
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
            assert!(self.0.is_null());
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
    use libvmaf_sys::{VmafConfiguration, VmafLogLevel_VMAF_LOG_LEVEL_NONE};

    use super::Vmaf;

    #[test]
    fn construct() {
        // Generate some dummy confiuguration since it's required by the constructor
        let config: VmafConfiguration = VmafConfiguration {
            log_level: VmafLogLevel_VMAF_LOG_LEVEL_NONE,
            n_threads: 1,
            n_subsample: 0,
            cpumask: 0,
        };
        let _vmaf = Vmaf::new(config).expect("Recieved error code from constructor");

        drop(_vmaf)
    }
}
