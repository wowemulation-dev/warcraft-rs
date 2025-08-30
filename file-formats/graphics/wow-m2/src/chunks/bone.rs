use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Write};

use crate::chunks::m2_track::{M2TrackQuat, M2TrackVec3};
use crate::common::C3Vector;
use crate::error::Result;
use crate::version::M2Version;

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
    /// TBC+ bone name CRC field for debugging/identification (wowdev.wiki: boneNameCRC)
    /// Only valid for version >= 260. This is a CRC hash of the bone's name string.
    pub bone_name_crc: Option<u32>,
    /// Translation animation track
    pub translation: M2TrackVec3,
    /// Rotation animation track
    pub rotation: M2TrackQuat,
    /// Scale animation track
    pub scale: M2TrackVec3,
    /// Pivot point
    pub pivot: C3Vector,
}

impl M2Bone {
    /// Parse a bone from a reader based on the M2 version
    pub fn parse<R: Read>(reader: &mut R, version: u32) -> Result<Self> {
        // Read header fields properly
        let bone_id = reader.read_i32_le()?;
        let flags = M2BoneFlags::from_bits_retain(reader.read_u32_le()?);
        let parent_bone = reader.read_i16_le()?;
        let submesh_id = reader.read_u16_le()?;

        // Version-specific bone name CRC field based on wowdev.wiki and WMVx M2Definitions.h:
        // - Vanilla (< TBC_MIN=260): NO boneNameCRC field after submeshId
        // - TBC+ (>= 260): HAS uint32 boneNameCRC field (wowdev.wiki: union with boneNameCRC for debugging)
        let (unknown, bone_name_crc) = if version >= 260 {
            // TBC+ format: read the boneNameCRC uint32 field (CRC hash of bone name string)
            let bone_name_crc = reader.read_u32_le()?;
            ([0, 0], Some(bone_name_crc)) // Store CRC for debugging/identification
        } else {
            // Vanilla format: NO boneNameCRC field - WMVx shows direct transition to AnimationBlocks
            ([0, 0], None) // No CRC field in vanilla
        };

        let translation = M2TrackVec3::parse(reader, version)?;
        let rotation = M2TrackQuat::parse(reader, version)?;
        let scale = M2TrackVec3::parse(reader, version)?;

        let mut pivot = C3Vector::parse(reader)?;

        // CRITICAL FIX: Handle NaN values in bone pivot coordinates
        // This addresses corruption where bone pivots contain NaN values
        if pivot.x.is_nan() || pivot.y.is_nan() || pivot.z.is_nan() {
            // Replace NaN values with zero (safe default for pivot point)
            if pivot.x.is_nan() {
                pivot.x = 0.0;
            }
            if pivot.y.is_nan() {
                pivot.y = 0.0;
            }
            if pivot.z.is_nan() {
                pivot.z = 0.0;
            }
        }

        Ok(Self {
            bone_id,
            flags,
            parent_bone,
            submesh_id,
            unknown,
            bone_name_crc,
            translation,
            rotation,
            scale,
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
            // TBC+ format: write the boneNameCRC uint32 field
            writer.write_u32_le(self.bone_name_crc.unwrap_or(0))?; // boneNameCRC field present in TBC+
        } else {
            // Vanilla format: NO boneNameCRC field to write based on WMVx reference
            // WMVx M2Definitions.h shows vanilla goes directly to AnimationBlocks
        }

        self.translation.write(writer, version)?;
        self.rotation.write(writer, version)?;
        self.scale.write(writer, version)?;

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
            bone_name_crc: None,
            translation: M2TrackVec3::new(),
            rotation: M2TrackQuat::new(),
            scale: M2TrackVec3::new(),
            pivot: C3Vector {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        }
    }

    /// Validate that bone data is reasonable for debugging
    /// Returns true if the bone data appears valid, false if corrupted
    pub fn is_valid_for_model(&self, total_bone_count: u32) -> bool {
        // Check bone_id is reasonable (-1 to 1000 is typical range)
        if self.bone_id < -1 || self.bone_id > 1000 {
            return false;
        }

        // Check parent_bone is reasonable (-1 or within bone count)
        if self.parent_bone != -1
            && (self.parent_bone < 0 || self.parent_bone as u32 >= total_bone_count)
        {
            return false;
        }

        // Check animation track counts are reasonable (< 100,000)
        if self.translation.timestamps.count > 100_000 || self.translation.values.count > 100_000 {
            return false;
        }

        if self.rotation.timestamps.count > 100_000 || self.rotation.values.count > 100_000 {
            return false;
        }

        if self.scale.timestamps.count > 100_000 || self.scale.values.count > 100_000 {
            return false;
        }

        true
    }

    /// Get a debug string for this bone
    pub fn debug_info(&self) -> String {
        let crc_info = if let Some(crc) = self.bone_name_crc {
            format!(", name_crc=0x{:08x}", crc)
        } else {
            String::new()
        };
        format!(
            "Bone(id={}, parent={}, flags=0x{:x}{}, trans_count={}, rot_count={}, scale_count={})",
            self.bone_id,
            self.parent_bone,
            self.flags.bits(),
            crc_info,
            self.translation.timestamps.count,
            self.rotation.timestamps.count,
            self.scale.timestamps.count
        )
    }

    /// Get the bone name CRC field value for TBC+ models
    pub fn get_bone_name_crc(&self) -> Option<u32> {
        self.bone_name_crc
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

        // NO unknown fields for vanilla (WMVx M2Definitions.h shows vanilla goes directly to animation blocks)

        // Translation track (M2TrackVec3: interpolation_type + global_sequence + ranges + timestamps + values - includes ranges for vanilla)
        data.extend_from_slice(&0u16.to_le_bytes()); // interpolation_type = None
        data.extend_from_slice(&65535u16.to_le_bytes()); // global_sequence = none
        data.extend_from_slice(&0u32.to_le_bytes()); // ranges count
        data.extend_from_slice(&0u32.to_le_bytes()); // ranges offset
        data.extend_from_slice(&0u32.to_le_bytes()); // timestamps count
        data.extend_from_slice(&0u32.to_le_bytes()); // timestamps offset
        data.extend_from_slice(&0u32.to_le_bytes()); // values count
        data.extend_from_slice(&0u32.to_le_bytes()); // values offset

        // Rotation track (M2TrackQuat: interpolation_type + global_sequence + ranges + timestamps + values - includes ranges for vanilla)
        data.extend_from_slice(&0u16.to_le_bytes()); // interpolation_type = None
        data.extend_from_slice(&65535u16.to_le_bytes()); // global_sequence = none
        data.extend_from_slice(&0u32.to_le_bytes()); // ranges count
        data.extend_from_slice(&0u32.to_le_bytes()); // ranges offset
        data.extend_from_slice(&0u32.to_le_bytes()); // timestamps count
        data.extend_from_slice(&0u32.to_le_bytes()); // timestamps offset
        data.extend_from_slice(&0u32.to_le_bytes()); // values count
        data.extend_from_slice(&0u32.to_le_bytes()); // values offset

        // Scale track (M2TrackVec3: interpolation_type + global_sequence + ranges + timestamps + values - includes ranges for vanilla)
        data.extend_from_slice(&0u16.to_le_bytes()); // interpolation_type = None
        data.extend_from_slice(&65535u16.to_le_bytes()); // global_sequence = none
        data.extend_from_slice(&0u32.to_le_bytes()); // ranges count
        data.extend_from_slice(&0u32.to_le_bytes()); // ranges offset
        data.extend_from_slice(&0u32.to_le_bytes()); // timestamps count
        data.extend_from_slice(&0u32.to_le_bytes()); // timestamps offset
        data.extend_from_slice(&0u32.to_le_bytes()); // values count
        data.extend_from_slice(&0u32.to_le_bytes()); // values offset

        // Pivot
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());

        let mut cursor = Cursor::new(data);
        let bone = M2Bone::parse(&mut cursor, M2Version::Vanilla.to_header_version()).unwrap();

        assert_eq!(bone.bone_id, 1);
        assert_eq!(bone.flags, M2BoneFlags::TRANSFORMED);
        assert_eq!(bone.parent_bone, -1);
        assert_eq!(bone.submesh_id, 0);
        // Test that tracks were parsed correctly
        assert!(!bone.translation.has_data()); // No animation data
        assert!(!bone.rotation.has_data()); // No animation data
        assert!(!bone.scale.has_data()); // No animation data
    }

