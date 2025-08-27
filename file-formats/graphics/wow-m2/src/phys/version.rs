use wow_data::prelude::*;
use wow_data_derive::{WowEnumFrom, WowHeaderR, WowHeaderW};

#[derive(
    Debug,
    Clone,
    Default,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    WowEnumFrom,
    WowHeaderR,
    WowHeaderW,
)]
#[wow_data(from_type=u16)]
pub enum PhysVersion {
    #[wow_data(expr = 0)]
    V0,
    #[wow_data(expr = 1)]
    V1,
    #[wow_data(expr = 2)]
    V2,
    #[wow_data(expr = 3)]
    V3,
    #[wow_data(expr = 4)]
    V4,
    #[wow_data(expr = 5)]
    V5,
    #[default]
    #[wow_data(expr = 6)]
    V6,
}

impl DataVersion for PhysVersion {}
