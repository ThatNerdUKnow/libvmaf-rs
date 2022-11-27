use std::mem;

use errno::Errno;
use libvmaf_sys::{vmaf_close, vmaf_init, VmafConfiguration, VmafContext};

struct Vmaf(*mut *mut VmafContext);

impl Vmaf {
    pub fn new(config: VmafConfiguration) -> Result<Vmaf, Errno> {
        // Allocate enough memmory for VmafContext
        let mut ctx: *mut libvmaf_sys::VmafContext =
            unsafe { libc::malloc(mem::size_of::<VmafContext>()) as *mut VmafContext };

        // Our first pointer should be non-null
        assert!(!ctx.is_null());
        let vmaf: Vmaf = Vmaf(&mut ctx);
        // After constructing Vmaf newtype, internal double pointer should also be non-null
        assert!(!vmaf.0.is_null());
        unsafe {
            assert!(!(*vmaf.0).is_null());
        }

        // Let vmaf do its thing with our pointer
        let err = unsafe { vmaf_init(vmaf.0, config) };

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
            assert!(!(*self.0).is_null());
            let err = vmaf_close(*self.0);

            if err < 0 {
                panic!("Got Error: {:?} when dropping Vmaf Context", Errno(-err));
            };
        }
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
    }
}
