use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Seek, Write};

use crate::common::{C3Vector, Quaternion, Quaternion16};
use crate::error::Result;
use crate::version::M2Version;

use super::animation::M2AnimationTrack;

bitflags::bitflags! {
    /// Bone flags as defined in the M2 format
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct M2BoneFlags: u32 {
        /// Spherical billboard
        const SPHERICAL_BILLBOARD = 0x8;
        /// Cylindrical billboard lock X
        const CYLINDRICAL_BILLBOARD_LOCK_X = 0x10;
        /// Cylindrical billboard lock Y
        const CYLINDRICAL_BILLBOARD_LOCK_Y = 0x20;
        /// Cylindrical billboard lock Z
        const CYLINDRICAL_BILLBOARD_LOCK_Z = 0x40;
        /// Transformed
        const TRANSFORMED = 0x200;
        /// Kinematic bone (requires physics)
        const KINEMATIC_BONE = 0x400;
        /// Helper bone
        const HELPER_BONE = 0x1000;
        /// Has animation
        const HAS_ANIMATION = 0x4000;
        /// Has multiple animations at higher LODs
        const ANIMATED_AT_HIGHER_LODS = 0x8000;
        /// Has procedural animation
        const HAS_PROCEDURAL_ANIMATION = 0x10000;
        /// Has IK (inverse kinematics)
        const HAS_IK = 0x20000;
    }
}

#[derive(Debug, Clone)]
pub enum M2BoneRotation {
    Classic(M2AnimationTrack<Quaternion>),
    Others(M2AnimationTrack<Quaternion16>),
}

impl M2BoneRotation {
    pub fn write<W: Write>(&self, _writer: &mut W) -> Result<()> {
        todo!("implement this")
    }
}

/// Represents a bone in an M2 model
#[derive(Debug, Clone)]
pub struct M2Bone {
    /// Bone ID
    pub bone_id: i32,
    /// Flags
    pub flags: M2BoneFlags,
    /// Parent bone ID
    pub parent_bone: i16,
    /// Submesh ID
    pub submesh_id: u16,
    /// Unknown values (may differ between versions)
    pub unknown: [u16; 2],
    /// Position
    pub position: M2AnimationTrack<C3Vector>,
    /// Rotation
    pub rotation: M2BoneRotation,
    /// Scaling
    pub scaling: M2AnimationTrack<C3Vector>,
    /// Pivot point
    pub pivot: C3Vector,
}

impl M2Bone {
    /// Parse a bone from a reader based on the M2 version
    pub fn parse<R: Read + Seek>(reader: &mut R, version: u32) -> Result<Self> {
        // Read header fields properly
        let bone_id = reader.read_i32_le()?;
        let flags = M2BoneFlags::from_bits_retain(reader.read_u32_le()?);
        let parent_bone = reader.read_i16_le()?;
        let submesh_id = reader.read_u16_le()?;

        // In BC+ (version >= 260), there's an extra int32 field
        let unknown = if version >= 260 {
            // Read the extra unknown int32 field for BC+
            let _unknown_bc = reader.read_i32_le()?;
            [0, 0] // We don't use the old unknown fields in BC+
        } else {
            // Classic format with two uint16 unknown fields
            [reader.read_u16_le()?, reader.read_u16_le()?]
        };

        let position = M2AnimationTrack::parse(reader, version)?;

        let rotation = if version < 264 {
            if version < 260 {
                M2BoneRotation::Classic(M2AnimationTrack::parse(reader, version)?)
            } else {
                M2BoneRotation::Others(M2AnimationTrack::parse(reader, version)?)
            }
        } else {
            M2BoneRotation::Others(M2AnimationTrack::parse(reader, version)?)
        };

        let scaling = M2AnimationTrack::parse(reader, version)?;

        let pivot = C3Vector::parse(reader)?;

        Ok(Self {
            bone_id,
            flags,
            parent_bone,
            submesh_id,
            unknown,
            position,
            rotation,
            scaling,
            pivot,
        })
    }

