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

        // To tell the truth I have no idea what this does
        ffmpeg_next::init()?;

        // Create format context from path
        let input = input(&path)?;

        // Get index of best video stream
        let input_stream: Stream = input
            .streams()
            .best(Type::Video)
            .ok_or(Error::StreamNotFound)?;
        let video_index = input_stream.index();

        // Instantiate an appropriate decoder for the input stream
        let context_decoder =
            ffmpeg_next::codec::context::Context::from_parameters(input_stream.parameters())?;
        let decoder = context_decoder.decoder().video()?;

        // Select which pixel format to scale to
        let pix_fmt = match format {
            VmafPixelFormat::VMAF_PIX_FMT_UNKNOWN => return Err(anyhow!("Unknown Pixel format!")),
            VmafPixelFormat::VMAF_PIX_FMT_YUV420P => Pixel::YUV420P,
            VmafPixelFormat::VMAF_PIX_FMT_YUV422P => Pixel::YUV422P,
            VmafPixelFormat::VMAF_PIX_FMT_YUV444P => Pixel::YUV444P,
            VmafPixelFormat::VMAF_PIX_FMT_YUV400P => {
                return Err(anyhow!("libavcodec does not support YUV400P"))
            }
        };

        // Generate scaler
        let scaler = Context::get(
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

        // This is an iterator of each packet in the selected video stream
        let packets = self
            .input
            .packets()
            .filter(|(stream, _packet)| stream.index() == self.video_index)
            .map(|(_stream, packet)| packet);

        for packet in packets {

            // Send each packet into our decoder
            match self.decoder.send_packet(&packet) {
                Ok(_) => (),
                Err(_) => continue,
            }

            // Allocate an empty frame for our decoder to use
            // the relationship of packet to frame is not 1:1, so 
            // if an error throws, just continue
            let mut frame = ffmpeg_next::frame::Video::empty();
            match self.decoder.receive_frame(&mut frame) {
                Ok(_) => {

                    // Allocate an empty frame for our scaler
                    // then scale the frame and yield the final frame
                    let mut scaled_frame = ffmpeg_next::frame::Video::empty();
                    self.scaler.run(&frame, &mut scaled_frame).unwrap();
                    return Some(scaled_frame);
                }
                Err(_) => continue,
            }
        }

        // Send eof to decoder so it can clean up
        self.decoder.send_eof().unwrap();
        None
    }
}

#[cfg(test)]
mod test {
    use super::Video;
    use std::path::Path;

    #[test]
    fn iterate() {
        let path = Path::new("./video/Big Buck Bunny 720P.m4v");

        let vid: Video = Video::new(
            &path,
            libvmaf_sys::VmafPixelFormat::VMAF_PIX_FMT_YUV444P,
            1920,
            1080,
        )
        .unwrap();

        for _frame in vid.into_iter() {
            // Do nothing
        }
    }
}
