use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Write};

use crate::chunks::animation::{M2AnimationBlock, M2AnimationTrack};
use crate::common::C3Vector;
use crate::error::Result;
use crate::version::M2Version;

bitflags::bitflags! {
    /// Camera flags as defined in the M2 format
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct M2CameraFlags: u16 {
        /// Camera uses custom UVs for positioning
        const CUSTOM_UV = 0x01;
        /// Auto-generated camera based on model
        const AUTO_GENERATED = 0x02;
        /// Camera is at global scene coordinates
        const GLOBAL_POSITION = 0x04;
    }
}

/// Represents a camera in an M2 model
#[derive(Debug, Clone)]
pub struct M2Camera {
    /// Camera type
    pub camera_type: u32,
    /// Field of view (in radians)
    pub fov: f32,
    /// Far clip distance
    pub far_clip: f32,
    /// Near clip distance
    pub near_clip: f32,
    /// Camera position animation
    pub position_animation: M2AnimationBlock<C3Vector>,
    /// Target position animation
    pub target_position_animation: M2AnimationBlock<C3Vector>,
    /// Roll animation (rotation around the view axis)
    pub roll_animation: M2AnimationBlock<f32>,
    /// Camera ID
    pub id: u32,
    /// Camera flags
    pub flags: M2CameraFlags,
}

impl M2Camera {
    /// Parse a camera from a reader based on the M2 version
    pub fn parse<R: Read>(reader: &mut R, version: u32) -> Result<Self> {
        let camera_type = reader.read_u32_le()?;
        let fov = reader.read_f32_le()?;
        let far_clip = reader.read_f32_le()?;
        let near_clip = reader.read_f32_le()?;

        let position_animation = M2AnimationBlock::parse(reader, version)?;
        let target_position_animation = M2AnimationBlock::parse(reader, version)?;
        let roll_animation = M2AnimationBlock::parse(reader, version)?;

        let id = reader.read_u32_le()?;

        // 2 bytes for flags, 2 bytes of padding
        let flags = M2CameraFlags::from_bits_retain(reader.read_u16_le()?);
        reader.read_u16_le()?; // Skip padding

        Ok(Self {
            camera_type,
            fov,
            far_clip,
            near_clip,
            position_animation,
            target_position_animation,
            roll_animation,
            id,
            flags,
        })
    }

    /// Write a camera to a writer based on the M2 version
    pub fn write<W: Write>(&self, writer: &mut W, _version: u32) -> Result<()> {
        writer.write_u32_le(self.camera_type)?;
        writer.write_f32_le(self.fov)?;
        writer.write_f32_le(self.far_clip)?;
        writer.write_f32_le(self.near_clip)?;

        self.position_animation.write(writer)?;
        self.target_position_animation.write(writer)?;
        self.roll_animation.write(writer)?;

        writer.write_u32_le(self.id)?;

        // 2 bytes for flags, 2 bytes of padding
        writer.write_u16_le(self.flags.bits())?;
        writer.write_u16_le(0)?; // Write padding

        Ok(())
    }

    /// Convert this camera to a different version (no version differences for cameras)
    pub fn convert(&self, _target_version: M2Version) -> Self {
        self.clone()
    }

    /// Create a new camera with default values
    pub fn new(id: u32) -> Self {
        Self {
            camera_type: 0,
            fov: 0.8726646, // 50 degrees in radians
            far_clip: 100.0,
            near_clip: 0.1,
            position_animation: M2AnimationBlock::new(M2AnimationTrack::new()),
            target_position_animation: M2AnimationBlock::new(M2AnimationTrack::new()),
            roll_animation: M2AnimationBlock::new(M2AnimationTrack::new()),
            id,
            flags: M2CameraFlags::empty(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_camera_parse_write() {
        let camera = M2Camera::new(1);

        // Test write
        let mut data = Vec::new();
        camera
            .write(&mut data, M2Version::Classic.to_header_version())
            .unwrap();

        // Test parse
        let mut cursor = Cursor::new(data);
        let parsed = M2Camera::parse(&mut cursor, M2Version::Classic.to_header_version()).unwrap();

        assert_eq!(parsed.camera_type, 0);
        assert_eq!(parsed.id, 1);
        assert_eq!(parsed.flags, M2CameraFlags::empty());
    }

    #[test]
    fn test_camera_flags() {
        let flags = M2CameraFlags::CUSTOM_UV | M2CameraFlags::AUTO_GENERATED;
        assert!(flags.contains(M2CameraFlags::CUSTOM_UV));
        assert!(flags.contains(M2CameraFlags::AUTO_GENERATED));
        assert!(!flags.contains(M2CameraFlags::GLOBAL_POSITION));
    }
}
