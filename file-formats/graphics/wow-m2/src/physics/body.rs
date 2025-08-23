use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{C3Vector, MagicStr};
use wow_data_derive::{WowHeaderR, WowHeaderW};

use crate::{M2Error, Result};

pub const BODY: MagicStr = *b"YDOB";
pub const BDY2: MagicStr = *b"2YDB";
pub const BDY3: MagicStr = *b"3YDB";
pub const BDY4: MagicStr = *b"4YDB";

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Version {
    V1,
    V2,
    V3,
    #[default]
    V4,
}

impl DataVersion for Version {}

impl TryFrom<MagicStr> for Version {
    type Error = M2Error;

    fn try_from(value: MagicStr) -> Result<Self> {
        Ok(match value {
            BODY => Self::V1,
            BDY2 => Self::V2,
            BDY3 => Self::V3,
            BDY4 => Self::V4,

            _ => {
                return Err(M2Error::ParseError(format!(
                    "Invalid body magic: {:?}",
                    value
                )));
            }
        })
    }
}

impl From<Version> for MagicStr {
    fn from(value: Version) -> Self {
        match value {
            Version::V1 => BODY,
            Version::V2 => BDY2,
            Version::V3 => BDY3,
            Version::V4 => BDY4,
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
pub enum BoneIndex {
    Padding([u8; 2]),

    #[wow_data(read_if = version >= Version::V3)]
    Some(i16),
}

impl Default for BoneIndex {
    fn default() -> Self {
        Self::Some(0)
    }
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = Version)]
pub enum ShapeIndex {
    #[wow_data(read_if = version <= Version::V2)]
    ModelBoneIndex(i16),

    ShapeIndex(i16),
}

impl Default for ShapeIndex {
    fn default() -> Self {
        Self::ShapeIndex(0)
    }
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = Version)]
pub enum ShapesBaseCount {
    #[wow_data(read_if = version <= Version::V2)]
    V1 {
        base: i32,
        count: i32,
    },

    V3 {
        count: i32,
        unkown: f32,
    },
}

impl Default for ShapesBaseCount {
    fn default() -> Self {
        Self::V3 {
            count: 0,
            unkown: 0.0,
        }
    }
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = Version)]
pub enum VE1<T: Default + WowHeaderR + WowHeaderW> {
    None,

    #[wow_data(read_if = version == Version::V1)]
    Some(T),
}

impl<T: Default + WowHeaderR + WowHeaderW> Default for VE1<T> {
    fn default() -> Self {
        Self::Some(T::default())
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

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = Version)]
pub enum VGTE4<T: Default + WowHeaderR + WowHeaderW> {
    None,

    #[wow_data(read_if = version >= Version::V4)]
    Some(T),
}

impl<T: Default + WowHeaderR + WowHeaderW> Default for VGTE4<T> {
    fn default() -> Self {
        Self::Some(T::default())
    }
}

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
#[wow_data(version = Version)]
pub struct Body {
    pub body_type: u16,
    #[wow_data(versioned)]
    pub bone_index: BoneIndex,
    pub position: C3Vector,
    #[wow_data(versioned)]
    pub shape_index: ShapeIndex,
    pub padding_x12: [u8; 2],
    #[wow_data(versioned)]
    pub shape_count: ShapesBaseCount,
    #[wow_data(versioned)]
    pub _x1c_1: VE1<[u8; 4]>,
    #[wow_data(versioned)]
    pub _x1c_2: VGTE2<f32>,
    #[wow_data(versioned)]
    pub drag: VGTE3<f32>,
    #[wow_data(versioned)]
    pub _x24: VGTE3<f32>,
    #[wow_data(versioned)]
    pub _x28: VGTE3<f32>,
    #[wow_data(versioned)]
    pub _x2c: VGTE4<[u8; 4]>,
}
