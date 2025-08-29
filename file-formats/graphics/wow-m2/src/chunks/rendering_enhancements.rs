//! Advanced rendering enhancement chunks for Legion+ M2 models
//!
//! This module provides support for advanced rendering features introduced in
//! Legion and later expansions, including extended particle systems, waterfall
//! effects, edge fading, model alpha calculations, and lighting details.

use crate::chunks::animation::{M2AnimationBlock, M2AnimationTrack, M2InterpolationType};
use crate::chunks::infrastructure::ChunkReader;
use crate::chunks::particle_emitter::M2ParticleEmitter;
use crate::chunks::texture_animation::M2TextureAnimation;
use crate::common::M2Array;
use crate::error::Result;
use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Write};

/// Helper function to create empty animation blocks for compatibility
fn create_empty_animation_block<T>() -> M2AnimationBlock<T> {
    let track = M2AnimationTrack {
        interpolation_type: M2InterpolationType::None,
        global_sequence: -1,
        timestamps: M2Array::new(0, 0),
        values: M2Array::new(0, 0),
    };
    M2AnimationBlock::new(track)
}

/// Extended particle data for EXPT chunks (version 1)
#[derive(Debug, Clone)]
pub struct ExtendedParticleData {
    /// Version identifier (1 for EXPT, 2 for EXP2)
    pub version: u8,
    /// Enhanced particle emitters
    pub enhanced_emitters: Vec<EnhancedEmitter>,
    /// Advanced particle systems
    pub particle_systems: Vec<AdvancedParticleSystem>,
}

impl ExtendedParticleData {
    /// Parse EXPT chunk (version 1)
    pub fn parse_expt<R: Read + std::io::Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        let version = 1;
        let mut enhanced_emitters = Vec::new();
        let mut particle_systems = Vec::new();

        // EXPT format: sequence of enhanced emitter definitions
        while !reader.is_at_end()? {
            let emitter_type = reader.read_u8()?;
            match emitter_type {
                0 => {
                    // Enhanced emitter
                    let enhanced = EnhancedEmitter::parse(reader)?;
                    enhanced_emitters.push(enhanced);
                }
                1 => {
                    // Advanced particle system
                    let system = AdvancedParticleSystem::parse(reader)?;
                    particle_systems.push(system);
                }
                _ => {
                    // Skip unknown types for forward compatibility
                    let skip_size = reader.read_u32_le()?;
                    let mut skip_buffer = vec![0u8; skip_size as usize];
                    reader.read_exact(&mut skip_buffer)?;
                }
            }
        }

        Ok(Self {
            version,
            enhanced_emitters,
            particle_systems,
        })
    }

    /// Parse EXP2 chunk (version 2)
    pub fn parse_exp2<R: Read + std::io::Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        let version = 2;
        let mut enhanced_emitters = Vec::new();
        let mut particle_systems = Vec::new();

        // EXP2 format: enhanced version with additional properties
        let emitter_count = reader.read_u32_le()?;
        let system_count = reader.read_u32_le()?;

        // Read enhanced emitters
        for _ in 0..emitter_count {
            let enhanced = EnhancedEmitter::parse_v2(reader)?;
            enhanced_emitters.push(enhanced);
        }

        // Read particle systems
        for _ in 0..system_count {
            let system = AdvancedParticleSystem::parse_v2(reader)?;
            particle_systems.push(system);
        }

        Ok(Self {
            version,
            enhanced_emitters,
            particle_systems,
        })
    }

    /// Write extended particle data to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        if self.version == 1 {
            // EXPT format
            for emitter in &self.enhanced_emitters {
                writer.write_u8(0)?; // Enhanced emitter type
                emitter.write(writer)?;
            }

            for system in &self.particle_systems {
                writer.write_u8(1)?; // Advanced particle system type
                system.write(writer)?;
            }
        } else {
            // EXP2 format
            writer.write_u32_le(self.enhanced_emitters.len() as u32)?;
            writer.write_u32_le(self.particle_systems.len() as u32)?;

            for emitter in &self.enhanced_emitters {
                emitter.write_v2(writer)?;
            }

            for system in &self.particle_systems {
                system.write_v2(writer)?;
            }
        }

        Ok(())
    }
}

/// Enhanced particle emitter with extended properties
#[derive(Debug, Clone)]
pub struct EnhancedEmitter {
    /// Base emitter from legacy system
    pub base_emitter: M2ParticleEmitter,
    /// Extended properties
    pub extended_properties: ExtendedEmitterProperties,
}

impl EnhancedEmitter {
    /// Parse version 1 enhanced emitter
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        // For now, create minimal structure
        // In a complete implementation, this would parse the full enhanced emitter format
        let extended_properties = ExtendedEmitterProperties {
            enhanced_blending_mode: reader.read_u8()?,
            particle_sorting_mode: reader.read_u8()?,
            texture_scaling_factor: reader.read_f32_le()?,
            advanced_physics_enabled: reader.read_u8()? != 0,
            collision_detection_enabled: reader.read_u8()? != 0,
            wind_influence_factor: reader.read_f32_le()?,
        };

        // Create a minimal base emitter for compatibility
        // In a real implementation, this would be parsed from the chunk data
        let base_emitter = M2ParticleEmitter {
            id: 0,
            flags: crate::chunks::particle_emitter::M2ParticleFlags::empty(),
            position: crate::common::C3Vector {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            bone_index: 0,
            texture_index: 0,
            model_filename: crate::common::M2Array::new(0, 0),
            parent_emitter: 0,
            geometry_model_unknown: 0,
            fallback_model_filename: None,
            blending_type: 0,
            emitter_type: crate::chunks::particle_emitter::M2ParticleEmitterType::Point,
            particle_type: 0,
            head_or_tail: 0,
            texture_file_data_ids: None,
            texture_tile_coordinates: crate::common::M2Array::new(0, 0),
            enable_encryption: None,
            multi_texture_param0: None,
            multi_texture_param1: None,
            lifetime: 0.0,
            emission_rate: 0.0,
            emission_area_length: 0.0,
            emission_area_width: 0.0,
            emission_velocity: 0.0,
            min_lifetime: 0.0,
            max_lifetime: 0.0,
            min_emission_rate: 0.0,
            max_emission_rate: 0.0,
            min_emission_area_length: 0.0,
            max_emission_area_length: 0.0,
            min_emission_area_width: 0.0,
            max_emission_area_width: 0.0,
            min_emission_velocity: 0.0,
            max_emission_velocity: 0.0,
            position_variation: 0.0,
            min_position_variation: 0.0,
            max_position_variation: 0.0,
            initial_size: 0.0,
            min_initial_size: 0.0,
            max_initial_size: 0.0,
            size_variation: 0.0,
            min_size_variation: 0.0,
            max_size_variation: 0.0,
            horizontal_range: 0.0,
            min_horizontal_range: 0.0,
            max_horizontal_range: 0.0,
            vertical_range: 0.0,
            min_vertical_range: 0.0,
            max_vertical_range: 0.0,
            gravity: 0.0,
            min_gravity: 0.0,
            max_gravity: 0.0,
            initial_velocity: 0.0,
            min_initial_velocity: 0.0,
            max_initial_velocity: 0.0,
            speed_variation: 0.0,
            min_speed_variation: 0.0,
            max_speed_variation: 0.0,
            rotation_speed: 0.0,
            min_rotation_speed: 0.0,
            max_rotation_speed: 0.0,
            initial_rotation: 0.0,
            min_initial_rotation: 0.0,
            max_initial_rotation: 0.0,
            mid_point_color: crate::chunks::color_animation::M2Color::transparent(),
            color_animation_speed: 0.0,
            color_median_time: 0.0,
            lifespan_unused: 0.0,
            emission_rate_unused: 0.0,
            unknown_1: 0,
            unknown_2: 0.0,
            emission_speed_animation: create_empty_animation_block(),
            emission_rate_animation: create_empty_animation_block(),
            emission_area_animation: create_empty_animation_block(),
            xy_scale_animation: create_empty_animation_block(),
            z_scale_animation: create_empty_animation_block(),
            color_animation: create_empty_animation_block(),
            transparency_animation: create_empty_animation_block(),
            size_animation: create_empty_animation_block(),
            intensity_animation: create_empty_animation_block(),
            z_source_animation: create_empty_animation_block(),
            particle_initial_state: None,
            particle_initial_state_variation: None,
            particle_convergence_time: None,
            physics_parameters: None,
        };

        Ok(Self {
            base_emitter,
            extended_properties,
        })
    }

