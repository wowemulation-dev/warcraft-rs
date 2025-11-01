//! MH2O - Multi-layer water system (WotLK+).
//!
//! MH2O replaced the legacy MCLQ chunk starting in Wrath of the Lich King (3.x).
//! It provides a more flexible system supporting:
//!
//! - Multiple liquid layers per chunk
//! - Partial tile coverage (8×8 grid)
//! - Four vertex data formats (LVF) for different use cases
//! - Fishable and deep water attributes
//!
//! ## Structure Hierarchy
//!
//! ```text
//! MH2O Chunk
//! ├─ 256 Headers (16×16 grid)
//! │  ├─ offset_instances → Instance array
//! │  ├─ layer_count (usually 0-1, occasionally 2+)
//! │  └─ offset_attributes → Mh2oAttributes (optional)
//! │
//! ├─ Instances (per layer)
//! │  ├─ liquid_type (LiquidTypeRec FK)
//! │  ├─ Position/dimensions in 8×8 tile grid
//! │  ├─ offset_exists_bitmap → Tile presence bitmap
//! │  └─ offset_vertex_data → Vertex data (format depends on LVF)
//! │
//! └─ Attributes (optional, 8×8 bitmaps)
//!    ├─ fishable - Fishing allowed
//!    └─ deep - Fatigue/deep water
//! ```
//!
//! ## Offset System
//!
//! **CRITICAL**: All offsets are relative to the start of MH2O chunk **data**
//! (after the 8-byte chunk header), NOT to the chunk header itself.
//! This differs from MCNK which uses offsets relative to chunk start.
//!
//! ## Liquid Vertex Formats (LVF)
//!
//! The instance's `liquid_object_or_lvf` field determines the vertex data format:
//!
//! - **LVF 0**: Height + Depth (5 bytes/vertex) - Basic water with transparency
//! - **LVF 1**: Height + UV (8 bytes/vertex) - Textured water
//! - **LVF 2**: Depth Only (1 byte/vertex) - Rare, transparency only
//! - **LVF 3**: Height + UV + Depth (9 bytes/vertex) - Full-featured water
//!
//! For WotLK (values < 42), LVF is the lower 2 bits of `liquid_object_or_lvf`.
//! For Cataclysm+ (values ≥ 42), format must be calculated from vertex data size.
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use wow_adt::chunks::mh2o::{Mh2oChunk, Mh2oHeader, Mh2oInstance};
//! use binrw::BinRead;
//! use std::io::Cursor;
//!
//! # fn example() -> binrw::BinResult<()> {
//! // Access water data from parsed ADT
//! # let chunk_data: Mh2oChunk = todo!();
//! let chunk = &chunk_data;
//!
//! // Iterate through all 256 map chunks
//! for (idx, entry) in chunk.entries.iter().enumerate() {
//!     if entry.header.has_liquid() {
//!         println!("Chunk {} has {} liquid layer(s)", idx, entry.header.layer_count);
//!
//!         for instance in &entry.instances {
//!             println!("  Liquid dimensions: {}×{}", instance.width, instance.height);
//!             println!("  Vertex count: {}", instance.vertex_count());
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Reference: <https://wowdev.wiki/ADT/v18#MH2O_chunk>

pub mod header;
pub mod instance;
pub mod vertex;

// Re-export main types
pub use header::{Mh2oAttributes, Mh2oHeader};
pub use instance::{LiquidVertexFormat, Mh2oInstance};
pub use vertex::{
    DepthOnlyVertex, HeightDepthVertex, HeightUvDepthVertex, HeightUvVertex, UvMapEntry,
    VertexDataArray,
};

/// Complete MH2O chunk with 256 entries (16×16 grid, WotLK+).
///
/// Each entry corresponds to one MCNK terrain chunk and contains:
/// - Header with offsets to instances and attributes
/// - Instance array (liquid layers)
/// - Optional attributes (fishable/deep water zones)
///
/// # Example
///
/// ```no_run
/// # use wow_adt::chunks::mh2o::Mh2oChunk;
/// # let water_data: Mh2oChunk = todo!();
/// // Iterate through all map chunks
/// for (idx, entry) in water_data.entries.iter().enumerate() {
///     if entry.header.has_liquid() {
///         let row = idx / 16;
///         let col = idx % 16;
///         println!("Chunk ({}, {}) has {} liquid layer(s)",
///             row, col, entry.header.layer_count);
///     }
/// }
/// ```
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ❌ Not present
/// - **TBC (2.4.3)**: ❌ Not present
/// - **WotLK (3.3.5a)**: ✅ Introduced (replaced MCLQ)
/// - **Cataclysm (4.3.4)**: ✅ Present
/// - **MoP (5.4.8)**: ✅ Present
#[derive(Debug, Clone, Default)]
pub struct Mh2oChunk {
    /// Water data entries for each map chunk (256 entries for 16×16 grid)
    pub entries: Vec<Mh2oEntry>,
}

