//! # World of Warcraft WMO (World Map Object) Parser
//!
//! This library provides comprehensive support for parsing, editing, validating, and converting
//! World of Warcraft WMO files across all game expansions from Classic to The War Within.
//!
//! ## Features
//!
//! - **Parsing**: Read WMO root and group files with full chunk support
//! - **Validation**: Verify file integrity and format compliance
//! - **Conversion**: Convert between different WoW expansion formats
//! - **Editing**: Modify WMO properties, geometry, and metadata
//! - **Export**: Export to common 3D formats (OBJ/MTL)
//! - **Type Safety**: Strongly typed structures for all WMO components
//!
//! ## Quick Start
//!
//! ```no_run
//! use std::fs::File;
//! use std::io::BufReader;
//! use wow_wmo::{parse_wmo, ParsedWmo};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Parse a WMO file
//! let file = File::open("example.wmo")?;
//! let mut reader = BufReader::new(file);
//! let wmo = parse_wmo(&mut reader)?;
//!
//! // Access WMO data based on type
//! match &wmo {
//!     ParsedWmo::Root(root) => {
//!         println!("Root file - Version: {}", root.version);
//!         println!("Groups: {}", root.n_groups);
//!         println!("Materials: {}", root.n_materials);
//!     }
//!     ParsedWmo::Group(group) => {
//!         println!("Group file - Version: {}", group.version);
//!         println!("Vertices: {}", group.n_vertices);
//!         println!("Triangles: {}", group.n_triangles);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Example: Validating a WMO File
//!
//! ```no_run
//! use std::fs::File;
//! use std::io::BufReader;
//! use wow_wmo::{WmoParser, WmoValidator};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Parse the WMO
//! let file = File::open("building.wmo")?;
//! let mut reader = BufReader::new(file);
//! let wmo = WmoParser::new().parse_root(&mut reader)?;
//!
//! // Validate it
//! let validator = WmoValidator::new();
//! let report = validator.validate_root(&wmo)?;
//!
//! if !report.errors.is_empty() {
//!     println!("Validation errors found:");
//!     for error in &report.errors {
//!         println!("  - {:?}", error);
//!     }
//! }
//!
//! if !report.warnings.is_empty() {
//!     println!("Validation warnings:");
//!     for warning in &report.warnings {
//!         println!("  - {:?}", warning);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Modules
//!
//! - [`chunk`]: Low-level chunk reading/writing functionality
//! - [`parser`]: WMO root file parser
//! - [`group_parser`]: WMO group file parser
//! - [`types`]: Common data types (Vec3, Color, BoundingBox, etc.)
//! - [`wmo_types`]: WMO root file structures
//! - [`wmo_group_types`]: WMO group file structures
//! - [`validator`]: File validation and integrity checking
//! - [`converter`]: Version conversion between expansions
//! - [`editor`]: High-level editing API
//! - [`writer`]: Binary serialization
//! - [`visualizer`]: 3D export functionality
//! - [`version`]: Version detection and feature support
//! - [`error`]: Error types and handling
//!
//! ## Supported Formats
//!
//! This library supports WMO files from all World of Warcraft expansions:
//! - Classic (1.12)
//! - The Burning Crusade (2.4.3)
//! - Wrath of the Lich King (3.3.5)
//! - Cataclysm (4.3.4)
//! - Mists of Pandaria (5.4.8)
//! - Warlords of Draenor (6.2.4)
//! - Legion (7.3.5)
//! - Battle for Azeroth (8.3.7)
//! - Shadowlands (9.2.7)
//! - Dragonflight (10.2.0)
//! - The War Within (11.0+)

pub mod api;
pub mod bsp;
pub mod chunk;
pub mod chunk_discovery;
pub mod chunk_header;
pub mod chunk_id;
pub mod chunks;
pub mod converter;
pub mod error;
pub mod file_type;
pub mod group_parser;
pub mod parser;
pub mod portal;
pub mod root_parser;
pub mod types;
pub mod validator;
pub mod version;
pub mod version_detection;
pub mod wmo_group_types;
pub mod wmo_types;
pub mod writer;

// Additional modules
pub mod editor;
pub mod visualizer;

// Test modules (only compiled for tests)
// #[cfg(test)]
// mod tests;  // Temporarily disabled while refactoring
#[cfg(test)]
mod missing_chunks_test;

pub use converter::WmoConverter;
pub use editor::WmoEditor;
pub use error::{Result, WmoError};
pub use group_parser::WmoGroupParser;
pub use parser::WmoParser;
pub use types::{BoundingBox, Color, Vec3};
pub use validator::{ValidationError, ValidationReport, ValidationWarning, WmoValidator};
pub use version::{WmoFeature, WmoVersion};
pub use visualizer::WmoVisualizer;
// Re-export all types from wmo_types
pub use wmo_types::{
    WmoConvexVolumePlane, WmoConvexVolumePlanes, WmoDoodadDef, WmoDoodadSet, WmoFlags,
    WmoGroupInfo, WmoHeader, WmoLight, WmoLightProperties, WmoLightType, WmoMaterial,
    WmoMaterialFlags, WmoPortal, WmoPortalReference, WmoRoot,
};

// Re-export all types from wmo_group_types (except WmoGroupFlags which conflicts)
pub use wmo_group_types::{
    TexCoord, WmoBatch, WmoBspNode, WmoGroup, WmoGroupFlags, WmoGroupHeader, WmoLiquid,
    WmoLiquidVertex, WmoMaterialInfo, WmoPlane,
};
pub use writer::WmoWriter;

// Portal culling exports
pub use portal::{
    AABB, Axis, ConvexHull, GroupLocationData, GroupPortalInfo, Plane, Portal, PortalCuller,
    PortalRef, VisibilityResult, WmoGroupLocator,
};

// BSP tree exports
pub use bsp::{BspAxisType, BspNodeExt, BspTree, point_in_group};

/// Re-export of chunk-related types
pub use chunk::Chunk;
pub use chunk_header::ChunkHeader;
pub use chunk_id::ChunkId;

/// Re-export API types from our binrw implementation
pub use api::{ParseResult, ParsedWmo, discover_wmo_chunks, parse_wmo, parse_wmo_with_metadata};

// Internal parsers are accessed via the ParsedWmo enum from api module

// Validation and conversion functionality will use the new parser
// These can be implemented later when the legacy structures are removed
