use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{MagicStr, Mat3x4};
use wow_data::utils::string_to_inverted_magic;
use wow_data_derive::{WowHeaderR, WowHeaderW};

use crate::physics::version::PhysVersion;
use crate::{M2Error, Result};

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
    type Error = M2Error;

    fn try_from(value: (PhysVersion, MagicStr)) -> Result<Self> {
        Ok(if value.0 <= PhysVersion::V1 {
            match value.1 {
                SHOJ => Self::V1,
                SHJ2 => Self::V3,

                _ => {
                    return Err(M2Error::ParseError(format!(
                        "Invalid shoulder joint magic: {:?}",
                        value
                    )));
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
                    )));
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

impl WowHeaderW for Version {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        let version: MagicStr = (*self).into();
        writer.wow_write(&version)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        4
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
