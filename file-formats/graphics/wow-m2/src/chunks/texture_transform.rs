use wow_data::prelude::*;
use wow_data::types::{C3Vector, Quaternion};
use wow_data_derive::{WowDataR, WowEnumFrom, WowHeaderR, WowHeaderW};

use crate::chunks::animation::M2AnimationTrackHeader;
use crate::version::MD20Version;

use super::animation::M2AnimationTrackData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, WowEnumFrom, WowHeaderR, WowHeaderW)]
#[wow_data(from_type=u16)]
pub enum M2TextureTransformType {
    #[wow_data(lit = 0)]
    None = 0,
    #[wow_data(lit = 1)]
    Scroll = 1,
    #[wow_data(lit = 2)]
    Rotate = 2,
    #[wow_data(lit = 3)]
    Scale = 3,
    #[wow_data(lit = 4)]
    Matrix = 4,
}

#[derive(Debug, Clone, Copy, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub enum M2TextureTransformIdType {
    #[wow_data(read_if = version >= MD20Version::BfAPlus)]
    Some {
        id: u32,
        transform_type: M2TextureTransformType,
    },
    None,
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
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
#[wow_data(version = MD20Version, header = M2TextureTransformHeader)]
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
