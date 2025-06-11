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
//! use wow_wmo::{parse_wmo, WmoVersion};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Parse a WMO file
//! let file = File::open("example.wmo")?;
//! let mut reader = BufReader::new(file);
//! let wmo = parse_wmo(&mut reader)?;
//!
//! // Access WMO data
//! println!("Version: {:?}", wmo.version);
//! println!("Groups: {}", wmo.groups.len());
//! println!("Materials: {}", wmo.materials.len());
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

pub mod chunk;
pub mod converter;
pub mod error;
pub mod group_parser;
pub mod parser;
pub mod types;
pub mod validator;
pub mod version;
pub mod wmo_group_types;
pub mod wmo_types;
pub mod writer;

// Additional modules
pub mod editor;
pub mod visualizer;

// Test module (only compiled for tests)
#[cfg(test)]
mod tests;

pub use converter::WmoConverter;
pub use editor::WmoEditor;
pub use error::{Result, WmoError};
pub use group_parser::WmoGroupParser;
pub use parser::{WmoParser, chunks};
pub use types::{BoundingBox, ChunkId, Color, Vec3};
pub use validator::{ValidationError, ValidationReport, ValidationWarning, WmoValidator};
pub use version::{WmoFeature, WmoVersion};
pub use visualizer::WmoVisualizer;
// Re-export all types from wmo_types
pub use wmo_types::{
    WmoDoodadDef, WmoDoodadSet, WmoFlags, WmoGroupInfo, WmoHeader, WmoLight, WmoLightProperties,
    WmoLightType, WmoMaterial, WmoMaterialFlags, WmoPortal, WmoPortalReference, WmoRoot,
};

// Re-export all types from wmo_group_types (except WmoGroupFlags which conflicts)
pub use wmo_group_types::{
    TexCoord, WmoBatch, WmoBspNode, WmoGroup, WmoGroupFlags, WmoGroupHeader, WmoLiquid,
    WmoLiquidVertex, WmoMaterialInfo, WmoPlane,
};
pub use writer::WmoWriter;

/// Re-export of chunk-related types
pub use chunk::{Chunk, ChunkHeader};

/// Parse a WMO root file from a reader
pub fn parse_wmo<R: std::io::Read + std::io::Seek>(reader: &mut R) -> Result<WmoRoot> {
    let parser = WmoParser::new();
    parser.parse_root(reader)
}

/// Parse a WMO group file from a reader
pub fn parse_wmo_group<R: std::io::Read + std::io::Seek>(
    reader: &mut R,
    group_index: u32,
) -> Result<WmoGroup> {
    let parser = WmoGroupParser::new();
    parser.parse_group(reader, group_index)
}

/// Validate a WMO file from a reader
pub fn validate_wmo<R: std::io::Read + std::io::Seek>(reader: &mut R) -> Result<bool> {
    // A simple validation just checks if we can parse the file without errors
    match parse_wmo(reader) {
        Ok(_) => Ok(true),
        Err(e) => {
            // If it's a format error, return false. Otherwise, propagate the error.
            match e {
                WmoError::InvalidFormat(_)
                | WmoError::InvalidMagic { .. }
                | WmoError::InvalidVersion(_)
                | WmoError::MissingRequiredChunk(_) => Ok(false),
                _ => Err(e),
            }
        }
    }
}

/// Perform detailed validation on a WMO root file
pub fn validate_wmo_detailed<R: std::io::Read + std::io::Seek>(
    reader: &mut R,
) -> Result<ValidationReport> {
    let wmo = parse_wmo(reader)?;
    let validator = WmoValidator::new();
    validator.validate_root(&wmo)
}

/// Perform detailed validation on a WMO group file
pub fn validate_wmo_group_detailed<R: std::io::Read + std::io::Seek>(
    reader: &mut R,
    group_index: u32,
) -> Result<ValidationReport> {
    let group = parse_wmo_group(reader, group_index)?;
    let validator = WmoValidator::new();
    validator.validate_group(&group)
}

/// Convert a WMO file from one version to another
pub fn convert_wmo<R, W>(reader: &mut R, writer: &mut W, target_version: WmoVersion) -> Result<()>
where
    R: std::io::Read + std::io::Seek,
    W: std::io::Write + std::io::Seek,
{
    let mut wmo = parse_wmo(reader)?;

    // Convert WMO to target version
    let converter = WmoConverter::new();
    converter.convert_root(&mut wmo, target_version)?;

    // Write converted WMO
    let writer_obj = WmoWriter::new();
    writer_obj.write_root(writer, &wmo, target_version)?;

    Ok(())
}

/// Convert a WMO group file from one version to another
pub fn convert_wmo_group<R, W>(
    reader: &mut R,
    writer: &mut W,
    target_version: WmoVersion,
    group_index: u32,
) -> Result<()>
where
    R: std::io::Read + std::io::Seek,
    W: std::io::Write + std::io::Seek,
{
    // Read root file first to get current version
    let wmo = parse_wmo(reader)?;
    let current_version = wmo.version;

    // Now read group file
    reader.rewind()?;
    let mut group = parse_wmo_group(reader, group_index)?;

    // Convert group to target version
    let converter = WmoConverter::new();
    converter.convert_group(&mut group, target_version, current_version)?;

    // Write converted group
    let writer_obj = WmoWriter::new();
    writer_obj.write_group(writer, &group, target_version)?;

    Ok(())
}
