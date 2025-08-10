use crate::M2Error;
use std::io::{Read, Seek, Write};
use wow_data::prelude::*;
use wow_data::types::WowCharArray;
use wow_data::{error::Result as WDResult, types::WowString};
use wow_data_derive::{WowHeaderR, WowHeaderW};

use crate::error::Result;

/// Texture type enum as defined in the M2 format
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum M2TextureType {
    /// Regular texture
    #[default]
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
    /// Accessories
    Accessories = 8,
    /// Custom type, not used
    Custom1 = 9,
    /// Custom type, not used
    Custom2 = 10,
    /// Custom type, not used
    Custom3 = 11,
}

impl TryFrom<u32> for M2TextureType {
    type Error = M2Error;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            0 => Ok(Self::Hardcoded),
            1 => Ok(Self::Body),
            2 => Ok(Self::Item),
            3 => Ok(Self::WeaponArmorBasic),
            4 => Ok(Self::WeaponBlade),
            5 => Ok(Self::WeaponHandle),
            6 => Ok(Self::Environment),
            7 => Ok(Self::Hair),
            8 => Ok(Self::Accessories),
            9 => Ok(Self::Custom1),
            10 => Ok(Self::Custom2),
            11 => Ok(Self::Custom3),
            _ => Err(M2Error::UnsupportedNumericVersion(value)),
        }
    }
}

impl From<M2TextureType> for u32 {
    fn from(value: M2TextureType) -> Self {
        match value {
            M2TextureType::Hardcoded => 0,
            M2TextureType::Body => 1,
            M2TextureType::Item => 2,
            M2TextureType::WeaponArmorBasic => 3,
            M2TextureType::WeaponBlade => 4,
            M2TextureType::WeaponHandle => 5,
            M2TextureType::Environment => 6,
            M2TextureType::Hair => 7,
            M2TextureType::Accessories => 8,
            M2TextureType::Custom1 => 9,
            M2TextureType::Custom2 => 10,
            M2TextureType::Custom3 => 11,
        }
    }
}

impl WowHeaderR for M2TextureType {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        let value: u32 = reader.wow_read()?;
        Ok(value.try_into()?)
    }
}
impl WowHeaderW for M2TextureType {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        let value: u32 = (*self).into();
        writer.wow_write(&value)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        4
    }
}

bitflags::bitflags! {
    /// Texture flags as defined in the M2 format
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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

impl WowHeaderR for M2TextureFlags {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        Ok(Self::from_bits_retain(reader.wow_read()?))
    }
}
impl WowHeaderW for M2TextureFlags {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        writer.wow_write(&self.bits())?;
        Ok(())
    }
    fn wow_size(&self) -> usize {
        4
    }
}

/// Represents a texture in an M2 model
#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
pub struct M2TextureHeader {
    /// Type of the texture
    pub texture_type: M2TextureType,
    /// Flags for this texture
    pub flags: M2TextureFlags,
    /// Filename of the texture
    pub filename: WowCharArray,
}

impl M2TextureHeader {
    /// Create a new texture with the given type and filename
    pub fn new(texture_type: M2TextureType, filename: WowCharArray) -> Self {
        Self {
            texture_type,
            flags: M2TextureFlags::empty(),
            filename,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct M2TextureData {
    pub filename: String,
}

impl WowDataR<M2TextureHeader> for M2TextureData {
    fn new_from_header<R: Read + Seek>(reader: &mut R, header: &M2TextureHeader) -> WDResult<Self> {
        Ok(Self {
            filename: String::from_wow_char_array(reader, header.filename.clone())?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct M2Texture {
    pub header: M2TextureHeader,
    pub data: M2TextureData,
}

// #[cfg(test)]
// mod tests {
//     use crate::common::{FixedString, M2Array};
//
//     use super::*;
//     use std::io::{Cursor, SeekFrom};
//
//     #[test]
//     fn test_texture_parse() {
//         let mut data = Vec::new();
//
//         let dummy = [0, 0, 0];
//         data.extend_from_slice(&dummy);
//
//         let filename_str = "test\0";
//         data.extend_from_slice(filename_str.as_bytes());
//
//         // Texture type (Body)
//         data.extend_from_slice(&1u32.to_le_bytes());
//
//         // Flags (WRAP_X | WRAP_Y)
//         data.extend_from_slice(&3u32.to_le_bytes());
//
//         data.extend_from_slice(&(filename_str.len() as u32).to_le_bytes());
//         data.extend_from_slice(&(dummy.len() as u32).to_le_bytes());
//
//         let mut cursor = Cursor::new(data);
//         cursor
//             .seek(SeekFrom::Start((filename_str.len() + dummy.len()) as u64))
//             .unwrap();
//         let texture =
//             M2Texture::parse(&mut cursor, M2Version::Classic.to_header_version()).unwrap();
//
//         assert_eq!(texture.texture_type, M2TextureType::Body);
//         assert_eq!(
//             texture.flags,
//             M2TextureFlags::WRAP_X | M2TextureFlags::WRAP_Y
//         );
//         assert_eq!(texture.filename.array.count, 5);
//         assert_eq!(texture.filename.array.offset, 3);
//     }
//
//     #[test]
//     fn test_texture_write() {
//         let texture = M2Texture {
//             texture_type: M2TextureType::Body,
//             flags: M2TextureFlags::WRAP_X | M2TextureFlags::WRAP_Y,
//             filename: M2ArrayString {
//                 string: FixedString { data: Vec::new() },
//                 array: M2Array::new(10, 0x100),
//             },
//         };
//
//         let mut data = Vec::new();
//         texture.write(&mut data).unwrap();
//
//         assert_eq!(
//             data,
//             [
//                 // Texture type (Body)
//                 1, 0, 0, 0, // Flags (WRAP_X | WRAP_Y)
//                 3, 0, 0, 0, // Filename
//                 10, 0, 0, 0, // count = 10
//                 0, 1, 0, 0, // offset = 0x100
//             ]
//         );
//     }
//
//     #[test]
//     fn test_texture_conversion() {
//         let texture = M2Texture {
//             texture_type: M2TextureType::Body,
//             flags: M2TextureFlags::WRAP_X | M2TextureFlags::WRAP_Y,
//             filename: M2ArrayString {
//                 string: FixedString { data: Vec::new() },
//                 array: M2Array::new(10, 0x100),
//             },
//         };
//
//         // Convert to Cataclysm (should be identical since there are no version differences)
//         let converted = texture.convert(M2Version::Cataclysm);
//
//         assert_eq!(converted.texture_type, texture.texture_type);
//         assert_eq!(converted.flags, texture.flags);
//         assert_eq!(converted.filename.array.count, texture.filename.array.count);
//         assert_eq!(
//             converted.filename.array.offset,
//             texture.filename.array.offset
//         );
//     }
// }
