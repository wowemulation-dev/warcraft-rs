//! MCBB - Blend Batches (Mists of Pandaria 5.x+)
//!
//! The MCBB chunk contains blend batch descriptors for the blend mesh system introduced
//! in Mists of Pandaria. Each blend batch references indices and vertices in the global
//! blend mesh arrays (MBMI/MBNV) for rendering smooth terrain transitions.
//!
//! # Format Specification
//!
//! Each blend batch entry is 20 bytes:
//! ```text
//! struct BlendBatch {
//!     mbmh_index: u32,      // Index into MBMH (blend mesh header) array
//!     index_count: u32,     // Number of indices in MBMI for this batch
//!     index_first: u32,     // First index offset (relative to mbmh.mbnv_base)
//!     vertex_count: u32,    // Number of vertices in MBNV for this batch
//!     vertex_first: u32,    // First vertex offset (relative to mbmh.mbnv_base)
//! }
//! ```
//!
//! **Size**: Variable (20 bytes per batch, max 256 batches per MCNK)
//!
//! # Purpose
//!
//! Blend batches enable efficient rendering of terrain texture blending by grouping
//! vertices and indices into renderable batches. Each batch references a specific
//! region of the global blend mesh data, allowing the renderer to:
//! - Minimize draw calls by batching blend geometry
//! - Support dynamic LOD by varying batch granularity
//! - Enable texture streaming with batch-level visibility culling
//!
//! # Version History
//!
//! - **5.0.1 (MoP)**: Introduced with blend mesh system
//! - **Max batches**: 256 per MCNK chunk (based on maximum chunk size)
//!
//! # References
//!
//! - **wowdev.wiki**: [ADT/v18](https://wowdev.wiki/ADT/v18) - Blend batch structure
//! - **WoWFormatLib**: ADT.Struct.cs - MCBB struct (5 u32 fields, 20 bytes)
//! - **wow.export**: ADTLoader.js - Blend batch parsing (MoP+ support)
//! - **noggit-red**: Not documented (MoP support incomplete)
//!
//! # Example
//!
//! ```rust
//! use wow_adt::chunks::mcnk::{McbbChunk, BlendBatch};
//!
//! // Create blend batch array with 2 batches
//! let batches = McbbChunk {
//!     batches: vec![
//!         BlendBatch {
//!             mbmh_index: 0,
//!             index_count: 96,
//!             index_first: 0,
//!             vertex_count: 64,
//!             vertex_first: 0,
//!         },
//!         BlendBatch {
//!             mbmh_index: 0,
//!             index_count: 48,
//!             index_first: 96,
//!             vertex_count: 32,
//!             vertex_first: 64,
//!         },
//!     ],
//! };
//!
//! assert_eq!(batches.count(), 2);
//! assert_eq!(batches.total_indices(), 144); // 96 + 48
//! assert_eq!(batches.total_vertices(), 96); // 64 + 32
//! ```

use binrw::{BinRead, BinWrite, helpers::until_eof};

/// MCBB - Blend Batches (Mists of Pandaria 5.x+)
///
/// Contains an array of blend batch descriptors for the blend mesh system.
/// Each batch groups indices and vertices for efficient terrain blending rendering.
///
/// # Structure
///
/// - Variable-length array of [`BlendBatch`] entries
/// - Each entry is 20 bytes (5 × u32 fields)
/// - Maximum 256 batches per MCNK chunk
/// - Chunk size determines batch count (size / 20)
///
/// # Usage
///
/// ```rust
/// use wow_adt::chunks::mcnk::{McbbChunk, BlendBatch};
///
/// let batches = McbbChunk {
///     batches: vec![
///         BlendBatch {
///             mbmh_index: 0,
///             index_count: 96,
///             index_first: 0,
///             vertex_count: 64,
///             vertex_first: 0,
///         },
///     ],
///  };
///
/// assert_eq!(batches.count(), 1);
/// ```
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite)]
pub struct McbbChunk {
    /// Array of blend batch descriptors.
    ///
    /// Each batch references a region of the global blend mesh data (MBMI/MBNV)
    /// for rendering. Batches are ordered by rendering priority.
    #[br(parse_with = until_eof)]
    pub batches: Vec<BlendBatch>,
}

/// Blend batch descriptor referencing blend mesh geometry.
///
/// Each batch groups indices and vertices from the global blend mesh arrays
/// (MBMI/MBNV) for efficient rendering. The offsets are relative to the
/// base specified in the MBMH (blend mesh header) entry.
///
/// # Format
///
/// ```text
/// struct BlendBatch {
///     mbmh_index: u32,      // Index into MBMH array
///     index_count: u32,     // Number of indices
///     index_first: u32,     // First index offset
///     vertex_count: u32,    // Number of vertices
///     vertex_first: u32,    // First vertex offset
/// }
/// ```
///
/// **Size**: 20 bytes
#[derive(Debug, Clone, Copy, PartialEq, Eq, BinRead, BinWrite)]
pub struct BlendBatch {
    /// Index into MBMH (blend mesh header) array.
    ///
    /// References which blend mesh header this batch belongs to. Multiple batches
    /// can share the same header for different LOD levels or rendering passes.
    pub mbmh_index: u32,

