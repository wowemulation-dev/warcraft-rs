//! High-level parser API for ADT terrain files.
//!
//! This module provides type-safe, version-aware parsing of World of Warcraft ADT
//! (A Dungeon Terrain) files using the two-pass architecture:
//!
//! **Pass 1 (Discovery)**: Fast chunk enumeration to detect version and file type
//! **Pass 2 (Parse)**: Type-safe extraction of chunk data into structured formats
//!
//! # Quick Start
//!
//! ```no_run
//! use std::fs::File;
//! use std::io::BufReader;
//! use wow_adt::api::parse_adt;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let file = File::open("world/maps/azeroth/azeroth_32_32.adt")?;
//! let mut reader = BufReader::new(file);
//! let adt = parse_adt(&mut reader)?;
//!
//! match adt {
//!     wow_adt::api::ParsedAdt::Root(root) => {
//!         println!("Version: {:?}", root.version);
//!         println!("Terrain chunks: {}", root.mcnk_chunks.len());
//!         println!("Textures: {}", root.textures.len());
//!
//!         if let Some(water) = &root.water_data {
//!             println!("Has WotLK+ water: {} chunks with water",
//!                 water.liquid_chunk_count());
//!         }
//!     }
//!     _ => {}
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Performance
//!
//! - Discovery phase: <10ms for typical ADT files (5-15 MB)
//! - Full parse: 100 ADT files in <5 seconds (50ms per file average)
//! - Memory: ≤2× raw file size peak

use std::io::{Read, Seek};
use std::time::{Duration, Instant};

use crate::chunk_discovery::{ChunkDiscovery, discover_chunks};
use crate::chunks::mh2o::Mh2oChunk;
use crate::chunks::{
    DoodadPlacement, MampChunk, MbbbChunk, MbmhChunk, MbmiChunk, MbnvChunk, McalChunk, McinChunk,
    MclyChunk, McnkChunk, MfboChunk, MhdrChunk, MtxfChunk, MtxpChunk, WmoPlacement,
};
use crate::error::Result;
use crate::file_type::AdtFileType;
use crate::version::AdtVersion;

