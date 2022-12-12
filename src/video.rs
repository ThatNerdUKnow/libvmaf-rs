use std::path::Path;

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
    decoder: ffmpeg_next::codec::decoder::Video,
    scaler: Context,
    video_index: usize,
    pixel_format: VmafPixelFormat,
}

impl Video {
    pub fn new(
        path: &dyn AsRef<Path>,
        format: VmafPixelFormat,
        w: u32,
        h: u32,
    ) -> Result<Video, anyhow::Error> {
        let input = input(&path)?;

        let input_stream: Stream = input
            .streams()
            .best(Type::Video)
            .ok_or(Error::StreamNotFound)?;
        let video_index = input_stream.index();

        let context_decoder =
            ffmpeg_next::codec::context::Context::from_parameters(input_stream.parameters())?;
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
        )?;

        Ok(Video {
            input,
            pixel_format: format,
            decoder,
            scaler,
            video_index,
        })
    }
}

impl Iterator for Video {
    type Item = ffmpeg_next::frame::Video;

    fn next(&mut self) -> Option<Self::Item> {
        self.input
            .packets()
            .filter(|(stream, _packet)| stream.index() == self.video_index)
            .map(|(_stream, packet)| packet)
            .map(|packet| {
                self.decoder.send_packet(&packet).unwrap();
                let mut frame = ffmpeg_next::frame::Video::empty();
                self.decoder.receive_frame(&mut frame).unwrap();
                let mut scaled_frame = ffmpeg_next::frame::Video::empty();
                self.scaler.run(&frame, &mut scaled_frame).unwrap();
                scaled_frame
            })
            .next()
    }
}
