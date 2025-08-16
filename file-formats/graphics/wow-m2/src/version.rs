use std::io::{Read, Seek, Write};
use wow_data::error::Result as WDResult;
use wow_data::prelude::*;

use wow_data::types::{DataVersion, WowHeaderR, WowHeaderW};

use crate::{M2Error, error::Result};

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum M2Version {
    ClassicV1,
    ClassicV2,
    ClassicV3,
    ClassicV4,

    TBCV1,
    TBCV2,
    TBCV3,
    TBCV4,

    WotLK,

    Cataclysm,

    MoPPlus,

    #[default]
    BfAPlus,
}

impl M2Version {
    pub fn try_from_header_version(version: u32) -> Option<Self> {
        match version {
            0x0100 => Some(Self::ClassicV1),
            0x0101 => Some(Self::ClassicV2),
            0x0102 => Some(Self::ClassicV3),
            0x0103 => Some(Self::ClassicV4),
            0x0104 => Some(Self::TBCV1),
            0x0105 => Some(Self::TBCV2),
            0x0106 => Some(Self::TBCV3),
            0x0107 => Some(Self::TBCV4),
            0x0108 => Some(Self::WotLK),
            0x0109 => Some(Self::Cataclysm),
            0x0110 => Some(Self::MoPPlus), // Also used in newest Cataclysm client
            // 0x0111 => Some(Self::), // haven't seen this version yet
            0x0112 => Some(Self::BfAPlus),
            _ => None,
        }
    }

    pub fn from_header_version(version: u32) -> Result<Self> {
        Self::try_from_header_version(version)
            .ok_or_else(|| M2Error::UnsupportedNumericVersion(version))
    }

    pub fn to_header_version(&self) -> u32 {
        match self {
            Self::ClassicV1 => 0x0100,
            Self::ClassicV2 => 0x0101,
            Self::ClassicV3 => 0x0102,
            Self::ClassicV4 => 0x0103,
            Self::TBCV1 => 0x0104,
            Self::TBCV2 => 0x0105,
            Self::TBCV3 => 0x0106,
            Self::TBCV4 => 0x0107,
            Self::WotLK => 0x0108,
            Self::Cataclysm => 0x0109,
            Self::MoPPlus => 0x0110,
            Self::BfAPlus => 0x0112,
        }
    }

    /// Check if a direct conversion path exists between two versions
    pub fn has_direct_conversion_path(&self, target: &Self) -> bool {
        // Adjacent versions typically have direct conversion paths
        let self_ord = *self as usize;
        let target_ord = *target as usize;

        (self_ord as isize - target_ord as isize).abs() == 1
    }
}

impl std::fmt::Display for M2Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<u32> for M2Version {
    type Error = M2Error;

    fn try_from(value: u32) -> std::result::Result<Self, Self::Error> {
        M2Version::from_header_version(value)
    }
}

impl From<M2Version> for u32 {
    fn from(value: M2Version) -> Self {
        value.to_header_version()
    }
}

impl DataVersion for M2Version {}

impl WowHeaderR for M2Version {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        let version: u32 = reader.wow_read()?;
        Ok(M2Version::from_header_version(version)?)
    }
}

impl WowHeaderW for M2Version {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        let version: u32 = (*self).into();
        writer.wow_write(&version)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        4
    }
}

#[cfg(test)]
mod tests {}