    /// Number of indices in MBMI for this batch.
    ///
    /// Specifies how many triangle indices to read from the MBMI array starting
    /// at `index_first + mbmh.mbnv_base`. Indices form triangles (count / 3 triangles).
    pub index_count: u32,

    /// First index offset in MBMI (relative to mbmh.mbnv_base).
    ///
    /// Starting position in the MBMI (blend mesh indices) array. The actual offset
    /// is `mbmh.mbnv_base + index_first` where mbmh is the referenced header.
    pub index_first: u32,

    /// Number of vertices in MBNV for this batch.
    ///
    /// Specifies how many vertices to read from the MBNV array starting at
    /// `vertex_first + mbmh.mbnv_base`. Vertices define the blend mesh geometry.
    pub vertex_count: u32,

    /// First vertex offset in MBNV (relative to mbmh.mbnv_base).
    ///
    /// Starting position in the MBNV (blend mesh vertices) array. The actual offset
    /// is `mbmh.mbnv_base + vertex_first` where mbmh is the referenced header.
    pub vertex_first: u32,
}

impl McbbChunk {
    /// Get the number of blend batches.
    ///
    /// # Returns
    ///
    /// Total count of blend batch descriptors.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use wow_adt::chunks::mcnk::{McbbChunk, BlendBatch};
    /// let batches = McbbChunk { batches: vec![BlendBatch::default()] };
    /// assert_eq!(batches.count(), 1);
    /// ```
    #[must_use]
    pub fn count(&self) -> usize {
        self.batches.len()
    }

    /// Check if chunk is empty (no batches).
    ///
    /// # Returns
    ///
    /// `true` if no blend batches are present.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use wow_adt::chunks::mcnk::McbbChunk;
    /// let empty = McbbChunk { batches: vec![] };
    /// assert!(empty.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.batches.is_empty()
    }

    /// Get total index count across all batches.
    ///
    /// # Returns
    ///
    /// Sum of index_count for all blend batches.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use wow_adt::chunks::mcnk::{McbbChunk, BlendBatch};
    /// let batches = McbbChunk {
    ///     batches: vec![
    ///         BlendBatch { mbmh_index: 0, index_count: 96, index_first: 0, vertex_count: 64, vertex_first: 0 },
    ///         BlendBatch { mbmh_index: 0, index_count: 48, index_first: 96, vertex_count: 32, vertex_first: 64 },
    ///     ],
    /// };
    /// assert_eq!(batches.total_indices(), 144); // 96 + 48
    /// ```
    #[must_use]
    pub fn total_indices(&self) -> u32 {
        self.batches.iter().map(|b| b.index_count).sum()
    }

    /// Get total vertex count across all batches.
    ///
    /// # Returns
    ///
    /// Sum of vertex_count for all blend batches.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use wow_adt::chunks::mcnk::{McbbChunk, BlendBatch};
    /// let batches = McbbChunk {
    ///     batches: vec![
    ///         BlendBatch { mbmh_index: 0, index_count: 96, index_first: 0, vertex_count: 64, vertex_first: 0 },
    ///         BlendBatch { mbmh_index: 0, index_count: 48, index_first: 96, vertex_count: 32, vertex_first: 64 },
    ///     ],
    /// };
    /// assert_eq!(batches.total_vertices(), 96); // 64 + 32
    /// ```
    #[must_use]
    pub fn total_vertices(&self) -> u32 {
        self.batches.iter().map(|b| b.vertex_count).sum()
    }

    /// Get triangle count across all batches.
    ///
    /// # Returns
    ///
    /// Total triangles (total_indices / 3).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use wow_adt::chunks::mcnk::{McbbChunk, BlendBatch};
    /// let batches = McbbChunk {
    ///     batches: vec![
    ///         BlendBatch { mbmh_index: 0, index_count: 96, index_first: 0, vertex_count: 64, vertex_first: 0 },
    ///     ],
    /// };
    /// assert_eq!(batches.triangle_count(), 32); // 96 / 3
    /// ```
    #[must_use]
    pub fn triangle_count(&self) -> u32 {
        self.total_indices() / 3
    }
}

impl Default for McbbChunk {
    /// Create empty blend batch array.
    ///
    /// # Returns
    ///
    /// `McbbChunk` with no batches.
    fn default() -> Self {
        Self {
            batches: Vec::new(),
        }
    }
}

