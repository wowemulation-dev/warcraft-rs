//! Particle system for M2 models
//!
//! This module provides runtime particle emission and simulation for M2 models.
//! It implements the three emitter types (planar, spherical, spline) with physics
//! simulation including gravity, drag, and wind forces.
//!
//! # Architecture
//!
//! - `Particle`: Individual particle with position, velocity, age, color, scale
//! - `ParticleEmitter`: Runtime state for a single emitter
//! - `EmitterParams`: Current frame parameters from animation tracks
//!
//! # Usage
//!
//! ```rust,ignore
//! use wow_m2::particles::{ParticleEmitter, EmitterParams};
//!
//! // Create emitter from parsed M2 data
//! let mut emitter = ParticleEmitter::new(&m2_particle_emitter);
//!
//! // Update each frame
//! let bone_transform = compute_bone_transform(emitter.bone_index());
//! emitter.update(dt_ms, &bone_transform, &animation_manager);
//!
//! // Get particle data for GPU upload
//! let particle_data = emitter.fill_texture_data();
//! ```
//!
//! # Reference
//!
//! Based on noclip.website's particle implementation:
//! `noclip.website/rust/src/wow/particles.rs`

mod emission;
mod emitter;
mod particle;

pub use emission::EmissionType;
pub use emitter::{EmitterParams, ParticleEmitter};
pub use particle::Particle;

/// Texels per particle in the GPU texture
/// - Texel 0: position.xyz + padding
/// - Texel 1: color.rgba
/// - Texel 2: scale.xy + padding
/// - Texel 3: tex_coord.xy + padding
pub const TEXELS_PER_PARTICLE: usize = 4;

/// Coordinate fix matrix for particle space conversion
/// Transforms from WoW particle space to standard 3D space
pub const PARTICLE_COORDINATE_FIX: [f32; 16] = [
    0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];
