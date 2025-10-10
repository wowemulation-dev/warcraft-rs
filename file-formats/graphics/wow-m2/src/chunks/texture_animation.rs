use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Seek, Write};

use crate::chunks::animation::{M2AnimationBlock, M2AnimationTrack};
use crate::error::Result;
use crate::version::M2Version;

/// Texture animation type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M2TextureAnimationType {
    /// No animation
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

impl M2TextureAnimationType {
    /// Parse from integer value
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Scroll),
            2 => Some(Self::Rotate),
            3 => Some(Self::Scale),
            4 => Some(Self::KeyFrame),
            _ => None,
        }
    }
}

/// Texture animation structure
#[derive(Debug, Clone)]
pub struct M2TextureAnimation {
    /// Animation type
    pub animation_type: M2TextureAnimationType,
    /// Animation for U coordinate
    pub translation_u: M2AnimationBlock<f32>,
    /// Animation for V coordinate
    pub translation_v: M2AnimationBlock<f32>,
    /// Rotation animation
    pub rotation: M2AnimationBlock<f32>,
    /// Scale U animation
    pub scale_u: M2AnimationBlock<f32>,
    /// Scale V animation
    pub scale_v: M2AnimationBlock<f32>,
}

impl M2TextureAnimation {
    /// Parse a texture animation from a reader
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let type_raw = reader.read_u16_le()?;
        let animation_type =
            M2TextureAnimationType::from_u16(type_raw).unwrap_or(M2TextureAnimationType::None);

        // Skip 2 bytes of padding
        reader.read_u16_le()?;

        let translation_u = M2AnimationBlock::parse(reader)?;
        let translation_v = M2AnimationBlock::parse(reader)?;
        let rotation = M2AnimationBlock::parse(reader)?;
        let scale_u = M2AnimationBlock::parse(reader)?;
        let scale_v = M2AnimationBlock::parse(reader)?;

        Ok(Self {
            animation_type,
            translation_u,
            translation_v,
            rotation,
            scale_u,
            scale_v,
        })
    }

    /// Write a texture animation to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u16_le(self.animation_type as u16)?;

        // Write 2 bytes of padding
        writer.write_u16_le(0)?;

        self.translation_u.write(writer)?;
        self.translation_v.write(writer)?;
        self.rotation.write(writer)?;
        self.scale_u.write(writer)?;
        self.scale_v.write(writer)?;

        Ok(())
    }

    /// Convert this texture animation to a different version (no version differences yet)
    pub fn convert(&self, _target_version: M2Version) -> Self {
        self.clone()
    }

    /// Create a new texture animation with default values
    pub fn new(animation_type: M2TextureAnimationType) -> Self {
        Self {
            animation_type,
            translation_u: M2AnimationBlock::new(M2AnimationTrack::default()),
            translation_v: M2AnimationBlock::new(M2AnimationTrack::default()),
            rotation: M2AnimationBlock::new(M2AnimationTrack::default()),
            scale_u: M2AnimationBlock::new(M2AnimationTrack::default()),
            scale_v: M2AnimationBlock::new(M2AnimationTrack::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_texture_animation_parse_write() {
        let mut data = Vec::new();

        // Animation type (Scroll)
        data.extend_from_slice(&1u16.to_le_bytes());

        // Padding
        data.extend_from_slice(&0u16.to_le_bytes());

        // Translation U animation track
        data.extend_from_slice(&1u16.to_le_bytes()); // Interpolation type (Linear)
        data.extend_from_slice(&(-1i16).to_le_bytes()); // Global sequence
        data.extend_from_slice(&0u32.to_le_bytes()); // Interpolation ranges count
        data.extend_from_slice(&0u32.to_le_bytes()); // Interpolation ranges offset
        data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps count
        data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps offset
        data.extend_from_slice(&0u32.to_le_bytes()); // Values count
        data.extend_from_slice(&0u32.to_le_bytes()); // Values offset

        // Translation V animation track
        data.extend_from_slice(&1u16.to_le_bytes()); // Interpolation type (Linear)
        data.extend_from_slice(&(-1i16).to_le_bytes()); // Global sequence
        data.extend_from_slice(&0u32.to_le_bytes()); // Interpolation ranges count
        data.extend_from_slice(&0u32.to_le_bytes()); // Interpolation ranges offset
        data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps count
        data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps offset
        data.extend_from_slice(&0u32.to_le_bytes()); // Values count
        data.extend_from_slice(&0u32.to_le_bytes()); // Values offset

        // Rotation animation track
        data.extend_from_slice(&1u16.to_le_bytes()); // Interpolation type (Linear)
        data.extend_from_slice(&(-1i16).to_le_bytes()); // Global sequence
        data.extend_from_slice(&0u32.to_le_bytes()); // Interpolation ranges count
        data.extend_from_slice(&0u32.to_le_bytes()); // Interpolation ranges offset
        data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps count
        data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps offset
        data.extend_from_slice(&0u32.to_le_bytes()); // Values count
        data.extend_from_slice(&0u32.to_le_bytes()); // Values offset

        // Scale U animation track
        data.extend_from_slice(&1u16.to_le_bytes()); // Interpolation type (Linear)
        data.extend_from_slice(&(-1i16).to_le_bytes()); // Global sequence
        data.extend_from_slice(&0u32.to_le_bytes()); // Interpolation ranges count
        data.extend_from_slice(&0u32.to_le_bytes()); // Interpolation ranges offset
        data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps count
        data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps offset
        data.extend_from_slice(&0u32.to_le_bytes()); // Values count
        data.extend_from_slice(&0u32.to_le_bytes()); // Values offset

        // Scale V animation track
        data.extend_from_slice(&1u16.to_le_bytes()); // Interpolation type (Linear)
        data.extend_from_slice(&(-1i16).to_le_bytes()); // Global sequence
        data.extend_from_slice(&0u32.to_le_bytes()); // Interpolation ranges count
        data.extend_from_slice(&0u32.to_le_bytes()); // Interpolation ranges offset
        data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps count
        data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps offset
        data.extend_from_slice(&0u32.to_le_bytes()); // Values count
        data.extend_from_slice(&0u32.to_le_bytes()); // Values offset

        let mut cursor = Cursor::new(data);
        let tex_anim = M2TextureAnimation::parse(&mut cursor).unwrap();

        assert_eq!(tex_anim.animation_type, M2TextureAnimationType::Scroll);

        // Test write
        let mut output = Vec::new();
        tex_anim.write(&mut output).unwrap();

        // Check output size (should be the same as input)
        assert_eq!(output.len(), cursor.get_ref().len());
    }
}
