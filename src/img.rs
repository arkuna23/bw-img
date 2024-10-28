#[cfg(feature = "img")]
pub use image::NormalImage;

use crate::BWDataErr;

pub trait ImageData {
    fn to_bw_data(&self) -> Result<Vec<u8>, BWDataErr>;
    fn image_config(&self) -> BWImageSize;

    #[inline(always)]
    fn parse_bw_image(&self) -> Result<BWImage, BWDataErr>
    where
        Self: Sized,
    {
        BWImage::parse(self)
    }
}

#[derive(Clone)]
pub struct RgbData<'a> {
    data: &'a [u8],
    height: u32,
    width: u32,
    bw_threshold: u8,
}

#[cfg(feature = "img")]
mod image {
    use image::GenericImageView;

    use crate::BWDataErr;

    use super::{is_white, ImageData};

    pub struct NormalImage<'a> {
        img: &'a image::DynamicImage,
        threshold: u8,
    }
    impl<'a> NormalImage<'a> {
        pub fn new(img: &'a image::DynamicImage) -> Self {
            Self {
                img,
                threshold: 128,
            }
        }

        pub fn set_bw_threshold(&mut self, threshold: u8) {
            self.threshold = threshold
        }
    }

    impl<'a> ImageData for NormalImage<'a> {
        fn to_bw_data(&self) -> Result<Vec<u8>, BWDataErr> {
            let width = self.img.width();
            let mut buf = vec![];
            let mut bytes = vec![];
            for (x, _, pix) in self.img.pixels() {
                buf.push(is_white(pix[0], pix[1], pix[2], self.threshold));

                if x == width - 1 || buf.len() == 8 {
                    bytes.push(super::to_bw_data_byte(&buf));
                    buf.clear();
                }
            }

            Ok(bytes)
        }

        fn image_config(&self) -> super::BWImageSize {
            super::BWImageSize {
                width: self.img.width(),
                height: self.img.height(),
            }
        }
    }
}

impl<'a> RgbData<'a> {
    pub fn new(data: &'a [u8], width: u32, height: u32) -> Self {
        Self {
            data,
            height,
            width,
            bw_threshold: 128,
        }
    }

    pub fn set_bw_threshold(&mut self, threshold: u8) {
        self.bw_threshold = threshold;
    }
}

#[inline(always)]
fn is_white(r: u8, g: u8, b: u8, threshold: u8) -> bool {
    let gray_value = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) as u8;
    gray_value > threshold
}

fn to_bw_data_byte(data: &[bool]) -> u8 {
    // 8 bits per byte, one bit presents one pixel, high bit is the first pixel
    let mut bw_bit = 0u8;
    for (i, bit) in data.iter().enumerate() {
        if *bit {
            bw_bit |= 1 << (7 - i);
        }
    }
    bw_bit
}

impl<'a> ImageData for RgbData<'a> {
    fn to_bw_data(&self) -> Result<Vec<u8>, BWDataErr> {
        Ok(self
            .data
            .chunks(3 * 8)
            .map(|c| {
                to_bw_data_byte(
                    &c.chunks(3)
                        .map(|pix| is_white(pix[0], pix[1], pix[2], self.bw_threshold))
                        .collect::<Vec<_>>(),
                )
            })
            .collect())
    }

    fn image_config(&self) -> BWImageSize {
        BWImageSize {
            width: self.width,
            height: self.height,
        }
    }
}

pub trait BWByteData {
    /// Get the iterator of the black and white data with specified len.(8 pixels per byte)
    fn bw_byte_iter(&self, len: usize) -> impl Iterator<Item = bool>;
}

