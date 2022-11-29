use std::ops::{Deref, DerefMut};

use errno::Errno;
pub use libvmaf_sys::VmafConfiguration;
use libvmaf_sys::{
    vmaf_close, vmaf_init, VmafContext, VmafLogLevel, VmafLogLevel_VMAF_LOG_LEVEL_DEBUG,
    VmafLogLevel_VMAF_LOG_LEVEL_ERROR, VmafLogLevel_VMAF_LOG_LEVEL_INFO,
    VmafLogLevel_VMAF_LOG_LEVEL_NONE, VmafLogLevel_VMAF_LOG_LEVEL_WARNING,
};

struct Vmaf(*mut VmafContext);

impl Vmaf {
    pub fn new(
        loglevel: LogLevel,
        n_threads: u32,
        n_subsample: u32,
        cpumask: u64,
    ) -> Result<Vmaf, Errno> {
        // Build configuration type
        let config = VmafConfiguration {
            log_level: loglevel.into(),
            n_threads: n_threads,
            n_subsample: n_subsample,
            cpumask: cpumask,
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

pub enum LogLevel {
    None,
    Debug,
    Info,
    Warning,
    Error,
}

impl Into<VmafLogLevel> for LogLevel {
    fn into(self) -> VmafLogLevel {
        match self {
            LogLevel::None => VmafLogLevel_VMAF_LOG_LEVEL_NONE,
            LogLevel::Debug => VmafLogLevel_VMAF_LOG_LEVEL_DEBUG,
            LogLevel::Info => VmafLogLevel_VMAF_LOG_LEVEL_INFO,
            LogLevel::Warning => VmafLogLevel_VMAF_LOG_LEVEL_WARNING,
            LogLevel::Error => VmafLogLevel_VMAF_LOG_LEVEL_ERROR,
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
    use super::{LogLevel, Vmaf};

    #[test]
    fn construct() {
        let _vmaf =
            Vmaf::new(LogLevel::Debug, 1, 0, 0).expect("Recieved error code from constructor");

        drop(_vmaf)
    }
}
