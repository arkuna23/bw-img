use crate::{img::BWImageSize, BWImage};

const MAGIC_NUMBER: &[u8; 4] = b"BWIM";

/// Parse the header of bw img file
pub fn parse_header<R: std::io::Read>(read: &mut R) -> super::Result<Option<BWImageSize>> {
    let mut header = [0u8; 16];
    if let Err(e) = read.read_exact(&mut header) {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            return Ok(None);
        } else {
            Err(e)?
        }
    }

    if &header[0..4] != MAGIC_NUMBER {
        return Err(super::BWError::FileHeader(format!(
            "img invalid magic number: {:?}",
            &header[0..4]
        )));
    }
    if u32::from_le_bytes([header[4], header[5], header[6], header[7]]) != 1 {
        return Err(super::BWError::FileHeader(format!(
            "invalid version number: {:?}",
            &header[4..8]
        )));
    }
    Ok(Some(BWImageSize {
        width: u32::from_le_bytes([header[8], header[9], header[10], header[11]]),
        height: u32::from_le_bytes([header[12], header[13], header[14], header[15]]),
    }))
}

/// write the header of bw img file
/// bw img file header format:
/// 0-3: magic number, "BWIM"
/// 4-7: version number, 1
/// 8-11: width, u32
/// 12-15: height, u32
pub fn write_header<W: std::io::Write>(write: &mut W, config: &BWImageSize) -> std::io::Result<()> {
    write.write_all(MAGIC_NUMBER)?;
    write.write_all(&1u32.to_le_bytes())?;
    write.write_all(&config.width.to_le_bytes())?;
    write.write_all(&config.height.to_le_bytes())?;
    Ok(())
}

/// Parse the bw image from file
pub fn parse_file<R: std::io::Read>(input: &mut R) -> super::Result<Option<(BWImage, u64)>> {
    Ok(match parse_header(input)? {
        Some(size) => {
            let len = size.get_padded_bytes_len();
            let mut data = vec![0u8; len as usize];
            input.read_exact(&mut data)?;
            Some((BWImage { size, pixels: data }, len + 16))
        }
        _ => None,
    })
}

/// Encode the bw image to file
pub fn encode_file<W: std::io::Write>(output: &mut W, img: &BWImage) -> super::Result<()> {
    write_header(output, &img.size)?;
    output.write_all(&img.pixels)?;
    output.flush()?;
    Ok(())
}

#[cfg(feature = "compress")]
pub mod compress {
    use std::io::Read;

    use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};

    use crate::{BWError, BWImage};

    pub struct DecompressIter<R: Read> {
        d: ZlibDecoder<R>,
        count: u32,
        position: u64,
    }

    impl<R: Read> Iterator for DecompressIter<R> {
        type Item = crate::Result<BWImage>;

        fn next(&mut self) -> Option<Self::Item> {
            match BWImage::parse_file(&mut self.d) {
                Ok(Some((img, size))) => {
                    self.count += 1;
                    self.position += size;
                    Some(Ok(img))
                }
                Ok(None) => None,
                Err(e) => Some(Err(BWError::Compression(
                    self.count as usize,
                    Box::new(e),
                    self.position,
                ))),
            }
        }
    }

    impl<R: Read> DecompressIter<R> {
        pub fn new(read: R) -> Self {
            Self {
                d: ZlibDecoder::new(read),
                count: 0,
                position: 0,
            }
        }
    }

    pub fn compress_imgs<W: std::io::Write>(imgs: &[BWImage], output: W) -> crate::Result<()> {
        let mut e = ZlibEncoder::new(output, Compression::best());
        for img in imgs {
            img.encode_as_file(&mut e)?;
        }
        e.finish()?;
        Ok(())
    }

    pub fn decompress_imgs<R: Read>(input: R) -> DecompressIter<R> {
        DecompressIter::new(input)
    }
}

#[cfg(feature = "video")]
pub mod video {
    use crate::{BWDataErr, BWImage, RgbData, VideoError};
    use ffmpeg_next::{
        codec::{self, packet::packet::Packet},
        decoder,
        format::{self, Pixel},
        frame::Video,
        software::scaling::{self, Flags},
    };
    use std::collections::LinkedList;

