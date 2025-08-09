use crate::M2Version;
use crate::chunks::animation::M2AnimationTrackHeader;
use wow_data::prelude::*;
use wow_data::types::{C2Vector, ColorA, VectorFp6_9, WowArray, WowCharArray};
use wow_data::{error::Result as WDResult, types::C3Vector};
use wow_data_derive::{WowHeaderRV, WowHeaderW};

use super::animation::{M2Box, M2FakeAnimationBlock, M2Range};

bitflags::bitflags! {
    /// Particle flags as defined in the M2 format
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl WowHeaderR for M2ParticleFlags {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        Ok(Self::from_bits_retain(reader.wow_read()?))
    }
}
impl WowHeaderW for M2ParticleFlags {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        writer.wow_write(&self.bits())?;
        Ok(())
    }
    fn wow_size(&self) -> usize {
        4
    }
}

/// Particle emitter type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M2ParticleEmitterType {
    /// Point emitter (particles spawn from a single point)
    Point = 0,
    /// Plane emitter (particles spawn within a 2D plane)
    Plane = 1,
    /// Sphere emitter (particles spawn within a 3D sphere)
    Sphere = 2,
    /// Spline emitter (particles follow a spline path)
    Spline = 3,
    /// Bone emitter (particles spawn from a bone)
    Bone = 4,
}

impl M2ParticleEmitterType {
    /// Parse from integer value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Point),
            1 => Some(Self::Plane),
            2 => Some(Self::Sphere),
            3 => Some(Self::Spline),
            4 => Some(Self::Bone),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, WowHeaderRV, WowHeaderW)]
#[wow_data(version = M2Version)]
enum M2ParticleEmitterBlending {
    Classic {
        blending_type: u16,
        emitter_type: u16,
    },

    #[wow_data(read_if = version >= M2Version::TBC)]
    Later {
        blending_type: u8,
        emitter_type: u8,
        particle_color_index: u16,
    },
}

#[derive(Debug, Clone, WowHeaderRV, WowHeaderW)]
#[wow_data(version = M2Version)]
enum M2ParticleEmitterMultiTextureParam {
    PreCata {
        particle_type: u8,
        head_or_tail: u8,
    },

    #[wow_data(read_if = version >= M2Version::Cataclysm)]
    AfterCata(u8),
}

#[derive(Debug, Clone, WowHeaderRV, WowHeaderW)]
#[wow_data(version = M2Version)]
enum M2ParticleEmitterLifespanVary {
    None,

    #[wow_data(read_if = version >= M2Version::WotLK)]
    Some(f32),
}

#[derive(Debug, Clone, WowHeaderRV, WowHeaderW)]
#[wow_data(version = M2Version)]
enum M2ParticleEmitterEmissionRateVary {
    None,

    #[wow_data(read_if = version >= M2Version::WotLK)]
    Some(f32),
}

#[derive(Debug, Clone, WowHeaderRV, WowHeaderW)]
#[wow_data(version = M2Version)]
enum M2ParticleEmitterColorAnimation {
    UpToTbc {
        mid_point: f32,
        color_values: [ColorA; 3],
        scale_values: [f32; 3],
        decay_uv_animation: [u16; 3],
        tail_uv_animation: [i16; 2],
        tail_decay_uv_animation: [i16; 2],
    },

    #[wow_data(read_if = version >= M2Version::WotLK)]
    Later {
        color_animation: M2FakeAnimationBlock<C3Vector>,
        alpha_animation: M2FakeAnimationBlock<u16>,
        scale_animation: M2FakeAnimationBlock<C2Vector>,
        scale_vary: C2Vector,
        head_cell_animation: M2FakeAnimationBlock<u16>,
        tail_cell_animation: M2FakeAnimationBlock<u16>,
    },
}

#[derive(Debug, Clone, WowHeaderRV, WowHeaderW)]
#[wow_data(version = M2Version)]
enum M2ParticleEmitterSpin {
    UpToTbc {
        spin: f32,
    },

    #[wow_data(read_if = version >= M2Version::WotLK)]
    Later {
        base_spin: f32,
        base_spin_vary: f32,
        spin: f32,
        spin_vary: f32,
    },
}

