pub mod file;
pub mod img;

use std::error::Error;

pub use img::*;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, BWError>;

#[derive(Error, Debug)]
pub enum BWError {
    #[error("error parsing bw image {0}: {1}, position: {2}")]
    Compression(usize, Box<BWError>, u64),
    #[error("err parsing file header: {0}")]
    FileHeader(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    BWDataErr(#[from] BWDataErr),
    #[cfg(feature = "video")]
    #[error(transparent)]
    VideoErr(#[from] VideoError),
}

#[derive(Error, Debug)]
pub enum BWDataErr {
    #[error("error parsing bw data: {0}")]
    Custom(Box<dyn Error + Send + Sync>),
    #[error("{0}x{1} is not divisible by 8, got {2} pixels")]
    WrongSize(u32, u32, usize),
}

#[cfg(feature = "video")]
#[derive(Error, Debug)]
pub enum VideoError {
    #[error(transparent)]
    FFMPEG(#[from] ffmpeg_next::Error),
    #[error("{0}")]
    Other(String),
}