/// Parsed root ADT file (main terrain file for all versions).
///
/// Contains all terrain data for a single 16×16 yard area:
/// - Heightmaps and vertex normals
/// - Texture layers and alpha blending
/// - Object placements (M2 models and WMOs)
/// - Version-specific features (water, flight bounds, etc.)
///
/// # Guarantees
///
/// - `version` correctly identifies format version
/// - All required chunks present (MHDR, MCIN, MTEX, MCNK)
/// - Version-specific chunks only present for compatible versions
/// - All indices valid within their respective arrays
/// - MCNK count ∈ [1, 256]
///
/// # Example
///
/// ```no_run
/// use wow_adt::api::{parse_adt, ParsedAdt};
/// use std::fs::File;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut file = File::open("terrain.adt")?;
/// let adt = parse_adt(&mut file)?;
///
/// if let ParsedAdt::Root(root) = adt {
///     println!("Version: {:?}", root.version);
///     println!("Terrain chunks: {}", root.mcnk_chunks.len());
///
///     if let Some(water) = &root.water_data {
///         // Access water data by chunk index
///         for (idx, entry) in water.entries.iter().enumerate() {
///             if entry.header.has_liquid() {
///                 let row = idx / 16;
///                 let col = idx % 16;
///                 println!("Chunk ({}, {}) has {} water layer(s)",
///                     row, col, entry.instances.len());
///             }
///         }
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct RootAdt {
    /// Detected ADT version
    pub version: AdtVersion,

    /// MHDR - Header with chunk offsets
    pub mhdr: MhdrChunk,

    /// MCIN - MCNK chunk index (256 entries)
    pub mcin: McinChunk,

    /// Texture filenames from MTEX chunk
    pub textures: Vec<String>,

    /// M2 model filenames from MMDX chunk
    pub models: Vec<String>,

    /// M2 model filename offsets from MMID chunk
    pub model_indices: Vec<u32>,

    /// WMO filenames from MWMO chunk
    pub wmos: Vec<String>,

    /// WMO filename offsets from MWID chunk
    pub wmo_indices: Vec<u32>,

    /// M2 model placements from MDDF chunk
    pub doodad_placements: Vec<DoodadPlacement>,

    /// WMO placements from MODF chunk
    pub wmo_placements: Vec<WmoPlacement>,

    /// MCNK terrain chunks (1-256 chunks)
    pub mcnk_chunks: Vec<McnkChunk>,

    /// MFBO - Flight boundaries (TBC+)
    pub flight_bounds: Option<MfboChunk>,

    /// MH2O - Advanced water system (WotLK+)
    ///
    /// Contains 256 entries (one per MCNK chunk) with liquid layer data.
    /// Each entry has header, instances, and optional attributes.
    pub water_data: Option<Mh2oChunk>,

    /// MTXF - Texture flags (WotLK 3.x+)
    ///
    /// Rendering flags for each texture controlling specularity,
    /// environment mapping, and animation.
    pub texture_flags: Option<MtxfChunk>,

    /// MAMP - Texture amplifier (Cataclysm+)
    pub texture_amplifier: Option<MampChunk>,

    /// MTXP - Texture parameters (MoP+)
    pub texture_params: Option<MtxpChunk>,

    /// MBMH - Blend mesh headers (MoP 5.x+)
    ///
    /// Headers describing blend mesh batches for smooth texture transitions.
    /// Each entry contains map_object_id, texture_id, and index/vertex ranges.
    pub blend_mesh_headers: Option<MbmhChunk>,

    /// MBBB - Blend mesh bounding boxes (MoP 5.x+)
    ///
    /// Bounding boxes for visibility culling of blend meshes.
    /// Each entry has map_object_id and min/max coordinates.
    pub blend_mesh_bounds: Option<MbbbChunk>,

    /// MBNV - Blend mesh vertices (MoP 5.x+)
    ///
    /// Vertex data for blend mesh system with position, normal, UV coordinates,
    /// and 3 RGBA color channels for texture blending.
    pub blend_mesh_vertices: Option<MbnvChunk>,

    /// MBMI - Blend mesh indices (MoP 5.x+)
    ///
    /// Triangle indices (u16) referencing MBNV vertex array.
    /// MCBB chunks in MCNK reference ranges within this array.
    pub blend_mesh_indices: Option<MbmiChunk>,
}

impl RootAdt {
    /// Check if this ADT has water data.
    #[must_use]
    pub fn has_water(&self) -> bool {
        self.water_data.is_some()
    }

    /// Check if this ADT has flight boundaries.
    #[must_use]
    pub fn has_flight_bounds(&self) -> bool {
        self.flight_bounds.is_some()
    }

    /// Get number of terrain chunks.
    #[must_use]
    pub fn terrain_chunk_count(&self) -> usize {
        self.mcnk_chunks.len()
    }

