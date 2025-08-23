use wow_data::prelude::*;
use wow_data_derive::{WowHeaderR, WowHeaderW};

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
pub struct FrequencyDamping {
    pub frequency_hz: f32,
    pub damping_ratio: f32,
}

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
pub struct TorqueMode {
    pub max_torque: f32,
    pub mode: u32,
}