    /// Parse version 2 enhanced emitter (EXP2)
    pub fn parse_v2<R: Read>(reader: &mut R) -> Result<Self> {
        // Version 2 has additional fields
        let mut emitter = Self::parse(reader)?;

        // Additional EXP2 fields (placeholder implementation)
        emitter.extended_properties.advanced_physics_enabled = true;
        emitter.extended_properties.collision_detection_enabled = reader.read_u8()? != 0;

        Ok(emitter)
    }

    /// Write enhanced emitter (version 1)
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u8(self.extended_properties.enhanced_blending_mode)?;
        writer.write_u8(self.extended_properties.particle_sorting_mode)?;
        writer.write_f32_le(self.extended_properties.texture_scaling_factor)?;
        writer.write_u8(if self.extended_properties.advanced_physics_enabled {
            1
        } else {
            0
        })?;
        writer.write_u8(if self.extended_properties.collision_detection_enabled {
            1
        } else {
            0
        })?;
        writer.write_f32_le(self.extended_properties.wind_influence_factor)?;
        Ok(())
    }

    /// Write enhanced emitter (version 2)
    pub fn write_v2<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.write(writer)?;
        // Additional EXP2 fields would go here
        Ok(())
    }
}

/// Extended properties for enhanced emitters
#[derive(Debug, Clone)]
pub struct ExtendedEmitterProperties {
    /// Enhanced blending mode
    pub enhanced_blending_mode: u8,
    /// Particle sorting mode for depth/alpha handling
    pub particle_sorting_mode: u8,
    /// Texture scaling factor
    pub texture_scaling_factor: f32,
    /// Enable advanced physics simulation
    pub advanced_physics_enabled: bool,
    /// Enable particle collision detection
    pub collision_detection_enabled: bool,
    /// Factor for wind influence on particles
    pub wind_influence_factor: f32,
}

/// Advanced particle system with additional capabilities
#[derive(Debug, Clone)]
pub struct AdvancedParticleSystem {
    /// System identifier
    pub system_id: u32,
    /// Maximum number of particles
    pub max_particles: u32,
    /// Particle spawn pattern type
    pub spawn_pattern: u8,
    /// System-wide physics properties
    pub physics_properties: ParticlePhysicsProperties,
}

impl AdvancedParticleSystem {
    /// Parse version 1 advanced particle system
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let system_id = reader.read_u32_le()?;
        let max_particles = reader.read_u32_le()?;
        let spawn_pattern = reader.read_u8()?;

        let physics_properties = ParticlePhysicsProperties {
            air_resistance: reader.read_f32_le()?,
            bounce_factor: reader.read_f32_le()?,
            friction_coefficient: reader.read_f32_le()?,
        };

        Ok(Self {
            system_id,
            max_particles,
            spawn_pattern,
            physics_properties,
        })
    }

    /// Parse version 2 advanced particle system (EXP2)
    pub fn parse_v2<R: Read>(reader: &mut R) -> Result<Self> {
        let mut system = Self::parse(reader)?;

        // Additional EXP2 fields
        system.physics_properties.air_resistance = reader.read_f32_le()?;

        Ok(system)
    }

    /// Write advanced particle system (version 1)
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32_le(self.system_id)?;
        writer.write_u32_le(self.max_particles)?;
        writer.write_u8(self.spawn_pattern)?;
        writer.write_f32_le(self.physics_properties.air_resistance)?;
        writer.write_f32_le(self.physics_properties.bounce_factor)?;
        writer.write_f32_le(self.physics_properties.friction_coefficient)?;
        Ok(())
    }

    /// Write advanced particle system (version 2)
    pub fn write_v2<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.write(writer)?;
        // Additional EXP2 fields would go here
        Ok(())
    }
}

/// Physics properties for particle systems
#[derive(Debug, Clone)]
pub struct ParticlePhysicsProperties {
    /// Air resistance factor
    pub air_resistance: f32,
    /// Bounce factor for collision
    pub bounce_factor: f32,
    /// Friction coefficient
    pub friction_coefficient: f32,
}

/// Parent animation blacklist (PABC chunk)
#[derive(Debug, Clone)]
pub struct ParentAnimationBlacklist {
    /// Animation sequences that should be blacklisted
    pub blacklisted_sequences: Vec<u16>,
}

impl ParentAnimationBlacklist {
    /// Parse PABC chunk
    pub fn parse<R: Read + std::io::Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        let count = reader.chunk_size() / 2; // Each sequence ID is 2 bytes
        let mut blacklisted_sequences = Vec::with_capacity(count as usize);

        for _ in 0..count {
            blacklisted_sequences.push(reader.read_u16_le()?);
        }

        Ok(Self {
            blacklisted_sequences,
        })
    }

    /// Write PABC chunk
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        for &sequence_id in &self.blacklisted_sequences {
            writer.write_u16_le(sequence_id)?;
        }
        Ok(())
    }
}

/// Parent animation data (PADC chunk)
#[derive(Debug, Clone)]
pub struct ParentAnimationData {
    /// Texture weight assignments
    pub texture_weights: Vec<TextureWeight>,
    /// Blending modes for animations
    pub blending_modes: Vec<BlendMode>,
}

impl ParentAnimationData {
    /// Parse PADC chunk
    pub fn parse<R: Read + std::io::Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        let weight_count = reader.read_u32_le()?;
        let mut texture_weights = Vec::with_capacity(weight_count as usize);

        for _ in 0..weight_count {
            let weight = TextureWeight {
                texture_index: reader.read_u16_le()?,
                weight_factor: reader.read_f32_le()?,
                blend_operation: reader.read_u8()?,
            };
            texture_weights.push(weight);
        }

        let mode_count = reader.read_u32_le()?;
        let mut blending_modes = Vec::with_capacity(mode_count as usize);

