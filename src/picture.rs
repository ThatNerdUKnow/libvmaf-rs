use std::{
    ffi::{c_int, c_uint},
    ops::{Deref, DerefMut},
};

use errno::Errno;
use libvmaf_sys::{vmaf_picture_alloc, vmaf_picture_unref, VmafPicture, VmafPixelFormat};

pub struct Picture {
    vmaf_picture: *mut VmafPicture,
}

impl Picture {
    pub fn new(
        pix_fmt: VmafPixelFormat,
        bpc: c_uint,
        w: c_uint,
        h: c_uint,
    ) -> Result<Picture, Errno> {
        let pic: *mut VmafPicture = std::ptr::null_mut();
        let err: i32 = unsafe { vmaf_picture_alloc(pic, pix_fmt, bpc, w, h) };
        match err {
            0 => Ok(Picture { vmaf_picture: pic }),
            _ => Err(Errno(-err)),
        }
    }
}

impl Deref for Picture {
    type Target = VmafPicture;

    fn deref(&self) -> &Self::Target {
        // Yeah i'm pretty sure this is a no-no
        unsafe { self.vmaf_picture.as_ref().unwrap() }
    }
}

impl DerefMut for Picture {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.vmaf_picture }
    }
}

impl Drop for Picture {
    fn drop(&mut self) {
        unsafe {
            vmaf_picture_unref(self.vmaf_picture);
        }
    }
}

#[cfg(test)]mod test{
    use libvmaf_sys::{VmafPixelFormat_VMAF_PIX_FMT_YUV422P, vmaf_version};

    use super::Picture;

#[test]
fn construct(){
    let _pic = Picture::new(VmafPixelFormat_VMAF_PIX_FMT_YUV422P,8,1920,1080).expect("Recieved error code from constructor");
}
}