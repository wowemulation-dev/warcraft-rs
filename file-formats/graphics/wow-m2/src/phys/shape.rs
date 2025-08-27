use wow_data::prelude::*;
use wow_data::types::{C3Vector, MagicStr, Mat3x4};
use wow_data::utils::string_to_inverted_magic;
use wow_data_derive::{WowEnumFrom, WowHeaderR, WowHeaderW};

pub const BOXS: MagicStr = string_to_inverted_magic("BOXS");
pub const CAPS: MagicStr = string_to_inverted_magic("CAPS");
pub const SPHS: MagicStr = string_to_inverted_magic("SPHS");
pub const SHAP: MagicStr = string_to_inverted_magic("SHAP");
pub const SHP2: MagicStr = string_to_inverted_magic("SHP2");

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, WowEnumFrom)]
#[wow_data(ty=MagicStr)]
pub enum Version {
    #[wow_data(ident=SHAP)]
    V1,
    #[default]
    #[wow_data(ident=SHP2)]
    V2,
}

impl DataVersion for Version {}

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, WowEnumFrom, WowHeaderR, WowHeaderW)]
#[wow_data(ty=u16)]
pub enum ShapeType {
    #[default]
    #[wow_data(lit = 0)]
    Box = 0,
    #[wow_data(lit = 1)]
    Capsule = 1,
    #[wow_data(lit = 2)]
    Sphere = 2,
    #[wow_data(lit = 3)]
    Polytope = 3,
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
