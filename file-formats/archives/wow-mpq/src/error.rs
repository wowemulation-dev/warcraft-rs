//! Error types for the MPQ library

use std::io;
use thiserror::Error;

/// Result type alias for MPQ operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for MPQ operations
#[derive(Error, Debug)]
pub enum Error {
    /// I/O error occurred
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Invalid MPQ format or corrupted archive
    #[error("Invalid MPQ format: {0}")]
    InvalidFormat(String),

    /// Unsupported MPQ version
    #[error("Unsupported MPQ version: {0}")]
    UnsupportedVersion(u16),

    /// File not found in archive
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Hash table error
    #[error("Hash table error: {0}")]
    HashTable(String),

    /// Block table error
    #[error("Block table error: {0}")]
    BlockTable(String),

    /// Encryption/decryption error
    #[error("Cryptography error: {0}")]
    Crypto(String),

    /// Compression/decompression error
    #[error("Compression error: {0}")]
    Compression(String),

    /// Signature verification failed
    #[error("Signature verification failed: {0}")]
    SignatureVerification(String),

    /// Invalid header location or alignment
    #[error("Invalid header: {0}")]
    InvalidHeader(String),

    /// Archive is read-only
    #[error("Archive is read-only")]
    ReadOnly,

    /// Operation not supported for this archive version
    #[error("Operation not supported for MPQ version {version}: {operation}")]
    OperationNotSupported {
        /// The MPQ version
        version: u16,
        /// The unsupported operation
        operation: String,
    },

    /// Invalid file size
    #[error("Invalid file size: expected {expected}, got {actual}")]
    InvalidFileSize {
        /// Expected size
        expected: u64,
        /// Actual size
        actual: u64,
    },

    /// Memory mapping error
    #[error("Memory mapping error: {0}")]
    MemoryMap(String),

    /// Invalid UTF-8 in filename
    #[error("Invalid UTF-8 in filename")]
    InvalidUtf8,

    /// Archive capacity exceeded
    #[error("Archive capacity exceeded: {0}")]
    CapacityExceeded(String),

    /// Checksum mismatch
    #[error("Checksum mismatch for {file}: expected {expected:08x}, got {actual:08x}")]
    ChecksumMismatch {
        /// File or table name
        file: String,
        /// Expected checksum
        expected: u32,
        /// Actual checksum
        actual: u32,
    },

    /// MD5 hash mismatch (v4 archives)
    #[error("MD5 hash mismatch for {table}")]
    MD5Mismatch {
        /// Table name
        table: String,
    },
}

impl Error {
    /// Create a new InvalidFormat error
    pub fn invalid_format<S: Into<String>>(msg: S) -> Self {
        Error::InvalidFormat(msg.into())
    }

    /// Create a new Crypto error
    pub fn crypto<S: Into<String>>(msg: S) -> Self {
        Error::Crypto(msg.into())
    }

    /// Create a new Compression error
    pub fn compression<S: Into<String>>(msg: S) -> Self {
        Error::Compression(msg.into())
    }

    /// Create a new HashTable error
    pub fn hash_table<S: Into<String>>(msg: S) -> Self {
        Error::HashTable(msg.into())
    }

    /// Create a new BlockTable error
    pub fn block_table<S: Into<String>>(msg: S) -> Self {
        Error::BlockTable(msg.into())
    }

    /// Check if this error indicates the archive is corrupted
    pub fn is_corruption(&self) -> bool {
        matches!(
            self,
            Error::InvalidFormat(_)
                | Error::ChecksumMismatch { .. }
                | Error::MD5Mismatch { .. }
                | Error::SignatureVerification(_)
                | Error::InvalidHeader(_)
        )
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Error::FileNotFound(_) | Error::ReadOnly | Error::OperationNotSupported { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::invalid_format("bad header");
        assert_eq!(err.to_string(), "Invalid MPQ format: bad header");

        let err = Error::FileNotFound("test.txt".to_string());
        assert_eq!(err.to_string(), "File not found: test.txt");
    }

    #[test]
    fn test_error_classification() {
        let corruption_err = Error::ChecksumMismatch {
            file: "test".to_string(),
            expected: 0x12345678,
            actual: 0x87654321,
        };
        assert!(corruption_err.is_corruption());
        assert!(!corruption_err.is_recoverable());

        let recoverable_err = Error::FileNotFound("missing.txt".to_string());
        assert!(!recoverable_err.is_corruption());
        assert!(recoverable_err.is_recoverable());
    }
}