impl Default for BlendBatch {
    /// Create default blend batch with zero values.
    ///
    /// # Returns
    ///
    /// `BlendBatch` with all fields set to 0.
    fn default() -> Self {
        Self {
            mbmh_index: 0,
            index_count: 0,
            index_first: 0,
            vertex_count: 0,
            vertex_first: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::BinReaderExt;
    use std::io::Cursor;

    #[test]
    fn test_blend_batch_size() {
        // BlendBatch must be exactly 20 bytes (5 × u32)
        assert_eq!(std::mem::size_of::<BlendBatch>(), 20);
    }

    #[test]
    fn test_mcbb_parse_single_batch() {
        let data: Vec<u8> = vec![
            0x00, 0x00, 0x00, 0x00, // mbmh_index: 0
            0x60, 0x00, 0x00, 0x00, // index_count: 96
            0x00, 0x00, 0x00, 0x00, // index_first: 0
            0x40, 0x00, 0x00, 0x00, // vertex_count: 64
            0x00, 0x00, 0x00, 0x00, // vertex_first: 0
        ];

        let mut cursor = Cursor::new(data);
        let mcbb: McbbChunk = cursor.read_le().expect("Failed to parse MCBB");

        assert_eq!(mcbb.count(), 1);
        assert_eq!(mcbb.batches[0].mbmh_index, 0);
        assert_eq!(mcbb.batches[0].index_count, 96);
        assert_eq!(mcbb.batches[0].index_first, 0);
        assert_eq!(mcbb.batches[0].vertex_count, 64);
        assert_eq!(mcbb.batches[0].vertex_first, 0);
    }

    #[test]
    fn test_mcbb_parse_multiple_batches() {
        let data: Vec<u8> = vec![
            // Batch 1
            0x00, 0x00, 0x00, 0x00, // mbmh_index: 0
            0x60, 0x00, 0x00, 0x00, // index_count: 96
            0x00, 0x00, 0x00, 0x00, // index_first: 0
            0x40, 0x00, 0x00, 0x00, // vertex_count: 64
            0x00, 0x00, 0x00, 0x00, // vertex_first: 0
            // Batch 2
            0x00, 0x00, 0x00, 0x00, // mbmh_index: 0
            0x30, 0x00, 0x00, 0x00, // index_count: 48
            0x60, 0x00, 0x00, 0x00, // index_first: 96
            0x20, 0x00, 0x00, 0x00, // vertex_count: 32
            0x40, 0x00, 0x00, 0x00, // vertex_first: 64
        ];

        let mut cursor = Cursor::new(data);
        let mcbb: McbbChunk = cursor.read_le().expect("Failed to parse MCBB");

        assert_eq!(mcbb.count(), 2);
        assert_eq!(mcbb.batches[1].index_first, 96);
        assert_eq!(mcbb.batches[1].vertex_first, 64);
    }

    #[test]
    fn test_mcbb_round_trip() {
        let original = McbbChunk {
            batches: vec![
                BlendBatch {
                    mbmh_index: 0,
                    index_count: 96,
                    index_first: 0,
                    vertex_count: 64,
                    vertex_first: 0,
                },
                BlendBatch {
                    mbmh_index: 1,
                    index_count: 48,
                    index_first: 96,
                    vertex_count: 32,
                    vertex_first: 64,
                },
            ],
        };

        // Serialize
        let mut buffer = Cursor::new(Vec::new());
        original
            .write_le(&mut buffer)
            .expect("Failed to write MCBB");

        // Deserialize
        buffer.set_position(0);
        let parsed: McbbChunk = buffer.read_le().expect("Failed to read MCBB");

        assert_eq!(parsed, original);
    }

    #[test]
    fn test_mcbb_empty() {
        let empty = McbbChunk::default();
        assert!(empty.is_empty());
        assert_eq!(empty.count(), 0);
        assert_eq!(empty.total_indices(), 0);
        assert_eq!(empty.total_vertices(), 0);
    }

    #[test]
    fn test_mcbb_total_indices() {
        let mcbb = McbbChunk {
            batches: vec![
                BlendBatch {
                    mbmh_index: 0,
                    index_count: 96,
                    index_first: 0,
                    vertex_count: 64,
                    vertex_first: 0,
                },
                BlendBatch {
                    mbmh_index: 0,
                    index_count: 48,
                    index_first: 96,
                    vertex_count: 32,
                    vertex_first: 64,
                },
            ],
        };

        assert_eq!(mcbb.total_indices(), 144); // 96 + 48
        assert_eq!(mcbb.total_vertices(), 96); // 64 + 32
        assert_eq!(mcbb.triangle_count(), 48); // 144 / 3
    }

    #[test]
    fn test_blend_batch_default() {
        let batch = BlendBatch::default();
        assert_eq!(batch.mbmh_index, 0);
        assert_eq!(batch.index_count, 0);
        assert_eq!(batch.index_first, 0);
        assert_eq!(batch.vertex_count, 0);
        assert_eq!(batch.vertex_first, 0);
    }

    #[test]
    fn test_mcbb_large_batch_count() {
        // Test with 10 batches (realistic scenario)
        let mut batches = Vec::new();
        for i in 0..10 {
            batches.push(BlendBatch {
                mbmh_index: i,
                index_count: 96,
                index_first: i * 96,
                vertex_count: 64,
                vertex_first: i * 64,
            });
        }

        let mcbb = McbbChunk { batches };
        assert_eq!(mcbb.count(), 10);
        assert_eq!(mcbb.total_indices(), 960); // 10 × 96
        assert_eq!(mcbb.total_vertices(), 640); // 10 × 64
    }
}
