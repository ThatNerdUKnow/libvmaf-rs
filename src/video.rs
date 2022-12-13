use std::path::Path;


use ffmpeg_next::{
    format::{context::Input, input},
    media::Type,
    Error as AVError, Stream,
};

pub struct Video {
    input: Input,
    decoder: ffmpeg_next::codec::decoder::Video,
    video_index: usize,
}

impl Video {
    pub fn new(
        path: &dyn AsRef<Path>,
    ) -> Result<Video, anyhow::Error> {

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
        let decoder = context_decoder.decoder().video()?;

        Ok(Video {
            input,
            decoder,
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

            while self.decoder.send_packet(&packet) != Err(AVError::Other { errno: libc::EAGAIN }){
                break;
            }

            // Allocate an empty frame for our decoder to use
            // the relationship of packet to frame is not 1:1, so 
            // if an error throws, just continue
            let mut frame = ffmpeg_next::frame::Video::empty();
            match self.decoder.receive_frame(&mut frame) {
                Ok(_) => {
                    return Some(frame);
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
        )
        .unwrap();

        for _frame in vid.into_iter() {
            // Do nothing
        }
    }
}