    /// Get number of textures.
    #[must_use]
    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }

    /// Get number of M2 models.
    #[must_use]
    pub fn model_count(&self) -> usize {
        self.models.len()
    }

    /// Get number of WMO objects.
    #[must_use]
    pub fn wmo_count(&self) -> usize {
        self.wmos.len()
    }

    // ========================================================================
    // Mutable Access Methods (for modify workflows)
    // ========================================================================

    /// Get mutable access to textures for replacement workflows.
    ///
    /// Use this to modify texture filenames in place. After modification,
    /// serialize using [`AdtBuilder::from_parsed()`](crate::builder::AdtBuilder::from_parsed).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use wow_adt::api::parse_adt;
    /// # use std::fs::File;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut file = File::open("terrain.adt")?;
    /// # let adt = parse_adt(&mut file)?;
    /// # if let wow_adt::api::ParsedAdt::Root(mut root) = adt {
    /// root.textures_mut()[0] = "terrain/grass_new.blp".to_string();
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    pub fn textures_mut(&mut self) -> &mut Vec<String> {
        &mut self.textures
    }

    /// Get mutable access to M2 model filenames.
    pub fn models_mut(&mut self) -> &mut Vec<String> {
        &mut self.models
    }

    /// Get mutable access to WMO filenames.
    pub fn wmos_mut(&mut self) -> &mut Vec<String> {
        &mut self.wmos
    }

    /// Get mutable access to M2 model placements.
    ///
    /// Use this to add, remove, or modify doodad placements.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use wow_adt::api::parse_adt;
    /// # use std::fs::File;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut file = File::open("terrain.adt")?;
    /// # let adt = parse_adt(&mut file)?;
    /// # if let wow_adt::api::ParsedAdt::Root(mut root) = adt {
    /// root.doodad_placements_mut()[0].position[2] += 10.0; // Z coordinate
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    pub fn doodad_placements_mut(&mut self) -> &mut Vec<DoodadPlacement> {
        &mut self.doodad_placements
    }

    /// Get mutable access to WMO placements.
    pub fn wmo_placements_mut(&mut self) -> &mut Vec<WmoPlacement> {
        &mut self.wmo_placements
    }

    /// Get mutable access to MCNK terrain chunks.
    ///
    /// Use this to modify terrain geometry, heights, textures, etc.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use wow_adt::api::parse_adt;
    /// # use std::fs::File;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut file = File::open("terrain.adt")?;
    /// # let adt = parse_adt(&mut file)?;
    /// # if let wow_adt::api::ParsedAdt::Root(mut root) = adt {
    /// if let Some(heights) = &mut root.mcnk_chunks_mut()[0].heights {
    ///     for height in &mut heights.heights {
    ///         *height += 5.0;
    ///     }
    /// }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    pub fn mcnk_chunks_mut(&mut self) -> &mut Vec<McnkChunk> {
        &mut self.mcnk_chunks
    }

    /// Get mutable access to water data (WotLK+).
    ///
    /// Returns `None` if this ADT has no water data.
    pub fn water_data_mut(&mut self) -> Option<&mut Mh2oChunk> {
        self.water_data.as_mut()
    }

    /// Get mutable access to flight boundaries (TBC+).
    ///
    /// Returns `None` if this ADT has no flight bounds.
    pub fn flight_bounds_mut(&mut self) -> Option<&mut MfboChunk> {
        self.flight_bounds.as_mut()
    }

    /// Get mutable access to texture flags (WotLK+).
    pub fn texture_flags_mut(&mut self) -> Option<&mut MtxfChunk> {
        self.texture_flags.as_mut()
    }

    /// Get mutable access to texture amplifier (Cataclysm+).
    pub fn texture_amplifier_mut(&mut self) -> Option<&mut MampChunk> {
        self.texture_amplifier.as_mut()
    }

    /// Get mutable access to texture parameters (MoP+).
    pub fn texture_params_mut(&mut self) -> Option<&mut MtxpChunk> {
        self.texture_params.as_mut()
    }
}

/// MCNK texture data container for split texture files.
///
/// In Cataclysm+ split architecture, texture files contain simplified MCNK chunks
/// that only hold texture layer and alpha map data. This structure represents the
/// per-chunk texture information.
///
/// # Format
///
/// Each texture file has 256 MCNK containers (16x16 grid), one for each terrain chunk.
/// Unlike root file MCNK chunks, these do not contain full headers or geometry data.
///
/// # References
///
/// - wowdev.wiki: ADT/v18#Texture Files (_tex0.adt)
#[derive(Debug, Clone)]
pub struct McnkChunkTexture {
    /// Chunk index in 16x16 grid (0-255)
    pub index: usize,

    /// MCLY - Texture layer definitions (up to 4 layers per chunk)
    pub layers: Option<MclyChunk>,

    /// MCAL - Alpha maps for texture blending
    pub alpha_maps: Option<McalChunk>,
}

/// MCNK object data container for split object files.
///
/// In Cataclysm+ split architecture, object files contain simplified MCNK chunks
/// that only hold references to M2 models and WMO objects placed within that chunk.
///
/// # Format
///
/// Each object file has 256 MCNK containers (16x16 grid), one for each terrain chunk.
/// The MCRD/MCRW chunks contain indices into the global MDDF/MODF arrays.
///
/// # References
///
/// - wowdev.wiki: ADT/v18#Object Files (_obj0.adt)
#[derive(Debug, Clone)]
pub struct McnkChunkObject {
    /// Chunk index in 16x16 grid (0-255)
    pub index: usize,

    /// MCRD - M2 doodad reference indices (Cataclysm+)
    ///
    /// Indices into the global MDDF array for M2 models placed in this chunk.
    pub doodad_refs: Vec<u32>,

