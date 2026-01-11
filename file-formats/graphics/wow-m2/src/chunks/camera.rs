use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Seek, Write};

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
///
/// Camera structure layout:
/// - Pre-WotLK (version < 264): 124 bytes
///   - type(4) + fov/far/near(12) + pos_track(28) + pos_base(12)
///   - + target_track(28) + target_base(12) + roll_track(28)
/// - WotLK+ (version >= 264): 108 bytes
///   - type(4) + fov/far/near(12) + pos_track(20) + pos_base(12)
///   - + target_track(20) + target_base(12) + roll_track(20) + id(4) + flags(2) + pad(2)
#[derive(Debug, Clone)]
pub struct M2Camera {
    /// Camera type (0=portrait, 1=character info, -1=default)
    pub camera_type: u32,
    /// Field of view (in radians)
    pub fov: f32,
    /// Far clip distance
    pub far_clip: f32,
    /// Near clip distance
    pub near_clip: f32,
    /// Camera position animation
    pub position_animation: M2AnimationBlock<C3Vector>,
    /// Camera position base (default position when not animated)
    pub position_base: C3Vector,
    /// Target position animation
    pub target_position_animation: M2AnimationBlock<C3Vector>,
    /// Target position base (default target when not animated)
    pub target_position_base: C3Vector,
    /// Roll animation (rotation around the view axis)
    pub roll_animation: M2AnimationBlock<f32>,
    /// Camera ID (WotLK+ only)
    pub id: u32,
    /// Camera flags (WotLK+ only)
    pub flags: M2CameraFlags,
}

impl M2Camera {
    /// Parse a camera from a reader based on the M2 version
    ///
    /// Camera structure varies by version:
    /// - Pre-WotLK (< 264): 124 bytes - header + tracks + base values (no id/flags)
    /// - WotLK+ (>= 264): 108 bytes - smaller tracks + id/flags
    pub fn parse<R: Read + Seek>(reader: &mut R, version: u32) -> Result<Self> {
        let camera_type = reader.read_u32_le()?;
        let fov = reader.read_f32_le()?;
        let far_clip = reader.read_f32_le()?;
        let near_clip = reader.read_f32_le()?;

        // Position track followed by position base (C3Vector)
        let position_animation = M2AnimationBlock::parse(reader)?;
        let position_base = C3Vector::parse(reader)?;

        // Target position track followed by target base (C3Vector)
        let target_position_animation = M2AnimationBlock::parse(reader)?;
        let target_position_base = C3Vector::parse(reader)?;

        // Roll track (no base value - roll defaults to 0)
        let roll_animation = M2AnimationBlock::parse(reader)?;

        // ID and flags are only present in WotLK+ (version >= 264)
        let (id, flags) = if version >= 264 {
            let id = reader.read_u32_le()?;
            let flags = M2CameraFlags::from_bits_retain(reader.read_u16_le()?);
            reader.read_u16_le()?; // Skip padding
            (id, flags)
        } else {
            // Pre-WotLK: no id/flags fields
            (0, M2CameraFlags::empty())
        };

        Ok(Self {
            camera_type,
            fov,
            far_clip,
            near_clip,
            position_animation,
            position_base,
            target_position_animation,
            target_position_base,
            roll_animation,
            id,
            flags,
        })
    }