    pub struct VideoIter {
        pub frame_count: u64,
        pub input_size: (u32, u32),
        pub output_size: (u32, u32),
        pub duration: u64,
        pub frame_rate: u32,
        decoder: decoder::Video,
        scaler: scaling::Context,
        packets: LinkedList<Packet>,
    }

    impl VideoIter {
        /// Create a new VideoIter to process video file into bw-images
        /// Please note that `ffmpeg_next::init` should be called before creating this
        pub fn new(
            path: &str,
            width: Option<u32>,
            height: Option<u32>,
        ) -> Result<Self, VideoError> {
            let mut ictx = format::input(path)?;
            let stream = ictx
                .streams()
                .best(ffmpeg_next::media::Type::Video)
                .ok_or_else(|| VideoError::Other("no video stream".into()))?;
            let vid_index = stream.index();
            let decoder = codec::Context::from_parameters(stream.parameters())?
                .decoder()
                .video()?;
            let scaler = scaling::Context::get(
                decoder.format(),
                decoder.width(),
                decoder.height(),
                Pixel::RGB24,
                width.unwrap_or_else(|| decoder.width()),
                height.unwrap_or_else(|| decoder.height()),
                Flags::BILINEAR,
            )?;

            let duration = stream.duration();
            let time_base = stream.time_base();
            let frame_rate = stream.avg_frame_rate();
            if duration == 0 {
                return Err(VideoError::Other("No frames".into()));
            }

            let secs = duration as f64 / (time_base.0 * time_base.1) as f64;
            let frame_rate = (frame_rate.0 / frame_rate.1) as f64;
            let frames = secs * frame_rate;

            let packets = ictx
                .packets()
                .filter(|(stream, _)| stream.index() == vid_index)
                .map(|(_, packet)| packet)
                .collect();

            Ok(Self {
                frame_count: frames as u64,
                input_size: (decoder.width(), decoder.height()),
                output_size: (scaler.output().width, scaler.output().height),
                decoder,
                scaler,
                duration: secs as u64,
                frame_rate: frame_rate as u32,
                packets,
            })
        }

        pub fn convert(&mut self, packet: Packet) -> crate::Result<(Vec<BWImage>, u64)> {
            self.decoder
                .send_packet(&packet)
                .map_err(VideoError::FFMPEG)?;

            let mut frames = vec![];
            let mut decoded = Video::empty();
            let mut skipped = 0;
            while self.decoder.receive_frame(&mut decoded).is_ok() {
                let mut scalled = Video::empty();
                self.scaler
                    .run(&decoded, &mut scalled)
                    .map_err(VideoError::FFMPEG)?;

                let (width, height) = (self.scaler.output().width, self.scaler.output().height);
                let data = scalled.data(0);

                let img = match BWImage::parse(&RgbData::new(data, width, height)) {
                    Ok(r) => r,
                    Err(e) => {
                        if let BWDataErr::WrongSize(_, _, _) = e {
                            skipped += 1;
                            continue;
                        } else {
                            Err(e)?
                        }
                    }
                };

                frames.push(img);
            }

            Ok((frames, skipped))
        }
    }

    impl Iterator for VideoIter {
        type Item = crate::Result<(Vec<BWImage>, u64)>;

        fn next(&mut self) -> Option<Self::Item> {
            self.packets.pop_front().map(|packet| self.convert(packet))
        }
    }

    pub fn convert_video(
        path: &str,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<VideoIter, VideoError> {
        VideoIter::new(path, width, height)
    }

    pub fn convert_video_to_bw_imgs(
        path: &str,
        width: Option<u32>,
        height: Option<u32>,
    ) -> crate::Result<(Vec<BWImage>, u64)> {
        let mut skipped = 0;
        let frames = VideoIter::new(path, width, height)?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flat_map(|(f, s)| {
                skipped += s;
                f
            })
            .collect();

        Ok((frames, skipped))
    }
}
