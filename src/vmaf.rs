use std::mem;

use errno::Errno;
use libvmaf_sys::{vmaf_close, vmaf_init, VmafConfiguration, VmafContext};

struct Vmaf(*mut *mut VmafContext);

impl Vmaf {
    pub fn new(config: VmafConfiguration) -> Result<Vmaf, Errno> {
        let mut ctx: *mut libvmaf_sys::VmafContext =
            unsafe { libc::malloc(mem::size_of::<VmafContext>()) as *mut VmafContext };

        assert!(!ctx.is_null());
        let vmaf: Vmaf = Vmaf(&mut ctx);
        assert!(!vmaf.0.is_null());
        let err = unsafe { vmaf_init(vmaf.0, config) };

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
        let _vmaf = Vmaf::new(VmafConfiguration {
            log_level: VmafLogLevel_VMAF_LOG_LEVEL_NONE,
            n_threads: 1,
            n_subsample: 0,
            cpumask: 0,
        })
        .expect("Recieved error code from constructor");
    }
}