    /// MCRW - WMO reference indices (Cataclysm+)
    ///
    /// Indices into the global MODF array for WMO objects placed in this chunk.
    pub wmo_refs: Vec<u32>,
}

/// Parsed texture file (Cataclysm+ `_tex0.adt`).
///
/// Contains texture-related data split from root ADT in Cataclysm+:
/// - Texture filenames
/// - Texture layer definitions (per-chunk)
/// - Alpha maps for blending (per-chunk)
/// - Texture parameters (MoP+)
///
/// # Format
///
/// Texture files contain MTEX (global texture list) and 256 MCNK containers
/// with texture layer and alpha map data for each terrain chunk.
///
/// # Example
///
/// ```no_run
/// use wow_adt::api::{parse_adt, ParsedAdt};
/// use std::fs::File;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut file = File::open("terrain_tex0.adt")?;
/// let adt = parse_adt(&mut file)?;
///
/// if let ParsedAdt::Tex0(tex) = adt {
///     println!("Textures: {}", tex.textures.len());
///     println!("Chunks with texture data: {}", tex.mcnk_textures.len());
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Tex0Adt {
    /// Detected ADT version
    pub version: AdtVersion,

    /// Texture filenames from MTEX chunk
    pub textures: Vec<String>,

    /// MTXP - Texture parameters (MoP+)
    pub texture_params: Option<MtxpChunk>,

    /// Per-chunk texture data (256 chunks in 16x16 grid)
    pub mcnk_textures: Vec<McnkChunkTexture>,
}

/// Parsed object file (Cataclysm+ `_obj0.adt`).
///
/// Contains object placement data split from root ADT in Cataclysm+:
/// - M2 model filenames and placements
/// - WMO filenames and placements
/// - Per-chunk object references (MCRD/MCRW)
///
/// # Format
///
/// Object files contain global object lists (MMDX/MWMO) and placement arrays
/// (MDDF/MODF), plus 256 MCNK containers with per-chunk object references.
///
/// # Example
///
/// ```no_run
/// use wow_adt::api::{parse_adt, ParsedAdt};
/// use std::fs::File;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut file = File::open("terrain_obj0.adt")?;
/// let adt = parse_adt(&mut file)?;
///
/// if let ParsedAdt::Obj0(obj) = adt {
///     println!("M2 models: {}", obj.models.len());
///     println!("WMO objects: {}", obj.wmos.len());
///     println!("Chunks with objects: {}", obj.mcnk_objects.len());
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Obj0Adt {
    /// Detected ADT version
    pub version: AdtVersion,

    /// M2 model filenames from MMDX chunk
    pub models: Vec<String>,

    /// M2 model filename offsets from MMID chunk
    pub model_indices: Vec<u32>,

    /// WMO filenames from MWMO chunk
    pub wmos: Vec<String>,

    /// WMO filename offsets from MWID chunk
    pub wmo_indices: Vec<u32>,

    /// M2 model placements from MDDF chunk
    pub doodad_placements: Vec<DoodadPlacement>,

    /// WMO placements from MODF chunk
    pub wmo_placements: Vec<WmoPlacement>,

    /// Per-chunk object references (256 chunks in 16x16 grid)
    pub mcnk_objects: Vec<McnkChunkObject>,
}

/// Parsed LOD file (Cataclysm+ `_lod.adt`).
///
/// Contains level-of-detail data for distant terrain rendering.
/// This format is minimally documented and currently a stub implementation.
#[derive(Debug, Clone)]
pub struct LodAdt {
    /// Detected ADT version
    pub version: AdtVersion,
}

// ============================================================================
// Type Aliases for Spec Compliance
// ============================================================================
//
// The split file specification uses names like `TextureAdt` and `ObjectAdt`,
// while our implementation uses `Tex0Adt` and `Obj0Adt` to reflect the actual
// file naming convention (`_tex0.adt`, `_obj0.adt`).
//
// These type aliases provide compatibility with the specification naming
// while preserving the more accurate implementation names.

/// Type alias for texture file structure (spec-compliant naming).
///
/// This is an alias for [`Tex0Adt`], providing compatibility with the
/// split file specification which uses `TextureAdt` as the canonical name.
///
/// # Example
///
/// ```rust
/// use wow_adt::api::TextureAdt;
/// // Equivalent to using Tex0Adt
/// ```
pub type TextureAdt = Tex0Adt;

