use crate::MD20Version;
use crate::chunks::animation::M2AnimationTrackHeader;
use wow_data::prelude::*;
use wow_data::types::{C2Vector, ColorA, VectorFp6_9, WowArray, WowCharArray};
use wow_data::{error::Result as WDResult, types::C3Vector};
use wow_data_derive::{WowDataR, WowEnumFrom, WowHeaderR, WowHeaderW};

use super::animation::{
    M2AnimationTrackData, M2Box, M2FakeAnimationBlockData, M2FakeAnimationBlockHeader, M2Range,
};

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, WowHeaderR, WowHeaderW)]
    #[wow_data(bitflags=u32)]
    pub struct M2ParticleFlags: u32 {
        /// Particles are billboarded
        const BILLBOARDED = 0x00000008;
        /// Particles stretch based on their velocity
        const AFFECTED_BY_VELOCITY = 0x00000010;
        /// Particles rotate around their central point
        const ROTATING = 0x00000020;
        /// Particles use random texture coordinate generation
        const RANDOMIZED = 0x00000040;
        /// Particles use tiling
        const TILED = 0x00000080;
        /// ModelParticleEmitterType::Plane should be treated as ModelParticleEmitterType::Sphere
        const SPHERE_AS_SOURCE = 0x00000100;
        /// The center of the sphere should be used as the source of the particles
        const USE_SPHERE_CENTER = 0x00000200;
        /// Enable lighting for particles
        const LIGHTING = 0x00000400;
        /// Use a Z-buffer test for particles
        const ZBUFFER_TEST = 0x00000800;
        /// Use particle bounds for culling
        const BOUND_TO_EMITTER = 0x00001000;
        /// Particles follow their emitter
        const FOLLOW_EMITTER = 0x00002000;
        /// Unknown, used in the Deeprun Tram subway
        const UNKNOWN_0x4000 = 0x00004000;
        /// Unknown, used in the Deeprun Tram subway
        const UNKNOWN_0x8000 = 0x00008000;
        /// Unknown, used in the character display window
        const UNKNOWN_0x10000 = 0x00010000;
        /// Random spawn position
        const RANDOM_SPAWN_POSITION = 0x00020000;
        /// Particles stretch based on particle size
        const PINNED = 0x00040000;
        /// Use XYZ rotation instead of just Z
        const XYZ_ROTATION = 0x00080000;
        /// Unknown, was added in WoD (6.x)
        const UNKNOWN_WOD = 0x00100000;
        /// Use physics settings for particles
        const PHYSICS = 0x00200000;
        /// Pinned on the Y axis instead of both XY
        const FIXED_Y = 0x00400000;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, WowEnumFrom)]
