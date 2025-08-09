use wow_data::prelude::*;
use wow_data::types::Color;
use wow_data_derive::{VWowHeaderR, WowHeaderW};

use crate::chunks::animation::M2AnimationTrackHeader;
use crate::version::M2Version;

/// Color animation structure
#[derive(Debug, Clone, VWowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub struct M2ColorAnimation {
    #[wow_data(versioned)]
    pub color: M2AnimationTrackHeader<Color>,

    #[wow_data(versioned)]
    pub alpha: M2AnimationTrackHeader<u16>,
}