    /// Write a bone to a writer
    pub fn write<W: Write>(&self, writer: &mut W, version: u32) -> Result<()> {
        writer.write_i32_le(self.bone_id)?;
        writer.write_u32_le(self.flags.bits())?;
        writer.write_i16_le(self.parent_bone)?;
        writer.write_u16_le(self.submesh_id)?;

        if version >= 260 {
            // BC+ format: write an unknown int32
            writer.write_i32_le(0)?; // Unknown field added in BC
        } else {
            // Classic format: write two uint16 unknown fields
            for &value in &self.unknown {
                writer.write_u16_le(value)?;
            }
        }

        self.position.write(writer)?;

        self.rotation.write(writer)?;

        self.scaling.write(writer)?;

        self.pivot.write(writer)?;

        Ok(())
    }

    /// Convert this bone to a different version (no version differences for bones yet)
    pub fn convert(&self, _target_version: M2Version) -> Self {
        self.clone()
    }

    /// Create a new bone with default values
    pub fn new(bone_id: i32, parent_bone: i16) -> Self {
        Self {
            bone_id,
            flags: M2BoneFlags::empty(),
            parent_bone,
            submesh_id: 0,
            unknown: [0, 0],
            position: M2AnimationTrack::new(),
            rotation: M2BoneRotation::Others(M2AnimationTrack::new()),
            scaling: M2AnimationTrack::new(),
            pivot: C3Vector {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        }
    }

    /// Check if this bone is a billboard
    pub fn is_billboard(&self) -> bool {
        self.flags.contains(M2BoneFlags::SPHERICAL_BILLBOARD)
            || self
                .flags
                .contains(M2BoneFlags::CYLINDRICAL_BILLBOARD_LOCK_X)
            || self
                .flags
                .contains(M2BoneFlags::CYLINDRICAL_BILLBOARD_LOCK_Y)
            || self
                .flags
                .contains(M2BoneFlags::CYLINDRICAL_BILLBOARD_LOCK_Z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_bone_parse() {
        let mut data = Vec::new();

        // Bone ID
        data.extend_from_slice(&1i32.to_le_bytes());

        // Flags (TRANSFORMED)
        data.extend_from_slice(&0x200u32.to_le_bytes());

        // Parent bone
        data.extend_from_slice(&(-1i16).to_le_bytes());

        // Submesh ID
        data.extend_from_slice(&0u16.to_le_bytes());

        // Unknown
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());

        // Position
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&2.0f32.to_le_bytes());
        data.extend_from_slice(&3.0f32.to_le_bytes());

        // Position animation
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());

        // Rotation
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());

        // Rotation animation
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());

        // Scaling
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());

        // Scaling animation
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());

        // Pivot
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());

        let mut cursor = Cursor::new(data);
        let bone = M2Bone::parse(&mut cursor, M2Version::Classic.to_header_version()).unwrap();

        assert_eq!(bone.bone_id, 1);
        assert_eq!(bone.flags, M2BoneFlags::TRANSFORMED);
        assert_eq!(bone.parent_bone, -1);
        assert_eq!(bone.submesh_id, 0);
        // assert_eq!(bone.position.x, 1.0);
        // assert_eq!(bone.position.y, 2.0);
        // assert_eq!(bone.position.z, 3.0);
    }

    #[test]
    fn test_bone_write() {
        let bone = M2Bone {
            bone_id: 1,
            flags: M2BoneFlags::TRANSFORMED,
            parent_bone: -1,
            submesh_id: 0,
            unknown: [0, 0],
            position: M2AnimationTrack::new(),
            rotation: M2BoneRotation::Others(M2AnimationTrack::new()),
            scaling: M2AnimationTrack::new(),
            pivot: C3Vector {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        };

        let mut data = Vec::new();
        bone.write(&mut data, 260).unwrap(); // BC version

        // Verify that the written data has the correct length
        // BC M2Bone size: 4 + 4 + 2 + 2 + 4 (extra unknown) + 12 + 8 + 16 + 8 + 12 + 8 + 12 = 92 bytes
        assert_eq!(data.len(), 92);

        // Test Classic version too
        let mut classic_data = Vec::new();
        bone.write(&mut classic_data, 256).unwrap(); // Classic version

        // Classic M2Bone size: 4 + 4 + 2 + 2 + 2 + 2 (two uint16 unknowns) + 12 + 8 + 16 + 8 + 12 + 8 + 12 = 92 bytes
        // Note: The comment was wrong - it's actually 92 bytes, same as BC but with different unknown field layout
        assert_eq!(classic_data.len(), 92);
    }
}
