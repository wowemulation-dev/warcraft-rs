use std::io;
use thiserror::Error;

/// Error types for WMO parsing and processing
#[derive(Error, Debug)]
pub enum WmoError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid magic identifier: expected {expected:?}, found {found:?}")]
    InvalidMagic { expected: [u8; 4], found: [u8; 4] },

    #[error("Unexpected end of file")]
    UnexpectedEof,

    #[error("Invalid chunk size: {0}")]
    InvalidChunkSize(u32),

    #[error("Invalid version: {0}")]
    InvalidVersion(u32),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Unsupported conversion: from version {from} to {to}")]
    UnsupportedConversion { from: u32, to: u32 },

    #[error("Missing required chunk: {0}")]
    MissingRequiredChunk(String),

    #[error("Duplicate chunk: {0}")]
    DuplicateChunk(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Invalid reference: {field} value {value} exceeds maximum {max}")]
    InvalidReference { field: String, value: u32, max: u32 },
}

/// Result type for WMO operations
pub type Result<T> = std::result::Result<T, WmoError>;