#[wow_data(from_type=u8)]
pub enum M2ParticleEmitterType {
    /// Point emitter (particles spawn from a single point)
    #[wow_data(expr = 0)]
    Point = 0,
    /// Plane emitter (particles spawn within a 2D plane)
    #[wow_data(expr = 1)]
    Plane = 1,
    /// Sphere emitter (particles spawn within a 3D sphere)
    #[wow_data(expr = 2)]
    Sphere = 2,
    /// Spline emitter (particles follow a spline path)
    #[wow_data(expr = 3)]
    Spline = 3,
    /// Bone emitter (particles spawn from a bone)
    #[wow_data(expr = 4)]
    Bone = 4,
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub enum M2ParticleEmitterBlending {
    Classic {
        blending_type: u16,
        emitter_type: u16,
    },

    #[wow_data(read_if = version >= MD20Version::TBCV1)]
    Later {
        blending_type: u8,
        emitter_type: u8,
        particle_color_index: u16,
    },
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub enum M2ParticleEmitterMultiTextureParam {
    PreCata {
        particle_type: u8,
        head_or_tail: u8,
    },

    #[wow_data(read_if = version >= MD20Version::Cataclysm)]
    AfterCata(u8),
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub enum M2ParticleEmitterLifespanVary {
    None,

    #[wow_data(read_if = version >= MD20Version::WotLK)]
    Some(f32),
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub enum M2ParticleEmitterEmissionRateVary {
    None,

    #[wow_data(read_if = version >= MD20Version::WotLK)]
    Some(f32),
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub enum M2ParticleEmitterColorAnimationHeader {
    UpToTbc {
        mid_point: f32,
        color_values: [ColorA; 3],
        scale_values: [f32; 3],
        decay_uv_animation: [u16; 3],
        tail_uv_animation: [i16; 2],
        tail_decay_uv_animation: [i16; 2],
    },

    #[wow_data(read_if = version >= MD20Version::WotLK)]
    Later {
        color_animation: M2FakeAnimationBlockHeader<C3Vector>,
        alpha_animation: M2FakeAnimationBlockHeader<u16>,
        scale_animation: M2FakeAnimationBlockHeader<C2Vector>,
        scale_vary: C2Vector,
        head_cell_animation: M2FakeAnimationBlockHeader<u16>,
        tail_cell_animation: M2FakeAnimationBlockHeader<u16>,
    },
}

#[derive(Debug, Clone)]
pub enum M2ParticleEmitterColorAnimation {
    UpToTbc,
    Later {
        color_animation: M2FakeAnimationBlockData<C3Vector>,
        alpha_animation: M2FakeAnimationBlockData<u16>,
        scale_animation: M2FakeAnimationBlockData<C2Vector>,
        head_cell_animation: M2FakeAnimationBlockData<u16>,
        tail_cell_animation: M2FakeAnimationBlockData<u16>,
    },
}

impl VWowDataR<MD20Version, M2ParticleEmitterColorAnimationHeader>
    for M2ParticleEmitterColorAnimation
{
    fn new_from_header<R: Read + Seek>(
        reader: &mut R,
        header: &M2ParticleEmitterColorAnimationHeader,
    ) -> WDResult<Self> {
        Ok(match header {
            M2ParticleEmitterColorAnimationHeader::Later {
                color_animation,
                alpha_animation,
                scale_animation,
                scale_vary: _,
                head_cell_animation,
                tail_cell_animation,
            } => Self::Later {
                color_animation: reader.new_from_header(color_animation)?,
                alpha_animation: reader.new_from_header(alpha_animation)?,
                scale_animation: reader.new_from_header(scale_animation)?,
                head_cell_animation: reader.new_from_header(head_cell_animation)?,
                tail_cell_animation: reader.new_from_header(tail_cell_animation)?,
            },
            M2ParticleEmitterColorAnimationHeader::UpToTbc {
                mid_point: _,
                color_values: _,
                scale_values: _,
                decay_uv_animation: _,
                tail_uv_animation: _,
                tail_decay_uv_animation: _,
            } => Self::UpToTbc,
        })
    }
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub enum M2ParticleEmitterSpin {
    UpToTbc {
        spin: f32,
    },

    #[wow_data(read_if = version >= MD20Version::WotLK)]
    Later {
        base_spin: f32,
        base_spin_vary: f32,
        spin: f32,
        spin_vary: f32,
    },
}

/// Represents a particle emitter in an M2 model
#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub struct M2ParticleEmitterOldHeader {
    pub id: u32,
    pub flags: M2ParticleFlags,
    pub position: C3Vector,
    pub bone_index: u16,
    pub texture_index: u16,
    pub model_filename: WowCharArray,
    pub recursion_model_filename: WowCharArray,
    #[wow_data(versioned)]
    pub blending_type: M2ParticleEmitterBlending,
    #[wow_data(versioned)]
    pub multi_texture_param: M2ParticleEmitterMultiTextureParam,
    pub texture_tile_rotation: u16,
    pub texture_dimension_rows: u16,
    pub texture_dimension_cols: u16,
    #[wow_data(versioned)]
    pub emission_speed: M2AnimationTrackHeader<f32>,
    #[wow_data(versioned)]
    pub speed_variation: M2AnimationTrackHeader<f32>,
    #[wow_data(versioned)]
    pub vertical_range: M2AnimationTrackHeader<f32>,
    #[wow_data(versioned)]
    pub horizontal_range: M2AnimationTrackHeader<f32>,
    #[wow_data(versioned)]
    pub gravity: M2AnimationTrackHeader<f32>,
    #[wow_data(versioned)]
    pub lifespan: M2AnimationTrackHeader<f32>,
    #[wow_data(versioned)]
    pub lifespan_vary: M2ParticleEmitterLifespanVary,
    #[wow_data(versioned)]
    pub emission_rate: M2AnimationTrackHeader<f32>,
    #[wow_data(versioned)]
    pub emission_rate_vary: M2ParticleEmitterEmissionRateVary,
    #[wow_data(versioned)]
    pub emission_area_length: M2AnimationTrackHeader<f32>,
    #[wow_data(versioned)]
    pub emission_area_width: M2AnimationTrackHeader<f32>,
    #[wow_data(versioned)]
    pub zsource: M2AnimationTrackHeader<f32>,
    #[wow_data(versioned)]
    pub color_animation: M2ParticleEmitterColorAnimationHeader,
    pub tail_length: f32,
    pub twinkle_speed: f32,
    pub twinkle_percent: f32,
    pub twinkle_scale: M2Range,
    pub burst_multiplier: f32,
    pub drag: f32,
    #[wow_data(versioned)]
    pub spin: M2ParticleEmitterSpin,
    pub tumble: M2Box,
    pub wind_vector: C3Vector,
    pub wind_time: f32,
    pub follow_speed_1: f32,
    pub follow_scale_1: f32,
    pub follow_speed_2: f32,
    pub follow_scale_2: f32,
    pub spline_points: WowArray<C3Vector>,
    #[wow_data(versioned)]
    pub enabled_in: M2AnimationTrackHeader<u8>,
}

#[derive(Debug, Clone, WowDataR)]
#[wow_data(version = MD20Version, header = M2ParticleEmitterOldHeader)]
pub struct M2ParticleEmitterOldData {
    pub model_filename: String,
    pub recursion_model_filename: String,
    #[wow_data(versioned)]
    pub emission_speed: M2AnimationTrackData<f32>,
    #[wow_data(versioned)]
    pub speed_variation: M2AnimationTrackData<f32>,
    #[wow_data(versioned)]
    pub vertical_range: M2AnimationTrackData<f32>,
    #[wow_data(versioned)]
    pub horizontal_range: M2AnimationTrackData<f32>,
    #[wow_data(versioned)]
    pub gravity: M2AnimationTrackData<f32>,
    #[wow_data(versioned)]
    pub lifespan: M2AnimationTrackData<f32>,
    #[wow_data(versioned)]
    pub emission_rate: M2AnimationTrackData<f32>,
    #[wow_data(versioned)]
    pub emission_area_length: M2AnimationTrackData<f32>,
    #[wow_data(versioned)]
    pub emission_area_width: M2AnimationTrackData<f32>,
    #[wow_data(versioned)]
    pub zsource: M2AnimationTrackData<f32>,
    #[wow_data(versioned)]
    pub color_animation: M2ParticleEmitterColorAnimation,
    pub spline_points: Vec<C3Vector>,
    #[wow_data(versioned)]
    pub enabled_in: M2AnimationTrackData<u8>,
}

#[derive(Debug, Clone)]
pub struct M2ParticleEmitterOld {
    pub header: M2ParticleEmitterOldHeader,
    pub data: M2ParticleEmitterOldData,
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub struct M2ParticleEmitterNewHeader {
    #[wow_data(versioned)]
    pub old_particle: M2ParticleEmitterOldHeader,
    pub multi_texture_param_0: [VectorFp6_9; 2],
    pub multi_texture_param_1: [VectorFp6_9; 2],
}

#[derive(Debug, Clone, WowDataR)]
#[wow_data(version = MD20Version, header = M2ParticleEmitterNewHeader)]
pub struct M2ParticleEmitterNewData {
    #[wow_data(versioned)]
    pub old_particle: M2ParticleEmitterOldData,
}

#[derive(Debug, Clone, WowHeaderW)]
#[wow_data(version = M2Version)]
pub enum M2ParticleEmitterHeader {
    PreCata(M2ParticleEmitterOldHeader),

    #[wow_data(read_if = version >= M2Version::Cataclysm)]
    PostCata(M2ParticleEmitterNewHeader),
}

impl VWowHeaderR<MD20Version> for M2ParticleEmitterHeader {
    fn wow_read<R: Read + Seek>(reader: &mut R, version: MD20Version) -> WDResult<Self> {
        Ok(if version >= MD20Version::Cataclysm {
            Self::PostCata(reader.wow_read_versioned(version)?)
        } else {
            Self::PreCata(reader.wow_read_versioned(version)?)
        })
    }
}

#[derive(Debug, Clone)]
pub enum M2ParticleEmitter {
    PreCata(M2ParticleEmitterOldData),
    PostCata(M2ParticleEmitterNewData),
}

impl VWowDataR<MD20Version, M2ParticleEmitterHeader> for M2ParticleEmitter {
    fn new_from_header<R: Read + Seek>(
        reader: &mut R,
        header: &M2ParticleEmitterHeader,
    ) -> WDResult<Self> {
        Ok(match header {
            M2ParticleEmitterHeader::PreCata(header) => {
                Self::PreCata(reader.v_new_from_header(header)?)
            }
            M2ParticleEmitterHeader::PostCata(header) => {
                Self::PostCata(reader.v_new_from_header(header)?)
            }
        })
    }
}