impl BWByteData for u8 {
    fn bw_byte_iter(&self, len: usize) -> impl Iterator<Item = bool> {
        (0..len).map(move |i| self & (1 << (7 - i)) != 0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BWImageSize {
    pub width: u32,
    pub height: u32,
}

impl BWImageSize {
    #[inline(always)]
    pub fn get_padded_bytes_len(&self) -> u64 {
        ((self.width as u64 + 7) / 8) * self.height as u64
    }
}

/// Black and white image
/// The image is stored as a 1-bit per pixel bitmap
/// The high bit is the first pixel
#[derive(Clone, Debug)]
pub struct BWImage {
    pub size: BWImageSize,
    pub pixels: Vec<u8>,
}

#[derive(Clone)]
pub struct BWIterState<'a> {
    pub size: BWImageSize,
    pub current: (u32, u32),
    pub pixels: &'a [u8],
}

#[derive(Clone, Debug)]
pub enum IterOutput {
    Byte { byte: u8, len: usize },
    NewLine,
}

pub trait IterDirection {
    /// Get the position after this iteration, and the byte of pixels or other result in this iteration
    fn next(&mut self, state: BWIterState) -> Option<((u32, u32), IterOutput)>;
}

impl<D: IterDirection> Iterator for BWByteIter<'_, D> {
    type Item = IterOutput;

    fn next(&mut self) -> Option<Self::Item> {
        self.direction
            .next(BWIterState {
                size: self.size,
                current: self.current,
                pixels: self.pixels,
            })
            .map(|(current, out)| {
                self.current = current;
                out
            })
    }
}

#[macro_export]
macro_rules! define_direction {
    ($type:ident, $impl_bo:tt) => {
        pub struct $type;

        impl $crate::img::IterDirection for $type $impl_bo
    };
}

pub mod iter_direction {
    use crate::{BWImageSize, IterOutput};

    define_direction!(Horizontal, {
        fn next(&mut self, state: crate::BWIterState) -> Option<((u32, u32), IterOutput)> {
            let (x, y) = state.current;
            let BWImageSize { width, height } = state.size;
            if y >= height {
                None
            } else if x >= width {
                Some(((0, y + 1), IterOutput::NewLine))
            } else {
                Some((
                    (x + 8, y),
                    IterOutput::Byte {
                        byte: state.pixels[((y * width + x) / 8) as usize],
                        len: 8.min((width - x) as usize),
                    },
                ))
            }
        }
    });
    define_direction!(Vertical, {
        fn next(&mut self, state: crate::BWIterState) -> Option<((u32, u32), IterOutput)> {
            let (x, y) = state.current;
            let BWImageSize { width, height } = state.size;
            if x >= width {
                None
            } else if y >= height {
                Some(((x + 1, 0), IterOutput::NewLine))
            } else {
                let mut byt = 0u8;
                let width_in_byte = ((width as f64) / 8f64).ceil() as u32;
                let (row_byte, rev_idx_at_byte) = (x / 8, (7 - x % 8));

                let from_byte = y * width_in_byte + row_byte;
                let mut len = 8;
                for i in 0..8 {
                    let pos = from_byte + i * width_in_byte;
                    if pos >= state.pixels.len() as u32 {
                        len = i as usize;
                        break;
                    }

                    byt |= ((state.pixels[pos as usize] >> rev_idx_at_byte) & 0b1) << (7 - i);
                }

                Some(((x, y + len as u32), IterOutput::Byte { byte: byt, len }))
            }
        }
    });
}

/// Iterator for black and white image
/// The iterator will iterate through the image in a specified direction
/// The iterator will return the position of the pixel and the value of the pixel(8 pixels per byte)
pub struct BWByteIter<'a, D: IterDirection> {
    size: BWImageSize,
    current: (u32, u32),
    pixels: &'a [u8],
    direction: D,
}

impl<'a, T: IterDirection> BWByteIter<'a, T> {
    pub fn new(size: &BWImageSize, pixels: &'a [u8], direction: T) -> Self {
        Self {
            direction,
            size: BWImageSize {
                width: size.width,
                height: size.height,
            },
            current: (0, 0),
            pixels,
        }
    }
}

impl BWImage {
    pub fn parse<T: ImageData>(data: &T) -> Result<Self, BWDataErr> {
        Ok(Self {
            size: data.image_config(),
            pixels: data.to_bw_data()?,
        })
    }

    #[inline(always)]
    pub fn parse_file<R: std::io::Read>(input: &mut R) -> super::Result<Option<(Self, u64)>> {
        crate::file::parse_file(input)
    }

    #[inline(always)]
    pub fn encode_as_file<W: std::io::Write>(&self, out: &mut W) -> super::Result<()> {
        crate::file::encode_file(out, self)
    }

    pub fn iterator<D: IterDirection>(&self, direction: D) -> BWByteIter<D> {
        BWByteIter::new(&self.size, &self.pixels, direction)
    }
}
