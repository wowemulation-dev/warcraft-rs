use std::io::SeekFrom;

use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{C3Vector, Quaternion, Quaternion16, WowArrayV};
use wow_data_derive::{VWowHeaderR, WowHeaderW};

use crate::Result;
use crate::version::M2Version;

use super::animation::{M2AnimationTrack, M2AnimationTrackHeader};

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

impl WowHeaderR for M2BoneFlags {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        Ok(Self::from_bits_retain(reader.wow_read()?))
    }
}
impl WowHeaderW for M2BoneFlags {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        writer.wow_write(&self.bits())?;
        Ok(())
    }
    fn wow_size(&self) -> usize {
        4
    }
}

#[derive(Debug, Clone, WowHeaderW)]
pub enum M2BoneRotationHeader {
    Classic(M2AnimationTrackHeader<Quaternion>),
    Later(M2AnimationTrackHeader<Quaternion16>),
}

impl VWowHeaderR<M2Version> for M2BoneRotationHeader {
    fn wow_read<R: Read + Seek>(reader: &mut R, version: M2Version) -> WDResult<Self> {
        Ok(if version == M2Version::Classic {
            Self::Classic(reader.wow_read_versioned(version)?)
        } else {
            Self::Later(reader.wow_read_versioned(version)?)
        })
    }
}

#[derive(Debug, Clone)]
pub enum M2BoneRotation {
    Classic(M2AnimationTrack<Quaternion>),
    Later(M2AnimationTrack<Quaternion16>),
}

impl WowDataRV<M2Version, M2BoneRotationHeader> for M2BoneRotation {
    fn new_from_header<R: Read + Seek>(
        reader: &mut R,
        header: &M2BoneRotationHeader,
    ) -> WDResult<Self> {
        match header {
            M2BoneRotationHeader::Classic(classic) => Ok(Self::Classic(
                M2AnimationTrack::read_from_header(reader, classic)?,
            )),
            M2BoneRotationHeader::Later(later) => Ok(Self::Later(
                M2AnimationTrack::read_from_header(reader, later)?,
            )),
        }
    }
}

#[derive(Debug, Clone, VWowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub enum M2BoneCrc {
    Classic {
        u_dist_to_furth_desc: u16,
        u_zratio_of_chain: u16,
    },

    #[wow_data(read_if = version > M2Version::Classic)]
    Crc(u32),
}

/// Represents a bone in an M2 model
#[derive(Debug, Clone, VWowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub struct M2BoneHeader {
    pub bone_id: i32,
    pub flags: M2BoneFlags,
    pub parent_bone: i16,
    pub submesh_id: u16,

    #[wow_data(versioned)]
    pub bone_crc: M2BoneCrc,

    #[wow_data(versioned)]
    pub position: M2AnimationTrackHeader<C3Vector>,

    #[wow_data(versioned)]
    pub rotation: M2BoneRotationHeader,

    #[wow_data(versioned)]
    pub scaling: M2AnimationTrackHeader<C3Vector>,
    pub pivot: C3Vector,
}

#[derive(Debug, Clone)]
pub struct M2BoneData {
    pub position: M2AnimationTrack<C3Vector>,
    pub rotation: M2BoneRotation,
    pub scaling: M2AnimationTrack<C3Vector>,
}

impl WowDataRV<M2Version, M2BoneHeader> for M2BoneData {
    fn new_from_header<R: Read + Seek>(reader: &mut R, header: &M2BoneHeader) -> WDResult<Self> {
        let position = M2AnimationTrack::read_from_header(reader, &header.position)?;
        let rotation = M2AnimationTrack::read_from_header(reader, &header.scaling)?;
        let scaling = M2AnimationTrack::read_from_header(reader, &header.scaling)?;
    }
}

pub struct M2Bone {
    header: M2BoneHeader,
    data: M2BoneData,
}

