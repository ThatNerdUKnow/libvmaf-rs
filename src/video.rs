use error_stack::{IntoReport, Result, ResultExt};
use ffmpeg_next::{
    codec::context::Context as Codec,
    codec::decoder::Video as VideoDecoder,
    format::{context::Input, input},
    frame::Video as VideoFrame,
    media::Type,
    software::scaling,
    software::scaling::Context as Scaler,
    threading::Type as ThreadingType,
    Error as AVError, Stream,
};
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};
use thiserror::Error;

/// The following trait represents the number of frames in the video stream
pub trait FrameNum{
    fn get_frames(&self) -> i64;
}

pub struct Video {
    input: Input,
    decoder: VideoDecoder,
    video_index: usize,
    scaler: Scaler,
    number_of_frames: i64,
}

#[derive(Error, Debug)]
pub enum VideoError {
    #[error("Encountered an error when creating video context {0}")]
    Construct(PathBuf),
}

impl Video {
    pub fn new(path: &dyn AsRef<Path>, w: u32, h: u32) -> Result<Video, VideoError> {
        // To tell the truth I have no idea what this does
        ffmpeg_next::init()
            .into_report()
            .change_context(VideoError::Construct(path.as_ref().to_owned()))?;

        // Create format context from path
        let input = input(&path)
            .into_report()
            .change_context(VideoError::Construct(path.as_ref().to_owned()))?;

        // Get index of best video stream
        let input_stream: Stream = input
            .streams()
            .best(Type::Video)
            .ok_or(AVError::StreamNotFound)
            .into_report()
            .change_context(VideoError::Construct(path.as_ref().to_owned()))?;

        let number_of_frames = input_stream.frames();

        let video_index = input_stream.index();

        // Instantiate an appropriate decoder for the input stream
        let context_decoder = Codec::from_parameters(input_stream.parameters())
            .into_report()
            .change_context(VideoError::Construct(path.as_ref().to_owned()))?;

        let mut decoder = context_decoder
            .decoder()
            .video()
            .into_report()
            .change_context(VideoError::Construct(path.as_ref().to_owned()))?;

        let mut threading_config = decoder.threading();
        threading_config.count = num_cpus::get();
        threading_config.kind = ThreadingType::Slice;

        decoder.set_threading(threading_config);

        let scaler = Scaler::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            decoder.format(),
            w,
            h,
            scaling::Flags::BILINEAR,
        )
        .into_report()
        .change_context(VideoError::Construct(path.as_ref().to_owned()))?;

        Ok(Video {
            input,
            decoder,
            video_index,
            scaler,
            number_of_frames
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
        self.decoder
            .send_eof()
            .into_report()
            .attach_printable("Encountered error when decoding video")
            .unwrap();
        None
    }
}

impl FrameNum for Video{
    fn get_frames(&self) -> i64 {
        self.number_of_frames
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
            let _picture: Picture = _frame.try_into().unwrap();
        }
    }
}
