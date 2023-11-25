use std::{
    default,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr,
    rc::Rc,
};

use libvmaf_sys::{
    vmaf_close, vmaf_init, vmaf_read_pictures, VmafConfiguration, VmafContext, VmafLogLevel,
    VmafPicture,
};
use ptrplus::AsPtr;

use crate::{
    error::FFIError,
    picture::{resolution::GetResolution, Picture, ValidRef},
    scoring::{model::Model, VmafScoring},
    vmaf::error::VmafError,
};

pub mod error;

pub struct Vmaf2<T: VmafState> {
    context: *mut VmafContext,
    model: Rc<dyn VmafScoring>,
    state: PhantomData<T>,
}

pub struct ReadFrames;

pub struct GetScores;

pub trait VmafState {}

impl VmafState for ReadFrames {}
impl VmafState for GetScores {}

impl<T: VmafState> Vmaf2<T> {
    pub fn new(
        log_level: VmafLogLevel,
        n_threads: u32,
        n_subsample: u32,
        cpumask: u64,
        model: Rc<Model>
    ) -> Result<Vmaf2<ReadFrames>, VmafError> {
        let config = VmafConfiguration {
            log_level,
            n_threads,
            n_subsample,
            cpumask,
        };

        let ctx: *mut libvmaf_sys::VmafContext = std::ptr::null_mut();

        debug_assert!(ctx.is_null());

        let mut vmaf: Vmaf2<ReadFrames> = Vmaf2 {
            context: ctx,
            model: model.clone(),
            state: PhantomData::default(),
        };

        let err = unsafe { vmaf_init(&mut vmaf.context, config) };

        FFIError::check_err(err).map_err(|e| VmafError::Construct)?;

        vmaf.model.clone().load(&mut vmaf)?;

        Ok(vmaf)
    }
}

impl Vmaf2<ReadFrames> {
    pub fn read_framepair(
        &mut self,
        reference: Picture<ValidRef>,
        distorted: Picture<ValidRef>,
        index: u32,
    ) -> Result<(), VmafError> {
        let ref_resolution = reference.get_resolution();
        let dist_resolution = distorted.get_resolution();

        if ref_resolution != dist_resolution {
            return Err(VmafError::MismatchedResolution(
                ref_resolution.clone(),
                dist_resolution.clone(),
            ));
        }

        let err = unsafe {
            vmaf_read_pictures(
                **self,
                reference.as_ptr() as *mut VmafPicture,
                distorted.as_ptr() as *mut VmafPicture,
                index,
            )
        };

        reference.consume();
        distorted.consume();
        FFIError::check_err(err)?;
        Ok(())
    }

    pub fn flush_framebuffers(self) -> Result<Vmaf2<GetScores>, VmafError> {
        let null: *mut VmafPicture = ptr::null_mut();
        let err = unsafe { vmaf_read_pictures(*self, null.clone(), null.clone(), 0) };
        FFIError::check_err(err)?;

        return Ok(Vmaf2 {
            context: self.context,
            model: self.model.clone(),
            state: PhantomData,
        });
    }
}

impl<T: VmafState> Deref for Vmaf2<T> {
    type Target = *mut VmafContext;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl<T: VmafState> Drop for Vmaf2<T> {
    fn drop(&mut self) {
        unsafe {
            assert!(!self.context.is_null());
            let err = vmaf_close(**self);
            FFIError::check_err(err).unwrap();
        }
    }
}
