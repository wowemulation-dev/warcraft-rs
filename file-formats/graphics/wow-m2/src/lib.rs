//! A parser for World of Warcraft M2 model files with version conversion support
//!
//! This library allows parsing, validating, and converting M2 model files between different versions
//! of the World of Warcraft client, with automatic format detection for SKIN and ANIM dependencies.
//!
//! # Format Evolution
//!
//! M2 files evolved significantly across WoW versions:
//! - **Classic/TBC**: Embedded SKIN and ANIM data within M2
//! - **WotLK**: External SKIN files introduced
//! - **Cataclysm/MoP**: External ANIM files (raw format)
//! - **Legion+**: ANIM files use chunked format with MAOF magic
//!
//! # Example
//! ```rust,no_run
//! use wow_m2::{
//!     M2Model, M2Format, SkinFile, AnimFile, AnimFormat, M2Version, M2Converter,
//!     CoordinateSystem, CoordinateTransformer, transform_position,
//!     skinning::{M2Skinner, SkinningOptions}
//! };
//!
//! // Load a model with format detection
//! let model_format = M2Model::load("path/to/model.m2").unwrap();
//! let model = model_format.model();
//!
//! // Print model info
//! println!("Model name: {:?}", model.name);
//! println!("Model version: {:?}", model.header.version());
//! println!("Vertices: {}", model.vertices.len());
//! println!("Is chunked format: {}", model_format.is_chunked());
//!
//! // Transform vertices using skinning system
//! let mut skinner = M2Skinner::new(&model.bones, SkinningOptions::default());
//! skinner.calculate_bind_pose();
//! let skinned_vertices = skinner.skin_vertices(&model.vertices);
//! for (i, vertex) in skinned_vertices.iter().enumerate() {
//!     println!("Skinned vertex {}: {:?}", i, vertex);
//! }
//!
//! // Transform coordinates for Blender
//! let transformer = CoordinateTransformer::new(CoordinateSystem::Blender);
//! for vertex in &model.vertices {
//!     let blender_pos = transformer.transform_position(vertex.position);
//!     println!("WoW: {:?} → Blender: {:?}", vertex.position, blender_pos);
//! }
//!
//! // Convert to a different version
//! let converter = M2Converter::new();
//! let converted = converter.convert(model, M2Version::MoP).unwrap();
//!
//! // Save the converted model
//! converted.save("path/to/converted.m2").unwrap();
//!
//! // Load SKIN files with automatic format detection
//! let skin_file = SkinFile::load("path/to/model00.skin").unwrap();
//! match &skin_file {
//!     SkinFile::New(skin) => println!("New format SKIN with version {}", skin.header.version),
//!     SkinFile::Old(skin) => println!("Old format SKIN with {} indices", skin.indices.len()),
//! }
//!
//! // Access data regardless of format
//! let indices = skin_file.indices();
//! let submeshes = skin_file.submeshes();
//! println!("SKIN has {} indices and {} submeshes", indices.len(), submeshes.len());
//!
//! // Load ANIM files with automatic format detection
//! let anim_file = AnimFile::load("path/to/model0-0.anim").unwrap();
//! match &anim_file.format {
//!     AnimFormat::Legacy => println!("Legacy format with {} sections", anim_file.sections.len()),
//!     AnimFormat::Modern => println!("Modern format with {} sections", anim_file.sections.len()),
//! }
//! ```
//!
//! # Vertex Skinning
//!
//! The library provides a comprehensive vertex skinning system that transforms vertices
//! from their bind pose using bone weights and transformations:
//!
//! ```rust,no_run
//! use wow_m2::{M2Model, skinning::{M2Skinner, SkinningOptions}};
//!
//! // Load a model
//! let model_format = M2Model::load("path/to/model.m2").unwrap();
//! let model = model_format.model();
//!
//! // Create skinner with options
//! let options = SkinningOptions {
//!     normalize_weights: true,
//!     weight_threshold: 0.001,
//!     validate_bone_indices: true,
//!     handle_invalid_indices: true,
//! };
//! let mut skinner = M2Skinner::new(&model.bones, options);
//!
//! // Calculate bind pose (rest position)
//! skinner.calculate_bind_pose();
//!
//! // Transform all vertices
//! let skinned_vertices = skinner.skin_vertices(&model.vertices);
//! println!("Transformed {} vertices", skinned_vertices.len());
//!
//! // Or transform individual vertices
//! for vertex in &model.vertices {
//!     let skinned_pos = skinner.skin_single_vertex(vertex);
//!     println!("Original: {:?} → Skinned: {:?}", vertex.position, skinned_pos);
//! }
//! ```
//!
//! # Vertex Validation
//!
//! The library supports different validation modes when parsing vertex data to handle corruption
//! in older WoW models while preserving valid static geometry:
//!
//! ```rust,no_run
//! use wow_m2::{ValidationMode, chunks::vertex::M2Vertex};
//! use std::io::Cursor;
//!
//! # let data = vec![0u8; 48]; // Mock vertex data
//! let mut cursor = Cursor::new(data);
//!
//! // Strict mode: fixes all zero-weight vertices (legacy behavior)
//! // Use when you need all vertices to be animated
//! let vertex = M2Vertex::parse_with_validation(
//!     &mut cursor, 256, Some(34), ValidationMode::Strict
//! );
//!
//! // Permissive mode: preserves valid static geometry, fixes corruption (default)
//! // Best balance between fixing corruption and preserving intentional data
//! let vertex = M2Vertex::parse_with_validation(
//!     &mut cursor, 256, Some(34), ValidationMode::Permissive
//! );
//!
//! // No validation: preserves all original data
//! // Use when you need exact original data, even if corrupted
//! let vertex = M2Vertex::parse_with_validation(
//!     &mut cursor, 256, Some(34), ValidationMode::None
//! );
//! ```

