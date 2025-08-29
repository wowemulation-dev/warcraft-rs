use std::io;
use thiserror::Error;

use crate::anim::AnimFormat;

/// Error types for M2 model parsing and processing
#[derive(Error, Debug)]
pub enum M2Error {
    /// I/O Error during reading or writing
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Invalid magic number in the file header
    #[error("Invalid magic number: expected '{expected}', got '{actual}'")]
    InvalidMagic { expected: String, actual: String },

    /// Unsupported file version
    #[error("Unsupported version: {0}")]
    UnsupportedVersion(String),

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

    /// ANIM format detection failed
    #[error("ANIM format detection failed: {0}")]
    AnimFormatError(String),

    /// Legacy ANIM parsing error
    #[error("Legacy ANIM parsing error: {0}")]
    LegacyAnimError(String),

    /// ANIM format conversion error
    #[error("ANIM format conversion error: from {from:?} to {to:?}: {reason}")]
    AnimConversionError {
        from: AnimFormat,
        to: AnimFormat,
        reason: String,
    },

    /// Invalid magic bytes in chunked format
    #[error("Invalid magic bytes: {0:?}")]
    InvalidMagicBytes([u8; 4]),

    /// Missing required MD21 chunk
    #[error("Missing required MD21 chunk")]
    MissingMD21Chunk,

    /// Malformed chunk header
    #[error("Malformed chunk header")]
    MalformedChunk,

    /// Unknown FileDataID
    #[error("Unknown FileDataID: {0}")]
    UnknownFileDataId(u32),

    /// Failed to resolve external file
    #[error("Failed to resolve external file: {0}")]
    ExternalFileError(String),
}

/// Result type using M2Error
pub type Result<T> = std::result::Result<T, M2Error>;
