use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Write};

use crate::chunks::animation::{M2AnimationBlock, M2AnimationTrack};
use crate::common::C3Vector;
use crate::error::Result;
use crate::version::M2Version;

/// Transform type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M2TextureTransformType {
    /// No transformation
    None = 0,
    /// Texture scrolling
    Scroll = 1,
    /// Texture rotation
    Rotate = 2,
    /// Texture scaling
    Scale = 3,
    /// Texture matrix transformation
    Matrix = 4,
}

impl M2TextureTransformType {
    /// Parse from integer value
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Scroll),
            2 => Some(Self::Rotate),
            3 => Some(Self::Scale),
            4 => Some(Self::Matrix),
            _ => None,
        }
    }
}

/// Represents a texture transform in an M2 model
/// Introduced in Legion (7.x)
#[derive(Debug, Clone)]
pub struct M2TextureTransform {
    /// Transform ID
    pub id: u32,
    /// Transform type
    pub transform_type: M2TextureTransformType,
    /// Translation animation (for scroll type)
    pub translation: M2AnimationBlock<C3Vector>,
    /// Rotation animation (for rotate type)
    pub rotation: M2AnimationBlock<C4Quaternion>,
    /// Scaling animation (for scale type)
    pub scaling: M2AnimationBlock<C3Vector>,
}

/// A quaternion for rotations and texture transforms
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct C4Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl C4Quaternion {
    /// Parse a quaternion from a reader
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let x = reader.read_f32_le()?;
        let y = reader.read_f32_le()?;
        let z = reader.read_f32_le()?;
        let w = reader.read_f32_le()?;

        Ok(Self { x, y, z, w })
    }

    /// Write a quaternion to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_f32_le(self.x)?;
        writer.write_f32_le(self.y)?;
        writer.write_f32_le(self.z)?;
        writer.write_f32_le(self.w)?;

        Ok(())
    }
}

impl M2TextureTransform {
    /// Parse a texture transform from a reader
    pub fn parse<R: Read>(reader: &mut R, version: u32) -> Result<Self> {
        let id = reader.read_u32_le()?;

        let transform_type_raw = reader.read_u16_le()?;
        let transform_type = M2TextureTransformType::from_u16(transform_type_raw)
            .unwrap_or(M2TextureTransformType::None);

        // Skip 2 bytes of padding
        reader.read_u16_le()?;

        let translation = M2AnimationBlock::parse(reader, version)?;
        let rotation = M2AnimationBlock::parse(reader, version)?;
        let scaling = M2AnimationBlock::parse(reader, version)?;

        Ok(Self {
            id,
            transform_type,
            translation,
            rotation,
            scaling,
        })
    }

    /// Write a texture transform to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32_le(self.id)?;
        writer.write_u16_le(self.transform_type as u16)?;

        // Write 2 bytes of padding
        writer.write_u16_le(0)?;

        self.translation.write(writer)?;
        self.rotation.write(writer)?;
        self.scaling.write(writer)?;

        Ok(())
    }

    /// Convert this texture transform to a different version
    pub fn convert(&self, _target_version: M2Version) -> Self {
        // No version-specific differences yet
        self.clone()
    }

    /// Create a new texture transform with default values
    pub fn new(id: u32, transform_type: M2TextureTransformType) -> Self {
        Self {
            id,
            transform_type,
            translation: M2AnimationBlock::new(M2AnimationTrack::new()),
            rotation: M2AnimationBlock::new(M2AnimationTrack::new()),
            scaling: M2AnimationBlock::new(M2AnimationTrack::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_c4quaternion_parse_write() {
        let quat = C4Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        };

        let mut data = Vec::new();
        quat.write(&mut data).unwrap();

        let mut cursor = Cursor::new(data);
        let parsed_quat = C4Quaternion::parse(&mut cursor).unwrap();

        assert_eq!(parsed_quat.x, 0.0);
        assert_eq!(parsed_quat.y, 0.0);
        assert_eq!(parsed_quat.z, 0.0);
        assert_eq!(parsed_quat.w, 1.0);
    }

    #[test]
    fn test_texture_transform_type() {
        assert_eq!(
            M2TextureTransformType::from_u16(0),
            Some(M2TextureTransformType::None)
        );
        assert_eq!(
            M2TextureTransformType::from_u16(1),
            Some(M2TextureTransformType::Scroll)
        );
        assert_eq!(
            M2TextureTransformType::from_u16(2),
            Some(M2TextureTransformType::Rotate)
        );
        assert_eq!(
            M2TextureTransformType::from_u16(3),
            Some(M2TextureTransformType::Scale)
        );
        assert_eq!(
            M2TextureTransformType::from_u16(4),
            Some(M2TextureTransformType::Matrix)
        );
        assert_eq!(M2TextureTransformType::from_u16(5), None);
    }
}