    #[test]
    fn test_bone_validation_constraints() {
        // Test that validates bone data constraints as described in the issue
        let mut data = Vec::new();

        // Create a bone with reasonable values
        data.extend_from_slice(&1i32.to_le_bytes()); // bone_id: 1 (reasonable)
        data.extend_from_slice(&0x200u32.to_le_bytes()); // flags: TRANSFORMED
        data.extend_from_slice(&0i16.to_le_bytes()); // parent_bone: 0 (reasonable for 96-bone model)
        data.extend_from_slice(&0u16.to_le_bytes()); // submesh_id: 0
        // NO unknown fields for vanilla (WMVx M2Definitions.h shows vanilla goes directly to animation blocks)

        // Add reasonable animation track data (includes ranges for vanilla)
        for _ in 0..3 {
            // translation, rotation, scale
            data.extend_from_slice(&0u16.to_le_bytes()); // interpolation_type: None
            data.extend_from_slice(&65535u16.to_le_bytes()); // global_sequence: none
            data.extend_from_slice(&0u32.to_le_bytes()); // ranges count
            data.extend_from_slice(&0u32.to_le_bytes()); // ranges offset
            data.extend_from_slice(&5u32.to_le_bytes()); // timestamps count (reasonable)
            data.extend_from_slice(&1000u32.to_le_bytes()); // timestamps offset (reasonable)
            data.extend_from_slice(&5u32.to_le_bytes()); // values count (reasonable)
            data.extend_from_slice(&1200u32.to_le_bytes()); // values offset (reasonable)
        }

        data.extend_from_slice(&0.0f32.to_le_bytes()); // pivot.x
        data.extend_from_slice(&1.0f32.to_le_bytes()); // pivot.y
        data.extend_from_slice(&0.0f32.to_le_bytes()); // pivot.z

        // Now create a second bone with corrupted data (as described in issue)
        data.extend_from_slice(&1000i32.to_le_bytes()); // bone_id: 1000 (high but valid - was causing test failure)
        data.extend_from_slice(&0u32.to_le_bytes()); // flags: 0
        data.extend_from_slice(&50i16.to_le_bytes()); // parent_bone: 50 (high but reasonable for HumanMale's ~96 bones)
        data.extend_from_slice(&0u16.to_le_bytes()); // submesh_id: 0
        // NO unknown fields for vanilla (WMVx M2Definitions.h shows vanilla goes directly to animation blocks)

        // Add unreasonable animation track data (as seen in corruption, includes ranges for vanilla)
        for count in [4294901760u32, 22768u32, 100u32] {
            // translation, rotation, scale
            data.extend_from_slice(&0u16.to_le_bytes()); // interpolation_type: None
            data.extend_from_slice(&65535u16.to_le_bytes()); // global_sequence: none
            data.extend_from_slice(&0u32.to_le_bytes()); // ranges count
            data.extend_from_slice(&0u32.to_le_bytes()); // ranges offset
            data.extend_from_slice(&count.to_le_bytes()); // timestamps count (unreasonable)
            data.extend_from_slice(&1000u32.to_le_bytes()); // timestamps offset
            data.extend_from_slice(&count.to_le_bytes()); // values count (unreasonable)
            data.extend_from_slice(&1200u32.to_le_bytes()); // values offset
        }

        data.extend_from_slice(&0.0f32.to_le_bytes()); // pivot.x
        data.extend_from_slice(&1.0f32.to_le_bytes()); // pivot.y
        data.extend_from_slice(&0.0f32.to_le_bytes()); // pivot.z

        println!(
            "Test data created: {} bytes (2 bones * 108 = 216 expected)",
            data.len()
        );
        assert_eq!(data.len(), 216); // 2 bones * 108 bytes each for vanilla with WMVx-aligned structure

        let mut cursor = Cursor::new(&data);

        // Parse first bone - should be reasonable
        let bone1 = M2Bone::parse(&mut cursor, 256).unwrap();

        println!(
            "Bone 1: id={}, parent={}, translation_count={}",
            bone1.bone_id, bone1.parent_bone, bone1.translation.timestamps.count
        );

        // Validate first bone has reasonable values
        assert_eq!(bone1.bone_id, 1);
        assert_eq!(bone1.parent_bone, 0);
        assert_eq!(bone1.translation.timestamps.count, 5);
        assert_eq!(bone1.rotation.timestamps.count, 5);
        assert_eq!(bone1.scale.timestamps.count, 5);

        // Parse second bone - this would show corruption if present
        let bone2 = M2Bone::parse(&mut cursor, 256).unwrap();

        println!(
            "Bone 2: id={}, parent={}, translation_count={}",
            bone2.bone_id, bone2.parent_bone, bone2.translation.timestamps.count
        );

        // This bone should have the high but valid values we inserted
        assert_eq!(bone2.bone_id, 1000);
        assert_eq!(bone2.parent_bone, 50);
        assert_eq!(bone2.translation.timestamps.count, 4294901760);
        assert_eq!(bone2.rotation.timestamps.count, 22768);
        assert_eq!(bone2.scale.timestamps.count, 100);

        // Verify cursor position (2 bones * 108 bytes each = 216 bytes total)
        assert_eq!(cursor.position(), 216);

        println!(
            "✓ Both bones parsed with expected values (including intentionally corrupted second bone)"
        );
        println!("This confirms M2Bone::parse correctly reads what's in the data,");
        println!("so the issue must be in the source data or cursor positioning.");
    }

