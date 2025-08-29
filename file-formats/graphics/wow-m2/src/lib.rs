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
//! use wow_m2::{M2Model, M2Format, SkinFile, AnimFile, AnimFormat, M2Version, M2Converter};
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

// Re-export main components
pub mod anim;
pub mod chunks;
pub mod common;
pub mod converter;
pub mod error;
pub mod file_resolver;
pub mod header;
pub mod io_ext;
pub mod model;
pub mod skin;
pub mod version;

// Re-export common types
pub use anim::{AnimFile, AnimFormat, AnimMetadata, AnimSection, MemoryUsage};
pub use converter::M2Converter;
pub use error::{M2Error, Result};
pub use file_resolver::{FileResolver, ListfileResolver, PathResolver};
pub use model::{M2Format, M2Model, parse_m2};
pub use skin::{OldSkin, Skin, SkinFile, load_skin, parse_skin};
pub use version::M2Version;

// Re-export BLP types from wow-blp crate for backwards compatibility
pub use wow_blp::BlpImage as BlpTexture;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
