use wow_data::prelude::*;
use wow_data::types::{MagicStr, Mat3x4};
use wow_data::utils::string_to_inverted_magic;
use wow_data_derive::{WowEnumFrom, WowHeaderR, WowHeaderW};

use super::common::{FrequencyDamping, TorqueMode};

pub const REVJ: MagicStr = string_to_inverted_magic("REVJ");
pub const REV2: MagicStr = string_to_inverted_magic("REV2");

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, WowEnumFrom)]
#[wow_data(from_type=MagicStr)]
pub enum Version {
    #[wow_data(ident = REVJ)]
    V1,
    #[default]
    #[wow_data(ident = REV2)]
    V2,
}

impl DataVersion for Version {}

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
pub struct JointRevolute {
    pub frame_a: Mat3x4,
    pub frame_b: Mat3x4,
    pub lower_angle: f32,
    pub upper_angle: f32,
    pub motor_tm: TorqueMode,
    #[wow_data(versioned)]
    pub motor_fd: VGTE2<FrequencyDamping>,
}