impl Mh2oChunk {
    /// Number of entries in MH2O chunk (one per MCNK chunk)
    pub const ENTRY_COUNT: usize = 256;

    /// Grid size (16×16 chunks)
    pub const GRID_SIZE: usize = 16;

    /// Create empty chunk with 256 default entries
    pub fn new() -> Self {
        Self {
            entries: vec![Mh2oEntry::default(); Self::ENTRY_COUNT],
        }
    }

    /// Get entry for specific grid position
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-15)
    /// * `y` - Row index (0-15)
    ///
    /// # Returns
    ///
    /// Entry reference or None if out of bounds
    pub fn get_entry(&self, x: usize, y: usize) -> Option<&Mh2oEntry> {
        if x >= Self::GRID_SIZE || y >= Self::GRID_SIZE {
            return None;
        }
        let index = y * Self::GRID_SIZE + x;
        self.entries.get(index)
    }

    /// Count entries with liquid data
    pub fn liquid_chunk_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.header.has_liquid())
            .count()
    }

    /// Check if any chunk has liquid data
    pub fn has_any_liquid(&self) -> bool {
        self.entries.iter().any(|e| e.header.has_liquid())
    }
}

/// Water data for a single map chunk (1/256 of ADT).
///
/// Contains header with offsets, parsed instances, optional vertex data, and attributes.
///
/// # Example
///
/// ```no_run
/// # use wow_adt::chunks::mh2o::Mh2oEntry;
/// # let entry: Mh2oEntry = todo!();
/// if entry.header.has_liquid() {
///     println!("Has {} liquid layer(s)", entry.instances.len());
///
///     // Check for vertex data
///     for (idx, vertex_data) in entry.vertex_data.iter().enumerate() {
///         if let Some(data) = vertex_data {
///             println!("Layer {} has {} vertices", idx, data.len());
///         }
///     }
///
///     if let Some(attrs) = &entry.attributes {
///         println!("Has {} fishable tiles", attrs.fishable_count());
///         println!("Has {} deep water tiles", attrs.deep_count());
///     }
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct Mh2oEntry {
    /// Header with offsets and layer count
    pub header: Mh2oHeader,

    /// Liquid instances (layers) for this chunk
    pub instances: Vec<Mh2oInstance>,

    /// Vertex data arrays for each instance (one per layer).
    ///
    /// This vector should have the same length as `instances`.
    /// Each element is `None` if the corresponding instance has no vertex data
    /// (uses min/max height instead), or `Some(VertexDataArray)` with the actual vertices.
    ///
    /// **Added in**: T098a-1 (MH2O vertex data implementation)
    pub vertex_data: Vec<Option<VertexDataArray>>,

    /// Exists bitmaps for each instance (one per layer).
    ///
    /// This vector should have the same length as `instances`.
    /// Each element is `None` if the instance has no exists bitmap (all tiles render),
    /// or `Some(u64)` with the 8×8 bitmap indicating which tiles should render.
    ///
    /// **Added in**: T098a-1 (MH2O vertex data implementation)
    pub exists_bitmaps: Vec<Option<u64>>,

    /// Optional attributes (fishable/deep water zones)
    pub attributes: Option<Mh2oAttributes>,
}

impl Mh2oEntry {
    /// Check if entry has liquid data
    pub fn has_liquid(&self) -> bool {
        self.header.has_liquid() && !self.instances.is_empty()
    }

    /// Check if entry has attributes
    pub fn has_attributes(&self) -> bool {
        self.attributes.is_some()
    }

    /// Get total layer count
    pub fn layer_count(&self) -> usize {
        self.instances.len()
    }
}
