use ::image::error::ImageError;
use thiserror::Error;

/// Errors that can occur during BLP conversion operations
#[derive(Debug, Error)]
pub enum Error {
    /// The requested mipmap level does not exist in the BLP file
    #[error("There is no image in the BLP mipmaps level {0}!")]
    MissingImage(usize),
    /// Error during image conversion operations
    #[error("Convertation error: {0}")]
    Convert(#[from] ImageError),
    /// Image width exceeds the maximum supported value of 65,535 pixels
    #[error("Maximum value for width is 65,535")]
    WidthTooLarge(u32),
    /// Image height exceeds the maximum supported value of 65,535 pixels
    #[error("Maximum value for height is 65,535")]
    HeightTooLarge(u32),
    /// Mismatch between header-declared size and actual pixel data size
    #[error(
        "Header sizes for mipmap {0} are {1}x{2}, but there are {3} pixels actually in content."
    )]
    MismatchSizes(usize, u32, u32, usize),
    /// Mismatch between expected and actual alpha channel data size
    #[error(
        "Header sizes for mipmap {0} are {1}x{2}, but there are {3} alpha values actually in content."
    )]
    MismatchAlphaSizes(usize, u32, u32, usize),
    /// Invalid alpha bit depth for Raw1 format (only 0, 1, 4, or 8 are valid)
    #[error("There are invalid alpha bits for the Raw1 format. Got {0}, expected: 0, 1, 4, 8.")]
    Raw1InvalidAlphaBits(u32),
    /// Color map does not contain exactly 256 entries as required
    #[error("Color map length {0}, 256 expected!")]
    ColorMapLengthInvalid(usize),
    /// Palette size mismatch (expected 256 colors)
    #[error("Expected palette of 256 colors, but got {0}")]
    PaletteWrongSize(usize),
    /// Failed to convert decompressed DXT1 data to raw format
    #[error("Failed to process bytes from DXT1 decomporession")]
    Dxt1RawConvertFail,
}
