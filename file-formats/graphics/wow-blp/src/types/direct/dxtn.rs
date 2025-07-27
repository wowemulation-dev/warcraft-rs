use super::super::{
    header::{BlpHeader, BlpVersion},
    locator::MipmapLocator,
};
use custom_debug::Debug;
use wow_utils::debug;

/// Which compression algorithm is used to compress the image
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DxtnFormat {
    /// DXT1 compression (BC1)
    Dxt1,
    /// DXT3 compression (BC2)
    Dxt3,
    /// DXT5 compression (BC3)
    Dxt5,
}

impl From<DxtnFormat> for texpresso::Format {
    fn from(v: DxtnFormat) -> texpresso::Format {
        match v {
            DxtnFormat::Dxt1 => texpresso::Format::Bc1,
            DxtnFormat::Dxt3 => texpresso::Format::Bc2,
            DxtnFormat::Dxt5 => texpresso::Format::Bc3,
        }
    }
}

impl DxtnFormat {
    /// Returns the block size in bytes for this DXT format
    pub fn block_size(&self) -> usize {
        match self {
            DxtnFormat::Dxt1 => 8,
            DxtnFormat::Dxt3 => 16,
            DxtnFormat::Dxt5 => 16,
        }
    }
}

/// DXT-compressed BLP image data
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlpDxtn {
    /// DXT compression format used
    pub format: DxtnFormat,
    /// Color map (palette) for DXT1 format
    #[debug(with = debug::trimmed_collection_fmt)]
    pub cmap: Vec<u32>,
    /// Mipmap levels
    pub images: Vec<DxtnImage>,
}

impl BlpDxtn {
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

/// Single mipmap level of DXT-compressed data
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DxtnImage {
    /// Raw DXT-compressed data
    #[debug(with = debug::trimmed_collection_fmt)]
    pub content: Vec<u8>,
}

impl DxtnImage {
    /// Get size in bytes of serialized image
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Check if the image has no data
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}
