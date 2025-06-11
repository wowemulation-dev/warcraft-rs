use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Write};

use crate::chunks::animation::M2AnimationBlock;
use crate::chunks::color_animation::M2Color;
use crate::common::{C2Vector, C3Vector, M2Array};
use crate::error::Result;
use crate::version::M2Version;

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

/// Represents a particle emitter in an M2 model
#[derive(Debug, Clone)]
pub struct M2ParticleEmitter {
    /// ID for this emitter
    pub id: u32,
    /// Flags controlling particle behavior
    pub flags: M2ParticleFlags,
    /// Position of the emitter
    pub position: C3Vector,
    /// Bone to attach the emitter to
    pub bone_index: u16,
    /// Texture coordinate (for UV coordinate generation)
    pub texture_index: u16,
    /// Geometry model filename (for complex shaped emitters)
    pub model_filename: M2Array<u8>,
    /// ID of another emitter this one is linked to
    pub parent_emitter: u16,
    /// Unknown geometry value
    pub geometry_model_unknown: u16,
    /// Explicit fallback model if main one fails to load
    pub fallback_model_filename: Option<M2Array<u8>>,
    /// Blending type
    pub blending_type: u8,
    /// Emitter type
    pub emitter_type: M2ParticleEmitterType,
    /// Particle type
    pub particle_type: u8,
    /// Head or tail
    pub head_or_tail: u8,
    /// Texture file IDs (for multi-texture particles)
    pub texture_file_data_ids: Option<M2Array<u32>>,
    /// Tile coordinates in the texture(s)
    pub texture_tile_coordinates: M2Array<C2Vector>,
    /// Flag to enable encryption (WoD and later)
    pub enable_encryption: Option<u8>,
    /// Multi-texture particle blend operation
    pub multi_texture_param0: Option<[u8; 4]>,
    /// Multi-texture particle blend flags
    pub multi_texture_param1: Option<[u8; 4]>,
    /// Time to live (in seconds)
    pub lifetime: f32,
    /// Time between emissions (in seconds)
    pub emission_rate: f32,
    /// Initial emission range
    pub emission_area_length: f32,
    /// Initial emission width
    pub emission_area_width: f32,
    /// Initial emission velocity
    pub emission_velocity: f32,
    /// Minimum lifetime of a particle
    pub min_lifetime: f32,
    /// Maximum lifetime of a particle
    pub max_lifetime: f32,
    /// Minimum emission rate
    pub min_emission_rate: f32,
    /// Maximum emission rate
    pub max_emission_rate: f32,
    /// Minimum emission area length
    pub min_emission_area_length: f32,
    /// Maximum emission area length
    pub max_emission_area_length: f32,
    /// Minimum emission area width
    pub min_emission_area_width: f32,
    /// Maximum emission area width
    pub max_emission_area_width: f32,
    /// Minimum emission velocity
    pub min_emission_velocity: f32,
    /// Maximum emission velocity
    pub max_emission_velocity: f32,
    /// Position variation (jitter)
    pub position_variation: f32,
    /// Minimum position variation
    pub min_position_variation: f32,
    /// Maximum position variation
    pub max_position_variation: f32,
    /// Initial size (diameter)
    pub initial_size: f32,
    /// Minimum initial size
    pub min_initial_size: f32,
    /// Maximum initial size
    pub max_initial_size: f32,
    /// Scaling factor for size over time
    pub size_variation: f32,
    /// Minimum size variation
    pub min_size_variation: f32,
    /// Maximum size variation
    pub max_size_variation: f32,
    /// Horizontal/vertical ratio
    pub horizontal_range: f32,
    /// Minimum horizontal range
    pub min_horizontal_range: f32,
    /// Maximum horizontal range
    pub max_horizontal_range: f32,
    /// Vertical range (slowdown)
    pub vertical_range: f32,
    /// Minimum vertical range
    pub min_vertical_range: f32,
    /// Maximum vertical range
    pub max_vertical_range: f32,
    /// Gravitational acceleration
    pub gravity: f32,
    /// Minimum gravity
    pub min_gravity: f32,
    /// Maximum gravity
    pub max_gravity: f32,
    /// Initial velocity
    pub initial_velocity: f32,
    /// Minimum initial velocity
    pub min_initial_velocity: f32,
    /// Maximum initial velocity
    pub max_initial_velocity: f32,
    /// Speed variation
    pub speed_variation: f32,
    /// Minimum speed variation
    pub min_speed_variation: f32,
    /// Maximum speed variation
    pub max_speed_variation: f32,
    /// Rotational speed (for ROTATING particles)
    pub rotation_speed: f32,
    /// Minimum rotation speed
    pub min_rotation_speed: f32,
    /// Maximum rotation speed
    pub max_rotation_speed: f32,
    /// Initial rotation
    pub initial_rotation: f32,
    /// Minimum initial rotation
    pub min_initial_rotation: f32,
    /// Maximum initial rotation
    pub max_initial_rotation: f32,
    /// Mid-point color animation
    pub mid_point_color: M2Color,
    /// Color/alpha animations
    pub color_animation_speed: f32,
    /// Time when the color/alpha fade to mid values
    pub color_median_time: f32,
    /// Duration of emission
    pub lifespan_unused: f32,
    /// Zero point of emission
    pub emission_rate_unused: f32,
    /// Unknown value 1
    pub unknown_1: u32,
    /// Unknown value 2
    pub unknown_2: f32,
    /// Animation for emission speed
    pub emission_speed_animation: M2AnimationBlock<f32>,
    /// Animation for emission rate
    pub emission_rate_animation: M2AnimationBlock<f32>,
    /// Animation for emission area
    pub emission_area_animation: M2AnimationBlock<f32>,
    /// Animation for X/Y scale (for non-particle plane)
    pub xy_scale_animation: M2AnimationBlock<C2Vector>,
    /// Animation for Z scale (for non-particle plane)
    pub z_scale_animation: M2AnimationBlock<f32>,
    /// Animation for particle colors
    pub color_animation: M2AnimationBlock<M2Color>,
    /// Animation for particle transparency
    pub transparency_animation: M2AnimationBlock<f32>,
    /// Animation for particle size (diameter)
    pub size_animation: M2AnimationBlock<f32>,
    /// Animation for intensity
    pub intensity_animation: M2AnimationBlock<f32>,
    /// Animation for Z source (height)
    pub z_source_animation: M2AnimationBlock<f32>,
    /// Base initial state for particles
    pub particle_initial_state: Option<u32>,
    /// Variation for initial state
    pub particle_initial_state_variation: Option<f32>,
    /// Convergence speed for particles
    pub particle_convergence_time: Option<f32>,
    /// Physical parameters (MoP+ with PHYSICS flag)
    pub physics_parameters: Option<[f32; 5]>,
}

