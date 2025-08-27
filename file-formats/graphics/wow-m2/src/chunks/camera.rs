use crate::chunks::animation::M2AnimationTrackHeader;
use crate::version::MD20Version;
use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::C3Vector;
use wow_data_derive::{WowDataR, WowHeaderR, WowHeaderW};

use super::animation::{M2AnimationTrackData, M2SplineKey};

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, WowHeaderR, WowHeaderW)]
    #[wow_data(bitflags=u16)]
    pub struct M2CameraFlags: u16 {
        /// Camera uses custom UVs for positioning
        const CUSTOM_UV = 0x01;
        /// Auto-generated camera based on model
        const AUTO_GENERATED = 0x02;
        /// Camera is at global scene coordinates
        const GLOBAL_POSITION = 0x04;
    }
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub enum M2CameraFov {
    None,

    #[wow_data(read_if = version < MD20Version::Cataclysm)]
    Some(f32),
}

#[derive(Debug, Clone, WowHeaderW)]
#[wow_data(version = M2Version)]
pub enum M2CameraFovAnimationHeader {
    None,

    #[wow_data(read_if = version >= M2Version::Cataclysm)]
    Some(M2AnimationTrackHeader<M2SplineKey<f32>>),
}

impl VWowHeaderR<MD20Version> for M2CameraFovAnimationHeader {
    fn wow_read<R: Read + Seek>(reader: &mut R, version: MD20Version) -> WDResult<Self> {
        Ok(if version >= MD20Version::Cataclysm {
            Self::Some(reader.wow_read_versioned(version)?)
        } else {
            Self::None
        })
    }
}

#[derive(Debug, Clone)]
pub enum M2CameraFovAnimation {
    None,

    Some(M2AnimationTrackData<M2SplineKey<f32>>),
}

impl VWowDataR<MD20Version, M2CameraFovAnimationHeader> for M2CameraFovAnimation {
    fn new_from_header<R: Read + Seek>(
        reader: &mut R,
        header: &M2CameraFovAnimationHeader,
    ) -> WDResult<Self> {
        Ok(match header {
            M2CameraFovAnimationHeader::Some(header) => {
                Self::Some(reader.v_new_from_header(header)?)
            }
            M2CameraFovAnimationHeader::None => Self::None,
        })
    }
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub struct M2CameraHeader {
    pub camera_type: u32,
    /// Field of view (in radians)
    #[wow_data(versioned)]
    pub fov: M2CameraFov,

    pub far_clip: f32,
    pub near_clip: f32,

    #[wow_data(versioned)]
    pub position_animation: M2AnimationTrackHeader<M2SplineKey<C3Vector>>,
    pub position_base: C3Vector,

    #[wow_data(versioned)]
    pub target_position_animation: M2AnimationTrackHeader<M2SplineKey<C3Vector>>,
    pub target_position_base: C3Vector,

    #[wow_data(versioned)]
    pub roll_animation: M2AnimationTrackHeader<M2SplineKey<f32>>,

    #[wow_data(versioned)]
    pub fov_animation: M2CameraFovAnimationHeader,
}

#[derive(Debug, Clone, WowDataR)]
#[wow_data(version = MD20Version, header = M2CameraHeader)]
pub struct M2CameraData {
    #[wow_data(versioned)]
    pub position_animation: M2AnimationTrackData<M2SplineKey<C3Vector>>,

    #[wow_data(versioned)]
    pub target_position_animation: M2AnimationTrackData<M2SplineKey<C3Vector>>,

    #[wow_data(versioned)]
    pub roll_animation: M2AnimationTrackData<M2SplineKey<f32>>,

    #[wow_data(versioned)]
    pub fov_animation: M2CameraFovAnimation,
}

#[derive(Debug, Clone)]
pub struct M2Camera {
    pub header: M2CameraHeader,
    pub data: M2CameraData,
}
