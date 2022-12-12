use std::{cell::Cell, path::Path};

use anyhow::anyhow;
use ffmpeg_next::{
    format::{context::Input, input, Pixel},
    media::Type,
    software::scaling::{context::Context, flag::Flags},
    Error, Stream,
};
use libvmaf_sys::VmafPixelFormat;

pub struct Video {
    input: Input,
    pixel_format: VmafPixelFormat,
    desired_width: u32,
    desired_height: u32,
}

impl Video<'_> {
    pub fn new(
        path: &dyn AsRef<Path>,
        format: VmafPixelFormat,
        w: u32,
        h: u32,
    ) -> Result<Video, anyhow::Error> {
        let context = input(&path)?;

        /*let input:Stream = context
            .streams()
            .best(Type::Video)
            .ok_or(Error::StreamNotFound)?;
        let video_index = input.index();

        let context_decoder =
            ffmpeg_next::codec::context::Context::from_parameters(input.parameters())?;
        let mut decoder = context_decoder.decoder().video()?;

        let pix_fmt = match format {
            VmafPixelFormat::VMAF_PIX_FMT_UNKNOWN => return Err(anyhow!("Unknown Pixel format!")),
            VmafPixelFormat::VMAF_PIX_FMT_YUV420P => Pixel::YUV420P,
            VmafPixelFormat::VMAF_PIX_FMT_YUV422P => Pixel::YUV422P,
            VmafPixelFormat::VMAF_PIX_FMT_YUV444P => Pixel::YUV444P,
            VmafPixelFormat::VMAF_PIX_FMT_YUV400P => {
                return Err(anyhow!("libavcodec does not support YUV400P"))
            }
        };

        let mut scaler = Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            pix_fmt,
            w,
            h,
            Flags::BILINEAR,
        )?;*/

        Ok(Video {
            input: context,
            pixel_format: format,
            desired_width: w,
            desired_height: h,
        })
    }
}
