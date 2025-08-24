use std::io;
use thiserror::Error;
use wow_data::error::WowDataError;

use crate::MD20Version;

/// Error types for M2 model parsing and processing
#[derive(Error, Debug)]
pub enum M2Error {
    /// I/O Error during reading or writing
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("wow-data error: {0}")]
    WowData(#[from] wow_data::error::WowDataError),

    /// Invalid magic number in the file header
    #[error("Invalid magic number: expected '{expected}', got '{actual}'")]
    InvalidMagic { expected: String, actual: String },

    /// Unsupported file version
    #[error("Unsupported version: {0}")]
    UnsupportedVersion(String),

    /// Unsupported numeric file version
    #[error("Unsupported numeric version: {0}")]
    UnsupportedNumericVersion(u32),

    /// Unsupported version conversion
    #[error("Unsupported conversion to version: {0}")]
    UnsupportedVersionWriting(MD20Version),

    /// Error during parsing
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Error during validation
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Error during version conversion
    #[error("Conversion error: cannot convert from version {from} to {to}: {reason}")]
    ConversionError { from: u32, to: u32, reason: String },

    /// Chunk error: missing expected chunk or invalid chunk
    #[error("Chunk error: {0}")]
    ChunkError(String),

    /// Reference error: invalid reference in the file
    #[error("Reference error: {0}")]
    ReferenceError(String),

    /// Internal error: something went wrong in the parser logic
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type using M2Error
pub type Result<T> = std::result::Result<T, M2Error>;

impl From<M2Error> for WowDataError {
    fn from(value: M2Error) -> Self {
        WowDataError::GenericError(format!("M2Error: {}", value))
    }
}