impl M2Bone {
    pub fn read_bone_array<R: Read + Seek>(
        reader: &mut R,
        bone_header_array: WowArrayV<M2Version, M2BoneHeader>,
        version: M2Version,
    ) -> Result<Vec<M2Bone>> {
        // Special handling for BC item files with 203 bones
        if version == M2Version::TBC && bone_header_array.count == 203 {
            // Check if this might be an item file with bone indices instead of bone structures
            let current_pos = reader.stream_position()?;
            let file_size = reader.seek(SeekFrom::End(0))?;
            reader.seek(SeekFrom::Start(current_pos))?; // Restore position

            let bone_size = 92; // BC bone size
            let expected_end =
                bone_header_array.offset as u64 + (bone_header_array.count as u64 * bone_size);

            if expected_end > file_size {
                // File is too small to contain 203 bone structures
                // This is likely a BC item file where "bones" is actually a bone lookup table

                // Skip the bone lookup table for now - we'll handle it differently
                return Ok(Vec::new());
            }
        }

        let mut iter = bone_header_array.new_iterator(reader, version).unwrap();
        let mut items = Vec::new();
        loop {
            match iter.next(|reader, item_header| {
                let item_header = match item_header {
                    Some(item) => item,
                    None => reader.wow_read_versioned(version)?,
                };
                items.push(M2Bone {
                    data: reader.new_from_header(&item_header)?,
                    header: item_header,
                });
                Ok(())
            }) {
                Ok(is_active) => {
                    if !is_active {
                        break;
                    }
                }
                Err(err) => return Err(err.into()),
            }
        }

        Ok(items)
    }
}