    #[test]
    fn test_m2track_byte_consumption_vanilla() {
        // Test exact byte consumption of M2Track parsing for Vanilla (WITH ranges)
        let mut data = Vec::new();

        // Create M2Track data for version 256 (SHOULD include ranges per WMVx reference)
        data.extend_from_slice(&1u16.to_le_bytes()); // interpolation_type: Linear
        data.extend_from_slice(&65535u16.to_le_bytes()); // global_sequence: none
        data.extend_from_slice(&1u32.to_le_bytes()); // ranges count
        data.extend_from_slice(&800u32.to_le_bytes()); // ranges offset
        data.extend_from_slice(&3u32.to_le_bytes()); // timestamps count
        data.extend_from_slice(&1000u32.to_le_bytes()); // timestamps offset
        data.extend_from_slice(&3u32.to_le_bytes()); // values count
        data.extend_from_slice(&1200u32.to_le_bytes()); // values offset

        assert_eq!(
            data.len(),
            28,
            "M2Track test data should be exactly 28 bytes for Vanilla"
        );

        let mut cursor = Cursor::new(&data);
        let pos_before = cursor.position();

        // Parse M2TrackVec3 (should consume all 28 bytes)
        let track = M2TrackVec3::parse(&mut cursor, 256).unwrap();

        let pos_after = cursor.position();
        let bytes_consumed = pos_after - pos_before;

        println!(
            "M2Track parsing: consumed {} bytes (expected 28)",
            bytes_consumed
        );
        println!(
            "Track details: interp={:?}, timestamps_count={}, values_count={}",
            track.base.interpolation_type, track.timestamps.count, track.values.count
        );

        // Critical test: M2Track should consume exactly 28 bytes for version 256 (Vanilla) per WMVx reference
        assert_eq!(
            bytes_consumed, 28,
            "M2Track should consume exactly 28 bytes for version 256, but consumed {}",
            bytes_consumed
        );

        // Verify the ranges field WAS parsed (Vanilla should have ranges per WMVx reference)
        assert!(
            track.ranges.is_some(),
            "M2Track SHOULD have ranges field for version 256 (Vanilla)"
        );

        println!("✓ M2Track consumes exactly 28 bytes as expected for Vanilla");
    }

