use std::io::SeekFrom;

use wow_data::error::Result as WDResult;
use wow_data::types::{C3Vector, Quaternion, Quaternion16, VWowDataR, WowArrayV};
use wow_data::{prelude::*, v_wow_collection};
use wow_data_derive::{WowDataR, WowHeaderR, WowHeaderW};

use crate::Result;
use crate::version::MD20Version;

use super::animation::{M2AnimationTrackData, M2AnimationTrackHeader};

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, WowHeaderR, WowHeaderW)]
    #[wow_data(bitflags=u32)]
    pub struct M2BoneFlags: u32 {
        const SPHERICAL_BILLBOARD = 0x8;
        const CYLINDRICAL_BILLBOARD_LOCK_X = 0x10;
        const CYLINDRICAL_BILLBOARD_LOCK_Y = 0x20;
        const CYLINDRICAL_BILLBOARD_LOCK_Z = 0x40;
        const TRANSFORMED = 0x200;
        /// Kinematic bone (requires physics)
        const KINEMATIC_BONE = 0x400;
        const HELPER_BONE = 0x1000;
        const HAS_ANIMATION = 0x4000;
        const ANIMATED_AT_HIGHER_LODS = 0x8000;
        const HAS_PROCEDURAL_ANIMATION = 0x10000;
        /// Has IK (inverse kinematics)
        const HAS_IK = 0x20000;
    }
}

#[derive(Debug, Clone, WowHeaderW)]
pub enum M2BoneRotationHeader {
    Classic(M2AnimationTrackHeader<Quaternion>),
    Later(M2AnimationTrackHeader<Quaternion16>),
}

impl VWowHeaderR<MD20Version> for M2BoneRotationHeader {
    fn wow_read<R: Read + Seek>(reader: &mut R, version: MD20Version) -> WDResult<Self> {
        Ok(if version <= MD20Version::ClassicV4 {
            Self::Classic(reader.wow_read_versioned(version)?)
        } else {
            Self::Later(reader.wow_read_versioned(version)?)
        })
    }
}

#[derive(Debug, Clone)]
pub enum M2BoneRotationData {
    Classic(M2AnimationTrackData<Quaternion>),
    Later(M2AnimationTrackData<Quaternion16>),
}

impl VWowDataR<MD20Version, M2BoneRotationHeader> for M2BoneRotationData {
    fn new_from_header<R: Read + Seek>(
        reader: &mut R,
        header: &M2BoneRotationHeader,
    ) -> WDResult<Self> {
        match header {
            M2BoneRotationHeader::Classic(classic) => {
                Ok(Self::Classic(reader.v_new_from_header(classic)?))
            }
            M2BoneRotationHeader::Later(later) => Ok(Self::Later(reader.v_new_from_header(later)?)),
        }
    }
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub enum M2BoneCrc {
    None,

    /// Unknown use
    #[wow_data(read_if = version >= MD20Version::TBCV1 && version <= MD20Version::TBCV4)]
    TBC([u8; 4]),

    #[wow_data(read_if = version >= MD20Version::WotLK)]
    Crc(u32),
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
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

#[derive(Debug, Clone, WowDataR)]
#[wow_data(version=MD20Version, header=M2BoneHeader)]
pub struct M2BoneData {
    #[wow_data(versioned)]
    pub position: M2AnimationTrackData<C3Vector>,
    #[wow_data(versioned)]
    pub rotation: M2BoneRotationData,
    #[wow_data(versioned)]
    pub scaling: M2AnimationTrackData<C3Vector>,
}

#[derive(Debug, Clone)]
pub struct M2Bone {
    pub header: M2BoneHeader,
    pub data: M2BoneData,
}

impl M2Bone {
    pub fn read_bone_array<R: Read + Seek>(
        reader: &mut R,
        bone_header_array: WowArrayV<MD20Version, M2BoneHeader>,
        version: MD20Version,
    ) -> Result<Vec<M2Bone>> {
        // Special handling for BC item files with 203 bones
        if version >= MD20Version::TBCV1
            && version <= MD20Version::TBCV4
            && bone_header_array.count == 203
        {
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

        Ok(v_wow_collection!(
            reader,
            version,
            bone_header_array,
            |reader, item_header| {
                M2Bone {
                    data: reader.v_new_from_header(&item_header)?,
                    header: item_header,
                }
            }
        ))
    }
}

impl M2Bone {
    /// Check if this bone is a billboard
    pub fn is_billboard(&self) -> bool {
        self.header.flags.contains(M2BoneFlags::SPHERICAL_BILLBOARD)
            || self
                .header
                .flags
                .contains(M2BoneFlags::CYLINDRICAL_BILLBOARD_LOCK_X)
            || self
                .header
                .flags
                .contains(M2BoneFlags::CYLINDRICAL_BILLBOARD_LOCK_Y)
            || self
                .header
                .flags
                .contains(M2BoneFlags::CYLINDRICAL_BILLBOARD_LOCK_Z)
    }
}
