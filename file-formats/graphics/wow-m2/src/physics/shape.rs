use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{C3Vector, MagicStr, Mat3x4};
use wow_data_derive::{WowHeaderR, WowHeaderW};

use crate::{M2Error, Result};

pub const BOXS: MagicStr = *b"SXOB";
pub const CAPS: MagicStr = *b"SPAC";
pub const SPHS: MagicStr = *b"SHPS";
pub const SHAP: MagicStr = *b"PAHS";
pub const SHP2: MagicStr = *b"2PHS";

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
            SHAP => Self::V1,
            SHP2 => Self::V2,

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
            Version::V1 => SHAP,
            Version::V2 => SHP2,
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

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
pub struct ShapeBox {
    pub a: Mat3x4,
    pub c: C3Vector,
}

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
pub struct ShapeCapsule {
    pub local_pos1: C3Vector,
    pub local_pos2: C3Vector,
    pub radius: f32,
}

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
pub struct ShapeSphere {
    pub local_pos: C3Vector,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ShapeType {
    #[default]
    Box = 0,
    Capsule = 1,
    Sphere = 2,
    Polytope = 3,
}

impl TryFrom<u16> for ShapeType {
    type Error = M2Error;

    fn try_from(value: u16) -> Result<Self> {
        match value {
            0 => Ok(Self::Box),
            1 => Ok(Self::Capsule),
            2 => Ok(Self::Sphere),
            3 => Ok(Self::Polytope),
            _ => Err(M2Error::ParseError(format!(
                "Invalid shape type value: {}",
                value
            ))),
        }
    }
}

impl From<ShapeType> for u16 {
    fn from(value: ShapeType) -> Self {
        match value {
            ShapeType::Box => 0,
            ShapeType::Capsule => 1,
            ShapeType::Sphere => 2,
            ShapeType::Polytope => 3,
        }
    }
}

impl WowHeaderR for ShapeType {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        Ok(u16::wow_read(reader)?.try_into()?)
    }
}

impl WowHeaderW for ShapeType {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        u16::wow_write(&(*self).into(), writer)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        u16::wow_size(&(*self).into())
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
pub struct Shape {
    pub shape_type: ShapeType,
    pub index: i16,
    pub _x04: [u8; 4],
    pub friction: f32,
    pub restitution: f32,
    pub density: f32,
    #[wow_data(versioned)]
    pub _x14: VGTE2<u32>,
    #[wow_data(versioned)]
    pub _x18: VGTE2<f32>,
    #[wow_data(versioned)]
    pub _x1c: VGTE2<u16>,
    #[wow_data(versioned)]
    pub _x1e: VGTE2<[u8; 2]>,
}
