use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{MagicStr, Mat3x4};
use wow_data::utils::string_to_inverted_magic;
use wow_data_derive::{WowHeaderR, WowHeaderW};

use crate::{M2Error, Result};

use super::common::{FrequencyDamping, TorqueMode};

pub const PRSJ: MagicStr = string_to_inverted_magic("PRSJ");
pub const PRS2: MagicStr = string_to_inverted_magic("PRS2");

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
            PRSJ => Self::V1,
            PRS2 => Self::V2,

            _ => {
                return Err(M2Error::ParseError(format!(
                    "Invalid prismatic joint magic: {:?}",
                    value
                )));
            }
        })
    }
}

impl From<Version> for MagicStr {
    fn from(value: Version) -> Self {
        match value {
            Version::V1 => PRSJ,
            Version::V2 => PRS2,
        }
    }
}

impl WowHeaderR for Version {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        let version: MagicStr = reader.wow_read()?;
        Ok(version.try_into()?)
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

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
#[wow_data(version = Version)]
pub struct JointPrismatic {
    pub frame_a: Mat3x4,
    pub frame_b: Mat3x4,
    pub lower_limit: f32,
    pub upper_limit: f32,
    pub _x68: f32,
    pub max_motor_force: f32,
    pub motor_tm: TorqueMode,
    #[wow_data(versioned)]
    pub motor_fd: VGTE2<FrequencyDamping>,
}
