use wow_data::prelude::*;
use wow_data_derive::{WowDataR, WowHeaderR, WowHeaderW};

use crate::version::MD20Version;

use super::animation::{M2AnimationTrackData, M2AnimationTrackHeader};

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub struct M2TransparencyAnimationHeader {
    #[wow_data(versioned)]
    pub alpha: M2AnimationTrackHeader<u16>,
}

#[derive(Debug, Clone, WowDataR)]
#[wow_data(version = MD20Version, header = M2TransparencyAnimationHeader)]
pub struct M2TransparencyAnimationData {
    #[wow_data(versioned)]
    pub alpha: M2AnimationTrackData<u16>,
}

#[derive(Debug, Clone)]
pub struct M2TransparencyAnimation {
    pub header: M2TransparencyAnimationHeader,
    pub data: M2TransparencyAnimationData,
}
