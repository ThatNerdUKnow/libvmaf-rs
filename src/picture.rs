use anyhow::anyhow;
use errno::Errno;
use ffmpeg_next::{format::Pixel, frame::Video as VideoFrame};
use libc::{self, c_void, memcpy};
pub use libvmaf_sys::VmafPixelFormat;
use libvmaf_sys::{vmaf_picture_alloc, vmaf_picture_unref, VmafPicture};
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
        bits_per_channel: c_uint,
        width: c_uint,
        height: c_uint,
    ) -> Result<Picture, Errno> {
        // Allocate memory for VmafPicture
        let pic: *mut VmafPicture =
            unsafe { libc::malloc(mem::size_of::<VmafPicture>()) as *mut VmafPicture };

        // Sanity check that our pointer is okay to use
        assert!(!pic.is_null());

        // Let libvmaf do their thing with our pointer
        let err: i32 = unsafe { vmaf_picture_alloc(pic, pix_fmt, bits_per_channel, width, height) };

        // Return an error if vmaf_picture_alloc returned an error code
        match err {
            0 => Ok(Picture { vmaf_picture: pic }),
            _ => Err(Errno(-err)),
        }
    }
}

impl TryFrom<VideoFrame> for Picture {
    type Error = anyhow::Error;

    fn try_from(frame: VideoFrame) -> Result<Self, Self::Error> {
        // Get pixel format
        let format = match frame.format() {
            Pixel::YUV420P | Pixel::YUV420P10LE | Pixel::YUV420P12LE | Pixel::YUV420P16LE => {
                VmafPixelFormat::VMAF_PIX_FMT_YUV420P
            }
            Pixel::YUV422P | Pixel::YUV422P10LE | Pixel::YUV422P12LE | Pixel::YUV422P16LE => {
                VmafPixelFormat::VMAF_PIX_FMT_YUV422P
            }
            Pixel::YUV444P | Pixel::YUV444P10LE | Pixel::YUV444P12LE | Pixel::YUV444P16LE => {
                VmafPixelFormat::VMAF_PIX_FMT_YUV444P
            }
            _ => VmafPixelFormat::VMAF_PIX_FMT_UNKNOWN,
        };

        // Get bits per channel
        let bits_per_channel: u32 = match frame.format() {
            Pixel::YUV420P | Pixel::YUV422P | Pixel::YUV444P => 8,
            Pixel::YUV420P10LE | Pixel::YUV422P10LE | Pixel::YUV444P10LE => 10,
            Pixel::YUV420P12LE | Pixel::YUV422P12LE | Pixel::YUV444P12LE => 12,
            Pixel::YUV420P16LE | Pixel::YUV422P16LE | Pixel::YUV444P16LE => 16,
            _ => return Err(anyhow!("Invalid pixel format: {:?}", frame.format())),
        };

        let picture = Picture::new(format, bits_per_channel, frame.width(), frame.height())?;

        let src = unsafe { frame.as_ptr() };
        let dst = *picture;
        // Fill pixel data
        let bytes_per_value: usize = match bits_per_channel {
            0..=8 => 1,
            _ => 2,
        };

        unsafe {
            for i in 0..3 {
                let mut src_data: *const c_void = (*src).data[i] as *const c_void;
                let mut dst_data = (*dst).data[i];

                for _ in 0..(*dst).h[i] {
                    memcpy(dst_data, src_data, bytes_per_value * (*dst).w[i] as usize);
                    src_data = src_data.add((*src).linesize[i].try_into()?);
                    dst_data = dst_data.add((*dst).stride[i].try_into()?);
                }
            }
        }

        Ok(picture)
    }
}

impl Deref for Picture {
    type Target = *mut VmafPicture;

    fn deref(&self) -> &Self::Target {
        assert!(!self.vmaf_picture.is_null());
        &self.vmaf_picture
    }
}

impl DerefMut for Picture {
    fn deref_mut(&mut self) -> &mut Self::Target {
        assert!(!self.vmaf_picture.is_null());
        &mut self.vmaf_picture
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
    use libvmaf_sys::VmafPixelFormat;

    use super::Picture;

    #[test]
    fn construct() {
        // Construct a new "Picture". Constructor and Drop should not panic
        let _pic = Picture::new(VmafPixelFormat::VMAF_PIX_FMT_YUV422P, 8, 1920, 1080)
            .expect("Recieved error code from constructor");
    }
}
