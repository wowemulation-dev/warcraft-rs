use custom_debug::Debug;
use wow_utils::debug;

use crate::io_ext::{ReadExt, WriteExt};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::error::{M2Error, Result};

/// Magic signature for BLP files ("BLP2")
pub const BLP2_MAGIC: [u8; 4] = *b"BLP2";

/// BLP compression type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlpCompressionType {
    /// JPEG compression (legacy, not used in modern clients)
    Jpeg = 0,
    /// Uncompressed paletted
    Uncompressed = 1,
    /// DXT compression
    Dxt = 2,
    /// Uncompressed ARGB
    UncompressedArgb = 3,
}

impl BlpCompressionType {
    /// Parse from integer value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Jpeg),
            1 => Some(Self::Uncompressed),
            2 => Some(Self::Dxt),
            3 => Some(Self::UncompressedArgb),
            _ => None,
        }
    }
}

/// BLP pixel format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlpPixelFormat {
    /// DXT1 compression (1-bit alpha)
    Dxt1 = 0,
    /// DXT3 compression (explicit alpha)
    Dxt3 = 1,
    /// DXT5 compression (interpolated alpha)
    Dxt5 = 2,
    /// ARGB 8888 (32-bit)
    Argb8888 = 3,
    /// RGB 888 (24-bit)
    Rgb888 = 4,
    /// ARGB 1555 (16-bit)
    Argb1555 = 5,
    /// ARGB 4444 (16-bit)
    Argb4444 = 6,
    /// RGB 565 (16-bit)
    Rgb565 = 7,
    /// DXT1 with 1-bit alpha
    DxtAlpha1 = 8,
    /// Index using alpha table
    AIndex8 = 9,
}

impl BlpPixelFormat {
    /// Parse from integer value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Dxt1),
            1 => Some(Self::Dxt3),
            2 => Some(Self::Dxt5),
            3 => Some(Self::Argb8888),
            4 => Some(Self::Rgb888),
            5 => Some(Self::Argb1555),
            6 => Some(Self::Argb4444),
            7 => Some(Self::Rgb565),
            8 => Some(Self::DxtAlpha1),
            9 => Some(Self::AIndex8),
            _ => None,
        }
    }

    /// Get the bits per pixel for this format
    pub fn bits_per_pixel(&self) -> u32 {
        match self {
            Self::Dxt1 | Self::DxtAlpha1 => 4, // 4 bits per pixel
            Self::Dxt3 | Self::Dxt5 => 8,      // 8 bits per pixel
            Self::Argb8888 => 32,              // 32 bits per pixel
            Self::Rgb888 => 24,                // 24 bits per pixel
            Self::Argb1555 | Self::Argb4444 | Self::Rgb565 => 16, // 16 bits per pixel
            Self::AIndex8 => 8,                // 8 bits per pixel
        }
    }

    /// Check if this format is compressed
    pub fn is_compressed(&self) -> bool {
        matches!(self, Self::Dxt1 | Self::Dxt3 | Self::Dxt5 | Self::DxtAlpha1)
    }
}

/// BLP file header
#[derive(Debug, Clone)]
pub struct BlpHeader {
    /// Magic signature ("BLP2")
    pub magic: [u8; 4],
    /// Compression type
    pub compression_type: BlpCompressionType,
    /// Alpha channel bits (0, 1, 4, or 8)
    pub alpha_bits: u8,
    /// Width of the texture
    pub width: u16,
    /// Height of the texture
    pub height: u16,
    /// Pixel format
    pub pixel_format: BlpPixelFormat,
    /// Mipmap levels
    pub mipmap_levels: u8,
}

impl BlpHeader {
    /// Parse a BLP header from a reader
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        // Read and check magic
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;

        if magic != BLP2_MAGIC {
            return Err(M2Error::InvalidMagic {
                expected: String::from_utf8_lossy(&BLP2_MAGIC).to_string(),
                actual: String::from_utf8_lossy(&magic).to_string(),
            });
        }

        // Read type
        let compression_type_raw = reader.read_u8()?;
        let compression_type =
            BlpCompressionType::from_u8(compression_type_raw).ok_or_else(|| {
                M2Error::ParseError(format!(
                    "Invalid BLP compression type: {compression_type_raw}"
                ))
            })?;

        // Read alpha bits
        let alpha_bits = reader.read_u8()?;

        // Read dimensions
        let width = reader.read_u16_le()?;
        let height = reader.read_u16_le()?;

        // Read pixel format
        let pixel_format_raw = reader.read_u8()?;
        let pixel_format = BlpPixelFormat::from_u8(pixel_format_raw).ok_or_else(|| {
            M2Error::ParseError(format!("Invalid BLP pixel format: {pixel_format_raw}"))
        })?;

        // Read mipmap levels
        let mipmap_levels = reader.read_u8()?;