        for _ in 0..mode_count {
            let mode = BlendMode {
                source_blend: reader.read_u8()?,
                dest_blend: reader.read_u8()?,
                alpha_test_threshold: reader.read_f32_le()?,
            };
            blending_modes.push(mode);
        }

        Ok(Self {
            texture_weights,
            blending_modes,
        })
    }

    /// Write PADC chunk
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32_le(self.texture_weights.len() as u32)?;
        for weight in &self.texture_weights {
            writer.write_u16_le(weight.texture_index)?;
            writer.write_f32_le(weight.weight_factor)?;
            writer.write_u8(weight.blend_operation)?;
        }

        writer.write_u32_le(self.blending_modes.len() as u32)?;
        for mode in &self.blending_modes {
            writer.write_u8(mode.source_blend)?;
            writer.write_u8(mode.dest_blend)?;
            writer.write_f32_le(mode.alpha_test_threshold)?;
        }

        Ok(())
    }
}

/// Texture weight assignment for parent animations
#[derive(Debug, Clone)]
pub struct TextureWeight {
    /// Index of the texture to weight
    pub texture_index: u16,
    /// Weight factor (0.0 to 1.0)
    pub weight_factor: f32,
    /// Blend operation type
    pub blend_operation: u8,
}

/// Blending mode configuration
#[derive(Debug, Clone)]
pub struct BlendMode {
    /// Source blend factor
    pub source_blend: u8,
    /// Destination blend factor
    pub dest_blend: u8,
    /// Alpha test threshold
    pub alpha_test_threshold: f32,
}

/// Waterfall effect data (WFV1/WFV2/WFV3 chunks)
#[derive(Debug, Clone)]
pub struct WaterfallEffect {
    /// Version (1, 2, or 3)
    pub version: u8,
    /// Effect parameters
    pub parameters: WaterfallParameters,
}

impl WaterfallEffect {
    /// Parse waterfall effect chunk
    pub fn parse<R: Read + std::io::Seek>(
        reader: &mut ChunkReader<R>,
        version: u8,
    ) -> Result<Self> {
        let parameters = match version {
            1 => WaterfallParameters::parse_v1(reader)?,
            2 => WaterfallParameters::parse_v2(reader)?,
            3 => WaterfallParameters::parse_v3(reader)?,
            _ => {
                return Err(crate::error::M2Error::ParseError(format!(
                    "Unsupported waterfall effect version: {}",
                    version
                )));
            }
        };

        Ok(Self {
            version,
            parameters,
        })
    }

    /// Write waterfall effect chunk
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        match self.version {
            1 => self.parameters.write_v1(writer),
            2 => self.parameters.write_v2(writer),
            3 => self.parameters.write_v3(writer),
            _ => Err(crate::error::M2Error::ParseError(format!(
                "Unsupported waterfall effect version: {}",
                self.version
            ))),
        }
    }
}

/// Parameters for waterfall effects
#[derive(Debug, Clone)]
pub struct WaterfallParameters {
    /// Flow velocity
    pub flow_velocity: f32,
    /// Turbulence factor
    pub turbulence: f32,
    /// Foam intensity
    pub foam_intensity: f32,
    /// Version-specific additional parameters
    pub additional_params: Vec<f32>,
}

impl WaterfallParameters {
    /// Parse version 1 parameters
    pub fn parse_v1<R: Read>(reader: &mut R) -> Result<Self> {
        let flow_velocity = reader.read_f32_le()?;
        let turbulence = reader.read_f32_le()?;
        let foam_intensity = reader.read_f32_le()?;

        Ok(Self {
            flow_velocity,
            turbulence,
            foam_intensity,
            additional_params: Vec::new(),
        })
    }

    /// Parse version 2 parameters
    pub fn parse_v2<R: Read>(reader: &mut R) -> Result<Self> {
        let mut params = Self::parse_v1(reader)?;

        // Version 2 adds splash parameters
        params.additional_params.push(reader.read_f32_le()?); // splash_intensity
        params.additional_params.push(reader.read_f32_le()?); // splash_radius

        Ok(params)
    }

    /// Parse version 3 parameters
    pub fn parse_v3<R: Read>(reader: &mut R) -> Result<Self> {
        let mut params = Self::parse_v2(reader)?;

        // Version 3 adds advanced flow control
        params.additional_params.push(reader.read_f32_le()?); // flow_direction_x
        params.additional_params.push(reader.read_f32_le()?); // flow_direction_y
        params.additional_params.push(reader.read_f32_le()?); // flow_direction_z

        Ok(params)
    }

    /// Write version 1 parameters
    pub fn write_v1<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_f32_le(self.flow_velocity)?;
        writer.write_f32_le(self.turbulence)?;
        writer.write_f32_le(self.foam_intensity)?;
        Ok(())
    }

    /// Write version 2 parameters
    pub fn write_v2<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.write_v1(writer)?;

        if self.additional_params.len() >= 2 {
            writer.write_f32_le(self.additional_params[0])?; // splash_intensity
            writer.write_f32_le(self.additional_params[1])?; // splash_radius
        } else {
            writer.write_f32_le(0.0)?;
            writer.write_f32_le(0.0)?;
        }

        Ok(())
    }

    /// Write version 3 parameters
    pub fn write_v3<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.write_v2(writer)?;

        if self.additional_params.len() >= 5 {
            writer.write_f32_le(self.additional_params[2])?; // flow_direction_x
            writer.write_f32_le(self.additional_params[3])?; // flow_direction_y  
            writer.write_f32_le(self.additional_params[4])?; // flow_direction_z
        } else {
            writer.write_f32_le(0.0)?;
            writer.write_f32_le(0.0)?;
            writer.write_f32_le(1.0)?; // Default downward flow
        }

        Ok(())
    }
}

/// Edge fade rendering data (EDGF chunk)
#[derive(Debug, Clone)]
pub struct EdgeFadeData {
    /// Fade distances for different LOD levels
    pub fade_distances: Vec<f32>,
    /// Fade factors for smooth transitions
    pub fade_factors: Vec<f32>,
}

impl EdgeFadeData {
    /// Parse EDGF chunk
    pub fn parse<R: Read + std::io::Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        let distance_count = reader.read_u32_le()?;
        let mut fade_distances = Vec::with_capacity(distance_count as usize);

        for _ in 0..distance_count {
            fade_distances.push(reader.read_f32_le()?);
        }

        let factor_count = reader.read_u32_le()?;
        let mut fade_factors = Vec::with_capacity(factor_count as usize);

        for _ in 0..factor_count {
            fade_factors.push(reader.read_f32_le()?);
        }

        Ok(Self {
            fade_distances,
            fade_factors,
        })
    }

    /// Write EDGF chunk
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32_le(self.fade_distances.len() as u32)?;
        for &distance in &self.fade_distances {
            writer.write_f32_le(distance)?;
        }

        writer.write_u32_le(self.fade_factors.len() as u32)?;
        for &factor in &self.fade_factors {
            writer.write_f32_le(factor)?;
        }

        Ok(())
    }
}

/// Model alpha calculation data (NERF chunk)
#[derive(Debug, Clone)]
pub struct ModelAlphaData {
    /// Alpha test threshold
    pub alpha_test_threshold: f32,
    /// Alpha blend mode
    pub blend_mode: AlphaBlendMode,
}

