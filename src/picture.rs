use error_stack::{Report, Result, ResultExt};
use ffmpeg_next::{format::Pixel, frame::Video as VideoFrame};
use libc::{self, c_void, memcpy};
pub use libvmaf_sys::VmafPixelFormat;
use libvmaf_sys::{vmaf_picture_alloc, vmaf_picture_unref, VmafPicture};
use ptrplus::{AsPtr, FromRaw, IntoRaw};
use std::{ffi::c_uint, marker::PhantomData, mem};

use crate::{error::FFIError, picture::error::PictureError};

pub mod error;
/// A safe wrapper around `*mut VmafPicture`
///
/// Unless you're trying to use a library besides FFMPEG for decoding video,
/// you shouldn't concern yourself with this struct
pub struct Picture<State: Consumable = ValidRef> {
    vmaf_picture: Option<*mut VmafPicture>,
    consumed: PhantomData<State>,
}

pub struct ValidRef;
impl Consumable for ValidRef {}
pub struct Consumed;
impl Consumable for Consumed {}
pub trait Consumable {}

impl Picture {
    pub fn new(
        pix_fmt: VmafPixelFormat,
        bits_per_channel: c_uint,
        width: c_uint,
        height: c_uint,
    ) -> Result<Picture, PictureError> {
        // Allocate memory for VmafPicture
        let pic: *mut VmafPicture =
            unsafe { libc::malloc(mem::size_of::<VmafPicture>()) as *mut VmafPicture };

        // Sanity check that our pointer is okay to use
        debug_assert!(!pic.is_null());

        // Let libvmaf do their thing with our pointer
        let err: i32 = unsafe { vmaf_picture_alloc(pic, pix_fmt, bits_per_channel, width, height) };

        // Return an error if vmaf_picture_alloc returned an error code
        FFIError::check_err(err).change_context(PictureError::Construct)?;

        Ok(Picture {
            vmaf_picture: Some(pic),
            consumed: PhantomData,
        })
    }

    /// This method is intended to be used when `self` is passed to a function that calls `vmaf_picture_unref` internally  
    /// Notably, `vmaf_read_pictures` does this
    pub fn consume(self) -> Picture<Consumed> {
        Picture {
            vmaf_picture: None,
            consumed: PhantomData,
        }
    }
}

impl TryFrom<VideoFrame> for Picture {
    type Error = Report<PictureError>;

    fn try_from(frame: VideoFrame) -> core::result::Result<Self, Self::Error> {
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
            _ => {
                return Err(Report::new(PictureError::Decode)
                    .attach_printable(format!("{:?}", frame.format())))
            }
        };

        let picture = Picture::new(format, bits_per_channel, frame.width(), frame.height())?;

        let src = unsafe { frame.as_ptr() };
        let dst = picture.as_ptr();
        // Fill pixel data
        let bytes_per_value: usize = match bits_per_channel {
            0..=8 => 1,
            _ => 2,
        };

        let conversion_handler = |e| {
            Err(Report::new(e)
                .change_context(PictureError::Decode)
                .attach_printable("When copying pixel data"))
        };

        unsafe {
            for i in 0..3 {
                let mut src_data: *const c_void = (*src).data[i] as *const c_void;
                let mut dst_data = (*dst).data[i];

                for _ in 0..(*dst).h[i] {
                    memcpy(dst_data, src_data, bytes_per_value * (*dst).w[i] as usize);

                    let linesize_src = match (*src).linesize[i].try_into() {
                        Ok(n) => n,
                        Err(e) => return conversion_handler(e).attach_printable("src"),
                    };

                    let linesize_dst = match (*dst).stride[i].try_into() {
                        Ok(n) => n,
                        Err(e) => return conversion_handler(e).attach_printable("dst"),
                    };

                    src_data = src_data.add(linesize_src);
                    dst_data = dst_data.add(linesize_dst);
                }
            }
        }

        Ok(picture)
    }
}

impl AsPtr for Picture<ValidRef> {
    type Raw = VmafPicture;

    fn as_ptr(&self) -> *const Self::Raw {
        self.vmaf_picture.unwrap()
    }
}

impl IntoRaw for Picture<ValidRef> {
    type Raw = VmafPicture;

    fn into_raw(self) -> *mut Self::Raw {
        self.vmaf_picture.unwrap()
    }
}

impl FromRaw<VmafPicture> for Picture {
    /// Safety warning! This function assumes `raw` hasn't been consumed yet! For reference on what I mean by "consume" please refer to `Picture.consume()`  
    /// If `raw` is a pointer which has previously been given to `libvmaf_sys::vmaf_read_pictures` this will
    /// cause a double free, as `vmaf_picture_unref` will be called twice!
    unsafe fn from_raw(raw: *mut VmafPicture) -> Self {
        Self {
            vmaf_picture: Some(raw),
            consumed: PhantomData,
        }
    }
}

impl<T: Consumable> Drop for Picture<T> {
    fn drop(&mut self) {
        // Allow FFI code to free its memory

        match self.vmaf_picture {
            Some(ptr) => unsafe {
                vmaf_picture_unref(ptr);
                libc::free(ptr as *mut c_void);
            },
            None => (),
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