        Ok(Self {
            magic,
            compression_type,
            alpha_bits,
            width,
            height,
            pixel_format,
            mipmap_levels,
        })
    }

    /// Write a BLP header to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Write magic
        writer.write_all(&self.magic)?;

        // Write compression type
        writer.write_u8(self.compression_type as u8)?;

        // Write alpha bits
        writer.write_u8(self.alpha_bits)?;

        // Write dimensions
        writer.write_u16_le(self.width)?;
        writer.write_u16_le(self.height)?;

        // Write pixel format
        writer.write_u8(self.pixel_format as u8)?;

        // Write mipmap levels
        writer.write_u8(self.mipmap_levels)?;

        Ok(())
    }
}

/// BLP mipmap level
#[derive(Debug, Clone)]
pub struct BlpMipmap {
    /// Mipmap data
    #[debug(with = debug::trimmed_collection_fmt)]
    pub data: Vec<u8>,
    /// Width of the mipmap
    pub width: u32,
    /// Height of the mipmap
    pub height: u32,
}

/// Represents a BLP texture file
#[derive(Debug, Clone)]
pub struct BlpTexture {
    /// BLP header
    pub header: BlpHeader,
    /// Mipmap data
    pub mipmaps: Vec<BlpMipmap>,
    /// Color palette (for paletted formats)
    #[debug(with = debug::option_trimmed_collection_fmt)]
    pub palette: Option<Vec<u32>>,
}

impl BlpTexture {
    /// Parse a BLP texture from a reader
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Parse header
        let header = BlpHeader::parse(reader)?;

        // Read mipmap offsets and sizes
        let mut mipmap_offsets = Vec::with_capacity(header.mipmap_levels as usize);
        let mut mipmap_sizes = Vec::with_capacity(header.mipmap_levels as usize);

        for _ in 0..header.mipmap_levels {
            mipmap_offsets.push(reader.read_u32_le()?);
        }

        // Skip 4 unused offsets
        reader.seek(SeekFrom::Current(4 * 4))?;

        for _ in 0..header.mipmap_levels {
            mipmap_sizes.push(reader.read_u32_le()?);
        }

        // Skip 4 unused sizes
        reader.seek(SeekFrom::Current(4 * 4))?;

        // Read palette for paletted formats
        let palette = if header.compression_type == BlpCompressionType::Uncompressed {
            let mut palette_data = Vec::with_capacity(256);
            for _ in 0..256 {
                palette_data.push(reader.read_u32_le()?);
            }
            Some(palette_data)
        } else {
            None
        };

        // Read mipmaps
        let mut mipmaps = Vec::with_capacity(header.mipmap_levels as usize);
        let width = header.width as u32;
        let height = header.height as u32;

        for i in 0..header.mipmap_levels as usize {
            let offset = mipmap_offsets[i];
            let size = mipmap_sizes[i];

            if offset > 0 && size > 0 {
                reader.seek(SeekFrom::Start(offset as u64))?;

                let mipmap_width = (width >> i as u32).max(1);
                let mipmap_height = (height >> i as u32).max(1);

                let mut data = vec![0u8; size as usize];
                reader.read_exact(&mut data)?;

                mipmaps.push(BlpMipmap {
                    data,
                    width: mipmap_width,
                    height: mipmap_height,
                });
            }
        }