impl ModelAlphaData {
    /// Parse NERF chunk
    pub fn parse<R: Read + std::io::Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        let alpha_test_threshold = reader.read_f32_le()?;
        let blend_mode_value = reader.read_u8()?;
        let blend_mode =
            AlphaBlendMode::from_u8(blend_mode_value).unwrap_or(AlphaBlendMode::Normal);

        Ok(Self {
            alpha_test_threshold,
            blend_mode,
        })
    }

    /// Write NERF chunk
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_f32_le(self.alpha_test_threshold)?;
        writer.write_u8(self.blend_mode as u8)?;
        Ok(())
    }
}

/// Alpha blending modes for model rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaBlendMode {
    /// Normal alpha blending
    Normal = 0,
    /// Additive blending
    Additive = 1,
    /// Multiplicative blending
    Multiplicative = 2,
    /// Alpha test only
    AlphaTest = 3,
}

impl AlphaBlendMode {
    /// Convert from u8 value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Normal),
            1 => Some(Self::Additive),
            2 => Some(Self::Multiplicative),
            3 => Some(Self::AlphaTest),
            _ => None,
        }
    }
}

/// Lighting detail data (DETL chunk)
#[derive(Debug, Clone)]
pub struct LightingDetails {
    /// Ambient lighting factor
    pub ambient_factor: f32,
    /// Diffuse lighting factor
    pub diffuse_factor: f32,
    /// Specular lighting factor
    pub specular_factor: f32,
}

impl LightingDetails {
    /// Parse DETL chunk
    pub fn parse<R: Read + std::io::Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        let ambient_factor = reader.read_f32_le()?;
        let diffuse_factor = reader.read_f32_le()?;
        let specular_factor = reader.read_f32_le()?;

        Ok(Self {
            ambient_factor,
            diffuse_factor,
            specular_factor,
        })
    }

    /// Write DETL chunk
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_f32_le(self.ambient_factor)?;
        writer.write_f32_le(self.diffuse_factor)?;
        writer.write_f32_le(self.specular_factor)?;
        Ok(())
    }
}

/// Recursive particle model IDs (RPID chunk)
#[derive(Debug, Clone)]
pub struct RecursiveParticleIds {
    /// FileDataIDs of models to load recursively
    pub model_ids: Vec<u32>,
}

impl RecursiveParticleIds {
    /// Parse RPID chunk
    pub fn parse<R: Read + std::io::Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        let count = reader.chunk_size() / 4; // Each ID is 4 bytes
        let mut model_ids = Vec::with_capacity(count as usize);

        for _ in 0..count {
            model_ids.push(reader.read_u32_le()?);
        }

        Ok(Self { model_ids })
    }

    /// Write RPID chunk
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        for &id in &self.model_ids {
            writer.write_u32_le(id)?;
        }
        Ok(())
    }
}

/// Geometry particle model IDs (GPID chunk)
#[derive(Debug, Clone)]
pub struct GeometryParticleIds {
    /// FileDataIDs of geometry models for particles
    pub model_ids: Vec<u32>,
}

impl GeometryParticleIds {
    /// Parse GPID chunk
    pub fn parse<R: Read + std::io::Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        let count = reader.chunk_size() / 4; // Each ID is 4 bytes
        let mut model_ids = Vec::with_capacity(count as usize);

        for _ in 0..count {
            model_ids.push(reader.read_u32_le()?);
        }

        Ok(Self { model_ids })
    }

    /// Write GPID chunk
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        for &id in &self.model_ids {
            writer.write_u32_le(id)?;
        }
        Ok(())
    }
}

/// TXAC texture animation chunk data
#[derive(Debug, Clone)]
pub struct TextureAnimationChunk {
    /// Extended texture animations beyond the standard set
    pub texture_animations: Vec<ExtendedTextureAnimation>,
}

impl TextureAnimationChunk {
    /// Parse TXAC chunk
    pub fn parse<R: Read + std::io::Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        let count = reader.read_u32_le()?;
        let mut texture_animations = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let extended_anim = ExtendedTextureAnimation::parse(reader)?;
            texture_animations.push(extended_anim);
        }

        Ok(Self { texture_animations })
    }

    /// Write TXAC chunk
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32_le(self.texture_animations.len() as u32)?;

        for anim in &self.texture_animations {
            anim.write(writer)?;
        }

        Ok(())
    }
}

/// Extended texture animation for TXAC chunk
#[derive(Debug, Clone)]
pub struct ExtendedTextureAnimation {
    /// Base texture animation
    pub base_animation: M2TextureAnimation,
    /// Extended properties for advanced effects
    pub extended_properties: ExtendedAnimationProperties,
}

impl ExtendedTextureAnimation {
    /// Parse extended texture animation
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        // Parse base texture animation first
        let base_animation = M2TextureAnimation::parse(reader)?;

        // Parse extended properties
        let extended_properties = ExtendedAnimationProperties {
            flow_direction: [
                reader.read_f32_le()?,
                reader.read_f32_le()?,
                reader.read_f32_le()?,
            ],
            speed_multiplier: reader.read_f32_le()?,
            turbulence_factor: reader.read_f32_le()?,
            animation_mode: ExtendedAnimationMode::from_u8(reader.read_u8()?)?,
            loop_behavior: LoopBehavior::from_u8(reader.read_u8()?)?,
            blend_mode: TextureBlendMode::from_u8(reader.read_u8()?)?,
            _padding: reader.read_u8()?, // Padding for alignment
        };

        Ok(Self {
            base_animation,
            extended_properties,
        })
    }

    /// Write extended texture animation
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Write base animation first
        self.base_animation.write(writer)?;

        // Write extended properties
        writer.write_f32_le(self.extended_properties.flow_direction[0])?;
        writer.write_f32_le(self.extended_properties.flow_direction[1])?;
        writer.write_f32_le(self.extended_properties.flow_direction[2])?;
        writer.write_f32_le(self.extended_properties.speed_multiplier)?;
        writer.write_f32_le(self.extended_properties.turbulence_factor)?;
        writer.write_u8(self.extended_properties.animation_mode as u8)?;
        writer.write_u8(self.extended_properties.loop_behavior as u8)?;
        writer.write_u8(self.extended_properties.blend_mode as u8)?;
        writer.write_u8(self.extended_properties._padding)?;

        Ok(())
    }
}

/// Extended properties for texture animations
#[derive(Debug, Clone)]
pub struct ExtendedAnimationProperties {
    /// Flow direction vector (x, y, z)
    pub flow_direction: [f32; 3],
    /// Speed multiplier for animation
    pub speed_multiplier: f32,
    /// Turbulence factor for realistic flow
    pub turbulence_factor: f32,
    /// Animation mode
    pub animation_mode: ExtendedAnimationMode,
    /// Loop behavior
    pub loop_behavior: LoopBehavior,
    /// Texture blend mode
    pub blend_mode: TextureBlendMode,
    /// Padding for alignment
    pub _padding: u8,
}