impl M2ParticleEmitter {
    /// Parse a particle emitter from a reader based on the M2 version
    pub fn parse<R: Read>(reader: &mut R, version: u32) -> Result<Self> {
        let id = reader.read_u32_le()?;
        let flags = M2ParticleFlags::from_bits_retain(reader.read_u32_le()?);
        let position = C3Vector::parse(reader)?;
        let bone_index = reader.read_u16_le()?;
        let texture_index = reader.read_u16_le()?;
        let model_filename = M2Array::parse(reader)?;
        let parent_emitter = reader.read_u16_le()?;
        let geometry_model_unknown = reader.read_u16_le()?;

        // Version-specific fields
        let (
            fallback_model_filename,
            blending_type,
            emitter_type,
            particle_type,
            head_or_tail,
            texture_file_data_ids,
            enable_encryption,
            multi_texture_param0,
            multi_texture_param1,
        ) = if let Some(m2_version) = M2Version::from_header_version(version) {
            if m2_version >= M2Version::Legion {
                // Legion and later have fallback model and texture file data IDs
                let fallback = M2Array::parse(reader)?;
                let blend = reader.read_u8()?;
                let emitter = M2ParticleEmitterType::from_u8(reader.read_u8()?)
                    .unwrap_or(M2ParticleEmitterType::Point);
                let particle = reader.read_u8()?;
                let head = reader.read_u8()?;
                let tex_file_ids = M2Array::parse(reader)?;

                // Extra fields for WoD and later
                let encryption = if m2_version >= M2Version::WoD {
                    Some(reader.read_u8()?)
                } else {
                    None
                };

                // Extra multi-texture params for BfA and later
                let (param0, param1) = if m2_version >= M2Version::BfA {
                    let mut p0 = [0u8; 4];
                    let mut p1 = [0u8; 4];

                    for item in &mut p0 {
                        *item = reader.read_u8()?;
                    }

                    for item in &mut p1 {
                        *item = reader.read_u8()?;
                    }

                    (Some(p0), Some(p1))
                } else {
                    (None, None)
                };

                (
                    Some(fallback),
                    blend,
                    emitter,
                    particle,
                    head,
                    Some(tex_file_ids),
                    encryption,
                    param0,
                    param1,
                )
            } else if m2_version >= M2Version::WoD {
                // WoD has encryption but no fallback model or texture file IDs
                let blend = reader.read_u8()?;
                let emitter = M2ParticleEmitterType::from_u8(reader.read_u8()?)
                    .unwrap_or(M2ParticleEmitterType::Point);
                let particle = reader.read_u8()?;
                let head = reader.read_u8()?;
                let encryption = Some(reader.read_u8()?);

                (
                    None, blend, emitter, particle, head, None, encryption, None, None,
                )
            } else {
                // Pre-WoD just has basic fields
                let blend = reader.read_u8()?;
                let emitter = M2ParticleEmitterType::from_u8(reader.read_u8()?)
                    .unwrap_or(M2ParticleEmitterType::Point);
                let particle = reader.read_u8()?;
                let head = reader.read_u8()?;

                (None, blend, emitter, particle, head, None, None, None, None)
            }
        } else {
            // Default to Classic format
            let blend = reader.read_u8()?;
            let emitter = M2ParticleEmitterType::from_u8(reader.read_u8()?)
                .unwrap_or(M2ParticleEmitterType::Point);
            let particle = reader.read_u8()?;
            let head = reader.read_u8()?;

            (None, blend, emitter, particle, head, None, None, None, None)
        };

        // Texture tile coordinates are in all versions
        let texture_tile_coordinates = M2Array::parse(reader)?;

        // Read common parameters
        let lifetime = reader.read_f32_le()?;
        let emission_rate = reader.read_f32_le()?;
        let emission_area_length = reader.read_f32_le()?;
        let emission_area_width = reader.read_f32_le()?;
        let emission_velocity = reader.read_f32_le()?;

        // Read min/max ranges
        let min_lifetime = reader.read_f32_le()?;
        let max_lifetime = reader.read_f32_le()?;
        let min_emission_rate = reader.read_f32_le()?;
        let max_emission_rate = reader.read_f32_le()?;
        let min_emission_area_length = reader.read_f32_le()?;
        let max_emission_area_length = reader.read_f32_le()?;
        let min_emission_area_width = reader.read_f32_le()?;
        let max_emission_area_width = reader.read_f32_le()?;
        let min_emission_velocity = reader.read_f32_le()?;
        let max_emission_velocity = reader.read_f32_le()?;

        // Read size parameters
        let position_variation = reader.read_f32_le()?;
        let min_position_variation = reader.read_f32_le()?;
        let max_position_variation = reader.read_f32_le()?;
        let initial_size = reader.read_f32_le()?;
        let min_initial_size = reader.read_f32_le()?;
        let max_initial_size = reader.read_f32_le()?;
        let size_variation = reader.read_f32_le()?;
        let min_size_variation = reader.read_f32_le()?;
        let max_size_variation = reader.read_f32_le()?;

        // Read movement parameters
        let horizontal_range = reader.read_f32_le()?;
        let min_horizontal_range = reader.read_f32_le()?;
        let max_horizontal_range = reader.read_f32_le()?;
        let vertical_range = reader.read_f32_le()?;
        let min_vertical_range = reader.read_f32_le()?;
        let max_vertical_range = reader.read_f32_le()?;
        let gravity = reader.read_f32_le()?;
        let min_gravity = reader.read_f32_le()?;
        let max_gravity = reader.read_f32_le()?;

        // Read velocity and rotation parameters
        let initial_velocity = reader.read_f32_le()?;
        let min_initial_velocity = reader.read_f32_le()?;
        let max_initial_velocity = reader.read_f32_le()?;
        let speed_variation = reader.read_f32_le()?;
        let min_speed_variation = reader.read_f32_le()?;
        let max_speed_variation = reader.read_f32_le()?;
        let rotation_speed = reader.read_f32_le()?;
        let min_rotation_speed = reader.read_f32_le()?;
        let max_rotation_speed = reader.read_f32_le()?;
        let initial_rotation = reader.read_f32_le()?;
        let min_initial_rotation = reader.read_f32_le()?;
        let max_initial_rotation = reader.read_f32_le()?;

        // Read color parameters
        let mid_point_color = M2Color::parse(reader)?;
        let color_animation_speed = reader.read_f32_le()?;
        let color_median_time = reader.read_f32_le()?;

        // Read unused/unknown parameters
        let lifespan_unused = reader.read_f32_le()?;
        let emission_rate_unused = reader.read_f32_le()?;
        let unknown_1 = reader.read_u32_le()?;
        let unknown_2 = reader.read_f32_le()?;

        // Read animation blocks
        let emission_speed_animation = M2AnimationBlock::parse(reader)?;
        let emission_rate_animation = M2AnimationBlock::parse(reader)?;
        let emission_area_animation = M2AnimationBlock::parse(reader)?;
        let xy_scale_animation = M2AnimationBlock::parse(reader)?;
        let z_scale_animation = M2AnimationBlock::parse(reader)?;
        let color_animation = M2AnimationBlock::parse(reader)?;
        let transparency_animation = M2AnimationBlock::parse(reader)?;
        let size_animation = M2AnimationBlock::parse(reader)?;
        let intensity_animation = M2AnimationBlock::parse(reader)?;
        let z_source_animation = M2AnimationBlock::parse(reader)?;

        // Additional fields for Legion and later
        let (particle_initial_state, particle_initial_state_variation, particle_convergence_time) =
            if let Some(m2_version) = M2Version::from_header_version(version) {
                if m2_version >= M2Version::Legion {
                    (
                        Some(reader.read_u32_le()?),
                        Some(reader.read_f32_le()?),
                        Some(reader.read_f32_le()?),
                    )
                } else {
                    (None, None, None)
                }
            } else {
                (None, None, None)
            };

        // Additional physics parameters for MoP+ with PHYSICS flag
        let physics_parameters = if let Some(m2_version) = M2Version::from_header_version(version) {
            if m2_version >= M2Version::MoP && flags.contains(M2ParticleFlags::PHYSICS) {
                let mut params = [0.0; 5];
                for item in &mut params {
                    *item = reader.read_f32_le()?;
                }
                Some(params)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            id,
            flags,
            position,
            bone_index,
            texture_index,
            model_filename,
            parent_emitter,
            geometry_model_unknown,
            fallback_model_filename,
            blending_type,
            emitter_type,
            particle_type,
            head_or_tail,
            texture_file_data_ids,
            texture_tile_coordinates,
            enable_encryption,
            multi_texture_param0,
            multi_texture_param1,
            lifetime,
            emission_rate,
            emission_area_length,
            emission_area_width,
            emission_velocity,
            min_lifetime,
            max_lifetime,
            min_emission_rate,
            max_emission_rate,
            min_emission_area_length,
            max_emission_area_length,
            min_emission_area_width,
            max_emission_area_width,
            min_emission_velocity,
            max_emission_velocity,
            position_variation,
            min_position_variation,
            max_position_variation,
            initial_size,
            min_initial_size,
            max_initial_size,
            size_variation,
            min_size_variation,
            max_size_variation,
            horizontal_range,
            min_horizontal_range,
            max_horizontal_range,
            vertical_range,
            min_vertical_range,
            max_vertical_range,
            gravity,
            min_gravity,
            max_gravity,
            initial_velocity,
            min_initial_velocity,
            max_initial_velocity,
            speed_variation,
            min_speed_variation,
            max_speed_variation,
            rotation_speed,
            min_rotation_speed,
            max_rotation_speed,
            initial_rotation,
            min_initial_rotation,
            max_initial_rotation,
            mid_point_color,
            color_animation_speed,
            color_median_time,
            lifespan_unused,
            emission_rate_unused,
            unknown_1,
            unknown_2,
            emission_speed_animation,
            emission_rate_animation,
            emission_area_animation,
            xy_scale_animation,
            z_scale_animation,
            color_animation,
            transparency_animation,
            size_animation,
            intensity_animation,
            z_source_animation,
            particle_initial_state,
            particle_initial_state_variation,
            particle_convergence_time,
            physics_parameters,
        })
    }