        Ok(Self {
            header,
            mipmaps,
            palette,
        })
    }

    /// Load a BLP texture from a file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        Self::parse(&mut file)
    }

    /// Save a BLP texture to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut file = File::create(path)?;
        self.write(&mut file)
    }

    /// Write a BLP texture to a writer
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        // Write header
        self.header.write(writer)?;

        // Calculate offsets
        let header_size = 20; // Magic + compression + alpha + width + height + format + mipmaps
        let mipmap_info_size = 8 * 4 + 8 * 4; // 8 offsets + 8 sizes
        let palette_size = if self.palette.is_some() { 256 * 4 } else { 0 };

        let mut current_offset = header_size + mipmap_info_size + palette_size;
        let mut mipmap_offsets = Vec::with_capacity(self.mipmaps.len());

        for mipmap in &self.mipmaps {
            mipmap_offsets.push(current_offset);
            current_offset += mipmap.data.len() as u32;
        }

        // Write mipmap offsets
        for &offset in &mipmap_offsets {
            writer.write_u32_le(offset)?;
        }

        // Write unused offsets
        for _ in mipmap_offsets.len()..8 {
            writer.write_u32_le(0)?;
        }

        // Write mipmap sizes
        for mipmap in &self.mipmaps {
            writer.write_u32_le(mipmap.data.len() as u32)?;
        }

        // Write unused sizes
        for _ in self.mipmaps.len()..8 {
            writer.write_u32_le(0)?;
        }

        // Write palette
        if let Some(ref palette) = self.palette {
            for &color in palette {
                writer.write_u32_le(color)?;
            }
        }

        // Write mipmap data
        for mipmap in &self.mipmaps {
            writer.write_all(&mipmap.data)?;
        }

        Ok(())
    }

    /// Create a new BLP texture with default values
    pub fn new(width: u16, height: u16, pixel_format: BlpPixelFormat) -> Self {
        let header = BlpHeader {
            magic: BLP2_MAGIC,
            compression_type: match pixel_format {
                BlpPixelFormat::Dxt1
                | BlpPixelFormat::Dxt3
                | BlpPixelFormat::Dxt5
                | BlpPixelFormat::DxtAlpha1 => BlpCompressionType::Dxt,
                BlpPixelFormat::Argb8888
                | BlpPixelFormat::Rgb888
                | BlpPixelFormat::Argb1555
                | BlpPixelFormat::Argb4444
                | BlpPixelFormat::Rgb565 => BlpCompressionType::UncompressedArgb,
                BlpPixelFormat::AIndex8 => BlpCompressionType::Uncompressed,
            },
            alpha_bits: match pixel_format {
                BlpPixelFormat::Argb8888 => 8,
                BlpPixelFormat::Argb4444 => 4,
                BlpPixelFormat::Argb1555 | BlpPixelFormat::Dxt1 | BlpPixelFormat::DxtAlpha1 => 1,
                _ => 0,
            },
            width,
            height,
            pixel_format,
            mipmap_levels: 1,
        };

        Self {
            header,
            mipmaps: Vec::new(),
            palette: if pixel_format == BlpPixelFormat::AIndex8 {
                Some(vec![0; 256])
            } else {
                None
            },
        }
    }

    /// Get the main mipmap data (highest resolution)
    pub fn main_data(&self) -> Option<&[u8]> {
        if self.mipmaps.is_empty() {
            None
        } else {
            Some(&self.mipmaps[0].data)
        }
    }

    /// Calculate the size of a mipmap level
    pub fn calculate_mipmap_size(&self, level: usize) -> usize {
        let width = (self.header.width as usize >> level).max(1);
        let height = (self.header.height as usize >> level).max(1);

        match self.header.compression_type {
            BlpCompressionType::Dxt => {
                match self.header.pixel_format {
                    BlpPixelFormat::Dxt1 | BlpPixelFormat::DxtAlpha1 => {
                        // DXT1 uses 4 bits per pixel
                        width.div_ceil(4) * height.div_ceil(4) * 8
                    }
                    BlpPixelFormat::Dxt3 | BlpPixelFormat::Dxt5 => {
                        // DXT3/5 uses 8 bits per pixel
                        width.div_ceil(4) * height.div_ceil(4) * 16
                    }
                    _ => 0,
                }
            }
            BlpCompressionType::UncompressedArgb => {
                // Calculate based on bits per pixel
                let bits_per_pixel = self.header.pixel_format.bits_per_pixel() as usize;
                (width * height * bits_per_pixel) / 8
            }
            BlpCompressionType::Uncompressed => {
                // Paletted format uses 1 byte per pixel index
                width * height
            }
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_blp_header_parse_write() {
        let header = BlpHeader {
            magic: BLP2_MAGIC,
            compression_type: BlpCompressionType::Dxt,
            alpha_bits: 8,
            width: 256,
            height: 256,
            pixel_format: BlpPixelFormat::Dxt5,
            mipmap_levels: 8,
        };

        let mut data = Vec::new();
        header.write(&mut data).unwrap();

        let mut cursor = Cursor::new(data);
        let parsed_header = BlpHeader::parse(&mut cursor).unwrap();

        assert_eq!(parsed_header.magic, BLP2_MAGIC);
        assert_eq!(parsed_header.compression_type, BlpCompressionType::Dxt);
        assert_eq!(parsed_header.alpha_bits, 8);
        assert_eq!(parsed_header.width, 256);
        assert_eq!(parsed_header.height, 256);
        assert_eq!(parsed_header.pixel_format, BlpPixelFormat::Dxt5);
        assert_eq!(parsed_header.mipmap_levels, 8);
    }

    #[test]
    fn test_pixel_format_properties() {
        assert_eq!(BlpPixelFormat::Dxt1.bits_per_pixel(), 4);
        assert_eq!(BlpPixelFormat::Dxt5.bits_per_pixel(), 8);
        assert_eq!(BlpPixelFormat::Argb8888.bits_per_pixel(), 32);
        assert_eq!(BlpPixelFormat::Rgb888.bits_per_pixel(), 24);
        assert_eq!(BlpPixelFormat::Rgb565.bits_per_pixel(), 16);

        assert!(BlpPixelFormat::Dxt1.is_compressed());
        assert!(BlpPixelFormat::Dxt5.is_compressed());
        assert!(!BlpPixelFormat::Argb8888.is_compressed());
        assert!(!BlpPixelFormat::Rgb888.is_compressed());
    }
}
