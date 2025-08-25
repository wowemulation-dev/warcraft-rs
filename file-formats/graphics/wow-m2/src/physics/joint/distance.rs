use wow_data::prelude::*;
use wow_data::types::{C3Vector, MagicStr};
use wow_data::utils::string_to_inverted_magic;
use wow_data_derive::{WowHeaderR, WowHeaderW};

pub const DSTJ: MagicStr = string_to_inverted_magic("DSTJ");

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
pub struct JointDistance {
    pub anchor_a: C3Vector,
    pub anchor_b: C3Vector,
    pub factor: f32,
}
