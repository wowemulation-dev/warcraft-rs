use wow_data::prelude::*;
use wow_data::types::{C3Vector, MagicStr};
use wow_data_derive::{WowHeaderR, WowHeaderW};

pub const DSTJ: MagicStr = *b"JTSD";

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
pub struct JointDistance {
    pub anchor_a: C3Vector,
    pub anchor_b: C3Vector,
    pub factor: f32,
}