/// Type alias for object file structure (spec-compliant naming).
///
/// This is an alias for [`Obj0Adt`], providing compatibility with the
/// split file specification which uses `ObjectAdt` as the canonical name.
///
/// # Example
///
/// ```rust
/// use wow_adt::api::ObjectAdt;
/// // Equivalent to using Obj0Adt
/// ```
pub type ObjectAdt = Obj0Adt;

/// Parsed ADT file with type-safe variant access.
///
/// Different ADT file types are represented as enum variants, allowing
/// type-safe access to file-specific data.
///
/// # Variants
///
/// - [`Root`](ParsedAdt::Root) - Main terrain file (all versions)
/// - [`Tex0`](ParsedAdt::Tex0) - Texture data (Cataclysm+)
/// - [`Tex1`](ParsedAdt::Tex1) - Additional textures (Cataclysm+)
/// - [`Obj0`](ParsedAdt::Obj0) - Object placements (Cataclysm+)
/// - [`Obj1`](ParsedAdt::Obj1) - Additional objects (Cataclysm+)
/// - [`Lod`](ParsedAdt::Lod) - Level-of-detail (Cataclysm+)
///
/// # Example
///
/// ```no_run
/// use wow_adt::api::{parse_adt, ParsedAdt};
/// use std::fs::File;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut file = File::open("terrain.adt")?;
/// let adt = parse_adt(&mut file)?;
///
/// match adt {
///     ParsedAdt::Root(root) => {
///         println!("Root ADT with {} chunks", root.mcnk_chunks.len());
///     }
///     ParsedAdt::Tex0(tex) => {
///         println!("Texture file with {} textures", tex.textures.len());
///     }
///     ParsedAdt::Obj0(obj) => {
///         println!("Object file with {} models", obj.models.len());
///     }
///     _ => {}
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub enum ParsedAdt {
    /// Root ADT file (main terrain)
    Root(Box<RootAdt>),

    /// Texture file 0 (Cataclysm+)
    Tex0(Tex0Adt),

    /// Texture file 1 (Cataclysm+)
    Tex1(Tex0Adt),

    /// Object file 0 (Cataclysm+)
    Obj0(Obj0Adt),

    /// Object file 1 (Cataclysm+)
    Obj1(Obj0Adt),

    /// Level-of-detail file (Cataclysm+)
    Lod(LodAdt),
}

impl ParsedAdt {
    /// Check if this is a root ADT file.
    #[must_use]
    pub const fn is_root(&self) -> bool {
        matches!(self, Self::Root(_))
    }

    /// Check if this is a split file (Cataclysm+).
    #[must_use]
    pub const fn is_split(&self) -> bool {
        !self.is_root()
    }

    /// Get the file type.
    #[must_use]
    pub const fn file_type(&self) -> AdtFileType {
        match self {
            Self::Root(_) => AdtFileType::Root,
            Self::Tex0(_) => AdtFileType::Tex0,
            Self::Tex1(_) => AdtFileType::Tex1,
            Self::Obj0(_) => AdtFileType::Obj0,
            Self::Obj1(_) => AdtFileType::Obj1,
            Self::Lod(_) => AdtFileType::Lod,
        }
    }

    /// Get the ADT version.
    #[must_use]
    pub fn version(&self) -> AdtVersion {
        match self {
            Self::Root(r) => r.version,
            Self::Tex0(t) | Self::Tex1(t) => t.version,
            Self::Obj0(o) | Self::Obj1(o) => o.version,
            Self::Lod(l) => l.version,
        }
    }
}

