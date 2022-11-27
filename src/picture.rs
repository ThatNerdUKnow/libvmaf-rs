use errno::Errno;
use libc;
use libvmaf_sys::{vmaf_picture_alloc, vmaf_picture_unref, VmafPicture, VmafPixelFormat};
use std::{
    ffi::c_uint,
    mem,
    ops::{Deref, DerefMut},
    ptr,
};

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
        // Allocate memory for VmafPicture
        let pic: *mut VmafPicture =
            unsafe { libc::malloc(mem::size_of::<VmafPicture>()) as *mut VmafPicture };

        // Sanity check that our pointer is okay to use
        assert!(!pic.is_null());

        // Let libvmaf do their thing with our pointer
        let err: i32 = unsafe { vmaf_picture_alloc(pic, pix_fmt, bpc, w, h) };

        // Return an error if vmaf_picture_alloc returned an error code
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
        assert!(!self.vmaf_picture.is_null());
        unsafe { &mut *self.vmaf_picture }
    }
}

impl Drop for Picture {
    fn drop(&mut self) {
        // Allow FFI code to free its memory
        unsafe {
            // Each pointer in the data array should be valid at this point
            (*self.vmaf_picture)
                .data
                .iter()
                .for_each(|p| assert!(!p.is_null()));

            // Decrease reference count of self.vmaf_picture, which should free memory of vmaf_picture.data
            let err = vmaf_picture_unref(self.vmaf_picture);

            // Now that libvmaf has freed data referenced by vmaf_picture, each pointer in the data array should be null
            (*self.vmaf_picture)
                .data
                .iter()
                .for_each(|p| assert!(p.is_null()));

            // If we recieved anything besides the "OK" code from libvmaf, panic
            if err < 0 {
                panic!("Got Error {:?} When dropping Picture", Errno(-err));
            };
        }

        // Our raw pointer to vmaf_picture should still be valid
        assert!(!self.vmaf_picture.is_null());

        // Deallocate data pointed to by vmaf_picture and nullify vmaf_picture
        unsafe {
            libc::free(self.vmaf_picture as *mut libc::c_void);
            self.vmaf_picture = ptr::null_mut();
            assert!(self.vmaf_picture.is_null());
        }
    }
}

#[cfg(test)]
mod test {
    use libvmaf_sys::VmafPixelFormat_VMAF_PIX_FMT_YUV422P;

    use super::Picture;

    #[test]
    fn construct() {
        // Construct a new "Picture". Constructor and Drop should not panic
        let _pic = Picture::new(VmafPixelFormat_VMAF_PIX_FMT_YUV422P, 8, 1920, 1080)
            .expect("Recieved error code from constructor");
    }
}
