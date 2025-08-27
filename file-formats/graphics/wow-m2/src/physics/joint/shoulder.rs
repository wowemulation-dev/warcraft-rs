use wow_data::error::{Result as WDResult, WowDataError};
use wow_data::types::{ChunkHeader, MagicStr, Mat3x4, VersionedChunk};
use wow_data::utils::string_to_inverted_magic;
use wow_data::{prelude::*, v_read_chunk_items};
use wow_data_derive::{WowHeaderR, WowHeaderW};

use crate::M2Error;
use crate::physics::version::PhysVersion;

use super::common::{FrequencyDamping, TorqueMode};

pub const SHOJ: MagicStr = string_to_inverted_magic("SHOJ");
pub const SHJ2: MagicStr = string_to_inverted_magic("SHJ2");

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Version {
    V1,
    V2,
    #[default]
    V3,
}

impl DataVersion for Version {}

impl TryFrom<(PhysVersion, MagicStr)> for Version {
    type Error = WowDataError;

    fn try_from(value: (PhysVersion, MagicStr)) -> WDResult<Self> {
        Ok(if value.0 <= PhysVersion::V1 {
            match value.1 {
                SHOJ => Self::V1,
                SHJ2 => Self::V3,

                _ => {
                    return Err(M2Error::ParseError(format!(
                        "Invalid shoulder joint magic: {:?}",
                        value
                    ))
                    .into());
                }
            }
        } else {
            match value.1 {
                SHOJ => Self::V2,
                SHJ2 => Self::V3,

                _ => {
                    return Err(M2Error::ParseError(format!(
                        "Invalid shoulder joint magic: {:?}",
                        value
                    ))
                    .into());
                }
            }
        })
    }
}

impl From<Version> for MagicStr {
    fn from(value: Version) -> Self {
        match value {
            Version::V1 => SHOJ,
            Version::V2 => SHOJ,
            Version::V3 => SHJ2,
        }
    }
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = Version)]
pub enum VGTE2<T: Default + WowHeaderR + WowHeaderW> {
    None,

    #[wow_data(read_if = version >= Version::V2)]
    Some(T),
}

impl<T: Default + WowHeaderR + WowHeaderW> Default for VGTE2<T> {
    fn default() -> Self {
        Self::Some(T::default())
    }
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = Version)]
pub enum VGTE3<T: Default + WowHeaderR + WowHeaderW> {
    None,

    #[wow_data(read_if = version >= Version::V3)]
    Some(T),
}

impl<T: Default + WowHeaderR + WowHeaderW> Default for VGTE3<T> {
    fn default() -> Self {
        Self::Some(T::default())
    }
}

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
#[wow_data(version = Version)]
pub struct JointShoulder {
    pub frame_a: Mat3x4,
    pub frame_b: Mat3x4,
    pub lower_twist_angle: f32,
    pub upper_twist_angle: f32,
    pub cone_angle: f32,
    #[wow_data(versioned)]
    pub motor_tm: VGTE2<TorqueMode>,
    #[wow_data(versioned)]
    pub motor_fd: VGTE3<FrequencyDamping>,
}

impl JointShoulder {
    pub fn wow_read_from_chunk<R: Read + Seek>(
        reader: &mut R,
        phys_version: PhysVersion,
        chunk_header: &ChunkHeader,
    ) -> WDResult<VersionedChunk<Version, Self>> {
        let version: Version = (phys_version, chunk_header.magic).try_into()?;

        Ok(VersionedChunk {
            version,
            items: v_read_chunk_items!(reader, version, chunk_header, Self),
        })
    }
}
