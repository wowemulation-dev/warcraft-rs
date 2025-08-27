use wow_data::prelude::*;
use wow_data_derive::{WowEnumFrom, WowHeaderR, WowHeaderW};

use crate::chunks::animation::M2AnimationTrackHeader;
use crate::version::MD20Version;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, WowEnumFrom, WowHeaderR, WowHeaderW)]
#[wow_data(from_type=u16)]
pub enum M2TextureAnimationType {
    /// No animation
    #[default]
    #[wow_data(lit = 0)]
    None = 0,
    #[wow_data(lit = 1)]
    Scroll = 1,
    #[wow_data(lit = 2)]
    Rotate = 2,
    #[wow_data(lit = 3)]
    Scale = 3,
    #[wow_data(lit = 4)]
    KeyFrame = 4,
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub struct M2TextureAnimation {
    pub animation_type: M2TextureAnimationType,
    /// Animation for U coordinate
    #[wow_data(versioned)]
    pub translation_u: M2AnimationTrackHeader<f32>,
    /// Animation for V coordinate
    #[wow_data(versioned)]
    pub translation_v: M2AnimationTrackHeader<f32>,
    #[wow_data(versioned)]
    pub rotation: M2AnimationTrackHeader<f32>,
    #[wow_data(versioned)]
    pub scale_u: M2AnimationTrackHeader<f32>,
    #[wow_data(versioned)]
    pub scale_v: M2AnimationTrackHeader<f32>,
}

impl M2TextureAnimation {
    /// Create a new texture animation with default values
    pub fn new(animation_type: M2TextureAnimationType) -> Self {
        Self {
            animation_type,
            translation_u: M2AnimationTrackHeader::new(),
            translation_v: M2AnimationTrackHeader::new(),
            rotation: M2AnimationTrackHeader::new(),
            scale_u: M2AnimationTrackHeader::new(),
            scale_v: M2AnimationTrackHeader::new(),
        }
    }
}
