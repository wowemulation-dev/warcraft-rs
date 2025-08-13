use crate::chunks::animation::M2AnimationTrackHeader;
use crate::version::M2Version;
use wow_data::prelude::*;
use wow_data::types::{C3Vector, Color, WowArray};
use wow_data_derive::{WowDataR, WowHeaderR, WowHeaderW};

use super::animation::M2AnimationTrackData;

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub enum M2RibbonEmitterRest {
    None,

    #[wow_data(read_if = version >= M2Version::WotLK)]
    Some {
        priority_plane: u16,
        ribbon_color_index: u8,
        texture_transform_lookup: u8,
    },
}

/// Represents a ribbon emitter in an M2 model
#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub struct M2RibbonEmitterHeader {
    pub id: u32,
    pub bone_index: u32,
    pub position: C3Vector,
    pub texture_indices: WowArray<u16>,
    pub material_indices: WowArray<u16>,
    #[wow_data(versioned)]
    pub color_animation: M2AnimationTrackHeader<Color>,
    #[wow_data(versioned)]
    pub alpha_animation: M2AnimationTrackHeader<u16>,
    #[wow_data(versioned)]
    pub height_above_animation: M2AnimationTrackHeader<f32>,
    #[wow_data(versioned)]
    pub height_below_animation: M2AnimationTrackHeader<f32>,
    pub edges_per_second: f32,
    pub edge_lifetime: f32,
    pub gravity: f32,
    pub texture_rows: u16,
    pub texture_cols: u16,
    #[wow_data(versioned)]
    pub texture_slot_animation: M2AnimationTrackHeader<u16>,
    #[wow_data(versioned)]
    pub visibility_animation: M2AnimationTrackHeader<u8>,
    #[wow_data(versioned)]
    pub rest: M2RibbonEmitterRest,
}

#[derive(Debug, Clone, WowDataR)]
#[wow_data(version = M2Version, header = M2RibbonEmitterHeader)]
pub struct M2RibbonEmitterData {
    pub texture_indices: Vec<u16>,
    pub material_indices: Vec<u16>,
    #[wow_data(versioned)]
    pub color_animation: M2AnimationTrackData<Color>,
    #[wow_data(versioned)]
    pub alpha_animation: M2AnimationTrackData<u16>,
    #[wow_data(versioned)]
    pub height_above_animation: M2AnimationTrackData<f32>,
    #[wow_data(versioned)]
    pub height_below_animation: M2AnimationTrackData<f32>,
    #[wow_data(versioned)]
    pub texture_slot_animation: M2AnimationTrackData<u16>,
    #[wow_data(versioned)]
    pub visibility_animation: M2AnimationTrackData<u8>,
}

#[derive(Debug, Clone)]
pub struct M2RibbonEmitter {
    pub header: M2RibbonEmitterHeader,
    pub data: M2RibbonEmitterData,
}
