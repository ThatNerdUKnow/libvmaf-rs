use ffmpeg_next::{
    codec::decoder::Video as VideoDecoder,
    format::{context::Input, input},
    frame::Video as VideoFrame,
    media::Type,
    software::scaling,
    software::scaling::Context as Scaler,
    Error as AVError, Stream, threading::Type as ThreadingType,
};
use std::path::Path;

pub struct Video {
    input: Input,
    decoder: VideoDecoder,
    video_index: usize,
    scaler: Scaler,
}

impl Video {
    pub fn new(path: &dyn AsRef<Path>, w: u32, h: u32) -> Result<Video, anyhow::Error> {
        // To tell the truth I have no idea what this does
        ffmpeg_next::init()?;

        // Create format context from path
        let input = input(&path)?;

        // Get index of best video stream
        let input_stream: Stream = input
            .streams()
            .best(Type::Video)
            .ok_or(AVError::StreamNotFound)?;
        let video_index = input_stream.index();

        // Instantiate an appropriate decoder for the input stream
        let context_decoder =
            ffmpeg_next::codec::context::Context::from_parameters(input_stream.parameters())?;
        let mut decoder = context_decoder.decoder().video()?;
        let mut threading_config = decoder.threading();
        threading_config.count = num_cpus::get();
        threading_config.kind = ThreadingType::Frame;

        decoder.set_threading(threading_config);

        let scaler = Scaler::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            decoder.format(),
            w,
            h,
            scaling::Flags::BILINEAR,
        )?;

        Ok(Video {
            input,
            decoder,
            video_index,
            scaler,
        })
    }
}

impl Iterator for Video {
    type Item = VideoFrame;

    fn next(&mut self) -> Option<Self::Item> {
        // This is an iterator of each packet in the selected video stream
        let packets = self
            .input
            .packets()
            .filter(|(stream, _packet)| stream.index() == self.video_index)
            .map(|(_stream, packet)| packet);

        for packet in packets {
            while self.decoder.send_packet(&packet)
                != Err(AVError::Other {
                    errno: libc::EAGAIN,
                })
            {
                break;
            }

            // Allocate an empty frame for our decoder to use
            // the relationship of packet to frame is not 1:1, so
            // if an error throws, just continue
            let mut frame = VideoFrame::empty();
            match self.decoder.receive_frame(&mut frame) {
                Ok(_) => {
                    let mut scaled_frame = VideoFrame::empty();
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
    use crate::picture::Picture;

    use super::Video;
    use std::path::Path;

    #[test]
    fn iterate() {
        let path = Path::new("./video/Big Buck Bunny 720P.m4v");

        let vid: Video = Video::new(&path, 1920, 1080).unwrap();

        for _frame in vid.into_iter() {
            // Do nothing
            let _picture:Picture = _frame.try_into().unwrap();
        }
    }
}
