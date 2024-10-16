use std::error::Error;

pub use image::NormalImage;

pub trait ImageData {
    fn to_bw_data(&self) -> Result<Vec<u8>, Box<dyn Error>>;
    fn image_config(&self) -> BWImageSize;

    #[inline(always)]
    fn parse_bw_image(&self) -> Result<BWImage, Box<dyn Error>>
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
        fn to_bw_data(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            let mut buf = vec![];
            let mut bytes = vec![];
            for (_, _, pix) in self.img.pixels() {
                buf.push(is_white(pix[0], pix[1], pix[2], self.threshold));

                if buf.len() == 8 {
                    bytes.push(super::to_bw_data_byte(&buf));
                    buf.clear();
                }
            }
            bytes.push(super::to_bw_data_byte(&buf));

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
    fn to_bw_data(&self) -> Result<Vec<u8>, Box<dyn Error>> {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BWImageSize {
    pub width: u32,
    pub height: u32,
}

/// Black and white image
/// The image is stored as a 1-bit per pixel bitmap
/// The high bit is the first pixel
#[derive(Clone, Debug)]
pub struct BWImage {
    pub size: BWImageSize,
    pub pixels: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub enum IterDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug)]
pub enum IterOutput {
    Byte { byte: u8, len: usize },
    NewLine,
}

/// Iterator for black and white image
/// The iterator will iterate through the image in a specified direction
/// The iterator will return the position of the pixel and the value of the pixel(8 pixels per byte)
pub struct BWByteIter<'a> {
    direction: IterDirection,
    byte_size: BWImageSize,
    current: (u32, u32),
    pixels: &'a [u8],
}

impl<'a> BWByteIter<'a> {
    pub fn new(direction: IterDirection, size: &BWImageSize, pixels: &'a [u8]) -> Self {
        let byte_size = match direction {
            IterDirection::Horizontal => BWImageSize {
                width: size.width,
                height: size.height,
            },
            IterDirection::Vertical => size.clone(),
        };
        Self {
            direction,
            byte_size,
            current: (0, 0),
            pixels,
        }
    }
}

impl<'a> Iterator for BWByteIter<'a> {
    type Item = IterOutput;

    fn next(&mut self) -> Option<Self::Item> {
        let (x, y) = self.current;
        let BWImageSize { width, height } = self.byte_size.clone();
        match self.direction {
            IterDirection::Horizontal => {
                if y >= height {
                    None
                } else if x >= width {
                    self.current = (0, y + 1);
                    Some(IterOutput::NewLine)
                } else {
                    self.current = (x + 8, y);

                    Some(IterOutput::Byte {
                        byte: self.pixels[((y * width + x) / 8) as usize],
                        len: 8.min((width - x) as usize),
                    })
                }
            }
            IterDirection::Vertical => {
                if x >= width {
                    None
                } else if y + 8 >= height {
                    self.current = (x + 1, 0);
                    Some(IterOutput::NewLine)
                } else {
                    self.current = (x, y + 8);

                    let mut byt = 0u8;
                    let from = x * width + y;
                    let mut len = 8;
                    for i in 0..8 {
                        let pos = from + i * width;
                        if pos >= self.pixels.len() as u32 {
                            len = i as usize;
                            break;
                        }

                        byt |= self.pixels[pos as usize] << (7 - i);
                    }

                    Some(IterOutput::Byte { byte: byt, len })
                }
            }
        }
    }
}

impl BWImage {
    pub fn parse<T: ImageData>(data: &T) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            size: data.image_config(),
            pixels: data.to_bw_data()?,
        })
    }

    #[inline(always)]
    pub fn parse_file<R: std::io::Read>(input: &mut R) -> super::Result<Option<Self>> {
        crate::file::parse_file(input)
    }

    #[inline(always)]
    pub fn encode_as_file<W: std::io::Write>(&self, out: &mut W) -> super::Result<()> {
        crate::file::encode_file(out, self)
    }

    pub fn iterator(&self, direction: IterDirection) -> BWByteIter {
        BWByteIter::new(direction, &self.size, &self.pixels)
    }
}
