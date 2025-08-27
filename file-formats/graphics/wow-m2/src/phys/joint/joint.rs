use wow_data::prelude::*;
use wow_data::types::MagicStr;
use wow_data::{error::Result as WDResult, utils::string_to_inverted_magic};
use wow_data_derive::{WowEnumFrom, WowHeaderR, WowHeaderW};

pub const JOIN: MagicStr = string_to_inverted_magic("JOIN");

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, WowEnumFrom)]
#[wow_data(ty=u16)]
pub enum JointType {
    #[default]
    #[wow_data(lit = 0)]
    Spherical = 0,
    #[wow_data(lit = 1)]
    Shoulder = 1,
    #[wow_data(lit = 2)]
    Weld = 2,
    #[wow_data(lit = 3)]
    Revolute = 3,
    #[wow_data(lit = 4)]
    Prismatic = 4,
    #[wow_data(lit = 5)]
    Distance = 5,
}

impl WowHeaderR for JointType {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        Ok(u16::wow_read(reader)?.try_into()?)
    }
}

impl WowHeaderW for JointType {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        u16::wow_write(&(*self).into(), writer)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        u16::wow_size(&(*self).into())
    }
}

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
pub struct Joint {
    pub body_a_idx: u32,
    pub body_b_idx: u32,
    pub _unknown: [u8; 4],
    pub joint_type: JointType,
    pub joint_id: i16,
}