/// Extended animation modes for TXAC
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedAnimationMode {
    /// Standard scrolling (legacy compatibility)
    StandardScroll = 0,
    /// Flowing liquid animation
    FlowingLiquid = 1,
    /// Turbulent flow with noise
    TurbulentFlow = 2,
    /// Whirlpool/vortex animation
    Vortex = 3,
    /// Wave motion
    Wave = 4,
}

impl ExtendedAnimationMode {
    /// Convert from u8 value
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::StandardScroll),
            1 => Ok(Self::FlowingLiquid),
            2 => Ok(Self::TurbulentFlow),
            3 => Ok(Self::Vortex),
            4 => Ok(Self::Wave),
            _ => Err(crate::error::M2Error::ParseError(format!(
                "Unknown extended animation mode: {}",
                value
            ))),
        }
    }
}

/// Loop behavior for animations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopBehavior {
    /// Loop infinitely
    Infinite = 0,
    /// Play once and stop
    Once = 1,
    /// Ping-pong (forward then reverse)
    PingPong = 2,
    /// Reverse loop
    Reverse = 3,
}

impl LoopBehavior {
    /// Convert from u8 value
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Infinite),
            1 => Ok(Self::Once),
            2 => Ok(Self::PingPong),
            3 => Ok(Self::Reverse),
            _ => Err(crate::error::M2Error::ParseError(format!(
                "Unknown loop behavior: {}",
                value
            ))),
        }
    }
}

/// Texture blend modes for TXAC
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureBlendMode {
    /// Normal blend
    Normal = 0,
    /// Additive blend
    Additive = 1,
    /// Multiply blend
    Multiply = 2,
    /// Screen blend
    Screen = 3,
    /// Overlay blend
    Overlay = 4,
}

impl TextureBlendMode {
    /// Convert from u8 value
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Normal),
            1 => Ok(Self::Additive),
            2 => Ok(Self::Multiply),
            3 => Ok(Self::Screen),
            4 => Ok(Self::Overlay),
            _ => Err(crate::error::M2Error::ParseError(format!(
                "Unknown texture blend mode: {}",
                value
            ))),
        }
    }
}

/// PGD1 particle geoset data chunk
#[derive(Debug, Clone)]
pub struct ParticleGeosetData {
    /// Geoset assignments for particle emitters
    pub geoset_assignments: Vec<ParticleGeosetEntry>,
}

impl ParticleGeosetData {
    /// Parse PGD1 chunk
    pub fn parse<R: Read + std::io::Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        let count = reader.chunk_size() / 2; // Each entry is 2 bytes (u16)
        let mut geoset_assignments = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let geoset = reader.read_u16_le()?;
            geoset_assignments.push(ParticleGeosetEntry { geoset });
        }

        Ok(Self { geoset_assignments })
    }

    /// Write PGD1 chunk
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        for entry in &self.geoset_assignments {
            writer.write_u16_le(entry.geoset)?;
        }
        Ok(())
    }
}

/// Particle geoset entry for PGD1 chunk
#[derive(Debug, Clone)]
pub struct ParticleGeosetEntry {
    /// Geoset ID that this particle emitter belongs to
    pub geoset: u16,
}

/// DBOC chunk data (purpose currently unknown)
#[derive(Debug, Clone)]
pub struct DbocChunk {
    /// Raw data for DBOC chunk (16 bytes observed)
    pub data: Vec<u8>,
}

impl DbocChunk {
    /// Parse DBOC chunk
    pub fn parse<R: Read + std::io::Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        let mut data = vec![0u8; reader.chunk_size() as usize];
        reader.read_exact(&mut data)?;

        Ok(Self { data })
    }

    /// Write DBOC chunk
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.data)?;
        Ok(())
    }
}

/// AFRA chunk data (purpose unknown, not observed in files yet)
#[derive(Debug, Clone)]
pub struct AfraChunk {
    /// Raw data for AFRA chunk
    pub data: Vec<u8>,
}

impl AfraChunk {
    /// Parse AFRA chunk
    pub fn parse<R: Read + std::io::Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        let mut data = vec![0u8; reader.chunk_size() as usize];
        reader.read_exact(&mut data)?;

        Ok(Self { data })
    }

    /// Write AFRA chunk
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.data)?;
        Ok(())
    }
}

/// DPIV chunk data (collision mesh for player housing)
#[derive(Debug, Clone)]
pub struct DpivChunk {
    /// Vertex position count
    pub vertex_pos_count: u32,
    /// Vertex position offset
    pub vertex_pos_offset: u32,
    /// Face normal count
    pub face_norm_count: u32,
    /// Face normal offset
    pub face_norm_offset: u32,
    /// Index count
    pub index_count: u32,
    /// Index offset
    pub index_offset: u32,
    /// Flags count
    pub flags_count: u32,
    /// Flags offset
    pub flags_offset: u32,
    /// Vertex positions
    pub vertex_positions: Vec<[f32; 3]>,
    /// Face normals
    pub face_normals: Vec<[f32; 3]>,
    /// Indices
    pub indices: Vec<u16>,
    /// Flags
    pub flags: Vec<u16>,
}

impl DpivChunk {
    /// Parse DPIV chunk
    pub fn parse<R: Read + std::io::Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        let chunk_start = reader.current_position()?;

        let vertex_pos_count = reader.read_u32_le()?;
        let vertex_pos_offset = reader.read_u32_le()?;
        let face_norm_count = reader.read_u32_le()?;
        let face_norm_offset = reader.read_u32_le()?;
        let index_count = reader.read_u32_le()?;
        let index_offset = reader.read_u32_le()?;
        let flags_count = reader.read_u32_le()?;
        let flags_offset = reader.read_u32_le()?;

        // Read vertex positions
        reader.seek_to_position(chunk_start + vertex_pos_offset as u64)?;
        let mut vertex_positions = Vec::with_capacity(vertex_pos_count as usize);
        for _ in 0..vertex_pos_count {
            let pos = [
                reader.read_f32_le()?,
                reader.read_f32_le()?,
                reader.read_f32_le()?,
            ];
            vertex_positions.push(pos);
        }

        // Read face normals
        reader.seek_to_position(chunk_start + face_norm_offset as u64)?;
        let mut face_normals = Vec::with_capacity(face_norm_count as usize);
        for _ in 0..face_norm_count {
            let normal = [
                reader.read_f32_le()?,
                reader.read_f32_le()?,
                reader.read_f32_le()?,
            ];
            face_normals.push(normal);
        }

        // Read indices
        reader.seek_to_position(chunk_start + index_offset as u64)?;
        let mut indices = Vec::with_capacity(index_count as usize);
        for _ in 0..index_count {
            indices.push(reader.read_u16_le()?);
        }

        // Read flags
        reader.seek_to_position(chunk_start + flags_offset as u64)?;
        let mut flags = Vec::with_capacity(flags_count as usize);
        for _ in 0..flags_count {
            flags.push(reader.read_u16_le()?);
        }

