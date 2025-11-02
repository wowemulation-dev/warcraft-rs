//! Error types for ADT file parsing and validation.
//!
//! This module implements a fail-fast error strategy where parsing halts at the first
//! critical failure. This approach ensures data integrity and prevents cascading errors
//! from corrupted or malformed ADT files.
//!
//! # Error Categories
//!
//! ## Critical Errors (halt parsing immediately)
//!
//! - [`AdtError::InvalidMagic`] - Chunk magic bytes don't match expected format
//! - [`AdtError::MissingRequiredChunk`] - Required chunk (MVER, MHDR, MCIN, MCNK) is missing
//! - [`AdtError::InvalidChunkCombination`] - Mutually exclusive chunks are present
//! - [`AdtError::OffsetOutOfBounds`] - Chunk offset exceeds file boundaries
//! - [`AdtError::InvalidChunkSize`] - Chunk size doesn't match format specification
//! - [`AdtError::InvalidSubchunkOffset`] - Subchunk offset exceeds parent chunk size
//! - [`AdtError::InvalidWaterStructure`] - Water data structure is malformed
//! - [`AdtError::InvalidTextureReference`] - Texture index exceeds available textures
//! - [`AdtError::InvalidModelReference`] - Model index exceeds available models
//! - [`AdtError::InvalidMcinEntry`] - MCIN entry references non-existent MCNK chunk
//! - [`AdtError::ChunkParseError`] - Generic chunk parsing failure with context
//! - [`AdtError::MemoryLimitExceeded`] - Allocation exceeds safety limits
//! - [`AdtError::VersionDetectionFailed`] - Cannot determine ADT format version
//! - [`AdtError::Io`] - Underlying I/O error
//! - [`AdtError::BinrwError`] - Binary parsing library error
//!
//! ## Warnings (logged but don't halt parsing)
//!
//! - [`AdtError::UnknownChunk`] - Unrecognized chunk encountered (skipped)
//! - [`AdtError::Utf8Error`] - Invalid UTF-8 in string data (lossy conversion applied)
//!
//! # Examples
//!
//! ```
//! use wow_adt::error::{AdtError, Result};
//! use wow_adt::ChunkId;
//!
//! fn validate_magic(magic: [u8; 4], expected: ChunkId, offset: u64) -> Result<()> {
//!     let found = ChunkId(magic);
//!     if found != expected {
//!         return Err(AdtError::InvalidMagic {
//!             expected,
//!             found,
//!             offset,
//!         });
//!     }
//!     Ok(())
//! }
//! ```
//!
//! # Error Context
//!
//! All errors include contextual information to aid debugging:
//!
//! - **File offsets** - Exact position in file where error occurred
//! - **Chunk context** - Which chunk was being parsed when error occurred
//! - **Expected vs actual** - What was expected and what was found
//! - **Index information** - Array indices for out-of-bounds references

use thiserror::Error;

use crate::ChunkId;

/// Result type alias using [`AdtError`] as the error type.
pub type Result<T> = std::result::Result<T, AdtError>;

/// Errors that can occur during ADT file parsing and validation.
///
/// This enum uses the fail-fast strategy: parsing halts at the first critical error.
/// This prevents cascading failures and ensures that subsequent operations don't work
/// with corrupted or incomplete data.
#[derive(Error, Debug)]
pub enum AdtError {
    /// Underlying I/O error occurred while reading ADT file.
    ///
    /// This is a critical error that halts parsing immediately.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Chunk magic bytes don't match expected format specification.
    ///
    /// This is a critical error indicating the file is corrupted or not a valid ADT.
    ///
    /// # Example
    ///
    /// ```text
    /// Expected MCNK chunk at offset 0x1000, but found MCLQ instead.
    /// ```
    #[error("Invalid magic bytes: expected {expected}, found {found} at offset {offset}")]
    InvalidMagic {
        /// Expected chunk identifier.
        expected: ChunkId,
        /// Actual chunk identifier found in file.
        found: ChunkId,
        /// File offset where invalid magic was encountered.
        offset: u64,
    },

    /// Required chunk is missing from ADT file.
    ///
    /// This is a critical error. ADT files must contain certain chunks (MVER, MHDR, MCIN, MCNK)
    /// to be considered valid.
    ///
    /// # Example
    ///
    /// ```text
    /// ADT file is missing required MHDR chunk - cannot parse terrain metadata.
    /// ```
    #[error("Missing required chunk: {0:?}")]
    MissingRequiredChunk(ChunkId),

