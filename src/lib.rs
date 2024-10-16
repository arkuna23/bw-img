pub mod img;
pub mod file;

pub use img::*;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, BWError>;

#[derive(Error, Debug)]
pub enum BWError {
    #[error("err parsing file header: {0}")]
    FileHeader(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