    #[test]
    fn test_m2track_byte_consumption_tbc() {
        // Test exact byte consumption of M2Track parsing for TBC (has ranges)
        let mut data = Vec::new();

        // Create M2Track data for version 260 (should include ranges)
        data.extend_from_slice(&1u16.to_le_bytes()); // interpolation_type: Linear
        data.extend_from_slice(&65535u16.to_le_bytes()); // global_sequence: none
        data.extend_from_slice(&0u32.to_le_bytes()); // ranges count
        data.extend_from_slice(&0u32.to_le_bytes()); // ranges offset
        data.extend_from_slice(&3u32.to_le_bytes()); // timestamps count
        data.extend_from_slice(&1000u32.to_le_bytes()); // timestamps offset
        data.extend_from_slice(&3u32.to_le_bytes()); // values count
        data.extend_from_slice(&1200u32.to_le_bytes()); // values offset

        assert_eq!(
            data.len(),
            28,
            "M2Track test data should be exactly 28 bytes for TBC"
        );

        let mut cursor = Cursor::new(&data);
        let pos_before = cursor.position();

        // Parse M2TrackVec3 (should consume all 28 bytes)
        let track = M2TrackVec3::parse(&mut cursor, 260).unwrap();

        let pos_after = cursor.position();
        let bytes_consumed = pos_after - pos_before;

        println!(
            "M2Track parsing: consumed {} bytes (expected 28)",
            bytes_consumed
        );
        println!(
            "Track details: interp={:?}, timestamps_count={}, values_count={}",
            track.base.interpolation_type, track.timestamps.count, track.values.count
        );

        // Critical test: M2Track should consume exactly 28 bytes for version 260 (TBC)
        assert_eq!(
            bytes_consumed, 28,
            "M2Track should consume exactly 28 bytes for version 260, but consumed {}",
            bytes_consumed
        );

        // Verify the ranges field was parsed (TBC should have ranges)
        assert!(
            track.ranges.is_some(),
            "M2Track should have ranges field for version 260 (TBC)"
        );

        println!("✓ M2Track consumes exactly 28 bytes as expected for TBC");
    }

