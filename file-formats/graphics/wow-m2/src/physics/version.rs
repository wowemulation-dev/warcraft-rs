use wow_data::error::Result as WDResult;
use wow_data::prelude::*;

use crate::{M2Error, Result};

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PhysVersion {
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    #[default]
    V6,
}

impl DataVersion for PhysVersion {}

impl TryFrom<u16> for PhysVersion {
    type Error = M2Error;

    fn try_from(value: u16) -> Result<Self> {
        Ok(match value {
            0 => Self::V0,
            1 => Self::V1,
            2 => Self::V2,
            3 => Self::V3,
            4 => Self::V4,
            5 => Self::V5,
            6 => Self::V6,
            _ => {
                return Err(M2Error::ParseError(format!(
                    "Invalid phys version: {}",
                    value
                )));
            }
        })
    }
}

impl From<PhysVersion> for u16 {
    fn from(value: PhysVersion) -> Self {
        match value {
            PhysVersion::V0 => 0,
            PhysVersion::V1 => 1,
            PhysVersion::V2 => 2,
            PhysVersion::V3 => 3,
            PhysVersion::V4 => 4,
            PhysVersion::V5 => 5,
            PhysVersion::V6 => 6,
        }
    }
}

impl WowHeaderR for PhysVersion {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        let version: u16 = reader.wow_read()?;
        Ok(version.try_into()?)
    }
}

impl WowHeaderW for PhysVersion {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        let version: u16 = (*self).into();
        writer.wow_write(&version)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        2
    }
}
