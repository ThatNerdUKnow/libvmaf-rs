use std::ffi::{c_int, c_uint};

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
    ) -> Result<Picture, c_int> {
        let pic: *mut VmafPicture = std::ptr::null_mut();
        let err: i32 = unsafe { vmaf_picture_alloc(pic, pix_fmt, bpc, w, h) };
        match err {
            0 => Ok(Picture { vmaf_picture: pic }),
            _ => Err(err),
        }
    }
}

impl Drop for Picture {
    fn drop(&mut self) {
        unsafe {
            vmaf_picture_unref(self.vmaf_picture);
        }
    }
}
