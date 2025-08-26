use wow_data::prelude::*;
use wow_data::types::{MagicStr, Mat3x4};
use wow_data::utils::string_to_inverted_magic;
use wow_data_derive::{WowHeaderR, WowHeaderW};

use crate::{M2Error, Result};

use super::common::{FrequencyDamping, TorqueMode};

pub const REVJ: MagicStr = string_to_inverted_magic("REVJ");
pub const REV2: MagicStr = string_to_inverted_magic("REV2");

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Version {
    V1,
    #[default]
    V2,
}

impl DataVersion for Version {}

impl TryFrom<MagicStr> for Version {
    type Error = M2Error;

    fn try_from(value: MagicStr) -> Result<Self> {
        Ok(match value {
            REVJ => Self::V1,
            REV2 => Self::V2,

            _ => {
                return Err(M2Error::ParseError(format!(
                    "Invalid revolute joint magic: {:?}",
                    value
                )));
            }
        })
    }
}

impl From<Version> for MagicStr {
    fn from(value: Version) -> Self {
        match value {
            Version::V1 => REVJ,
            Version::V2 => REV2,
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

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
#[wow_data(version = Version)]
pub struct JointRevolute {
    pub frame_a: Mat3x4,
    pub frame_b: Mat3x4,
    pub lower_angle: f32,
    pub upper_angle: f32,
    pub motor_tm: TorqueMode,
    #[wow_data(versioned)]
    pub motor_fd: VGTE2<FrequencyDamping>,
}
