//! Two-pass parser for World of Warcraft ADT terrain files using binrw.
//!
//! This library implements a clean, type-safe parser for ADT (A Dungeon Terrain) files
//! from World of Warcraft using the binrw serialization framework. It supports all
//! ADT format versions from Vanilla (1.12.1) through Mists of Pandaria (5.4.8).
//!
//! ## Architecture
//!
//! The parser follows a two-pass architecture:
//!
//! **Pass 1 (Discovery)**: Fast chunk enumeration without parsing chunk data. Identifies
//! version, file type, and chunk locations for selective parsing.
//!
//! **Pass 2 (Parse)**: Type-safe extraction of chunk data into binrw-derived structures
//! with automatic offset resolution.
//!
//! ## Supported Versions
//!
//! - **Vanilla (1.x)** - Basic terrain chunks (MVER, MHDR, MCNK)
//! - **The Burning Crusade (2.x)** - Flight boundaries (MFBO chunk)
//! - **Wrath of the Lich King (3.x)** - Enhanced water/lava system (MH2O chunk)
//! - **Cataclysm (4.x)** - Split file architecture, texture amplifiers (MAMP chunk)
//! - **Mists of Pandaria (5.x)** - Texture parameters (MTXP chunk)
//!
//! Version detection is automatic based on chunk presence and structure analysis.
//!
//! ## Quick Start
//!
//! ```no_run
//! use std::fs::File;
//! use std::io::BufReader;
//! use wow_adt::{parse_adt, ParsedAdt};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Parse any ADT file with automatic version detection
//! let file = File::open("world/maps/azeroth/azeroth_32_32.adt")?;
//! let mut reader = BufReader::new(file);
//! let adt = parse_adt(&mut reader)?;
//!
//! // Access terrain data based on file type
//! match adt {
//!     ParsedAdt::Root(root) => {
//!         println!("Version: {:?}", root.version);
//!         println!("Terrain chunks: {}", root.mcnk_chunks.len());
//!         println!("Textures: {}", root.textures.len());
//!
//!         // Access water data (WotLK+)
//!         if let Some(water) = &root.water_data {
//!             for (idx, entry) in water.entries.iter().enumerate() {
//!                 if entry.header.has_liquid() {
//!                     println!("Chunk {} has {} water layer(s)",
//!                         idx, entry.instances.len());
//!                 }
//!             }
//!         }
//!     }
//!     ParsedAdt::Tex0(tex) => {
//!         println!("Texture file with {} entries", tex.textures.len());
//!     }
//!     ParsedAdt::Obj0(obj) => {
//!         println!("Object file with {} models", obj.models.len());
//!     }
//!     _ => {}
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Example: Fast Chunk Discovery
//!
//! ```no_run
//! use std::fs::File;
//! use wow_adt::{discover_chunks, ChunkId};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut file = File::open("terrain.adt")?;
//! let discovery = discover_chunks(&mut file)?;
//!
//! println!("Total chunks: {}", discovery.total_chunks);
//!
//! // Check for specific chunks without full parsing
//! if discovery.has_chunk(ChunkId::MH2O) {
//!     println!("File contains advanced water (WotLK+)");
//! }
//!
//! // Selective parsing: only parse specific chunks
//! for chunk_id in discovery.chunk_types() {
//!     println!("Found chunk: {}", chunk_id.as_str());
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Example: Building ADT Files
//!
//! ```no_run
//! use wow_adt::builder::AdtBuilder;
//! use wow_adt::{AdtVersion, DoodadPlacement};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
//! ## Example: Loading Split Files (Cataclysm+)
//!
//! ```no_run
//! use wow_adt::AdtSet;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Load complete split file set (automatically discovers _tex0, _obj0, _lod files)
//! let adt_set = AdtSet::load_from_path("World/Maps/Azeroth/Azeroth_30_30.adt")?;
//!
//! // Check if complete set
//! if adt_set.is_complete() {
//!     println!("Complete Cataclysm+ split file set");
//!     println!("Version: {:?}", adt_set.version());
//! }
//!
//! // Merge split files into unified structure
//! let merged = adt_set.merge()?;
//! println!("Merged {} MCNK chunks", merged.mcnk_chunks.len());
//! println!("Textures: {}", merged.textures.len());
//! println!("Models: {}", merged.models.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Features
//!
//! - **Automatic version detection** - Identifies WoW client version from chunk analysis
//! - **Split file support** - Cataclysm+ `_tex0`, `_obj0`, `_obj1`, `_lod` file handling
//! - **Type-safe parsing** - binrw derives for zero-overhead serialization
//! - **Fast discovery** - <10ms chunk inventory for selective parsing
//! - **Builder API** - Fluent builder for programmatically constructing ADT files
//!
//! ## Modules
//!
//! - [`adt_set`] - High-level API for loading complete split file sets (Cataclysm+)
//! - [`api`] - Core parser API (parse_adt, ParsedAdt enum)
//! - [`builder`] - Fluent builder API for constructing ADT files
//! - [`merger`] - Utilities for merging split files into unified structures
//! - [`split_set`] - Split file discovery and path management
//! - [`chunk_discovery`] - Discovery phase for fast chunk enumeration
//! - [`chunk_header`] - ChunkHeader binrw structure (8-byte magic + size)
//! - [`chunk_id`] - ChunkId type with reversed magic constants
//! - [`version`] - AdtVersion enum and detection logic
//! - [`file_type`] - AdtFileType enum (Root, Tex0, Obj0, etc.)
//! - [`error`] - AdtError types with detailed context
//! - [`chunks`] - Chunk structure definitions (MVER, MHDR, MCNK, etc.)
//!
//! ## References
//!
//! Based on information from:
//! - [WoW.dev ADT Format](https://wowdev.wiki/ADT) - Format specification
//! - [TrinityCore](https://github.com/TrinityCore/TrinityCore) - Server reference
//! - [noggit-red](https://github.com/Marlamin/noggit-red) - Map editor reference

