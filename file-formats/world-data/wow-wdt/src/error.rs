//! Error types for the WDT library

use std::io;
use thiserror::Error;

/// Result type alias for WDT operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during WDT operations
#[derive(Error, Debug)]
pub enum Error {
    /// I/O error occurred
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Invalid magic bytes for a chunk
    #[error("Invalid chunk magic: expected {expected:?}, found {found:?}")]
    InvalidMagic { expected: [u8; 4], found: [u8; 4] },

    /// Unexpected chunk size
    #[error("Invalid chunk size for {chunk}: expected {expected}, found {found}")]
    InvalidChunkSize {
        chunk: String,
        expected: usize,
        found: usize,
    },

    /// Invalid WDT version
    #[error("Invalid WDT version: expected 18, found {0}")]
    InvalidVersion(u32),

    /// Missing required chunk
    #[error("Missing required chunk: {0}")]
    MissingChunk(String),

    /// Invalid data within a chunk
    #[error("Invalid {chunk} data: {message}")]
    InvalidChunkData { chunk: String, message: String },

    /// Version conversion error
    #[error("Cannot convert from {from} to {to}: {reason}")]
    ConversionError {
        from: String,
        to: String,
        reason: String,
    },

    /// Validation error
    #[error("Validation failed: {0}")]
    ValidationError(String),

    /// Unsupported feature for version
    #[error("Feature {feature} is not supported in version {version}")]
    UnsupportedFeature { feature: String, version: String },

    /// String encoding error
    #[error("Invalid string encoding in {context}: {message}")]
    StringError { context: String, message: String },

    /// File size exceeded limit
    #[error("File size {size} exceeds limit {limit} for {context}")]
    SizeLimit {
        size: usize,
        limit: usize,
        context: String,
    },
}

impl Error {
    /// Create an invalid magic error with chunk name
    pub fn invalid_magic_str(_chunk: &str, expected: &[u8; 4], found: &[u8; 4]) -> Self {
        Error::InvalidMagic {
            expected: *expected,
            found: *found,
        }
    }

    /// Create an invalid chunk data error
    pub fn invalid_data(chunk: impl Into<String>, message: impl Into<String>) -> Self {
        Error::InvalidChunkData {
            chunk: chunk.into(),
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Error::ValidationError(message.into())
    }
}