/// Diagnostic metadata for format research and debugging.
///
/// Contains performance metrics and warnings collected during parsing.
/// Useful for format analysis, debugging, and performance profiling.
///
/// # Example
///
/// ```no_run
/// use wow_adt::api::parse_adt_with_metadata;
/// use std::fs::File;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut file = File::open("terrain.adt")?;
/// let (adt, metadata) = parse_adt_with_metadata(&mut file)?;
///
/// println!("Version: {:?}", metadata.version);
/// println!("Discovery: {:?}", metadata.discovery_duration);
/// println!("Parse: {:?}", metadata.parse_duration);
/// println!("Chunks: {}", metadata.chunk_count);
///
/// for warning in &metadata.warnings {
///     eprintln!("Warning: {}", warning);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct AdtMetadata {
    /// Detected ADT version
    pub version: AdtVersion,

    /// Detected file type
    pub file_type: AdtFileType,

    /// Total number of chunks discovered
    pub chunk_count: usize,

    /// Time spent in discovery phase
    pub discovery_duration: Duration,

    /// Time spent in parse phase
    pub parse_duration: Duration,

    /// Warning messages collected during parsing
    pub warnings: Vec<String>,

    /// Full chunk discovery results
    pub discovery: ChunkDiscovery,
}

/// Parse ADT file with automatic version detection and type-safe chunk extraction.
///
/// This is the primary entry point for parsing ADT files. It automatically detects
/// the file type and version, then parses all chunks into type-safe structures.
///
/// # Arguments
///
/// * `reader` - Seekable input stream (File, Cursor, etc.)
///
/// # Returns
///
/// - `Ok(ParsedAdt)` - Successfully parsed ADT with appropriate variant
/// - `Err(AdtError)` - Parsing failed with detailed error context
///
/// # Behavior
///
/// 1. **Discovery Phase**: Enumerate chunks, detect version and file type
/// 2. **Parse Phase**: Extract chunk data into type-safe structures
/// 3. **Validation**: Verify required chunks and structural integrity
/// 4. **Error Handling**: Fail-fast on critical errors
///
/// # Example
///
/// ```no_run
/// use wow_adt::api::{parse_adt, ParsedAdt};
/// use std::fs::File;
/// use std::io::BufReader;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let file = File::open("world/maps/azeroth/azeroth_32_32.adt")?;
/// let mut reader = BufReader::new(file);
/// let adt = parse_adt(&mut reader)?;
///
/// match adt {
///     ParsedAdt::Root(root) => {
///         println!("Loaded {} terrain chunks", root.mcnk_chunks.len());
///         println!("Textures: {:?}", root.textures);
///     }
///     ParsedAdt::Tex0(tex) => {
///         println!("Texture file with {} entries", tex.textures.len());
///     }
///     _ => {}
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Performance
///
/// - Discovery: <10ms for typical files
/// - Full parse: ~50ms average per file
/// - Memory: ≤2× raw file size peak
pub fn parse_adt<R: Read + Seek>(reader: &mut R) -> Result<ParsedAdt> {
    let (adt, _metadata) = parse_adt_with_metadata(reader)?;
    Ok(adt)
}

