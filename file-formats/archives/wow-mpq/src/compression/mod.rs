//! Compression and decompression algorithms for MPQ files

mod algorithms;
mod compress;
mod decompress;
mod methods;

// Re-export the main public API
pub use compress::compress;
pub use decompress::{decompress, decompress_secure};
pub use methods::{CompressionMethod, flags};

// Re-export security types for public use
pub use crate::security::{DecompressionMonitor, SecurityLimits, SessionTracker};

/// Error conversion helpers for compression algorithms
pub(crate) mod error_helpers {
    use crate::Error;

    /// Convert a compression-related error to an MPQ Error with algorithm context
    pub(crate) fn compression_error(algorithm: &str, err: impl std::fmt::Display) -> Error {
        Error::compression(format!("{algorithm} compression failed: {err}"))
    }

    /// Convert a decompression-related error to an MPQ Error with algorithm context  
    pub(crate) fn decompression_error(algorithm: &str, err: impl std::fmt::Display) -> Error {
        Error::compression(format!("{algorithm} decompression failed: {err}"))
    }
}
