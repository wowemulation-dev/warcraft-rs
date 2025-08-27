use std::io::{Read, Seek, Write};
use wow_data::error::Result as WDResult;
use wow_data::prelude::*;

use wow_data::types::{DataVersion, WowHeaderR, WowHeaderW};
use wow_data_derive::WowEnumFrom;

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, WowEnumFrom)]
#[wow_data(ty=u32)]
pub enum MD20Version {
    #[wow_data(lit = 0x0, default)]
    Unknown,

    #[wow_data(lit = 0x0100)]
    ClassicV1,
    #[wow_data(lit = 0x0101)]
    ClassicV2,
    #[wow_data(lit = 0x0102)]
    ClassicV3,
    #[wow_data(lit = 0x0103)]
    ClassicV4,

    #[wow_data(lit = 0x0104)]
    TBCV1,
    #[wow_data(lit = 0x0105)]
    TBCV2,
    #[wow_data(lit = 0x0106)]
    TBCV3,
    #[wow_data(lit = 0x0107)]
    TBCV4,

    #[wow_data(lit = 0x0108)]
    WotLK,

    #[wow_data(lit = 0x0109)]
    Cataclysm,

    #[wow_data(lit = 0x0110)]
    MoPPlus,

    #[default]
    #[wow_data(lit = 0x0112)]
    BfAPlus,
}

impl MD20Version {
    /// Check if a direct conversion path exists between two versions
    pub fn has_direct_conversion_path(&self, target: &Self) -> bool {
        // Adjacent versions typically have direct conversion paths
        let self_ord = *self as usize;
        let target_ord = *target as usize;

        (self_ord as isize - target_ord as isize).abs() == 1
    }
}

impl std::fmt::Display for MD20Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl DataVersion for MD20Version {}

impl WowHeaderR for MD20Version {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        let version: u32 = reader.wow_read()?;
        Ok(MD20Version::try_from(version)?)
    }
}

impl WowHeaderW for MD20Version {
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
