use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{C3Vector, Quaternion};
use wow_data_derive::{WowDataR, WowHeaderR, WowHeaderW};

use crate::M2Error;
use crate::chunks::animation::M2AnimationTrackHeader;
use crate::error::Result;
use crate::version::M2Version;

use super::animation::M2AnimationTrackData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M2TextureTransformType {
    None = 0,
    Scroll = 1,
    Rotate = 2,
    Scale = 3,
    Matrix = 4,
}

impl TryFrom<u16> for M2TextureTransformType {
    type Error = M2Error;

    fn try_from(value: u16) -> Result<Self> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Scroll),
            2 => Ok(Self::Rotate),
            3 => Ok(Self::Scale),
            4 => Ok(Self::Matrix),
            _ => Err(M2Error::UnsupportedNumericVersion(value as u32)),
        }
    }
}

impl From<M2TextureTransformType> for u16 {
    fn from(value: M2TextureTransformType) -> Self {
        match value {
            M2TextureTransformType::None => 0,
            M2TextureTransformType::Scroll => 1,
            M2TextureTransformType::Rotate => 2,
            M2TextureTransformType::Scale => 3,
            M2TextureTransformType::Matrix => 4,
        }
    }
}

impl WowHeaderR for M2TextureTransformType {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        let value: u16 = reader.wow_read()?;
        Ok(value.try_into()?)
    }
}
impl WowHeaderW for M2TextureTransformType {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        let value: u16 = (*self).into();
        writer.wow_write(&value)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        2
    }
}

#[derive(Debug, Clone, Copy, WowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub enum M2TextureTransformIdType {
    #[wow_data(read_if = version >= M2Version::BfAPlus)]
    Some {
        id: u32,
        transform_type: M2TextureTransformType,
    },
    None,
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub struct M2TextureTransformHeader {
    #[wow_data(versioned)]
    pub id_type: M2TextureTransformIdType,

    #[wow_data(versioned)]
    pub translation: M2AnimationTrackHeader<C3Vector>,

    #[wow_data(versioned)]
    pub rotation: M2AnimationTrackHeader<Quaternion>,

    #[wow_data(versioned)]
    pub scaling: M2AnimationTrackHeader<C3Vector>,
}

#[derive(Debug, Clone, WowDataR)]
#[wow_data(version = M2Version, header = M2TextureTransformHeader)]
pub struct M2TextureTransformData {
    #[wow_data(versioned)]
    pub translation: M2AnimationTrackData<C3Vector>,

    #[wow_data(versioned)]
    pub rotation: M2AnimationTrackData<Quaternion>,

    #[wow_data(versioned)]
    pub scaling: M2AnimationTrackData<C3Vector>,
}

#[derive(Debug, Clone)]
pub struct M2TextureTransform {
    pub header: M2TextureTransformHeader,
    pub data: M2TextureTransformData,
}