// Public API modules
pub mod adt_set;
pub mod api;
pub mod builder;
pub mod chunk_discovery;
pub mod chunk_header;
pub mod chunk_id;
pub mod chunks;
pub mod error;
pub mod file_type;
pub mod merger;
pub mod split_set;
pub mod version;

// Internal parser modules
pub(crate) mod root_parser;
pub(crate) mod split_parser;

// Public re-exports for convenience
pub use adt_set::AdtSet;
pub use api::{
    AdtMetadata, LodAdt, McnkChunkObject, McnkChunkTexture, Obj0Adt, ObjectAdt, ParsedAdt, RootAdt,
    Tex0Adt, TextureAdt, parse_adt, parse_adt_with_metadata,
};
pub use builder::{AdtBuilder, BuiltAdt};
pub use chunk_discovery::{ChunkDiscovery, ChunkLocation, discover_chunks};
pub use chunk_header::ChunkHeader;
pub use chunk_id::ChunkId;
pub use error::{AdtError, Result};
pub use file_type::AdtFileType;
pub use version::AdtVersion;

// Chunk structure re-exports
pub use chunks::{
    // MCNK terrain chunks
    AlphaFormat,
    AlphaMap,
    // MH2O water chunks
    DepthOnlyVertex,
    // Placement chunks
    DoodadPlacement,
    HeightDepthVertex,
    HeightUvDepthVertex,
    HeightUvVertex,
    LiquidType,
    LiquidVertexFormat,
    // Simple chunks
    MampChunk,
    McalChunk,
    MccvChunk,
    McinChunk,
    McinEntry,
    MclqChunk,
    MclyChunk,
    MclyFlags,
    MclyLayer,
    McnkChunk,
    McnkFlags,
    McnkHeader,
    McnrChunk,
    McrfChunk,
    McseChunk,
    McshChunk,
    McvtChunk,
    MddfChunk,
    MfboChunk,
    Mh2oAttributes,
    Mh2oChunk,
    Mh2oEntry,
    Mh2oHeader,
    Mh2oInstance,
    MhdrChunk,
    // String chunks
    MmdxChunk,
    MmidChunk,
    ModfChunk,
    MtexChunk,
    MtxpChunk,
    MverChunk,
    MwidChunk,
    MwmoChunk,
    SoundEmitter,
    UvMapEntry,
    VertexColor,
    VertexNormal,
    WmoPlacement,
};
