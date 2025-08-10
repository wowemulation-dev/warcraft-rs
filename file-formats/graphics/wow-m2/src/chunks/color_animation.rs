use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::Color;
use wow_data_derive::{WowDataR, WowHeaderR, WowHeaderW};

use crate::chunks::animation::M2AnimationTrackHeader;
use crate::version::M2Version;

use super::animation::M2AnimationTrackData;

/// Color animation structure
#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub struct M2ColorAnimationHeader {
    #[wow_data(versioned)]
    pub color: M2AnimationTrackHeader<Color>,

    #[wow_data(versioned)]
    pub alpha: M2AnimationTrackHeader<u16>,
}

// #[derive(Debug, Clone, WowDataR)]
// #[wow_data(version = M2Version, header = M2ColorAnimationHeader)]
// #[wow_data(version = M2Version)]
#[derive(Debug, Clone)]
pub struct M2ColorAnimationData {
    // #[wow_data(versioned = true)]
    pub color: M2AnimationTrackData<Color>,
    pub alpha: M2AnimationTrackData<u16>,
}

#[derive(Debug, Clone)]
pub struct M2ColorAnimation {
    pub header: M2ColorAnimationHeader,
    pub data: M2ColorAnimationData,
}

// impl VWowDataR<M2Version, M2ColorAnimationHeader> for M2ColorAnimationData {
//     fn new_from_header<R: Read + Seek>(
//         reader: &mut R,
//         header: &M2ColorAnimationHeader,
//     ) -> WDResult<Self> {
//         Ok(Self {
//             color: reader.vnew_from_header(&header.color)?,
//             alpha: reader.vnew_from_header(&header.alpha)?,
//         })
//     }
// }