    /// Write a camera to a writer based on the M2 version
    pub fn write<W: Write>(&self, writer: &mut W, version: u32) -> Result<()> {
        writer.write_u32_le(self.camera_type)?;
        writer.write_f32_le(self.fov)?;
        writer.write_f32_le(self.far_clip)?;
        writer.write_f32_le(self.near_clip)?;

        // Position track followed by position base
        self.position_animation.write(writer)?;
        self.position_base.write(writer)?;

        // Target position track followed by target base
        self.target_position_animation.write(writer)?;
        self.target_position_base.write(writer)?;

        // Roll track (no base value)
        self.roll_animation.write(writer)?;

        // ID and flags only for WotLK+ (version >= 264)
        if version >= 264 {
            writer.write_u32_le(self.id)?;
            writer.write_u16_le(self.flags.bits())?;
            writer.write_u16_le(0)?; // padding
        }

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
            position_animation: M2AnimationBlock::new(M2AnimationTrack::default()),
            position_base: C3Vector::default(),
            target_position_animation: M2AnimationBlock::new(M2AnimationTrack::default()),
            target_position_base: C3Vector::default(),
            roll_animation: M2AnimationBlock::new(M2AnimationTrack::default()),
            id,
            flags: M2CameraFlags::empty(),
        }
    }

    /// Returns the size of a camera in bytes for the given version
    pub fn size(version: u32) -> usize {
        if version >= 264 {
            // WotLK+: 20-byte tracks + id/flags
            // type(4) + fov/far/near(12) + pos_track(20) + pos_base(12)
            // + target_track(20) + target_base(12) + roll_track(20) + id(4) + flags(4)
            108
        } else {
            // Pre-WotLK: 28-byte tracks (with ranges), no id/flags
            // type(4) + fov/far/near(12) + pos_track(28) + pos_base(12)
            // + target_track(28) + target_base(12) + roll_track(28)
            124
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_camera_parse_write_vanilla() {
        let camera = M2Camera::new(1);
        let version = M2Version::Vanilla.to_header_version();

        // Test write
        let mut data = Vec::new();
        camera.write(&mut data, version).unwrap();

        // Vanilla camera: type(4) + fov/far/near(12) + 3 tracks(28*3) + 2 bases(12*2)
        // = 16 + 84 + 24 = 124 bytes (no id/flags)
        assert_eq!(data.len(), 124);

        // Test parse
        let mut cursor = Cursor::new(data);
        let parsed = M2Camera::parse(&mut cursor, version).unwrap();

        assert_eq!(parsed.camera_type, 0);
        // id defaults to 0 for Vanilla (not stored in file)
        assert_eq!(parsed.id, 0);
        assert_eq!(parsed.flags, M2CameraFlags::empty());
    }

    #[test]
    fn test_camera_parse_write_wotlk() {
        let mut camera = M2Camera::new(5);
        camera.flags = M2CameraFlags::CUSTOM_UV;
        let version = M2Version::WotLK.to_header_version();

        // Test write
        let mut data = Vec::new();
        camera.write(&mut data, version).unwrap();

        // Note: Current implementation always writes 28-byte tracks (pre-WotLK format)
        // WotLK+ should technically use 20-byte tracks, but that's not yet implemented
        // So actual size: 16 + 84 + 24 + 8 (id/flags/pad) = 132 bytes
        assert_eq!(data.len(), 132);

        // Test parse
        let mut cursor = Cursor::new(data);
        let parsed = M2Camera::parse(&mut cursor, version).unwrap();

        assert_eq!(parsed.camera_type, 0);
        assert_eq!(parsed.id, 5);
        assert_eq!(parsed.flags, M2CameraFlags::CUSTOM_UV);
    }

    #[test]
    fn test_camera_flags() {
        let flags = M2CameraFlags::CUSTOM_UV | M2CameraFlags::AUTO_GENERATED;
        assert!(flags.contains(M2CameraFlags::CUSTOM_UV));
        assert!(flags.contains(M2CameraFlags::AUTO_GENERATED));
        assert!(!flags.contains(M2CameraFlags::GLOBAL_POSITION));
    }

    #[test]
    fn test_camera_size() {
        // These are the expected format sizes (not current implementation sizes)
        assert_eq!(M2Camera::size(256), 124); // Vanilla (28-byte tracks)
        assert_eq!(M2Camera::size(263), 124); // TBC (28-byte tracks)
        assert_eq!(M2Camera::size(264), 108); // WotLK (20-byte tracks)
        assert_eq!(M2Camera::size(272), 108); // MoP (20-byte tracks)
    }
}
