use super::super::{
    header::{BlpHeader, BlpVersion},
    locator::MipmapLocator,
};

/// Raw BGRA format used in BLP2
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlpRaw3 {
    /// The cmap field array is the colour look up table used for an indexed
    /// colour model. Each element represents 24 bit RGB colour component values
    /// in the order of 0xBBGGRR. The final byte is alignment padding and will
    /// not alter the decoded image in any way. One might be able to improve the
    /// file compressibility by carefully choosing padding values.
    pub cmap: Vec<u32>,
    /// Image itself and all mipmaps levels. If there are no mipmaps,
    /// the length of the vector is 1.
    pub images: Vec<Raw3Image>,
}

impl BlpRaw3 {
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

/// Each mipmap contains what appears to be
/// 32 bit BGRA data. `alpha_bits` seems to represent a set of bit flags
/// rather than depth, as all images of this type seem to have 4 bytes per
/// pixel regardless of depth, and it has been seen to exceed 8. Their
/// meaning is unknown.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Raw3Image {
    /// BGRA pixel data (4 bytes per pixel)
    pub pixels: Vec<u32>,
}

impl Raw3Image {
    /// Get size in bytes of serialized image
    pub fn len(&self) -> usize {
        self.pixels.len() * 4
    }

    /// Check if the image has no pixels
    pub fn is_empty(&self) -> bool {
        self.pixels.is_empty()
    }
}
