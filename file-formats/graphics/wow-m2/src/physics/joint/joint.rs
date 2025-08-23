use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::MagicStr;
use wow_data_derive::{WowHeaderR, WowHeaderW};

use crate::{M2Error, Result};

pub const JOIN: MagicStr = *b"NIOJ";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum JointType {
    #[default]
    Spherical = 0,
    Shoulder = 1,
    Weld = 2,
    Revolute = 3,
    Prismatic = 4,
    Distance = 5,
}

impl TryFrom<u16> for JointType {
    type Error = M2Error;

    fn try_from(value: u16) -> Result<Self> {
        match value {
            0 => Ok(Self::Spherical),
            1 => Ok(Self::Shoulder),
            2 => Ok(Self::Weld),
            3 => Ok(Self::Revolute),
            4 => Ok(Self::Prismatic),
            5 => Ok(Self::Distance),
            _ => Err(M2Error::ParseError(format!(
                "Invalid joint type value: {}",
                value
            ))),
        }
    }
}

impl From<JointType> for u16 {
    fn from(value: JointType) -> Self {
        match value {
            JointType::Spherical => 0,
            JointType::Shoulder => 1,
            JointType::Weld => 2,
            JointType::Revolute => 3,
            JointType::Prismatic => 4,
            JointType::Distance => 5,
        }
    }
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
