use super::direct::*;
use super::header::*;
use super::jpeg::*;
pub use super::version::BlpVersion;

/// Maximum width that BLP image can have due limitation
/// of mipmaping storage.
pub const BLP_MAX_WIDTH: u32 = 65535;
/// Maximum height that BLP image can have due limitation
/// of mipmaping storage.
pub const BLP_MAX_HEIGHT: u32 = 65535;

/// Parsed information from BLP file. The structure of the type
/// strictly follows how the file is stored on the disk for
/// easy encoding/decoding and further transformations.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlpImage {
    /// File header containing metadata
    pub header: BlpHeader,
    /// Actual image data
    pub content: BlpContent,
}

impl BlpImage {
    /// Get total amount of images encoded in the content
    pub fn image_count(&self) -> usize {
        match &self.content {
            BlpContent::Dxt1(v) => v.images.len(),
            BlpContent::Dxt3(v) => v.images.len(),
            BlpContent::Dxt5(v) => v.images.len(),
            BlpContent::Raw1(v) => v.images.len(),
            BlpContent::Raw3(v) => v.images.len(),
            BlpContent::Jpeg(v) => v.images.len(),
        }
    }

    /// If the image is encoded jpeg, return the content
    pub fn content_jpeg(&self) -> Option<&BlpJpeg> {
        self.content.jpeg()
    }

    /// If the image is direct encoded with BLP1 format, return the content
    pub fn content_raw1(&self) -> Option<&BlpRaw1> {
        self.content.raw1()
    }

    /// If the image is direct encoded with raw3 BLP2 format, return the content
    pub fn content_raw3(&self) -> Option<&BlpRaw3> {
        self.content.raw3()
    }

    /// If the image is DXT1 encoded, return the content
    pub fn content_dxt1(&self) -> Option<&BlpDxtn> {
        self.content.dxt1()
    }

    /// If the image is DXT3 encoded, return the content
    pub fn content_dxt3(&self) -> Option<&BlpDxtn> {
        self.content.dxt3()
    }

    /// If the image is DXT5 encoded, return the content
    pub fn content_dxt5(&self) -> Option<&BlpDxtn> {
        self.content.dxt5()
    }

    /// Get the compression type used for this BLP image
    pub fn compression_type(&self) -> CompressionType {
        match &self.content {
            BlpContent::Jpeg(_) => CompressionType::Jpeg,
            BlpContent::Raw1(_) => CompressionType::Raw1,
            BlpContent::Raw3(_) => CompressionType::Raw3,
            BlpContent::Dxt1(_) => CompressionType::Dxt1,
            BlpContent::Dxt3(_) => CompressionType::Dxt3,
            BlpContent::Dxt5(_) => CompressionType::Dxt5,
        }
    }

    /// Get the alpha bit depth for this BLP image
    pub fn alpha_bit_depth(&self) -> u8 {
        self.header.alpha_bits() as u8
    }

    /// Find the best mipmap level for a target resolution.
    /// Returns the mipmap level closest to the target size.
    pub fn best_mipmap_for_size(&self, target_size: u32) -> usize {
        let image_count = self.image_count();
        if image_count == 0 {
            return 0;
        }

        let mut best_level = 0;
        let mut best_diff = u32::MAX;

        for level in 0..image_count {
            let (width, height) = self.header.mipmap_size(level);
            let size = width.max(height);
            let diff = if size >= target_size {
                size - target_size
            } else {
                target_size - size
            };

            if diff < best_diff {
                best_diff = diff;
                best_level = level;
            }
        }

        best_level
    }

    /// Get information about all mipmap levels
    pub fn mipmap_info(&self) -> Vec<MipMapInfo> {
        let mut info = Vec::new();

        for level in 0..self.image_count() {
            let (width, height) = self.header.mipmap_size(level);
            let data_size = match &self.content {
                BlpContent::Jpeg(jpeg) => jpeg.images.get(level).map(|img| img.len()).unwrap_or(0),
                BlpContent::Raw1(raw) => raw
                    .images
                    .get(level)
                    .map(|img| img.indexed_rgb.len() + img.indexed_alpha.len())
                    .unwrap_or(0),
                BlpContent::Raw3(raw) => raw
                    .images
                    .get(level)
                    .map(|img| img.pixels.len() * 4)
                    .unwrap_or(0),
                BlpContent::Dxt1(dxt) => dxt
                    .images
                    .get(level)
                    .map(|img| img.content.len())
                    .unwrap_or(0),
                BlpContent::Dxt3(dxt) => dxt
                    .images
                    .get(level)
                    .map(|img| img.content.len())
                    .unwrap_or(0),
                BlpContent::Dxt5(dxt) => dxt
                    .images
                    .get(level)
                    .map(|img| img.content.len())
                    .unwrap_or(0),
            };

            info.push(MipMapInfo {
                level,
                width,
                height,
                data_size,
                pixel_count: width * height,
            });
        }

        info
    }

