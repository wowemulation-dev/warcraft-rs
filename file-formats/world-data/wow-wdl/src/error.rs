//! Error handling for WDL parsing

use std::io;
use thiserror::Error;

/// Errors that can occur when working with WDL files
#[derive(Debug, Error)]
pub enum WdlError {
    /// An I/O error occurred
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Invalid magic value in the file header
    #[error("Invalid magic value: expected '{expected}', found '{found}'")]
    InvalidMagic {
        /// The expected magic value
        expected: String,
        /// The actual magic value found
        found: String,
    },

    /// Unsupported WDL version
    #[error("Unsupported WDL version: {0}")]
    UnsupportedVersion(u32),

    /// Error when parsing WDL data
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Data validation failed
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Error when converting between WDL versions
    #[error("Version conversion error: {0}")]
    VersionConversionError(String),

    /// Unexpected end of file
    #[error("Unexpected end of file")]
    UnexpectedEof,

    /// Unexpected chunk found
    #[error("Unexpected chunk type: {0}")]
    UnexpectedChunk(String),
}

/// Type alias for Results from WDL operations
pub type Result<T> = std::result::Result<T, WdlError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = WdlError::ParseError("Test error".to_string());
        assert_eq!(format!("{}", error), "Parse error: Test error");

        let error = WdlError::InvalidMagic {
            expected: "MVER".to_string(),
            found: "ABCD".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Invalid magic value: expected 'MVER', found 'ABCD'"
        );
    }
}
