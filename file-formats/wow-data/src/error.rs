use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WowDataError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Unsupported numeric version: {0}")]
    UnsupportedNumericVersion(u32),

    #[error("Conversion error: cannot convert from version {from} to {to}: {reason}")]
    ConversionError { from: u32, to: u32, reason: String },

    #[error("Generic error: {0}")]
    GenericError(String),
}

pub type Result<T> = std::result::Result<T, WowDataError>;
