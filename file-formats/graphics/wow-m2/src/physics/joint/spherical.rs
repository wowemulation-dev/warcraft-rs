use wow_data::prelude::*;
use wow_data::types::{C3Vector, MagicStr};
use wow_data::utils::string_to_inverted_magic;
use wow_data_derive::{WowHeaderR, WowHeaderW};

pub const SPHJ: MagicStr = string_to_inverted_magic("SPHJ");

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
pub struct JointSpherical {
    pub anchor_a: C3Vector,
    pub anchor_b: C3Vector,
    pub friction_torque: f32,
}