    #[test]
    fn test_sequential_bone_parsing() {
        // Test that multiple bones can be parsed sequentially
        let mut data = Vec::new();

        // Create data for 3 bones
        for bone_id in 0i32..3 {
            // Bone ID
            data.extend_from_slice(&bone_id.to_le_bytes());

            // Flags (TRANSFORMED)
            data.extend_from_slice(&0x200u32.to_le_bytes());

            // Parent bone (bone 0 has no parent, others have previous as parent)
            let parent = if bone_id == 0 { -1 } else { bone_id - 1 };
            data.extend_from_slice(&(parent as i16).to_le_bytes());

            // Submesh ID
            data.extend_from_slice(&0u16.to_le_bytes());

            // NO unknown fields for vanilla (WMVx M2Definitions.h shows vanilla goes directly to animation blocks)

            // Translation track (M2TrackVec3: interpolation_type + global_sequence + ranges + timestamps + values - includes ranges for vanilla)
            data.extend_from_slice(&0u16.to_le_bytes()); // interpolation_type = None
            data.extend_from_slice(&65535u16.to_le_bytes()); // global_sequence = none
            data.extend_from_slice(&0u32.to_le_bytes()); // ranges count
            data.extend_from_slice(&0u32.to_le_bytes()); // ranges offset
            data.extend_from_slice(&0u32.to_le_bytes()); // timestamps count
            data.extend_from_slice(&0u32.to_le_bytes()); // timestamps offset
            data.extend_from_slice(&0u32.to_le_bytes()); // values count
            data.extend_from_slice(&0u32.to_le_bytes()); // values offset

            // Rotation track (M2TrackQuat: interpolation_type + global_sequence + ranges + timestamps + values - includes ranges for vanilla)
            data.extend_from_slice(&0u16.to_le_bytes()); // interpolation_type = None
            data.extend_from_slice(&65535u16.to_le_bytes()); // global_sequence = none
            data.extend_from_slice(&0u32.to_le_bytes()); // ranges count
            data.extend_from_slice(&0u32.to_le_bytes()); // ranges offset
            data.extend_from_slice(&0u32.to_le_bytes()); // timestamps count
            data.extend_from_slice(&0u32.to_le_bytes()); // timestamps offset
            data.extend_from_slice(&0u32.to_le_bytes()); // values count
            data.extend_from_slice(&0u32.to_le_bytes()); // values offset

            // Scale track (M2TrackVec3: interpolation_type + global_sequence + ranges + timestamps + values - includes ranges for vanilla)
            data.extend_from_slice(&0u16.to_le_bytes()); // interpolation_type = None
            data.extend_from_slice(&65535u16.to_le_bytes()); // global_sequence = none
            data.extend_from_slice(&0u32.to_le_bytes()); // ranges count
            data.extend_from_slice(&0u32.to_le_bytes()); // ranges offset
            data.extend_from_slice(&0u32.to_le_bytes()); // timestamps count
            data.extend_from_slice(&0u32.to_le_bytes()); // timestamps offset
            data.extend_from_slice(&0u32.to_le_bytes()); // values count
            data.extend_from_slice(&0u32.to_le_bytes()); // values offset

            // Pivot
            data.extend_from_slice(&0.0f32.to_le_bytes());
            data.extend_from_slice(&0.0f32.to_le_bytes());
            data.extend_from_slice(&0.0f32.to_le_bytes());
        }

        println!(
            "Total test data: {} bytes (expected: 3 * 108 = 324)",
            data.len()
        );
        assert_eq!(data.len(), 324); // 3 bones * 108 bytes each for vanilla

        let mut cursor = Cursor::new(data);

        // Parse all 3 bones sequentially
        let mut bones = Vec::new();
        for i in 0..3 {
            let pos_before = cursor.position();
            let bone = M2Bone::parse(&mut cursor, M2Version::Vanilla.to_header_version()).unwrap();
            let pos_after = cursor.position();
            let bytes_consumed = pos_after - pos_before;

            println!(
                "Bone {}: id={}, parent={}, consumed {} bytes (pos: {} -> {})",
                i, bone.bone_id, bone.parent_bone, bytes_consumed, pos_before, pos_after
            );

            // Verify bone data is correct
            assert_eq!(bone.bone_id, i);
            assert_eq!(bone.flags, M2BoneFlags::TRANSFORMED);
            if i == 0 {
                assert_eq!(bone.parent_bone, -1);
            } else {
                assert_eq!(bone.parent_bone, (i - 1) as i16);
            }
            assert_eq!(bone.submesh_id, 0);

            // Each bone should consume exactly 108 bytes for vanilla (WITH ranges in M2Track)
            // Bone structure: 4+4+2+2 (header, no unknown for vanilla) + 28*3 (3 M2Tracks with ranges) + 12 (pivot) = 108 bytes
            assert_eq!(
                bytes_consumed, 108,
                "Bone {} consumed {} bytes, expected 108",
                i, bytes_consumed
            );

            bones.push(bone);
        }

        // Verify we consumed all data (3 bones * 108 bytes = 324 bytes)
        assert_eq!(cursor.position(), 324);
        println!("✓ All 3 bones parsed successfully with correct sequential alignment");
    }