    /// Get total file size estimation (excluding external mipmaps)
    pub fn estimated_file_size(&self) -> usize {
        let header_size = BlpHeader::size(self.header.version);
        let content_size = match &self.content {
            BlpContent::Jpeg(jpeg) => {
                jpeg.header.len() + jpeg.images.iter().map(|img| img.len()).sum::<usize>()
            }
            BlpContent::Raw1(raw) => {
                raw.cmap.len() * 4 + // palette size
                raw.images.iter().map(|img| {
                    img.indexed_rgb.len() + img.indexed_alpha.len()
                }).sum::<usize>()
            }
            BlpContent::Raw3(raw) => raw
                .images
                .iter()
                .map(|img| img.pixels.len() * 4)
                .sum::<usize>(),
            BlpContent::Dxt1(dxt) => dxt
                .images
                .iter()
                .map(|img| img.content.len())
                .sum::<usize>(),
            BlpContent::Dxt3(dxt) => dxt
                .images
                .iter()
                .map(|img| img.content.len())
                .sum::<usize>(),
            BlpContent::Dxt5(dxt) => dxt
                .images
                .iter()
                .map(|img| img.content.len())
                .sum::<usize>(),
        };

        header_size + content_size
    }

    /// Get compression efficiency (uncompressed size vs compressed size)
    pub fn compression_ratio(&self) -> f32 {
        let uncompressed_size = self
            .mipmap_info()
            .iter()
            .map(|info| info.width * info.height * 4) // RGBA
            .sum::<u32>() as f32;

        let compressed_size = self.estimated_file_size() as f32;

        if compressed_size > 0.0 {
            uncompressed_size / compressed_size
        } else {
            1.0
        }
    }
}

/// Information about a single mipmap level
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MipMapInfo {
    /// Mipmap level (0 = original)
    pub level: usize,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Size of compressed data in bytes
    pub data_size: usize,
    /// Total pixel count
    pub pixel_count: u32,
}

/// Compression type enumeration for easy inspection
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompressionType {
    /// JPEG compressed image data
    Jpeg,
    /// Palettized 256-color format with alpha
    Raw1,
    /// Uncompressed RGBA format  
    Raw3,
    /// DXT1 compression (no alpha or 1-bit alpha)
    Dxt1,
    /// DXT3 compression (explicit alpha)
    Dxt3,
    /// DXT5 compression (interpolated alpha)
    Dxt5,
}

/// Collects all possible content types with actual data
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BlpContent {
    /// JPEG compressed image data
    Jpeg(BlpJpeg),
    /// Used with direct type for BLP0/BLP1 and raw compression in BLP2
    Raw1(BlpRaw1),
    /// Used with direct type for BLP2, encodes RGBA bitmap.
    Raw3(BlpRaw3),
    /// BLP2 DXT1 compression (no alpha)
    Dxt1(BlpDxtn),
    /// BLP2 DXT3 compression (with alpha)
    Dxt3(BlpDxtn),
    /// BLP2 DXT5 compression (with alpha)
    Dxt5(BlpDxtn),
}

impl BlpContent {
    /// Get the content tag type for this content
    pub fn tag(&self) -> BlpContentTag {
        match self {
            BlpContent::Jpeg { .. } => BlpContentTag::Jpeg,
            BlpContent::Raw1 { .. } => BlpContentTag::Direct,
            BlpContent::Raw3 { .. } => BlpContentTag::Direct,
            BlpContent::Dxt1 { .. } => BlpContentTag::Direct,
            BlpContent::Dxt3 { .. } => BlpContentTag::Direct,
            BlpContent::Dxt5 { .. } => BlpContentTag::Direct,
        }
    }

    /// Get JPEG content if this is JPEG encoded
    pub fn jpeg(&self) -> Option<&BlpJpeg> {
        match self {
            BlpContent::Jpeg(v) => Some(v),
            _ => None,
        }
    }

    /// Get RAW1 content if this is RAW1 encoded
    pub fn raw1(&self) -> Option<&BlpRaw1> {
        match self {
            BlpContent::Raw1(v) => Some(v),
            _ => None,
        }
    }

    /// Get RAW3 content if this is RAW3 encoded
    pub fn raw3(&self) -> Option<&BlpRaw3> {
        match self {
            BlpContent::Raw3(v) => Some(v),
            _ => None,
        }
    }

    /// Get DXT1 content if this is DXT1 encoded
    pub fn dxt1(&self) -> Option<&BlpDxtn> {
        match self {
            BlpContent::Dxt1(v) => Some(v),
            _ => None,
        }
    }

    /// Get DXT3 content if this is DXT3 encoded
    pub fn dxt3(&self) -> Option<&BlpDxtn> {
        match self {
            BlpContent::Dxt3(v) => Some(v),
            _ => None,
        }
    }

    /// Get DXT5 content if this is DXT5 encoded
    pub fn dxt5(&self) -> Option<&BlpDxtn> {
        match self {
            BlpContent::Dxt5(v) => Some(v),
            _ => None,
        }
    }
}
