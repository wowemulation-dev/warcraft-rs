// error.rs - Error types for ADT parser

// use std::fmt;
use thiserror::Error;

/// Result type alias for ADT parser operations
pub type Result<T> = std::result::Result<T, AdtError>;

/// Error types for ADT parser
#[derive(Debug, Error)]
pub enum AdtError {
    /// I/O error during file operations
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid magic signature in chunk header
    #[error("Invalid magic: expected {expected}, found {found}")]
    InvalidMagic { expected: String, found: String },

    /// Unsupported file version
    #[error("Unsupported version: {0}")]
    UnsupportedVersion(u32),

    /// Missing required chunk
    #[error("Missing chunk: {0}")]
    MissingChunk(String),

    /// Invalid chunk size
    #[error("Invalid chunk size for {chunk}: {size} (expected {expected})")]
    InvalidChunkSize {
        chunk: String,
        size: u32,
        expected: u32,
    },

    /// End of file reached unexpectedly
    #[error("Unexpected end of file")]
    UnexpectedEof,

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Version conversion not supported
    #[error("Cannot convert from {from} to {to}")]
    VersionConversionUnsupported { from: String, to: String },

    /// Parsing error
    #[error("Parsing error: {0}")]
    ParseError(String),

    /// Not implemented yet
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Invalid version string
    #[error("Invalid version: {0}")]
    InvalidVersion(String),

    /// Invalid file size
    #[error("Invalid file size: {0}")]
    InvalidFileSize(String),
}
