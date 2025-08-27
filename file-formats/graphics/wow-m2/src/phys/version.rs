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
#[wow_data(ty=u16)]
pub enum PhysVersion {
    #[wow_data(lit = 0)]
    V0,
    #[wow_data(lit = 1)]
    V1,
    #[wow_data(lit = 2)]
    V2,
    #[wow_data(lit = 3)]
    V3,
    #[wow_data(lit = 4)]
    V4,
    #[wow_data(lit = 5)]
    V5,
    #[default]
    #[wow_data(lit = 6)]
    V6,
}

impl DataVersion for PhysVersion {}
