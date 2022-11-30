use std::ops::{Deref, DerefMut};

use errno::Errno;
pub use libvmaf_sys::VmafConfiguration;
use libvmaf_sys::{vmaf_close, vmaf_init, VmafContext};
pub use libvmaf_sys::VmafLogLevel;
struct Vmaf(*mut VmafContext);

impl Vmaf {
    pub fn new(
        log_level: VmafLogLevel,
        n_threads: u32,
        n_subsample: u32,
        cpumask: u64,
    ) -> Result<Vmaf, Errno> {
        // Build configuration type
        let config = VmafConfiguration {
            log_level,
            n_threads,
            n_subsample,
            cpumask,
        };
        // Allocate enough memmory for VmafContext
        let ctx: *mut libvmaf_sys::VmafContext = std::ptr::null_mut();

        // Our first pointer should be non-null
        assert!(ctx.is_null());

        let mut vmaf: Vmaf = Vmaf(ctx);
        // Let vmaf do its thing with our pointer
        let err = unsafe { vmaf_init(&mut *vmaf, config) };

        // ctx should no longer be null at this point
        assert!(!(*vmaf).is_null());

        // Return an error if vmaf_init returned an error code
        match err {
            0 => Ok(vmaf),
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
    use super::Vmaf;
    use libvmaf_sys::VmafLogLevel;

    #[test]
    fn construct() {
        let _vmaf = Vmaf::new(VmafLogLevel::VMAF_LOG_LEVEL_DEBUG, 1, 0, 0)
            .expect("Recieved error code from constructor");

        drop(_vmaf)
    }
}
