use crate::types::{
    header::{BlpHeader, BlpVersion},
    locator::MipmapLocator,
};

/// Raw indexed color format used in BLP0 and BLP1
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlpRaw1 {
    /// The cmap field array is the colour look up table used for an indexed
    /// colour model. Each element represents 24 bit RGB colour component values
    /// in the order of 0xBBGGRR. The final byte is alignment padding and will
    /// not alter the decoded image in any way. One might be able to improve the
    /// file compressibility by carefully choosing padding values.
    pub cmap: Vec<u32>,
    /// Image itself and all mipmaps levels. If there are no mipmaps,
    /// the length of the vector is 1.
    pub images: Vec<Raw1Image>,
}

impl BlpRaw1 {
    /// Predict internal locator to write down mipmaps
    pub fn mipmap_locator(&self, version: BlpVersion) -> MipmapLocator {
        let mut offsets = [0; 16];
        let mut sizes = [0; 16];
        let mut cur_offset = BlpHeader::size(version) + self.cmap.len() * 4;
        for (i, image) in self.images.iter().take(16).enumerate() {
            offsets[i] = cur_offset as u32;
            sizes[i] = image.len() as u32;
            cur_offset += image.len();
        }

        MipmapLocator::Internal { offsets, sizes }
    }
}

/// Single mipmap level of indexed color data
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Raw1Image {
    /// BGR component values can be obtained by using indexedRGB values as an
    /// index in lutBGR. When producing such values using color matching be
    /// aware of the linear nature of the color space. For best results it is
    /// recommended that color matching be performed in sRGB or other perceptual
    /// color spaces.
    pub indexed_rgb: Vec<u8>,

    /// Alpha component can be obtained by breaking indexedAlpha into a bit
    /// field of alphaBits bit length fragments and then using the bit fragment
    /// as the alpha value for the pixel. The alpha pixel components are ordered
    /// from least significant to most significant bit with bytes following the
    /// same pixel order as indexedRGB. Since the alpha is to alphaBits
    /// precision it may need to be resampled to 8 bits be useful depending on
    /// the imaging framework used.
    ///
    /// Example of different alpha packing in a byte:
    ///
    /// ```text
    /// MSB <-> LSB where number indicates the sequential pixel the bits belong to
    /// ALPHA_8B -> 11111111
    /// ALPHA_4B -> 22221111
    /// ALPHA_1B -> 87654321
    /// ```
    pub indexed_alpha: Vec<u8>,
}

impl Raw1Image {
    /// Get size in bytes of serialized image
    pub fn len(&self) -> usize {
        self.indexed_rgb.len() + self.indexed_alpha.len()
    }

    /// Check if the image has no data
    pub fn is_empty(&self) -> bool {
        self.indexed_rgb.is_empty() && self.indexed_alpha.is_empty()
    }
}