    /// Write a particle emitter to a writer based on the M2 version
    pub fn write<W: Write>(&self, writer: &mut W, version: u32) -> Result<()> {
        writer.write_u32_le(self.id)?;
        writer.write_u32_le(self.flags.bits())?;
        self.position.write(writer)?;
        writer.write_u16_le(self.bone_index)?;
        writer.write_u16_le(self.texture_index)?;
        self.model_filename.write(writer)?;
        writer.write_u16_le(self.parent_emitter)?;
        writer.write_u16_le(self.geometry_model_unknown)?;

        // Version-specific fields
        if let Some(m2_version) = M2Version::from_header_version(version) {
            if m2_version >= M2Version::Legion {
                // Legion and later have fallback model and texture file data IDs
                if let Some(ref fallback) = self.fallback_model_filename {
                    fallback.write(writer)?;
                } else {
                    M2Array::<u8>::new(0, 0).write(writer)?;
                }

                writer.write_u8(self.blending_type)?;
                writer.write_u8(self.emitter_type as u8)?;
                writer.write_u8(self.particle_type)?;
                writer.write_u8(self.head_or_tail)?;

                if let Some(ref tex_file_ids) = self.texture_file_data_ids {
                    tex_file_ids.write(writer)?;
                } else {
                    M2Array::<u32>::new(0, 0).write(writer)?;
                }

                // Extra fields for WoD and later
                if m2_version >= M2Version::WoD {
                    writer.write_u8(self.enable_encryption.unwrap_or(0))?;
                }

                // Extra multi-texture params for BfA and later
                if m2_version >= M2Version::BfA {
                    if let Some(param0) = self.multi_texture_param0 {
                        for &val in &param0 {
                            writer.write_u8(val)?;
                        }
                    } else {
                        for _ in 0..4 {
                            writer.write_u8(0)?;
                        }
                    }

                    if let Some(param1) = self.multi_texture_param1 {
                        for &val in &param1 {
                            writer.write_u8(val)?;
                        }
                    } else {
                        for _ in 0..4 {
                            writer.write_u8(0)?;
                        }
                    }
                }
            } else if m2_version >= M2Version::WoD {
                // WoD has encryption but no fallback model or texture file IDs
                writer.write_u8(self.blending_type)?;
                writer.write_u8(self.emitter_type as u8)?;
                writer.write_u8(self.particle_type)?;
                writer.write_u8(self.head_or_tail)?;
                writer.write_u8(self.enable_encryption.unwrap_or(0))?;
            } else {
                // Pre-WoD just has basic fields
                writer.write_u8(self.blending_type)?;
                writer.write_u8(self.emitter_type as u8)?;
                writer.write_u8(self.particle_type)?;
                writer.write_u8(self.head_or_tail)?;
            }
        } else {
            // Default to Classic format
            writer.write_u8(self.blending_type)?;
            writer.write_u8(self.emitter_type as u8)?;
            writer.write_u8(self.particle_type)?;
            writer.write_u8(self.head_or_tail)?;
        }

        // Texture tile coordinates are in all versions
        self.texture_tile_coordinates.write(writer)?;

        // Write common parameters
        writer.write_f32_le(self.lifetime)?;
        writer.write_f32_le(self.emission_rate)?;
        writer.write_f32_le(self.emission_area_length)?;
        writer.write_f32_le(self.emission_area_width)?;
        writer.write_f32_le(self.emission_velocity)?;

        // Write min/max ranges
        writer.write_f32_le(self.min_lifetime)?;
        writer.write_f32_le(self.max_lifetime)?;
        writer.write_f32_le(self.min_emission_rate)?;
        writer.write_f32_le(self.max_emission_rate)?;
        writer.write_f32_le(self.min_emission_area_length)?;
        writer.write_f32_le(self.max_emission_area_length)?;
        writer.write_f32_le(self.min_emission_area_width)?;
        writer.write_f32_le(self.max_emission_area_width)?;
        writer.write_f32_le(self.min_emission_velocity)?;
        writer.write_f32_le(self.max_emission_velocity)?;

        // Write size parameters
        writer.write_f32_le(self.position_variation)?;
        writer.write_f32_le(self.min_position_variation)?;
        writer.write_f32_le(self.max_position_variation)?;
        writer.write_f32_le(self.initial_size)?;
        writer.write_f32_le(self.min_initial_size)?;
        writer.write_f32_le(self.max_initial_size)?;
        writer.write_f32_le(self.size_variation)?;
        writer.write_f32_le(self.min_size_variation)?;
        writer.write_f32_le(self.max_size_variation)?;

        // Write movement parameters
        writer.write_f32_le(self.horizontal_range)?;
        writer.write_f32_le(self.min_horizontal_range)?;
        writer.write_f32_le(self.max_horizontal_range)?;
        writer.write_f32_le(self.vertical_range)?;
        writer.write_f32_le(self.min_vertical_range)?;
        writer.write_f32_le(self.max_vertical_range)?;
        writer.write_f32_le(self.gravity)?;
        writer.write_f32_le(self.min_gravity)?;
        writer.write_f32_le(self.max_gravity)?;

        // Write velocity and rotation parameters
        writer.write_f32_le(self.initial_velocity)?;
        writer.write_f32_le(self.min_initial_velocity)?;
        writer.write_f32_le(self.max_initial_velocity)?;
        writer.write_f32_le(self.speed_variation)?;
        writer.write_f32_le(self.min_speed_variation)?;
        writer.write_f32_le(self.max_speed_variation)?;
        writer.write_f32_le(self.rotation_speed)?;
        writer.write_f32_le(self.min_rotation_speed)?;
        writer.write_f32_le(self.max_rotation_speed)?;
        writer.write_f32_le(self.initial_rotation)?;
        writer.write_f32_le(self.min_initial_rotation)?;
        writer.write_f32_le(self.max_initial_rotation)?;

        // Write color parameters
        self.mid_point_color.write(writer)?;
        writer.write_f32_le(self.color_animation_speed)?;
        writer.write_f32_le(self.color_median_time)?;

        // Write unused/unknown parameters
        writer.write_f32_le(self.lifespan_unused)?;
        writer.write_f32_le(self.emission_rate_unused)?;
        writer.write_u32_le(self.unknown_1)?;
        writer.write_f32_le(self.unknown_2)?;

        // Write animation blocks
        self.emission_speed_animation.write(writer)?;
        self.emission_rate_animation.write(writer)?;
        self.emission_area_animation.write(writer)?;
        self.xy_scale_animation.write(writer)?;
        self.z_scale_animation.write(writer)?;
        self.color_animation.write(writer)?;
        self.transparency_animation.write(writer)?;
        self.size_animation.write(writer)?;
        self.intensity_animation.write(writer)?;
        self.z_source_animation.write(writer)?;

        // Additional fields for Legion and later
        if let Some(m2_version) = M2Version::from_header_version(version) {
            if m2_version >= M2Version::Legion {
                writer.write_u32_le(self.particle_initial_state.unwrap_or(0))?;
                writer.write_f32_le(self.particle_initial_state_variation.unwrap_or(0.0))?;
                writer.write_f32_le(self.particle_convergence_time.unwrap_or(0.0))?;
            }
        }

        // Additional physics parameters for MoP+ with PHYSICS flag
        if let Some(m2_version) = M2Version::from_header_version(version) {
            if m2_version >= M2Version::MoP && self.flags.contains(M2ParticleFlags::PHYSICS) {
                if let Some(params) = self.physics_parameters {
                    for &val in &params {
                        writer.write_f32_le(val)?;
                    }
                } else {
                    // Write default values if no physics parameters are provided
                    for _ in 0..5 {
                        writer.write_f32_le(0.0)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Convert this particle emitter to a different version
    pub fn convert(&self, target_version: M2Version) -> Self {
        let mut new_emitter = self.clone();

        // Handle version-specific conversions
        if target_version >= M2Version::Legion && self.fallback_model_filename.is_none() {
            // When upgrading to Legion or later, add fallback model filename and texture file data IDs if missing
            new_emitter.fallback_model_filename = Some(M2Array::new(0, 0));
            new_emitter.texture_file_data_ids = Some(M2Array::new(0, 0));
        } else if target_version < M2Version::Legion {
            // When downgrading to pre-Legion, remove fallback model filename and texture file data IDs
            new_emitter.fallback_model_filename = None;
            new_emitter.texture_file_data_ids = None;
        }

        if target_version >= M2Version::WoD && self.enable_encryption.is_none() {
            // When upgrading to WoD or later, add encryption if missing
            new_emitter.enable_encryption = Some(0);
        } else if target_version < M2Version::WoD {
            // When downgrading to pre-WoD, remove encryption
            new_emitter.enable_encryption = None;
        }

        if target_version >= M2Version::BfA && self.multi_texture_param0.is_none() {
            // When upgrading to BfA or later, add multi-texture params if missing
            new_emitter.multi_texture_param0 = Some([0, 0, 0, 0]);
            new_emitter.multi_texture_param1 = Some([0, 0, 0, 0]);
        } else if target_version < M2Version::BfA {
            // When downgrading to pre-BfA, remove multi-texture params
            new_emitter.multi_texture_param0 = None;
            new_emitter.multi_texture_param1 = None;
        }

        if target_version >= M2Version::Legion && self.particle_initial_state.is_none() {
            // When upgrading to Legion or later, add particle state if missing
            new_emitter.particle_initial_state = Some(0);
            new_emitter.particle_initial_state_variation = Some(0.0);
            new_emitter.particle_convergence_time = Some(0.0);
        } else if target_version < M2Version::Legion {
            // When downgrading to pre-Legion, remove particle state
            new_emitter.particle_initial_state = None;
            new_emitter.particle_initial_state_variation = None;
            new_emitter.particle_convergence_time = None;
        }

        new_emitter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_emitter_flags() {
        let flags = M2ParticleFlags::BILLBOARDED | M2ParticleFlags::ROTATING;
        assert!(flags.contains(M2ParticleFlags::BILLBOARDED));
        assert!(flags.contains(M2ParticleFlags::ROTATING));
        assert!(!flags.contains(M2ParticleFlags::PHYSICS));
    }
}