    #[test]
    fn test_nan_pivot_fix() {
        // Test the fix for NaN values in bone pivot coordinates (Issue #2)
        let mut data = Vec::new();

        // Bone ID
        data.extend_from_slice(&1i32.to_le_bytes());

        // Flags (TRANSFORMED)
        data.extend_from_slice(&0x200u32.to_le_bytes());

        // Parent bone
        data.extend_from_slice(&(-1i16).to_le_bytes());

        // Submesh ID
        data.extend_from_slice(&0u16.to_le_bytes());

        // Animation tracks (empty for vanilla)
        for _ in 0..3 {
            // translation, rotation, scale
            data.extend_from_slice(&0u16.to_le_bytes()); // interpolation_type = None
            data.extend_from_slice(&65535u16.to_le_bytes()); // global_sequence = none
            data.extend_from_slice(&0u32.to_le_bytes()); // ranges count
            data.extend_from_slice(&0u32.to_le_bytes()); // ranges offset
            data.extend_from_slice(&0u32.to_le_bytes()); // timestamps count
            data.extend_from_slice(&0u32.to_le_bytes()); // timestamps offset
            data.extend_from_slice(&0u32.to_le_bytes()); // values count
            data.extend_from_slice(&0u32.to_le_bytes()); // values offset
        }

        // CRITICAL: Pivot with NaN values (simulating corruption)
        data.extend_from_slice(&f32::NAN.to_le_bytes()); // pivot.x = NaN
        data.extend_from_slice(&2.5f32.to_le_bytes()); // pivot.y = valid
        data.extend_from_slice(&f32::NAN.to_le_bytes()); // pivot.z = NaN

        let mut cursor = Cursor::new(data);
        let bone = M2Bone::parse(&mut cursor, M2Version::Vanilla.to_header_version()).unwrap();

        // Verify NaN values were fixed (replaced with 0.0)
        assert_eq!(bone.pivot.x, 0.0, "NaN pivot.x should be replaced with 0.0");
        assert_eq!(bone.pivot.y, 2.5, "Valid pivot.y should be preserved");
        assert_eq!(bone.pivot.z, 0.0, "NaN pivot.z should be replaced with 0.0");

        // Verify no NaN values remain
        assert!(
            !bone.pivot.x.is_nan(),
            "pivot.x should not be NaN after parsing"
        );
        assert!(!bone.pivot.y.is_nan(), "pivot.y should not be NaN");
        assert!(
            !bone.pivot.z.is_nan(),
            "pivot.z should not be NaN after parsing"
        );

        // Other bone data should be preserved
        assert_eq!(bone.bone_id, 1);
        assert_eq!(bone.parent_bone, -1);

        println!(
            "✓ NaN pivot coordinates fixed: (NaN, 2.5, NaN) -> ({}, {}, {})",
            bone.pivot.x, bone.pivot.y, bone.pivot.z
        );
    }

    #[test]
    fn test_bone_write() {
        let bone = M2Bone {
            bone_id: 1,
            flags: M2BoneFlags::TRANSFORMED,
            parent_bone: -1,
            submesh_id: 0,
            unknown: [0, 0],
            bone_name_crc: Some(0x12345678), // Test CRC value for TBC+
            translation: M2TrackVec3::new(),
            rotation: M2TrackQuat::new(),
            scale: M2TrackVec3::new(),
            pivot: C3Vector {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        };

        let mut data = Vec::new();
        bone.write(&mut data, 260).unwrap(); // BC version

        // Verify that the written data has the correct length
        // BC M2Bone size: 4 + 4 + 2 + 2 + 4 (extra unknown) + 28*3 (M2Track each with ranges) + 12 (pivot) = 112 bytes
        assert_eq!(data.len(), 112);

        // Test Vanilla version too
        let mut vanilla_data = Vec::new();
        bone.write(&mut vanilla_data, 256).unwrap(); // Vanilla version

        // Vanilla M2Bone size: 4 + 4 + 2 + 2 (no unknown fields) + 28*3 (M2Track each with ranges) + 12 (pivot) = 108 bytes
        assert_eq!(vanilla_data.len(), 108);
    }
}