    /// Mutually exclusive chunks are present in the same file.
    ///
    /// This is a critical error. Some chunks cannot coexist because they represent
    /// incompatible format versions or conflicting data structures.
    ///
    /// # Example
    ///
    /// ```text
    /// File contains both MCLQ (old water) and MH2O (new water) chunks.
    /// ```
    #[error("Invalid chunk combination: {chunk1:?} and {chunk2:?} cannot coexist")]
    InvalidChunkCombination {
        /// First conflicting chunk.
        chunk1: ChunkId,
        /// Second conflicting chunk.
        chunk2: ChunkId,
    },

    /// Chunk offset exceeds file boundaries.
    ///
    /// This is a critical error indicating corrupted offset data or truncated file.
    ///
    /// # Example
    ///
    /// ```text
    /// MCIN entry references MCNK at offset 0x50000, but file is only 0x40000 bytes.
    /// ```
    #[error(
        "Offset out of bounds for chunk {chunk:?}: offset {offset} at file position {file_position}"
    )]
    OffsetOutOfBounds {
        /// Chunk being accessed.
        chunk: ChunkId,
        /// Offset value that exceeds bounds.
        offset: u32,
        /// File position where invalid offset was read.
        file_position: u64,
    },

    /// Chunk size doesn't match format specification.
    ///
    /// This is a critical error. Fixed-size chunks must have exact expected sizes.
    ///
    /// # Example
    ///
    /// ```text
    /// MVER chunk must be exactly 4 bytes, found 8 bytes instead.
    /// ```
    #[error("Invalid chunk size for {chunk:?}: expected {expected}, got {actual}")]
    InvalidChunkSize {
        /// Chunk with invalid size.
        chunk: ChunkId,
        /// Expected size in bytes.
        expected: usize,
        /// Actual size in bytes.
        actual: usize,
    },

    /// Subchunk offset exceeds parent chunk boundaries.
    ///
    /// This is a critical error. Subchunk offsets must be relative to parent chunk start
    /// and cannot exceed parent chunk size.
    ///
    /// # Example
    ///
    /// ```text
    /// MCNK subchunk offset 0x2000 exceeds MCNK chunk size of 0x1FC0.
    /// ```
    #[error(
        "Invalid subchunk offset for {parent:?}: offset {offset} exceeds chunk size {chunk_size}"
    )]
    InvalidSubchunkOffset {
        /// Parent chunk containing the subchunk.
        parent: ChunkId,
        /// Subchunk offset that exceeds bounds.
        offset: u32,
        /// Total size of parent chunk.
        chunk_size: u32,
    },

    /// Water data structure is malformed or contains invalid data.
    ///
    /// This is a critical error for tiles with water features.
    ///
    /// # Example
    ///
    /// ```text
    /// MH2O chunk has water layers but missing height data.
    /// ```
    #[error("Invalid water structure: {0}")]
    InvalidWaterStructure(String),

    /// Texture index exceeds available texture count.
    ///
    /// This is a critical error. All texture references must be valid indices into
    /// the texture list defined in MTEX chunk.
    ///
    /// # Example
    ///
    /// ```text
    /// Layer references texture index 15, but only 10 textures are defined in MTEX.
    /// ```
    #[error("Invalid texture reference: index {index} exceeds texture count {count}")]
    InvalidTextureReference {
        /// Invalid texture index.
        index: u32,
        /// Total number of available textures.
        count: u32,
    },

    /// Model index exceeds available model count.
    ///
    /// This is a critical error. All model references must be valid indices into
    /// the model lists defined in MMDX/MMID or MWMO/MWID chunks.
    ///
    /// # Example
    ///
    /// ```text
    /// MCRF references M2 model index 50, but only 30 models defined in MMDX.
    /// ```
    #[error("Invalid model reference: index {index} exceeds model count {count}")]
    InvalidModelReference {
        /// Invalid model index.
        index: u32,
        /// Total number of available models.
        count: u32,
    },

    /// MCIN entry references non-existent MCNK chunk.
    ///
    /// This is a critical error. MCIN (chunk index) entries must reference valid MCNK chunks.
    /// ADT files should have exactly 256 MCNK chunks (16x16 grid).
    ///
    /// # Example
    ///
    /// ```text
    /// MCIN entry 100 has non-zero offset but corresponding MCNK chunk is missing.
    /// ```
    #[error("Invalid MCIN entry at index {index}: references non-existent MCNK")]
    InvalidMcinEntry {
        /// Index of invalid MCIN entry.
        index: usize,
    },

    /// Generic chunk parsing error with context.
    ///
    /// This is a critical error used when more specific error types don't apply.
    ///
    /// # Example
    ///
    /// ```text
    /// Failed to parse MCLY chunk at offset 0x3000: layer count mismatch.
    /// ```
    #[error("Chunk parse error for {chunk:?} at offset {offset}: {details}")]
    ChunkParseError {
        /// Chunk that failed to parse.
        chunk: ChunkId,
        /// File offset where error occurred.
        offset: u64,
        /// Detailed error description.
        details: String,
    },

    /// Binary parsing library error.
    ///
    /// This is a critical error from the underlying binrw library.
    #[error("binrw error: {0}")]
    BinrwError(String),

    /// UTF-8 conversion error encountered in string data.
    ///
    /// This is a WARNING, not a critical error. When invalid UTF-8 is encountered,
    /// lossy conversion is applied (invalid sequences replaced with �) and parsing continues.
    ///
    /// # Example
    ///
    /// ```text
    /// Texture filename contains invalid UTF-8 byte 0xFF, using lossy conversion.
    /// ```
    #[error("UTF-8 conversion error: {0} (using lossy conversion)")]
    Utf8Error(String),

    /// Unknown chunk encountered during parsing.
    ///
    /// This is a WARNING, not a critical error. Unknown chunks are skipped and parsing continues.
    /// This allows for forward compatibility with newer ADT format versions.
    ///
    /// # Example
    ///
    /// ```text
    /// Unknown chunk 'MXYZ' at offset 0x5000, skipping to next chunk.
    /// ```
    #[error("Unknown chunk encountered: {magic:?} at offset {offset} (skipping)")]
    UnknownChunk {
        /// Raw magic bytes of unknown chunk.
        magic: [u8; 4],
        /// File offset where unknown chunk was found.
        offset: u64,
    },

    /// Memory allocation exceeds safety limits.
    ///
    /// This is a critical error to prevent memory exhaustion attacks from malicious
    /// or corrupted files claiming enormous allocation sizes.
    ///
    /// # Example
    ///
    /// ```text
    /// Chunk claims to need 2GB allocation, exceeds 100MB limit.
    /// ```
    #[error("Memory limit exceeded: attempted to allocate {requested} bytes, limit is {limit}")]
    MemoryLimitExceeded {
        /// Number of bytes requested.
        requested: usize,
        /// Maximum allowed allocation size.
        limit: usize,
    },

    /// Cannot determine ADT format version.
    ///
    /// This is a critical error. Version detection is required to parse format-specific chunks.
    ///
    /// # Example
    ///
    /// ```text
    /// MVER chunk contains unknown version 19, expected 18 (WotLK).
    /// ```
    #[error("Version detection failed: {0}")]
    VersionDetectionFailed(String),
}

impl From<binrw::Error> for AdtError {
    fn from(err: binrw::Error) -> Self {
        AdtError::BinrwError(format!("{err}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_formats_correctly() {
        let err = AdtError::InvalidMagic {
            expected: ChunkId::MCNK,
            found: ChunkId::MCLQ,
            offset: 0x1000,
        };
        let display = format!("{err}");
        assert!(display.contains("MCNK"));
        assert!(display.contains("MCLQ"));
        assert!(display.contains("4096"));
    }

    #[test]
    fn error_context_preserved() {
        let err = AdtError::InvalidTextureReference {
            index: 15,
            count: 10,
        };
        assert!(matches!(err, AdtError::InvalidTextureReference { .. }));
    }

    #[test]
    fn io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let adt_err: AdtError = io_err.into();
        assert!(matches!(adt_err, AdtError::Io(_)));
    }

    #[test]
    fn binrw_error_conversion() {
        let binrw_err = binrw::Error::AssertFail {
            pos: 0x100,
            message: "test assertion failed".into(),
        };
        let adt_err: AdtError = binrw_err.into();
        assert!(matches!(adt_err, AdtError::BinrwError(_)));
    }
}