/// Represents a particle emitter in an M2 model
#[derive(Debug, Clone, WowHeaderRV, WowHeaderW)]
#[wow_data(version = M2Version)]
pub struct M2ParticleEmitterOld {
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
    pub color_animation: M2ParticleEmitterColorAnimation,
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

impl M2ParticleEmitterOld {
    // /// Convert this particle emitter to a different version
    // pub fn convert(&self, target_version: M2Version) -> Self {
    //     let mut new_emitter = self.clone();
    //
    //     // Handle version-specific conversions
    //     if target_version >= M2Version::Legion && self.fallback_model_filename.is_none() {
    //         // When upgrading to Legion or later, add fallback model filename and texture file data IDs if missing
    //         new_emitter.fallback_model_filename = Some(WowArray::new(0, 0));
    //         new_emitter.texture_file_data_ids = Some(WowArray::new(0, 0));
    //     } else if target_version < M2Version::Legion {
    //         // When downgrading to pre-Legion, remove fallback model filename and texture file data IDs
    //         new_emitter.fallback_model_filename = None;
    //         new_emitter.texture_file_data_ids = None;
    //     }
    //
    //     if target_version >= M2Version::WoD && self.enable_encryption.is_none() {
    //         // When upgrading to WoD or later, add encryption if missing
    //         new_emitter.enable_encryption = Some(0);
    //     } else if target_version < M2Version::WoD {
    //         // When downgrading to pre-WoD, remove encryption
    //         new_emitter.enable_encryption = None;
    //     }
    //
    //     if target_version >= M2Version::BfA && self.multi_texture_param0.is_none() {
    //         // When upgrading to BfA or later, add multi-texture params if missing
    //         new_emitter.multi_texture_param0 = Some([0, 0, 0, 0]);
    //         new_emitter.multi_texture_param1 = Some([0, 0, 0, 0]);
    //     } else if target_version < M2Version::BfA {
    //         // When downgrading to pre-BfA, remove multi-texture params
    //         new_emitter.multi_texture_param0 = None;
    //         new_emitter.multi_texture_param1 = None;
    //     }
    //
    //     if target_version >= M2Version::Legion && self.particle_initial_state.is_none() {
    //         // When upgrading to Legion or later, add particle state if missing
    //         new_emitter.particle_initial_state = Some(0);
    //         new_emitter.particle_initial_state_variation = Some(0.0);
    //         new_emitter.particle_convergence_time = Some(0.0);
    //     } else if target_version < M2Version::Legion {
    //         // When downgrading to pre-Legion, remove particle state
    //         new_emitter.particle_initial_state = None;
    //         new_emitter.particle_initial_state_variation = None;
    //         new_emitter.particle_convergence_time = None;
    //     }
    //
    //     new_emitter
    // }
}

#[derive(Debug, Clone, WowHeaderRV, WowHeaderW)]
#[wow_data(version = M2Version)]
pub struct M2ParticleEmitterNew {
    #[wow_data(versioned)]
    pub old_particle: M2ParticleEmitterOld,
    pub multi_texture_param_0: [VectorFp6_9; 2],
    pub multi_texture_param_1: [VectorFp6_9; 2],
}

#[derive(Debug, Clone, WowHeaderW)]
#[wow_data(version = M2Version)]
pub enum M2ParticleEmitter {
    PreCata(M2ParticleEmitterOld),

    #[wow_data(read_if = version >= M2Version::Cataclysm)]
    PostCata(M2ParticleEmitterNew),
}

impl WowHeaderRV<M2Version> for M2ParticleEmitter {
    fn wow_read<R: Read + Seek>(reader: &mut R, version: M2Version) -> WDResult<Self> {
        Ok(if version >= M2Version::Cataclysm {
            Self::PostCata(reader.wow_read_versioned(version)?)
        } else {
            Self::PreCata(reader.wow_read_versioned(version)?)
        })
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_particle_emitter_flags() {
//         let flags = M2ParticleFlags::BILLBOARDED | M2ParticleFlags::ROTATING;
//         assert!(flags.contains(M2ParticleFlags::BILLBOARDED));
//         assert!(flags.contains(M2ParticleFlags::ROTATING));
//         assert!(!flags.contains(M2ParticleFlags::PHYSICS));
//     }
// }
