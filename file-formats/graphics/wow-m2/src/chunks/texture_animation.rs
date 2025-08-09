use crate::M2Error;
use std::io::{Read, Seek, Write};
use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data_derive::{WowHeaderRV, WowHeaderW};

use crate::chunks::animation::M2AnimationTrackHeader;
use crate::error::Result;
use crate::version::M2Version;

/// Texture animation type enum
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum M2TextureAnimationType {
    /// No animation
    #[default]
    None = 0,
    /// Scroll animation
    Scroll = 1,
    /// Rotate animation
    Rotate = 2,
    /// Scale animation
    Scale = 3,
    /// Key frame animation
    KeyFrame = 4,
}

impl TryFrom<u16> for M2TextureAnimationType {
    type Error = M2Error;

    fn try_from(value: u16) -> Result<Self> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Scroll),
            2 => Ok(Self::Rotate),
            3 => Ok(Self::Scale),
            4 => Ok(Self::KeyFrame),
            _ => Err(M2Error::UnsupportedNumericVersion(value as u32)),
        }
    }
}

impl From<M2TextureAnimationType> for u16 {
    fn from(value: M2TextureAnimationType) -> Self {
        match value {
            M2TextureAnimationType::None => 0,
            M2TextureAnimationType::Scroll => 1,
            M2TextureAnimationType::Rotate => 2,
            M2TextureAnimationType::Scale => 3,
            M2TextureAnimationType::KeyFrame => 4,
        }
    }
}

impl WowHeaderR for M2TextureAnimationType {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        let value: u16 = reader.wow_read()?;
        Ok(value.try_into()?)
    }
}
impl WowHeaderW for M2TextureAnimationType {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        let value: u16 = (*self).into();
        writer.wow_write(&value)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        2
    }
}

/// Texture animation structure
#[derive(Debug, Clone, WowHeaderRV, WowHeaderW)]
#[wow_data(version = M2Version)]
pub struct M2TextureAnimation {
    /// Animation type
    pub animation_type: M2TextureAnimationType,
    /// Animation for U coordinate
    #[wow_data(versioned)]
    pub translation_u: M2AnimationTrackHeader<f32>,
    /// Animation for V coordinate
    #[wow_data(versioned)]
    pub translation_v: M2AnimationTrackHeader<f32>,
    /// Rotation animation
    #[wow_data(versioned)]
    pub rotation: M2AnimationTrackHeader<f32>,
    /// Scale U animation
    #[wow_data(versioned)]
    pub scale_u: M2AnimationTrackHeader<f32>,
    /// Scale V animation
    #[wow_data(versioned)]
    pub scale_v: M2AnimationTrackHeader<f32>,
}

impl M2TextureAnimation {
    /// Create a new texture animation with default values
    pub fn new(animation_type: M2TextureAnimationType) -> Self {
        Self {
            animation_type,
            translation_u: M2AnimationTrackHeader::new(),
            translation_v: M2AnimationTrackHeader::new(),
            rotation: M2AnimationTrackHeader::new(),
            scale_u: M2AnimationTrackHeader::new(),
            scale_v: M2AnimationTrackHeader::new(),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::io::Cursor;
//
//     #[test]
//     fn test_texture_animation_parse_write() {
//         let mut data = Vec::new();
//
//         // Animation type (Scroll)
//         data.extend_from_slice(&1u16.to_le_bytes());
//
//         // Padding
//         data.extend_from_slice(&0u16.to_le_bytes());
//
//         // Translation U animation track
//         data.extend_from_slice(&1u16.to_le_bytes()); // Interpolation type (Linear)
//         data.extend_from_slice(&(-1i16).to_le_bytes()); // Global sequence
//         data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps count
//         data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps offset
//         data.extend_from_slice(&0u32.to_le_bytes()); // Values count
//         data.extend_from_slice(&0u32.to_le_bytes()); // Values offset
//
//         // Translation V animation track
//         data.extend_from_slice(&1u16.to_le_bytes()); // Interpolation type (Linear)
//         data.extend_from_slice(&(-1i16).to_le_bytes()); // Global sequence
//         data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps count
//         data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps offset
//         data.extend_from_slice(&0u32.to_le_bytes()); // Values count
//         data.extend_from_slice(&0u32.to_le_bytes()); // Values offset
//
//         // Rotation animation track
//         data.extend_from_slice(&1u16.to_le_bytes()); // Interpolation type (Linear)
//         data.extend_from_slice(&(-1i16).to_le_bytes()); // Global sequence
//         data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps count
//         data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps offset
//         data.extend_from_slice(&0u32.to_le_bytes()); // Values count
//         data.extend_from_slice(&0u32.to_le_bytes()); // Values offset
//
//         // Scale U animation track
//         data.extend_from_slice(&1u16.to_le_bytes()); // Interpolation type (Linear)
//         data.extend_from_slice(&(-1i16).to_le_bytes()); // Global sequence
//         data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps count
//         data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps offset
//         data.extend_from_slice(&0u32.to_le_bytes()); // Values count
//         data.extend_from_slice(&0u32.to_le_bytes()); // Values offset
//
//         // Scale V animation track
//         data.extend_from_slice(&1u16.to_le_bytes()); // Interpolation type (Linear)
//         data.extend_from_slice(&(-1i16).to_le_bytes()); // Global sequence
//         data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps count
//         data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps offset
//         data.extend_from_slice(&0u32.to_le_bytes()); // Values count
//         data.extend_from_slice(&0u32.to_le_bytes()); // Values offset
//
//         let mut cursor = Cursor::new(data);
//         let tex_anim = M2TextureAnimation::parse(&mut cursor, 264).unwrap();
//
//         assert_eq!(tex_anim.animation_type, M2TextureAnimationType::Scroll);
//
//         // Test write
//         let mut output = Vec::new();
//         tex_anim.write(&mut output).unwrap();
//
//         // Check output size (should be the same as input)
//         assert_eq!(output.len(), cursor.get_ref().len());
//     }
// }
