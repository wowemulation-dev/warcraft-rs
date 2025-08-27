use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data_derive::WowEnumFrom;

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, WowEnumFrom)]
#[wow_data(ty=u16)]
pub enum PhysVersion {
    #[wow_data(lit = 0)]
    V0,
    #[wow_data(lit = 1)]
    V1,
    #[wow_data(lit = 2)]
    V2,
    #[wow_data(lit = 3)]
    V3,
    #[wow_data(lit = 4)]
    V4,
    #[wow_data(lit = 5)]
    V5,
    #[default]
    #[wow_data(lit = 6)]
    V6,
}

impl DataVersion for PhysVersion {}

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
