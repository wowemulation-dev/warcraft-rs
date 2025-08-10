use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{C3Vector, Color};
use wow_data_derive::{WowDataR, WowHeaderR, WowHeaderW};

use crate::M2Error;
use crate::chunks::animation::M2AnimationTrackHeader;
use crate::error::Result;
use crate::version::M2Version;

use super::animation::M2AnimationTrackData;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct M2LightFlags: u16 {
        /// Light is directional (otherwise it's a point light)
        const DIRECTIONAL = 0x01;
        /// Unknown flag from Blood Elf "BE_hairSynthesizer.m2"
        const UNKNOWN_BE_HAIR = 0x02;
    }
}

impl WowHeaderR for M2LightFlags {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        Ok(Self::from_bits_retain(reader.wow_read()?))
    }
}
impl WowHeaderW for M2LightFlags {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        writer.wow_write(&self.bits())?;
        Ok(())
    }
    fn wow_size(&self) -> usize {
        2
    }
}

/// Light type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M2LightType {
    /// Directional light (like the sun)
    Directional = 0,
    /// Point light (emits light in all directions)
    Point = 1,
    /// Spot light (emits light in a cone)
    Spot = 2,
    /// Ambient light (global illumination)
    Ambient = 3,
}

impl TryFrom<u16> for M2LightType {
    type Error = M2Error;

    fn try_from(value: u16) -> Result<Self> {
        match value {
            0 => Ok(Self::Directional),
            1 => Ok(Self::Point),
            2 => Ok(Self::Spot),
            3 => Ok(Self::Ambient),
            _ => Err(M2Error::UnsupportedNumericVersion(value as u32)),
        }
    }
}

impl From<M2LightType> for u16 {
    fn from(value: M2LightType) -> Self {
        match value {
            M2LightType::Directional => 0,
            M2LightType::Point => 1,
            M2LightType::Spot => 2,
            M2LightType::Ambient => 3,
        }
    }
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
#[wow_data(version = M2Version)]
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
    // /// Light ID
    // pub id: u32,
    // /// Light flags
    // pub flags: M2LightFlags,
}

#[derive(Debug, Clone, WowDataR)]
#[wow_data(version = M2Version, header = M2LightHeader)]
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
