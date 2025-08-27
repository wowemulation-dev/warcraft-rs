use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{C3Vector, Color};
use wow_data_derive::{WowDataR, WowEnumFrom, WowHeaderR, WowHeaderW};

use crate::chunks::animation::M2AnimationTrackHeader;
use crate::version::MD20Version;

use super::animation::M2AnimationTrackData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, WowEnumFrom)]
#[wow_data(ty=u16)]
pub enum M2LightType {
    /// Directional light (like the sun)
    #[wow_data(lit = 0)]
    Directional = 0,
    /// Point light (emits light in all directions)
    #[wow_data(lit = 1)]
    Point = 1,
    /// Spot light (emits light in a cone)
    #[wow_data(lit = 2)]
    Spot = 2,
    /// Ambient light (global illumination)
    #[wow_data(lit = 3)]
    Ambient = 3,
}

impl WowHeaderR for M2LightType {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        let value: u16 = reader.wow_read()?;
        Ok(value.try_into()?)
    }
}
impl WowHeaderW for M2LightType {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        let value: u16 = (*self).into();
        writer.wow_write(&value)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        2
    }
}

/// Represents a light in an M2 model
#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub struct M2LightHeader {
    pub light_type: M2LightType,
    /// Bone to attach the light to
    pub bone_index: u16,
    pub position: C3Vector,
    #[wow_data(versioned)]
    pub ambient_color_animation: M2AnimationTrackHeader<Color>,
    #[wow_data(versioned)]
    pub ambient_intensity: M2AnimationTrackHeader<f32>,
    #[wow_data(versioned)]
    pub diffuse_color_animation: M2AnimationTrackHeader<Color>,
    #[wow_data(versioned)]
    pub diffuse_intensity: M2AnimationTrackHeader<f32>,
    /// Attenuation start animation (where light begins to fade)
    #[wow_data(versioned)]
    pub attenuation_start_animation: M2AnimationTrackHeader<f32>,
    /// Attenuation end animation (where light fully fades)
    #[wow_data(versioned)]
    pub attenuation_end_animation: M2AnimationTrackHeader<f32>,
    #[wow_data(versioned)]
    pub visibility_animation: M2AnimationTrackHeader<u8>,
}

#[derive(Debug, Clone, WowDataR)]
#[wow_data(version = MD20Version, header = M2LightHeader)]
pub struct M2LightData {
    #[wow_data(versioned)]
    pub ambient_color_animation: M2AnimationTrackData<Color>,
    #[wow_data(versioned)]
    pub ambient_intensity: M2AnimationTrackData<f32>,
    #[wow_data(versioned)]
    pub diffuse_color_animation: M2AnimationTrackData<Color>,
    #[wow_data(versioned)]
    pub diffuse_intensity: M2AnimationTrackData<f32>,
    /// Attenuation start animation (where light begins to fade)
    #[wow_data(versioned)]
    pub attenuation_start_animation: M2AnimationTrackData<f32>,
    /// Attenuation end animation (where light fully fades)
    #[wow_data(versioned)]
    pub attenuation_end_animation: M2AnimationTrackData<f32>,
    #[wow_data(versioned)]
    pub visibility_animation: M2AnimationTrackData<u8>,
}

#[derive(Debug, Clone)]
pub struct M2Light {
    pub header: M2LightHeader,
    pub data: M2LightData,
}
