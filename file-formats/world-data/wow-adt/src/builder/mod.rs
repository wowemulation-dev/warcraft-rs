//! Builder API for programmatically constructing ADT terrain files.
//!
//! This module provides a fluent API for building valid ADT files with:
//! - Automatic offset calculation
//! - Progressive validation
//! - Version-aware serialization
//!
//! # Quick Start
//!
//! ```no_run
//! use wow_adt::builder::AdtBuilder;
//! use wow_adt::AdtVersion;
//! use wow_adt::DoodadPlacement;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let adt = AdtBuilder::new()
//!     .with_version(AdtVersion::WotLK)
//!     .add_texture("terrain/grass_01.blp")
//!     .add_texture("terrain/dirt_01.blp")
//!     .add_model("doodad/tree_01.m2")
//!     .add_doodad_placement(DoodadPlacement {
//!         name_id: 0,
//!         unique_id: 1,
//!         position: [1000.0, 1000.0, 100.0],
//!         rotation: [0.0, 0.0, 0.0],
//!         scale: 1024,
//!         flags: 0,
//!     })
//!     .build()?;
//!
//! adt.write_to_file("world/maps/custom/custom_32_32.adt")?;
//! # Ok(())
//! # }
//! ```
//!
//! # Validation
//!
//! The builder performs progressive validation:
//! - **Method call**: Basic parameter checks (non-null, valid ranges)
//! - **Add operations**: Format validation (file extensions, path format)
//! - **Build call**: Structural validation (required chunks, reference validity)
//! - **Serialization**: Final checks (offset overflow, size limits)

mod adt_builder;
mod built_adt;
mod serializer;
pub mod validation;

pub use adt_builder::AdtBuilder;
pub use built_adt::BuiltAdt;
