//! A parser for World of Warcraft M2 model files with version conversion support
//!
//! This library allows parsing, validating, and converting M2 model files between different versions
//! of the World of Warcraft client.
//!
//! # Example
//! ```rust,no_run
//! use wow_m2::model::M2Model;
//! use wow_m2::version::M2Version;
//! use wow_m2::converter::M2Converter;
//!
//! // Load a model
//! let model = M2Model::load("path/to/model.m2").unwrap();
//!
//! // Print model info
//! println!("Model name: {:?}", model.name);
//! println!("Model version: {:?}", model.header.version());
//! println!("Vertices: {}", model.vertices.len());
//!
//! // Convert to a different version
//! let converter = M2Converter::new();
//! let converted = converter.convert(&model, M2Version::MoP).unwrap();
//!
//! // Save the converted model
//! converted.save("path/to/converted.m2").unwrap();
//! ```

// Re-export main components
pub mod anim;
pub mod blp;
pub mod chunks;
pub mod common;
pub mod converter;
pub mod error;
pub mod header;
pub mod io_ext;
pub mod model;
pub mod skin;
pub mod version;

// Re-export common types
pub use anim::AnimFile;
pub use blp::BlpTexture;
pub use converter::M2Converter;
pub use error::{M2Error, Result};
pub use model::M2Model;
pub use skin::{OldSkin, Skin};
pub use version::M2Version;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