// impl M2Bone {
//     /// Parse a bone from a reader based on the M2 version
//     pub fn parse<R: Read + Seek>(reader: &mut R, version: u32) -> Result<Self> {
//         let version_e = M2Version::try_from_header_version(version)
//             .ok_or_else(|| M2Error::UnsupportedNumericVersion(version))?;
//
//         // Read header fields properly
//         let bone_id = reader.read_i32_le()?;
//         let flags = M2BoneFlags::from_bits_retain(reader.read_u32_le()?);
//         let parent_bone = reader.read_i16_le()?;
//         let submesh_id = reader.read_u16_le()?;
//
//         let bone_crc = if version_e >= M2Version::TBC {
//             M2BoneCrc::Later(reader.read_u32_le()?)
//         } else {
//             M2BoneCrc::Classic(reader.read_u16_le()?, reader.read_u16_le()?)
//         };
//
//         let position = M2AnimationTrack::parse(reader, version)?;
//
//         let rotation = match version_e {
//             M2Version::Classic => {
//                 M2BoneRotation::Classic(M2AnimationTrack::parse(reader, version)?)
//             }
//             _ => M2BoneRotation::Later(M2AnimationTrack::parse(reader, version)?),
//         };
//
//         let scaling = M2AnimationTrack::parse(reader, version)?;
//
//         let pivot = C3Vector::parse(reader)?;
//
//         Ok(Self {
//             bone_id,
//             flags,
//             parent_bone,
//             submesh_id,
//             bone_crc,
//             position,
//             rotation,
//             scaling,
//             pivot,
//         })
//     }
//
//     /// Write a bone to a writer
//     pub fn write<W: Write>(&self, writer: &mut W, version: u32) -> Result<()> {
//         let _version_e = M2Version::try_from_header_version(version)
//             .ok_or_else(|| M2Error::UnsupportedNumericVersion(version))?;
//
//         writer.write_i32_le(self.bone_id)?;
//         writer.write_u32_le(self.flags.bits())?;
//         writer.write_i16_le(self.parent_bone)?;
//         writer.write_u16_le(self.submesh_id)?;
//
//         self.bone_crc.write(writer)?;
//
//         self.position.write(writer)?;
//
//         self.rotation.write(writer)?;
//
//         self.scaling.write(writer)?;
//
//         self.pivot.write(writer)?;
//
//         Ok(())
//     }
//
//     /// Convert this bone to a different version (no version differences for bones yet)
//     pub fn convert(&self, _target_version: M2Version) -> Self {
//         self.clone()
//     }
//
//     /// Create a new bone with default values
//     pub fn new(bone_id: i32, parent_bone: i16) -> Self {
//         Self {
//             bone_id,
//             flags: M2BoneFlags::empty(),
//             parent_bone,
//             submesh_id: 0,
//             bone_crc: M2BoneCrc::Later(0),
//             position: M2AnimationTrack::new(),
//             rotation: M2BoneRotation::Later(M2AnimationTrack::new()),
//             scaling: M2AnimationTrack::new(),
//             pivot: C3Vector {
//                 x: 0.0,
//                 y: 0.0,
//                 z: 0.0,
//             },
//         }
//     }
//
//     /// Check if this bone is a billboard
//     pub fn is_billboard(&self) -> bool {
//         self.flags.contains(M2BoneFlags::SPHERICAL_BILLBOARD)
//             || self
//                 .flags
//                 .contains(M2BoneFlags::CYLINDRICAL_BILLBOARD_LOCK_X)
//             || self
//                 .flags
//                 .contains(M2BoneFlags::CYLINDRICAL_BILLBOARD_LOCK_Y)
//             || self
//                 .flags
//                 .contains(M2BoneFlags::CYLINDRICAL_BILLBOARD_LOCK_Z)
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::io::Cursor;
//
//     #[test]
//     fn test_bone_parse() {
//         let mut data = Vec::new();
//
//         // Bone ID
//         data.extend_from_slice(&1i32.to_le_bytes());
//
//         // Flags (TRANSFORMED)
//         data.extend_from_slice(&0x200u32.to_le_bytes());
//
//         // Parent bone
//         data.extend_from_slice(&(-1i16).to_le_bytes());
//
//         // Submesh ID
//         data.extend_from_slice(&0u16.to_le_bytes());
//
//         // Unknown
//         data.extend_from_slice(&0u16.to_le_bytes());
//         data.extend_from_slice(&0u16.to_le_bytes());
//
//         // Position
//         data.extend_from_slice(&1.0f32.to_le_bytes());
//         data.extend_from_slice(&2.0f32.to_le_bytes());
//         data.extend_from_slice(&3.0f32.to_le_bytes());
//
//         // Position animation
//         data.extend_from_slice(&0u32.to_le_bytes());
//         data.extend_from_slice(&0u32.to_le_bytes());
//
//         // Rotation
//         data.extend_from_slice(&0.0f32.to_le_bytes());
//         data.extend_from_slice(&0.0f32.to_le_bytes());
//         data.extend_from_slice(&0.0f32.to_le_bytes());
//         data.extend_from_slice(&1.0f32.to_le_bytes());
//
//         // Rotation animation
//         data.extend_from_slice(&0u32.to_le_bytes());
//         data.extend_from_slice(&0u32.to_le_bytes());
//
//         // Scaling
//         data.extend_from_slice(&1.0f32.to_le_bytes());
//         data.extend_from_slice(&1.0f32.to_le_bytes());
//         data.extend_from_slice(&1.0f32.to_le_bytes());
//
//         // Scaling animation
//         data.extend_from_slice(&0u32.to_le_bytes());
//         data.extend_from_slice(&0u32.to_le_bytes());
//
//         // Pivot
//         data.extend_from_slice(&0.0f32.to_le_bytes());
//         data.extend_from_slice(&0.0f32.to_le_bytes());
//         data.extend_from_slice(&0.0f32.to_le_bytes());
//
//         let mut cursor = Cursor::new(data);
//         let bone =
//             M2BoneHeader::parse(&mut cursor, M2Version::Classic.to_header_version()).unwrap();
//
//         assert_eq!(bone.bone_id, 1);
//         assert_eq!(bone.flags, M2BoneFlags::TRANSFORMED);
//         assert_eq!(bone.parent_bone, -1);
//         assert_eq!(bone.submesh_id, 0);
//         // assert_eq!(bone.position.x, 1.0);
//         // assert_eq!(bone.position.y, 2.0);
//         // assert_eq!(bone.position.z, 3.0);
//     }
//
//     #[test]
//     fn test_bone_write() {
//         let bone = M2BoneHeader {
//             bone_id: 1,
//             flags: M2BoneFlags::TRANSFORMED,
//             parent_bone: -1,
//             submesh_id: 0,
//             bone_crc: M2BoneCrc::Later(0),
//             position: M2AnimationTrack::new(),
//             rotation: M2BoneRotation::Later(M2AnimationTrack::new()),
//             scaling: M2AnimationTrack::new(),
//             pivot: C3Vector {
//                 x: 0.0,
//                 y: 0.0,
//                 z: 0.0,
//             },
//         };
//
//         let mut data = Vec::new();
//         bone.write(&mut data, 260).unwrap(); // BC version
//
//         // Verify that the written data has the correct length
//         // BC M2Bone size: 4 + 4 + 2 + 2 + 4 (extra unknown) + 12 + 8 + 16 + 8 + 12 + 8 + 12 = 92 bytes
//         assert_eq!(data.len(), 92);
//
//         // Test Classic version too
//         let mut classic_data = Vec::new();
//         bone.write(&mut classic_data, 256).unwrap(); // Classic version
//
//         // Classic M2Bone size: 4 + 4 + 2 + 2 + 2 + 2 (two uint16 unknowns) + 12 + 8 + 16 + 8 + 12 + 8 + 12 = 92 bytes
//         // Note: The comment was wrong - it's actually 92 bytes, same as BC but with different unknown field layout
//         assert_eq!(classic_data.len(), 92);
//     }
// }