/// Parse ADT file with diagnostic metadata for debugging and format research.
///
/// Similar to [`parse_adt`] but also returns metadata containing:
/// - Chunk discovery results
/// - Detected version and reasoning
/// - File type identification
/// - Parse timing statistics
/// - Warning messages
///
/// # Arguments
///
/// * `reader` - Seekable input stream
///
/// # Returns
///
/// - `Ok((ParsedAdt, AdtMetadata))` - Parsed ADT with metadata
/// - `Err(AdtError)` - Parsing failed
///
/// # Example
///
/// ```no_run
/// use wow_adt::api::parse_adt_with_metadata;
/// use std::fs::File;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut file = File::open("terrain.adt")?;
/// let (adt, metadata) = parse_adt_with_metadata(&mut file)?;
///
/// println!("Version: {:?}", metadata.version);
/// println!("File type: {:?}", metadata.file_type);
/// println!("Chunks discovered: {}", metadata.chunk_count);
/// println!("Discovery time: {:?}", metadata.discovery_duration);
/// println!("Parse time: {:?}", metadata.parse_duration);
///
/// for warning in metadata.warnings {
///     eprintln!("Warning: {}", warning);
/// }
/// # Ok(())
/// # }
/// ```
pub fn parse_adt_with_metadata<R: Read + Seek>(reader: &mut R) -> Result<(ParsedAdt, AdtMetadata)> {
    // Discovery phase
    let discovery_start = Instant::now();
    let discovery = discover_chunks(reader)?;
    let discovery_duration = discovery_start.elapsed();

    // Detect version and file type
    let version = AdtVersion::from_discovery(&discovery);
    let file_type = AdtFileType::from_discovery(&discovery);

    log::debug!(
        "Discovered {} chunks, version: {:?}, file type: {:?}",
        discovery.total_chunks,
        version,
        file_type
    );

    // Parse phase - route to appropriate parser
    let parse_start = Instant::now();
    let (adt, warnings) = match file_type {
        AdtFileType::Root => {
            let (root, warnings) = crate::root_parser::parse_root_adt(reader, &discovery, version)?;
            (ParsedAdt::Root(Box::new(root)), warnings)
        }
        AdtFileType::Tex0 | AdtFileType::Tex1 => {
            let (tex, warnings) = crate::split_parser::parse_tex_adt(reader, &discovery, version)?;
            let adt = if file_type == AdtFileType::Tex0 {
                ParsedAdt::Tex0(tex)
            } else {
                ParsedAdt::Tex1(tex)
            };
            (adt, warnings)
        }
        AdtFileType::Obj0 | AdtFileType::Obj1 => {
            let (obj, warnings) = crate::split_parser::parse_obj_adt(reader, &discovery, version)?;
            let adt = if file_type == AdtFileType::Obj0 {
                ParsedAdt::Obj0(obj)
            } else {
                ParsedAdt::Obj1(obj)
            };
            (adt, warnings)
        }
        AdtFileType::Lod => {
            let (lod, warnings) = crate::split_parser::parse_lod_adt(reader, &discovery, version)?;
            (ParsedAdt::Lod(lod), warnings)
        }
    };
    let parse_duration = parse_start.elapsed();

    let metadata = AdtMetadata {
        version,
        file_type,
        chunk_count: discovery.total_chunks,
        discovery_duration,
        parse_duration,
        warnings,
        discovery,
    };

    Ok((adt, metadata))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsed_adt_is_root() {
        let root = RootAdt {
            version: AdtVersion::WotLK,
            mhdr: MhdrChunk::default(),
            mcin: McinChunk {
                entries: McinChunk::default().entries,
            },
            textures: vec![],
            models: vec![],
            model_indices: vec![],
            wmos: vec![],
            wmo_indices: vec![],
            doodad_placements: vec![],
            wmo_placements: vec![],
            mcnk_chunks: vec![],
            flight_bounds: None,
            water_data: None,
            texture_flags: None,
            texture_amplifier: None,
            texture_params: None,
            blend_mesh_headers: None,
            blend_mesh_bounds: None,
            blend_mesh_vertices: None,
            blend_mesh_indices: None,
        };

        let parsed = ParsedAdt::Root(Box::new(root));
        assert!(parsed.is_root());
        assert!(!parsed.is_split());
        assert_eq!(parsed.file_type(), AdtFileType::Root);
    }

    #[test]
    fn test_parsed_adt_is_split() {
        let tex = Tex0Adt {
            version: AdtVersion::Cataclysm,
            textures: vec![],
            texture_params: None,
            mcnk_textures: vec![],
        };

        let parsed = ParsedAdt::Tex0(tex);
        assert!(!parsed.is_root());
        assert!(parsed.is_split());
        assert_eq!(parsed.file_type(), AdtFileType::Tex0);
    }

    #[test]
    fn test_root_adt_helpers() {
        let root = RootAdt {
            version: AdtVersion::WotLK,
            mhdr: MhdrChunk::default(),
            mcin: McinChunk {
                entries: McinChunk::default().entries,
            },
            textures: vec!["texture1.blp".into(), "texture2.blp".into()],
            models: vec!["model1.m2".into()],
            model_indices: vec![0],
            wmos: vec![],
            wmo_indices: vec![],
            doodad_placements: vec![],
            wmo_placements: vec![],
            mcnk_chunks: vec![],
            flight_bounds: None,
            water_data: None,
            texture_flags: None,
            texture_amplifier: None,
            texture_params: None,
            blend_mesh_headers: None,
            blend_mesh_bounds: None,
            blend_mesh_vertices: None,
            blend_mesh_indices: None,
        };

        assert_eq!(root.texture_count(), 2);
        assert_eq!(root.model_count(), 1);
        assert_eq!(root.wmo_count(), 0);
        assert_eq!(root.terrain_chunk_count(), 0);
        assert!(!root.has_water());
        assert!(!root.has_flight_bounds());
    }
}
