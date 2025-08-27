use wow_data::prelude::*;
use wow_data::types::{C3Vector, MagicStr};
use wow_data::utils::string_to_inverted_magic;
use wow_data_derive::{WowEnumFrom, WowHeaderR, WowHeaderW};

pub const BODY: MagicStr = string_to_inverted_magic("BODY");
pub const BDY2: MagicStr = string_to_inverted_magic("BDY2");
pub const BDY3: MagicStr = string_to_inverted_magic("BDY3");
pub const BDY4: MagicStr = string_to_inverted_magic("BDY4");

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, WowEnumFrom)]
#[wow_data(from_type=MagicStr)]
pub enum Version {
    #[wow_data(ident=BODY)]
    V1,
    #[wow_data(ident=BDY2)]
    V2,
    #[wow_data(ident=BDY3)]
    V3,
    #[wow_data(ident=BDY4)]
    V4,
}

impl DataVersion for Version {}

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
        _unknown: f32,
    },
}

impl Default for ShapesBaseCount {
    fn default() -> Self {
        Self::V3 {
            count: 0,
            _unknown: 0.0,
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
    pub _x1c: VGTE2<f32>,
    #[wow_data(versioned)]
    pub drag: VGTE3<f32>,
    #[wow_data(versioned)]
    pub _x24: VGTE3<f32>,
    #[wow_data(versioned)]
    pub _x28: VGTE3<f32>,
    #[wow_data(versioned)]
    pub _x2c: VGTE4<[u8; 4]>,
}