        Ok(Self {
            vertex_pos_count,
            vertex_pos_offset,
            face_norm_count,
            face_norm_offset,
            index_count,
            index_offset,
            flags_count,
            flags_offset,
            vertex_positions,
            face_normals,
            indices,
            flags,
        })
    }

    /// Write DPIV chunk
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32_le(self.vertex_pos_count)?;
        writer.write_u32_le(self.vertex_pos_offset)?;
        writer.write_u32_le(self.face_norm_count)?;
        writer.write_u32_le(self.face_norm_offset)?;
        writer.write_u32_le(self.index_count)?;
        writer.write_u32_le(self.index_offset)?;
        writer.write_u32_le(self.flags_count)?;
        writer.write_u32_le(self.flags_offset)?;

        // Note: For write, we would need to calculate proper offsets and write data
        // This is a simplified implementation that assumes sequential data layout
        for pos in &self.vertex_positions {
            writer.write_f32_le(pos[0])?;
            writer.write_f32_le(pos[1])?;
            writer.write_f32_le(pos[2])?;
        }

        for normal in &self.face_normals {
            writer.write_f32_le(normal[0])?;
            writer.write_f32_le(normal[1])?;
            writer.write_f32_le(normal[2])?;
        }

        for &index in &self.indices {
            writer.write_u16_le(index)?;
        }

        for &flag in &self.flags {
            writer.write_u16_le(flag)?;
        }

        Ok(())
    }
}

/// PSBC - Parent Sequence Bounds Chunk
#[derive(Debug, Clone)]
pub struct ParentSequenceBounds {
    /// Animation sequence bounds data
    pub sequence_bounds: Vec<SequenceBounds>,
}

/// Sequence bounds information
#[derive(Debug, Clone)]
pub struct SequenceBounds {
    /// Minimum bounds
    pub min_bounds: [f32; 3],
    /// Maximum bounds
    pub max_bounds: [f32; 3],
    /// Radius
    pub radius: f32,
}

impl ParentSequenceBounds {
    /// Parse PSBC chunk
    pub fn parse<R: Read + std::io::Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        let mut sequence_bounds = Vec::new();

        // Each sequence bound is 28 bytes (6 floats + 1 float)
        while !reader.is_at_end()? {
            let min_bounds = [
                reader.read_f32_le()?,
                reader.read_f32_le()?,
                reader.read_f32_le()?,
            ];
            let max_bounds = [
                reader.read_f32_le()?,
                reader.read_f32_le()?,
                reader.read_f32_le()?,
            ];
            let radius = reader.read_f32_le()?;

            sequence_bounds.push(SequenceBounds {
                min_bounds,
                max_bounds,
                radius,
            });
        }

        Ok(Self { sequence_bounds })
    }

    /// Write PSBC chunk
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        for bounds in &self.sequence_bounds {
            writer.write_f32_le(bounds.min_bounds[0])?;
            writer.write_f32_le(bounds.min_bounds[1])?;
            writer.write_f32_le(bounds.min_bounds[2])?;
            writer.write_f32_le(bounds.max_bounds[0])?;
            writer.write_f32_le(bounds.max_bounds[1])?;
            writer.write_f32_le(bounds.max_bounds[2])?;
            writer.write_f32_le(bounds.radius)?;
        }
        Ok(())
    }
}

/// PEDC - Parent Event Data Chunk
#[derive(Debug, Clone)]
pub struct ParentEventData {
    /// Event data entries
    pub event_entries: Vec<ParentEventEntry>,
}

/// Parent event entry
#[derive(Debug, Clone)]
pub struct ParentEventEntry {
    /// Event identifier
    pub event_id: u32,
    /// Data blob
    pub data: Vec<u8>,
    /// Timestamp
    pub timestamp: u32,
}

impl ParentEventData {
    /// Parse PEDC chunk
    pub fn parse<R: Read + std::io::Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        let mut event_entries = Vec::new();

        while !reader.is_at_end()? {
            let event_id = reader.read_u32_le()?;
            let data_size = reader.read_u32_le()?;
            let timestamp = reader.read_u32_le()?;

            let mut data = vec![0u8; data_size as usize];
            reader.read_exact(&mut data)?;

            event_entries.push(ParentEventEntry {
                event_id,
                data,
                timestamp,
            });
        }

        Ok(Self { event_entries })
    }

    /// Write PEDC chunk
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        for entry in &self.event_entries {
            writer.write_u32_le(entry.event_id)?;
            writer.write_u32_le(entry.data.len() as u32)?;
            writer.write_u32_le(entry.timestamp)?;
            writer.write_all(&entry.data)?;
        }
        Ok(())
    }
}

/// PCOL - Collision Mesh Data Chunk
#[derive(Debug, Clone)]
pub struct CollisionMeshData {
    /// Collision vertices
    pub vertices: Vec<[f32; 3]>,
    /// Collision faces
    pub faces: Vec<CollisionFace>,
    /// Material properties
    pub materials: Vec<CollisionMaterial>,
}

/// Collision face
#[derive(Debug, Clone)]
pub struct CollisionFace {
    /// Vertex indices
    pub indices: [u16; 3],
    /// Material index
    pub material_index: u16,
}

/// Collision material properties
#[derive(Debug, Clone)]
pub struct CollisionMaterial {
    /// Material flags
    pub flags: u32,
    /// Friction coefficient
    pub friction: f32,
    /// Restitution coefficient
    pub restitution: f32,
}

impl CollisionMeshData {
    /// Parse PCOL chunk
    pub fn parse<R: Read + std::io::Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        let vertex_count = reader.read_u32_le()?;
        let face_count = reader.read_u32_le()?;
        let material_count = reader.read_u32_le()?;

        let mut vertices = Vec::with_capacity(vertex_count as usize);
        for _ in 0..vertex_count {
            vertices.push([
                reader.read_f32_le()?,
                reader.read_f32_le()?,
                reader.read_f32_le()?,
            ]);
        }

        let mut faces = Vec::with_capacity(face_count as usize);
        for _ in 0..face_count {
            faces.push(CollisionFace {
                indices: [
                    reader.read_u16_le()?,
                    reader.read_u16_le()?,
                    reader.read_u16_le()?,
                ],
                material_index: reader.read_u16_le()?,
            });
        }

        let mut materials = Vec::with_capacity(material_count as usize);
        for _ in 0..material_count {
            materials.push(CollisionMaterial {
                flags: reader.read_u32_le()?,
                friction: reader.read_f32_le()?,
                restitution: reader.read_f32_le()?,
            });
        }

        Ok(Self {
            vertices,
            faces,
            materials,
        })
    }

    /// Write PCOL chunk
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32_le(self.vertices.len() as u32)?;
        writer.write_u32_le(self.faces.len() as u32)?;
        writer.write_u32_le(self.materials.len() as u32)?;

        for vertex in &self.vertices {
            writer.write_f32_le(vertex[0])?;
            writer.write_f32_le(vertex[1])?;
            writer.write_f32_le(vertex[2])?;
        }

        for face in &self.faces {
            writer.write_u16_le(face.indices[0])?;
            writer.write_u16_le(face.indices[1])?;
            writer.write_u16_le(face.indices[2])?;
            writer.write_u16_le(face.material_index)?;
        }

        for material in &self.materials {
            writer.write_u32_le(material.flags)?;
            writer.write_f32_le(material.friction)?;
            writer.write_f32_le(material.restitution)?;
        }

        Ok(())
    }
}

