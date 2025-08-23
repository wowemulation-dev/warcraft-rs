use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{MagicStr, Mat3x4};
use wow_data_derive::{WowHeaderR, WowHeaderW};

use crate::{M2Error, Result};

use super::common::FrequencyDamping;

pub const WELJ: MagicStr = *b"JLEW";
pub const WLJ2: MagicStr = *b"2JLW";
pub const WLJ3: MagicStr = *b"3JLW";

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Version {
    V1,
    V2,
    #[default]
    V3,
}

impl DataVersion for Version {}

impl TryFrom<MagicStr> for Version {
    type Error = M2Error;

    fn try_from(value: MagicStr) -> Result<Self> {
        Ok(match value {
            WELJ => Self::V1,
            WLJ2 => Self::V2,
            WLJ3 => Self::V3,

            _ => {
                return Err(M2Error::ParseError(format!(
                    "Invalid weld joint magic: {:?}",
                    value
                )));
            }
        })
    }
}

impl From<Version> for MagicStr {
    fn from(value: Version) -> Self {
        match value {
            Version::V1 => WELJ,
            Version::V2 => WLJ2,
            Version::V3 => WLJ3,
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
pub struct JointWeld {
    pub frame_a: Mat3x4,
    pub frame_b: Mat3x4,
    pub angular: FrequencyDamping,
    #[wow_data(versioned)]
    pub linear: VGTE2<FrequencyDamping>,
    #[wow_data(versioned)]
    pub _x70: VGTE3<f32>,
}
