use crate::types::BlpVersion;
use thiserror::Error;

/// Errors that can occur during BLP encoding operations
#[derive(Debug, Error)]
pub enum Error {
    /// Image width exceeds BLP format maximum of 65,535 pixels
    #[error("BLP supports width up to 65,535, the width: {0}")]
    WidthTooHigh(u32),
    /// Image height exceeds BLP format maximum of 65,535 pixels
    #[error("BLP supports height up to 65,535, the width: {0}")]
    HeightTooHigh(u32),
    /// The specified BLP version does not support external mipmap files
    #[error("External mipmaps are not supported for the version {0}")]
    ExternalMipmapsNotSupported(BlpVersion),
    /// Mipmap data offset is invalid or out of bounds
    #[error("Invalid offset {offset} for mipmap {mipmap}, filled bytes {filled}")]
    InvalidOffset {
        /// Index of the mipmap with invalid offset
        mipmap: usize,
        /// The invalid offset value
        offset: usize,
        /// Number of bytes already written
        filled: usize,
    },
    /// Mipmap size in header doesn't match actual data size
    #[error("Size of mipmap {mipmap} in header {in_header} doesn't match actual {actual}")]
    InvalidMipmapSize {
        /// Index of the mipmap with size mismatch
        mipmap: usize,
        /// Size declared in the header
        in_header: usize,
        /// Actual size of the mipmap data
        actual: usize,
    },
    /// Filesystem operation failed
    #[error("Failed to proceed {0}, due: {1}")]
    FileSystem(std::path::PathBuf, std::io::Error),
    /// Invalid or malformed file name for BLP file
    #[error("Name of root file is malformed: {0}")]
    FileNameInvalid(std::path::PathBuf),
}
