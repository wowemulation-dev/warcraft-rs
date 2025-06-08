//! Parser for World of Warcraft BLP (texture) files.
//!
//! This crate provides support for reading and writing BLP texture files used in
//! World of Warcraft. BLP is Blizzard's proprietary texture format that supports
//! various compression methods including JPEG compression and palettized images.
//!
//! # Status
//!
//! 🚧 **Under Construction** - This crate is not yet fully implemented.
//!
//! # Examples
//!
//! ```no_run
//! use wow_blp::BlpTexture;
//!
//! // This is a placeholder example for future implementation
//! let texture = BlpTexture::placeholder();
//! println!("Texture format: {:?}", texture.format());
//! ```

#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use std::fmt;

/// Compression format used in BLP files
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlpCompression {
    /// JPEG compressed texture (BLP0)
    Jpeg,
    /// Palettized texture with optional alpha (BLP1)
    Palettized,
    /// DirectX compressed texture (BLP2)
    DirectX,
    /// Uncompressed BGRA texture
    Uncompressed,
}

/// A BLP texture file
#[derive(Debug)]
pub struct BlpTexture {
    /// Compression format
    compression: BlpCompression,
    /// Width in pixels
    width: u32,
    /// Height in pixels
    height: u32,
}

impl BlpTexture {
    /// Create a placeholder texture for demonstration
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_blp::BlpTexture;
    ///
    /// let texture = BlpTexture::placeholder();
    /// assert_eq!(texture.width(), 256);
    /// assert_eq!(texture.height(), 256);
    /// ```
    pub fn placeholder() -> Self {
        Self {
            compression: BlpCompression::Jpeg,
            width: 256,
            height: 256,
        }
    }

    /// Get the compression format
    pub fn format(&self) -> BlpCompression {
        self.compression
    }

    /// Get the texture width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the texture height
    pub fn height(&self) -> u32 {
        self.height
    }
}

impl fmt::Display for BlpTexture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BLP Texture ({}x{}, {:?})",
            self.width, self.height, self.compression
        )
    }
}

/// Error type for BLP operations
#[derive(Debug, thiserror::Error)]
pub enum BlpError {
    /// Invalid BLP file format
    #[error("Invalid BLP format: {0}")]
    InvalidFormat(String),

    /// Unsupported compression type
    #[error("Unsupported compression: {0}")]
    UnsupportedCompression(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for BLP operations
pub type Result<T> = std::result::Result<T, BlpError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        let texture = BlpTexture::placeholder();
        assert_eq!(texture.width(), 256);
        assert_eq!(texture.height(), 256);
        assert_eq!(texture.format(), BlpCompression::Jpeg);
    }

    #[test]
    fn test_display() {
        let texture = BlpTexture::placeholder();
        let display = format!("{}", texture);
        assert!(display.contains("256x256"));
        assert!(display.contains("Jpeg"));
    }
}
