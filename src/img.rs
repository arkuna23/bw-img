use std::error::Error;

pub trait ImageData {
    fn to_bw_data(&self) -> Result<Vec<u8>, Box<dyn Error>>;
    fn image_config(&self) -> BWImageConfig;
}

#[derive(Clone)]
pub struct RgbImage<'a> {
    data: &'a [u8],
    height: u32,
    width: u32,
    bw_threshold: u8,
}

impl<'a> RgbImage<'a> {
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

impl<'a> ImageData for RgbImage<'a> {
    fn to_bw_data(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(self
            .data
            .chunks(3 * 8)
            .map(|c| {
                // 8 bits per byte, one bit presents one pixel, high bit is the first pixel
                let mut bw_bit = 0u8;
                for (i, bit) in c.chunks(3).enumerate() {
                    let gray_value = (0.299 * bit[0] as f32
                        + 0.587 * bit[1] as f32
                        + 0.114 * bit[2] as f32) as u8;
                    if gray_value > self.bw_threshold {
                        bw_bit |= 1 << (7 - i);
                    }
                }
                bw_bit
            })
            .collect())
    }

    fn image_config(&self) -> BWImageConfig {
        BWImageConfig {
            width: self.width,
            height: self.height,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BWImageConfig {
    pub width: u32,
    pub height: u32,
}

/// Black and white image
/// The image is stored as a 1-bit per pixel bitmap
/// The high bit is the first pixel
#[derive(Clone, Debug)]
pub struct BWImage {
    pub config: BWImageConfig,
    pub data: Vec<u8>,
}

impl BWImage {
    pub fn parse(data: impl ImageData) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            config: data.image_config(),
            data: data.to_bw_data()?,
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
}
