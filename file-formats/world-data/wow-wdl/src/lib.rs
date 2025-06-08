//! Parser for World of Warcraft WDL (World Detail Level) files.
//!
//! This crate provides support for reading and writing WDL files used in
//! World of Warcraft. WDL files store low-resolution versions of terrain
//! used for distant terrain rendering and the world map interface.
//!
//! # Overview
//!
//! WDL files contain low-resolution heightmap data for entire continents,
//! providing:
//! - 17x17 heightmap points per map tile (ADT)
//! - Hole information for terrain gaps
//! - Optional low-resolution world object placements
//! - Support for efficient distant terrain rendering
//! - Data for world map and minimap generation
//!
//! ## File Structure
//!
//! WDL files use a chunk-based format similar to other WoW files:
//! - **MVER**: Version information (always first)
//! - **MAOF**: Map Area Offset table (64x64 grid of file offsets)
//! - **MARE**: Map Area heightmap data (17x17 outer + 16x16 inner heights)
//! - **MAHO**: Map Area hole data (16 uint16 bitmasks)
//! - **MWMO/MWID/MODF**: WMO placement data (pre-Legion)
//! - **ML\*\***: Model placement chunks (Legion+)
//!
//! # Examples
//!
//! ```rust,no_run
//! use std::fs::File;
//! use std::io::BufReader;
//! use wow_wdl::parser::WdlParser;
//!
//! // Open a WDL file
//! let file = File::open("path/to/file.wdl").unwrap();
//! let mut reader = BufReader::new(file);
//!
//! // Parse the file
//! let parser = WdlParser::new();
//! let wdl_file = parser.parse(&mut reader).unwrap();
//!
//! // Use the data
//! println!("WDL version: {}", wdl_file.version);
//! println!("Map tiles: {}", wdl_file.heightmap_tiles.len());
//!
//! // Get heightmap for a specific tile
//! if let Some(tile) = wdl_file.heightmap_tiles.get(&(32, 32)) {
//!     println!("Tile 32,32 has {} height values", tile.outer_values.len());
//! }
//! ```
//!
//! ## Version Conversion
//!
//! ```rust,no_run
//! use wow_wdl::parser::WdlParser;
//! use wow_wdl::version::WdlVersion;
//! use wow_wdl::conversion::convert_wdl_file;
//! use std::fs::File;
//! use std::io::{BufReader, BufWriter};
//!
//! // Parse an existing file
//! let file = File::open("input.wdl").unwrap();
//! let mut reader = BufReader::new(file);
//! let parser = WdlParser::new();
//! let wdl_file = parser.parse(&mut reader).unwrap();
//!
//! // Convert to Legion version
//! let legion_file = convert_wdl_file(&wdl_file, WdlVersion::Legion).unwrap();
//!
//! // Save the converted file
//! let output = File::create("output.wdl").unwrap();
//! let mut writer = BufWriter::new(output);
//! let legion_parser = WdlParser::with_version(WdlVersion::Legion);
//! legion_parser.write(&mut writer, &legion_file).unwrap();
//! ```
//!
//! See the `examples` directory for more detailed usage examples.

#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod conversion;
pub mod error;
pub mod parser;
pub mod types;
pub mod validation;
pub mod version;

// Re-export primary types
pub use error::{Result, WdlError};
pub use types::WdlFile;
pub use version::WdlVersion;