// Re-export main components
pub mod anim;
pub mod animation;
pub mod chunks;
pub mod common;
pub mod converter;
pub mod coordinate;
pub mod embedded_skin;
pub mod error;
pub mod file_resolver;
pub mod header;
pub mod io_ext;
pub mod model;
pub mod model_animation_resolver;
pub mod model_enhanced;
pub mod particles;
pub mod skin;
pub mod skinning;
pub mod version;

// Re-export common types
pub use anim::{AnimFile, AnimFormat, AnimMetadata, AnimSection, MemoryUsage};
pub use animation::{
    AnimSequence, AnimationManager, AnimationState, BoneFlags, BoneTransformComputer, ComputedBone,
    Fixedi16, Lerp, Mat4 as AnimMat4, Quat, ResolvedBone, ResolvedTrack, Vec3 as AnimVec3,
};
pub use chunks::particle_emitter::{M2ParticleEmitter, M2ParticleEmitterType, M2ParticleFlags};
pub use chunks::vertex::ValidationMode;
pub use converter::M2Converter;
pub use coordinate::{
    CoordinateSystem, CoordinateTransformer, transform_position, transform_quaternion,
};
pub use error::{M2Error, Result};
pub use file_resolver::{FileResolver, ListfileResolver, PathResolver};
pub use model::{M2Format, M2Model, parse_m2};
pub use model_animation_resolver::{M2ModelAnimationExt, ResolvedBoneAnimation};
pub use model_enhanced::{
    AnimationInfo, BoneInfo, BoundingBox, EnhancedModelData, MaterialInfo, ModelStats, TextureInfo,
};
pub use particles::{EmissionType, EmitterParams, Particle, ParticleEmitter, TEXELS_PER_PARTICLE};
pub use skin::{OldSkin, Skin, SkinFile, load_skin, parse_skin};
pub use skinning::{BoneTransform, M2Skinner, SkinningOptions};
pub use version::M2Version;

// Re-export BLP types from wow-blp crate for backwards compatibility
pub use wow_blp::BlpImage as BlpTexture;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
