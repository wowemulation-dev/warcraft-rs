use wow_data::prelude::*;
use wow_data_derive::{WowEnumFrom, WowHeaderR, WowHeaderW};

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, WowHeaderR, WowHeaderW)]
    #[wow_data(bitflags=u16)]
    pub struct M2RenderFlags: u16 {
        const UNLIT = 0x01;
        const UNFOGGED = 0x02;
        const NO_BACKFACE_CULLING = 0x04;
        const NO_ZBUFFER = 0x08;
        const AFFECTED_BY_PROJECTION = 0x10;
        const DEPTH_TEST = 0x20;
        const DEPTH_WRITE = 0x40;
        const UNUSED = 0x80;
        const SHADOW_BATCH_1 = 0x100;
        const SHADOW_BATCH_2 = 0x200;
        const UNKNOWN_400 = 0x400;
        const UNKNOWN_800 = 0x800;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, WowHeaderR, WowHeaderW)]
    #[wow_data(bitflags=u16)]
    pub struct M2BlendMode: u16 {
        const OPAQUE = 0;
        const ALPHA_KEY = 1;
        const ALPHA = 2;
        const NO_ALPHA_ADD = 3;
        const ADD = 4;
        const MOD = 5;
        const MOD2X = 6;
        const BLEND_ADD = 7;
    }
}

/// Material texture uv transformations
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, WowEnumFrom, WowHeaderR, WowHeaderW)]
#[wow_data(ty=u16)]
pub enum M2TexTransformType {
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
    Stretch = 4,
    #[wow_data(lit = 5)]
    Camera = 5,
}

/// Represents a material layer (render flags) in an M2 model
/// This corresponds to the ModelRenderFlagsM2 structure in WMVx
#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
pub struct M2Material {
    pub flags: M2RenderFlags,
    pub blend_mode: M2BlendMode,
}

impl M2Material {
    /// Create a new material with default values
    pub fn new(blend_mode: M2BlendMode) -> Self {
        Self {
            flags: M2RenderFlags::DEPTH_TEST | M2RenderFlags::DEPTH_WRITE,
            blend_mode,
        }
    }
}
