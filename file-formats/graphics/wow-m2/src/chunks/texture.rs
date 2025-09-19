use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Seek, Write};

use crate::common::M2ArrayString;
use crate::error::Result;
use crate::version::M2Version;

/// Texture type enum as defined in the M2 format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M2TextureType {
    /// Regular texture
    Hardcoded = 0,
    /// Body + clothes
    Body = 1,
    /// Item, capes
    Item = 2,
    /// Weapon, armor (armorless)
    WeaponArmorBasic = 3,
    /// Weapon blade
    WeaponBlade = 4,
    /// Weapon handle
    WeaponHandle = 5,
    /// Environment
    Environment = 6,
    /// Hair, beard
    Hair = 7,
    /// Skin extra (accessories)
    SkinExtra = 8,
    /// Inventory art
    UiSkin = 9,
    /// Tauren mane
    TaurenMane = 10,
    /// Monster skin 1
    Monster1 = 11,
    /// Monster skin 2
    Monster2 = 12,
    /// Monster skin 3
    Monster3 = 13,
    /// Item icon
    ItemIcon = 14,
    /// Unknown
    Unknown = 255,
}

impl M2TextureType {
    /// Parse from integer value
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Hardcoded),
            1 => Some(Self::Body),
            2 => Some(Self::Item),
            3 => Some(Self::WeaponArmorBasic),
            4 => Some(Self::WeaponBlade),
            5 => Some(Self::WeaponHandle),
            6 => Some(Self::Environment),
            7 => Some(Self::Hair),
            8 => Some(Self::SkinExtra),
            9 => Some(Self::UiSkin),
            10 => Some(Self::TaurenMane),
            11 => Some(Self::Monster1),
            12 => Some(Self::Monster2),
            13 => Some(Self::Monster3),
            14 => Some(Self::ItemIcon),
            _ => None,
        }
    }
}

bitflags::bitflags! {
    /// Texture flags as defined in the M2 format
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct M2TextureFlags: u32 {
        /// Texture is wrapped horizontally
        const WRAP_X = 0x01;
        /// Texture is wrapped vertically
        const WRAP_Y = 0x02;
        /// Texture will not be replaced by other textures
        /// (character customization texture replacement)
        const NOT_REPLACEABLE = 0x04;
    }
}

/// Represents a texture in an M2 model
#[derive(Debug, Clone)]
pub struct M2Texture {
    /// Type of the texture
    pub texture_type: M2TextureType,
    /// Flags for this texture
    pub flags: M2TextureFlags,
    /// Filename of the texture
    pub filename: M2ArrayString,
}

impl M2Texture {
    /// Parse a texture from a reader based on the M2 version
    pub fn parse<R: Read + Seek>(reader: &mut R, _version: u32) -> Result<Self> {
        let texture_type_raw = reader.read_u32_le()?;
        let texture_type =
            M2TextureType::from_u32(texture_type_raw).unwrap_or(M2TextureType::Unknown);

        let flags = M2TextureFlags::from_bits_retain(reader.read_u32_le()?);
        let filename = M2ArrayString::parse(reader)?;

        Ok(Self {
            texture_type,
            flags,
            filename,
        })
    }

    /// Write a texture to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32_le(self.texture_type as u32)?;
        writer.write_u32_le(self.flags.bits())?;
        self.filename.write(writer)?;

        Ok(())
    }

    /// Convert this texture to a different version (no version differences for textures)
    pub fn convert(&self, _target_version: M2Version) -> Self {
        self.clone()
    }

    /// Create a new texture with the given type and filename
    pub fn new(texture_type: M2TextureType, filename: M2ArrayString) -> Self {
        Self {
            texture_type,
            flags: M2TextureFlags::empty(),
            filename,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::common::{FixedString, M2Array};

    use super::*;
    use std::io::{Cursor, SeekFrom};

    #[test]
    fn test_texture_parse() {
        let mut data = Vec::new();

        let dummy = [0, 0, 0];
        data.extend_from_slice(&dummy);

        let filename_str = "test\0";
        data.extend_from_slice(filename_str.as_bytes());

        // Texture type (Body)
        data.extend_from_slice(&1u32.to_le_bytes());

        // Flags (WRAP_X | WRAP_Y)
        data.extend_from_slice(&3u32.to_le_bytes());

        data.extend_from_slice(&(filename_str.len() as u32).to_le_bytes());
        data.extend_from_slice(&(dummy.len() as u32).to_le_bytes());

        let mut cursor = Cursor::new(data);
        cursor
            .seek(SeekFrom::Start((filename_str.len() + dummy.len()) as u64))
            .unwrap();
        let texture =
            M2Texture::parse(&mut cursor, M2Version::Vanilla.to_header_version()).unwrap();

        assert_eq!(texture.texture_type, M2TextureType::Body);
        assert_eq!(
            texture.flags,
            M2TextureFlags::WRAP_X | M2TextureFlags::WRAP_Y
        );
        assert_eq!(texture.filename.array.count, 5);
        assert_eq!(texture.filename.array.offset, 3);
    }

    #[test]
    fn test_texture_write() {
        let texture = M2Texture {
            texture_type: M2TextureType::Body,
            flags: M2TextureFlags::WRAP_X | M2TextureFlags::WRAP_Y,
            filename: M2ArrayString {
                string: FixedString { data: Vec::new() },
                array: M2Array::new(10, 0x100),
            },
        };

        let mut data = Vec::new();
        texture.write(&mut data).unwrap();

        assert_eq!(
            data,
            [
                // Texture type (Body)
                1, 0, 0, 0, // Flags (WRAP_X | WRAP_Y)
                3, 0, 0, 0, // Filename
                10, 0, 0, 0, // count = 10
                0, 1, 0, 0, // offset = 0x100
            ]
        );
    }

    #[test]
    fn test_texture_conversion() {
        let texture = M2Texture {
            texture_type: M2TextureType::Body,
            flags: M2TextureFlags::WRAP_X | M2TextureFlags::WRAP_Y,
            filename: M2ArrayString {
                string: FixedString { data: Vec::new() },
                array: M2Array::new(10, 0x100),
            },
        };

        // Convert to Cataclysm (should be identical since there are no version differences)
        let converted = texture.convert(M2Version::Cataclysm);

        assert_eq!(converted.texture_type, texture.texture_type);
        assert_eq!(converted.flags, texture.flags);
        assert_eq!(converted.filename.array.count, texture.filename.array.count);
        assert_eq!(
            converted.filename.array.offset,
            texture.filename.array.offset
        );
    }
}