/// PFDC - Physics File Data Chunk
#[derive(Debug, Clone)]
pub struct PhysicsFileDataChunk {
    /// Raw physics file data
    pub physics_data: Vec<u8>,
    /// Physics properties
    pub properties: PhysicsProperties,
}

/// Physics properties
#[derive(Debug, Clone)]
pub struct PhysicsProperties {
    /// Mass
    pub mass: f32,
    /// Center of mass
    pub center_of_mass: [f32; 3],
    /// Inertia tensor
    pub inertia_tensor: [f32; 9],
    /// Physics flags
    pub flags: u32,
}

impl PhysicsFileDataChunk {
    /// Parse PFDC chunk
    pub fn parse<R: Read + std::io::Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        let mass = reader.read_f32_le()?;
        let center_of_mass = [
            reader.read_f32_le()?,
            reader.read_f32_le()?,
            reader.read_f32_le()?,
        ];

        let mut inertia_tensor = [0.0f32; 9];
        for item in &mut inertia_tensor {
            *item = reader.read_f32_le()?;
        }

        let flags = reader.read_u32_le()?;

        // Read remaining data as physics blob
        let mut physics_data = Vec::new();
        reader.read_to_end(&mut physics_data)?;

        Ok(Self {
            physics_data,
            properties: PhysicsProperties {
                mass,
                center_of_mass,
                inertia_tensor,
                flags,
            },
        })
    }

    /// Write PFDC chunk
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_f32_le(self.properties.mass)?;
        writer.write_f32_le(self.properties.center_of_mass[0])?;
        writer.write_f32_le(self.properties.center_of_mass[1])?;
        writer.write_f32_le(self.properties.center_of_mass[2])?;

        for &tensor_val in &self.properties.inertia_tensor {
            writer.write_f32_le(tensor_val)?;
        }

        writer.write_u32_le(self.properties.flags)?;
        writer.write_all(&self.physics_data)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunks::infrastructure::{ChunkHeader, ChunkReader};
    use std::io::Cursor;

    #[test]
    fn test_parent_animation_blacklist() {
        let data = vec![
            0x01, 0x00, // Sequence 1
            0x05, 0x00, // Sequence 5
            0x0A, 0x00, // Sequence 10
        ];

        let header = ChunkHeader {
            magic: *b"PABC",
            size: data.len() as u32,
        };
        let cursor = Cursor::new(data);
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        let blacklist = ParentAnimationBlacklist::parse(&mut chunk_reader).unwrap();
        assert_eq!(blacklist.blacklisted_sequences, vec![1, 5, 10]);
    }

    #[test]
    fn test_waterfall_effect_v1() {
        let data = vec![
            0x00, 0x00, 0x80, 0x3F, // flow_velocity: 1.0
            0x00, 0x00, 0x00, 0x3F, // turbulence: 0.5
            0x00, 0x00, 0x40, 0x3F, // foam_intensity: 0.75
        ];

        let header = ChunkHeader {
            magic: *b"WFV1",
            size: data.len() as u32,
        };
        let cursor = Cursor::new(data);
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        let effect = WaterfallEffect::parse(&mut chunk_reader, 1).unwrap();
        assert_eq!(effect.version, 1);
        assert_eq!(effect.parameters.flow_velocity, 1.0);
        assert!(effect.parameters.additional_params.is_empty());
    }

    #[test]
    fn test_edge_fade_data() {
        let mut data = Vec::new();
        // Distance count
        data.extend_from_slice(&2u32.to_le_bytes());
        // Distances
        data.extend_from_slice(&10.0f32.to_le_bytes());
        data.extend_from_slice(&20.0f32.to_le_bytes());
        // Factor count
        data.extend_from_slice(&2u32.to_le_bytes());
        // Factors
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.8f32.to_le_bytes());

        let header = ChunkHeader {
            magic: *b"EDGF",
            size: data.len() as u32,
        };
        let cursor = Cursor::new(data);
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        let fade_data = EdgeFadeData::parse(&mut chunk_reader).unwrap();
        assert_eq!(fade_data.fade_distances, vec![10.0, 20.0]);
        assert_eq!(fade_data.fade_factors, vec![0.5, 0.8]);
    }

    #[test]
    fn test_alpha_blend_mode_conversion() {
        assert_eq!(AlphaBlendMode::from_u8(0), Some(AlphaBlendMode::Normal));
        assert_eq!(AlphaBlendMode::from_u8(1), Some(AlphaBlendMode::Additive));
        assert_eq!(
            AlphaBlendMode::from_u8(2),
            Some(AlphaBlendMode::Multiplicative)
        );
        assert_eq!(AlphaBlendMode::from_u8(3), Some(AlphaBlendMode::AlphaTest));
        assert_eq!(AlphaBlendMode::from_u8(99), None);
    }

    #[test]
    fn test_particle_model_ids() {
        let data = vec![
            0x01, 0x02, 0x03, 0x04, // ID: 67305985
            0x05, 0x06, 0x07, 0x08, // ID: 134678021
        ];

        let header = ChunkHeader {
            magic: *b"RPID",
            size: data.len() as u32,
        };
        let cursor = Cursor::new(data);
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        let rpid = RecursiveParticleIds::parse(&mut chunk_reader).unwrap();
        assert_eq!(rpid.model_ids.len(), 2);
        assert_eq!(rpid.model_ids[0], 0x04030201); // Little-endian
        assert_eq!(rpid.model_ids[1], 0x08070605);
    }

    #[test]
    fn test_particle_geoset_data() {
        let data = vec![
            0x01, 0x00, // Geoset 1
            0x05, 0x00, // Geoset 5
            0x0A, 0x00, // Geoset 10
        ];

        let header = ChunkHeader {
            magic: *b"PGD1",
            size: data.len() as u32,
        };
        let cursor = Cursor::new(data);
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        let pgd1 = ParticleGeosetData::parse(&mut chunk_reader).unwrap();
        assert_eq!(pgd1.geoset_assignments.len(), 3);
        assert_eq!(pgd1.geoset_assignments[0].geoset, 1);
        assert_eq!(pgd1.geoset_assignments[1].geoset, 5);
        assert_eq!(pgd1.geoset_assignments[2].geoset, 10);
    }

    #[test]
    fn test_parent_sequence_bounds() {
        let mut data = Vec::new();

        // First sequence bounds (28 bytes)
        data.extend_from_slice(&(-10.0f32).to_le_bytes()); // min_x
        data.extend_from_slice(&(-5.0f32).to_le_bytes()); // min_y
        data.extend_from_slice(&(-2.0f32).to_le_bytes()); // min_z
        data.extend_from_slice(&10.0f32.to_le_bytes()); // max_x
        data.extend_from_slice(&5.0f32.to_le_bytes()); // max_y
        data.extend_from_slice(&2.0f32.to_le_bytes()); // max_z
        data.extend_from_slice(&15.0f32.to_le_bytes()); // radius

        let header = ChunkHeader {
            magic: *b"PSBC",
            size: data.len() as u32,
        };
        let cursor = Cursor::new(data);
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        let psbc = ParentSequenceBounds::parse(&mut chunk_reader).unwrap();
        assert_eq!(psbc.sequence_bounds.len(), 1);
        assert_eq!(psbc.sequence_bounds[0].min_bounds, [-10.0, -5.0, -2.0]);
        assert_eq!(psbc.sequence_bounds[0].max_bounds, [10.0, 5.0, 2.0]);
        assert_eq!(psbc.sequence_bounds[0].radius, 15.0);
    }

    #[test]
    fn test_parent_event_data() {
        let mut data = Vec::new();

        // First event
        data.extend_from_slice(&1u32.to_le_bytes()); // event_id
        data.extend_from_slice(&4u32.to_le_bytes()); // data_size
        data.extend_from_slice(&1000u32.to_le_bytes()); // timestamp
        data.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]); // data

        let header = ChunkHeader {
            magic: *b"PEDC",
            size: data.len() as u32,
        };
        let cursor = Cursor::new(data);
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        let pedc = ParentEventData::parse(&mut chunk_reader).unwrap();
        assert_eq!(pedc.event_entries.len(), 1);
        assert_eq!(pedc.event_entries[0].event_id, 1);
        assert_eq!(pedc.event_entries[0].timestamp, 1000);
        assert_eq!(pedc.event_entries[0].data, vec![0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_collision_mesh_data() {
        let mut data = Vec::new();

        // Header
        data.extend_from_slice(&1u32.to_le_bytes()); // vertex_count
        data.extend_from_slice(&1u32.to_le_bytes()); // face_count
        data.extend_from_slice(&1u32.to_le_bytes()); // material_count

        // Vertex
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&2.0f32.to_le_bytes());
        data.extend_from_slice(&3.0f32.to_le_bytes());

        // Face
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&1u16.to_le_bytes());
        data.extend_from_slice(&2u16.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes()); // material_index

        // Material
        data.extend_from_slice(&1u32.to_le_bytes()); // flags
        data.extend_from_slice(&0.5f32.to_le_bytes()); // friction
        data.extend_from_slice(&0.8f32.to_le_bytes()); // restitution

        let header = ChunkHeader {
            magic: *b"PCOL",
            size: data.len() as u32,
        };
        let cursor = Cursor::new(data);
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        let pcol = CollisionMeshData::parse(&mut chunk_reader).unwrap();
        assert_eq!(pcol.vertices.len(), 1);
        assert_eq!(pcol.faces.len(), 1);
        assert_eq!(pcol.materials.len(), 1);
        assert_eq!(pcol.vertices[0], [1.0, 2.0, 3.0]);
        assert_eq!(pcol.materials[0].friction, 0.5);
    }

    #[test]
    fn test_texture_animation_chunk() {
        let mut data = Vec::new();

        // Count of texture animations
        data.extend_from_slice(&1u32.to_le_bytes());

        // Base texture animation data (simplified)
        data.extend_from_slice(&1u16.to_le_bytes()); // Animation type (Scroll)
        data.extend_from_slice(&0u16.to_le_bytes()); // Padding

        // Add minimal animation block data (would be more complex in reality)
        for _ in 0..5 {
            // Each animation block: interpolation_type, global_sequence, timestamps, values
            data.extend_from_slice(&0u16.to_le_bytes()); // Interpolation type
            data.extend_from_slice(&(-1i16).to_le_bytes()); // Global sequence
            data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps count
            data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps offset
            data.extend_from_slice(&0u32.to_le_bytes()); // Values count
            data.extend_from_slice(&0u32.to_le_bytes()); // Values offset
        }

        // Extended properties
        data.extend_from_slice(&1.0f32.to_le_bytes()); // Flow direction X
        data.extend_from_slice(&0.0f32.to_le_bytes()); // Flow direction Y
        data.extend_from_slice(&0.0f32.to_le_bytes()); // Flow direction Z
        data.extend_from_slice(&1.5f32.to_le_bytes()); // Speed multiplier
        data.extend_from_slice(&0.2f32.to_le_bytes()); // Turbulence factor
        data.extend_from_slice(&1u8.to_le_bytes()); // Animation mode (FlowingLiquid)
        data.extend_from_slice(&0u8.to_le_bytes()); // Loop behavior (Infinite)
        data.extend_from_slice(&0u8.to_le_bytes()); // Blend mode (Normal)
        data.extend_from_slice(&0u8.to_le_bytes()); // Padding

        let header = ChunkHeader {
            magic: *b"TXAC",
            size: data.len() as u32,
        };
        let cursor = Cursor::new(data);
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        let txac = TextureAnimationChunk::parse(&mut chunk_reader).unwrap();
        assert_eq!(txac.texture_animations.len(), 1);
        assert_eq!(
            txac.texture_animations[0]
                .extended_properties
                .animation_mode,
            ExtendedAnimationMode::FlowingLiquid
        );
        assert_eq!(
            txac.texture_animations[0]
                .extended_properties
                .speed_multiplier,
            1.5
        );
    }

    #[test]
    fn test_dboc_chunk() {
        let data = vec![
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10,
        ]; // 16 bytes

        let header = ChunkHeader {
            magic: *b"DBOC",
            size: data.len() as u32,
        };
        let cursor = Cursor::new(data.clone());
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        let dboc = DbocChunk::parse(&mut chunk_reader).unwrap();
        assert_eq!(dboc.data, data);
    }

    #[test]
    fn test_extended_animation_mode_conversion() {
        assert_eq!(
            ExtendedAnimationMode::from_u8(0).unwrap(),
            ExtendedAnimationMode::StandardScroll
        );
        assert_eq!(
            ExtendedAnimationMode::from_u8(1).unwrap(),
            ExtendedAnimationMode::FlowingLiquid
        );
        assert_eq!(
            ExtendedAnimationMode::from_u8(4).unwrap(),
            ExtendedAnimationMode::Wave
        );
        assert!(ExtendedAnimationMode::from_u8(99).is_err());
    }

    #[test]
    fn test_physics_file_data_chunk() {
        let mut data = Vec::new();

        // Physics properties
        data.extend_from_slice(&10.0f32.to_le_bytes()); // mass
        data.extend_from_slice(&1.0f32.to_le_bytes()); // center_of_mass[0]
        data.extend_from_slice(&2.0f32.to_le_bytes()); // center_of_mass[1]
        data.extend_from_slice(&3.0f32.to_le_bytes()); // center_of_mass[2]

        // Inertia tensor (9 floats)
        for i in 0..9 {
            data.extend_from_slice(&((i + 1) as f32).to_le_bytes());
        }

        data.extend_from_slice(&0x12345678u32.to_le_bytes()); // flags

        // Additional physics data
        data.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]);

        let header = ChunkHeader {
            magic: *b"PFDC",
            size: data.len() as u32,
        };
        let cursor = Cursor::new(data);
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        let pfdc = PhysicsFileDataChunk::parse(&mut chunk_reader).unwrap();
        assert_eq!(pfdc.properties.mass, 10.0);
        assert_eq!(pfdc.properties.center_of_mass, [1.0, 2.0, 3.0]);
        assert_eq!(pfdc.properties.flags, 0x12345678);
        assert_eq!(pfdc.physics_data, vec![0xAA, 0xBB, 0xCC, 0xDD]);
    }
}
