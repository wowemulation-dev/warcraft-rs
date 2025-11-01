//! Blend Mesh System Chunks (Mists of Pandaria 5.x+)
//!
//! The blend mesh system introduced in Mists of Pandaria enables smooth terrain
//! texture transitions through dedicated geometry that blends between different
//! terrain types. The system consists of four interconnected chunks:
//!
//! - **MBMH**: Blend mesh headers (metadata and index ranges)
//! - **MBBB**: Bounding boxes for visibility culling
//! - **MBNV**: Vertex data (position, normal, UV, colors)
//! - **MBMI**: Triangle indices into vertex array
//!
//! # System Architecture
//!
//! ```text
//! MBMH (Headers)        MBBB (Bounds)         MBNV (Vertices)      MBMI (Indices)
//! ┌─────────────┐       ┌─────────────┐       ┌─────────────┐      ┌────────┐
//! │ Header 0    │  ───  │ BBox 0      │       │ Vertex 0    │ ◄─── │ Index 0│
//! │  mbnv:0+64  │       │  min/max    │       │  pos,norm   │      │ Index 1│
//! │  mbmi:0+96  │       └─────────────┘       │  uv,colors  │      │ Index 2│
//! ├─────────────┤       ┌─────────────┐       ├─────────────┤      ├────────┤
//! │ Header 1    │  ───  │ BBox 1      │       │ Vertex 64   │ ◄─── │ Index N│
//! │  mbnv:64+32 │       │  min/max    │       │  ...        │      │  ...   │
//! │  mbmi:96+48 │       └─────────────┘       └─────────────┘      └────────┘
//! └─────────────┘
//! ```
//!
//! Each blend mesh header (MBMH) references a contiguous range of vertices (MBNV)
//! and indices (MBMI). The MCBB chunk (per-MCNK) further subdivides these meshes
//! into renderable batches.
//!
//! # Version History
//!
//! - **5.0.1 (MoP)**: Initial blend mesh system
//! - **Storage**: Root ADT and LOD split files
//!
//! # References
//!
//! - **wowdev.wiki**: [ADT/v18](https://wowdev.wiki/ADT/v18) - Complete blend mesh specification
//! - **WoWFormatLib**: ADT.Struct.cs - Chunk IDs defined (implementation incomplete)
//! - **wow.export**: ADTLoader.js - MCBB parsing (blend mesh chunks marked TODO)
//! - **noggit-red**: Limited MoP support (blend mesh system not fully implemented)

use binrw::{BinRead, BinWrite, helpers::until_eof};

/// MBMH - Blend Mesh Header (Mists of Pandaria 5.x+)
///
/// Contains metadata for blend mesh instances, including texture references
/// and index ranges into the global MBNV/MBMI arrays. Each header can reference
///multiple textures for the same map object.
///
/// # Format
///
/// ```text
/// struct MbmhEntry {
///     map_object_id: u32,   // Unique identifier
///     texture_id: u32,      // Linked WMO texture reference
///     unknown: u32,         // Always 0
///     mbmi_count: u32,      // Index count in MBMI
///     mbnv_count: u32,      // Vertex count in MBNV
///     mbmi_start: u32,      // Starting index in MBMI
///     mbnv_start: u32,      // Starting vertex in MBNV
/// }
/// ```
///
/// **Size**: 28 bytes per entry
///
/// Reference: <https://wowdev.wiki/ADT/v18#MBMH_chunk>
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite)]
pub struct MbmhChunk {
    /// Array of blend mesh headers.
    ///
    /// Each header describes a blend mesh instance with its vertex/index ranges.
    #[br(parse_with = until_eof)]
    pub entries: Vec<MbmhEntry>,
}

/// Blend mesh header entry describing mesh metadata and index ranges.
///
/// # Format
///
/// ```text
/// struct MbmhEntry {
///     map_object_id: u32,   // Unique ID
///     texture_id: u32,      // WMO texture ref
///     unknown: u32,         // Padding/reserved
///     mbmi_count: u32,      // Index count
///     mbnv_count: u32,      // Vertex count
///     mbmi_start: u32,      // Index start
///     mbnv_start: u32,      // Vertex start
/// }
/// ```
///
/// **Size**: 28 bytes
#[derive(Debug, Clone, Copy, PartialEq, Eq, BinRead, BinWrite)]
pub struct MbmhEntry {
    /// Unique blend mesh identifier.
    ///
    /// Used to match with bounding boxes (MBBB) and batches (MCBB).
    pub map_object_id: u32,

    /// Texture ID for this blend mesh.
    ///
    /// References a WMO texture for rendering. Multiple headers can share
    /// the same map_object_id but use different textures.
    pub texture_id: u32,

    /// Unknown field (always 0 in observed files).
    ///
    /// Possibly reserved for future use or alignment padding.
    pub unknown: u32,

    /// Number of indices in MBMI for this mesh.
    ///
    /// Defines how many triangle indices to read starting at `mbmi_start`.
    /// Triangle count = `mbmi_count / 3`.
    pub mbmi_count: u32,

    /// Number of vertices in MBNV for this mesh.
    ///
    /// Defines how many vertices to read starting at `mbnv_start`.
    pub mbnv_count: u32,

    /// Starting index in MBMI array.
    ///
    /// Absolute offset into the global MBMI chunk. Indices at positions
    /// `[mbmi_start .. mbmi_start + mbmi_count]` belong to this mesh.
    pub mbmi_start: u32,

    /// Starting vertex in MBNV array.
    ///
    /// Absolute offset into the global MBNV chunk. Vertices at positions
    /// `[mbnv_start .. mbnv_start + mbnv_count]` belong to this mesh.
    pub mbnv_start: u32,
}

/// MBBB - Blend Mesh Bounding Boxes (Mists of Pandaria 5.x+)
///
/// Contains axis-aligned bounding boxes for blend meshes, one-to-one with MBMH entries.
/// Used for visibility culling and spatial queries.
///
/// # Format
///
/// ```text
/// struct MbbbEntry {
///     map_object_id: u32,   // Matches MBMH entry
///     bounding_box: CAaBox, // Min/max corners (24 bytes)
/// }
/// ```
///
/// **Size**: 28 bytes per entry
///
/// Reference: <https://wowdev.wiki/ADT/v18#MBBB_chunk>
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite)]
pub struct MbbbChunk {
    /// Array of bounding boxes, one per MBMH entry.
    ///
    /// The order matches MBMH entries (entry N bounds mesh N).
    #[br(parse_with = until_eof)]
    pub entries: Vec<MbbbEntry>,
}

/// Bounding box entry for blend mesh visibility culling.
///
/// # Format
///
/// ```text
/// struct MbbbEntry {
///     map_object_id: u32,   // Identifier
///     bounding_box: {
///         min: [f32; 3],    // Minimum corner
///         max: [f32; 3],    // Maximum corner
///     }
/// }
/// ```
///
/// **Size**: 28 bytes
#[derive(Debug, Clone, Copy, PartialEq, BinRead, BinWrite)]
pub struct MbbbEntry {
    /// Map object ID matching the corresponding MBMH entry.
    pub map_object_id: u32,

    /// Minimum bounding box corner (X, Y, Z).
    pub min: [f32; 3],

    /// Maximum bounding box corner (X, Y, Z).
    pub max: [f32; 3],
}

/// MBNV - Blend Mesh Vertices (Mists of Pandaria 5.x+)
///
/// Contains vertex data for blend meshes including position, normal, texture coordinates,
/// and up to 3 color channels. Different vertex formats use color channels differently:
///
/// - **PN**: Position + Normal (colors unused)
/// - **PNC**: Position + Normal + Color[0]
/// - **PNC2**: Position + Normal + Color[0,1]
/// - **PNC2T**: Position + Normal + Color[0,2] + Tangent
///
/// # Format
///
/// ```text
/// struct MbnvEntry {
///     position: [f32; 3],   // Vertex position
///     normal: [f32; 3],     // Surface normal
///     uv: [f32; 2],         // Texture coordinates
///     color: [[u8; 4]; 3],  // Color data (RGBA × 3)
/// }
/// ```
///
/// **Size**: 44 bytes per vertex
///
/// Reference: <https://wowdev.wiki/ADT/v18#MBNV_chunk>
#[derive(Debug, Clone, PartialEq, BinRead, BinWrite, Default)]
pub struct MbnvChunk {
    /// Array of blend mesh vertices.
    ///
    /// Vertices are referenced by MBMI indices. Multiple MBMH entries can share
    /// vertices if their ranges overlap (though this is rare).
    #[br(parse_with = until_eof)]
    pub vertices: Vec<MbnvVertex>,
}

/// Blend mesh vertex with position, normal, UV, and color data.
///
/// # Format
///
/// ```text
/// struct MbnvVertex {
///     position: [f32; 3],   // XYZ coordinates
///     normal: [f32; 3],     // Normal vector
///     uv: [f32; 2],         // Texture coords
///     color: [[u8; 4]; 3],  // RGBA × 3 channels
/// }
/// ```
///
/// **Size**: 44 bytes
#[derive(Debug, Clone, Copy, PartialEq, BinRead, BinWrite)]
pub struct MbnvVertex {
    /// Vertex position in world coordinates (X, Y, Z).
    pub position: [f32; 3],

    /// Surface normal vector (X, Y, Z), typically normalized.
    pub normal: [f32; 3],

    /// Texture coordinates (U, V) for blend texture sampling.
    pub uv: [f32; 2],

    /// Color channels (RGBA format, 4 bytes each × 3 channels).
    ///
    /// Usage depends on vertex format:
    /// - **PN**: All unused
    /// - **PNC**: color[0] used
    /// - **PNC2**: color[0,1] used
    /// - **PNC2T**: color[0,2] used
    pub color: [[u8; 4]; 3],
}

/// MBMI - Blend Mesh Indices (Mists of Pandaria 5.x+)
///
/// Contains triangle indices referencing vertices in the MBNV array.
/// Indices are u16 values, allowing up to 65,536 vertices per mesh.
///
/// # Format
///
/// ```text
/// struct MbmiEntry {
///     index: u16,  // Index into MBNV array
/// }
/// ```
///
/// **Size**: 2 bytes per index (3 indices = 1 triangle)
///
/// Reference: <https://wowdev.wiki/ADT/v18#MBMI_chunk>
#[derive(Debug, Clone, PartialEq, Eq, BinRead, BinWrite, Default)]
pub struct MbmiChunk {
    /// Array of vertex indices forming triangles.
    ///
    /// Indices are grouped in sets of 3 to form triangles. Each index references
    /// a vertex in the MBNV chunk. Triangle N uses indices `[3*N, 3*N+1, 3*N+2]`.
    #[br(parse_with = until_eof)]
    pub indices: Vec<u16>,
}

// ============================================================================
// Helper Methods
// ============================================================================

impl MbmhChunk {
    /// Get the number of blend mesh headers.
    #[must_use]
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Check if chunk is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get total vertex count across all meshes.
    #[must_use]
    pub fn total_vertices(&self) -> u32 {
        self.entries.iter().map(|e| e.mbnv_count).sum()
    }

    /// Get total index count across all meshes.
    #[must_use]
    pub fn total_indices(&self) -> u32 {
        self.entries.iter().map(|e| e.mbmi_count).sum()
    }
}

impl MbbbChunk {
    /// Get the number of bounding boxes.
    #[must_use]
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Check if chunk is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl MbbbEntry {
    /// Get bounding box center point.
    #[must_use]
    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) / 2.0,
            (self.min[1] + self.max[1]) / 2.0,
            (self.min[2] + self.max[2]) / 2.0,
        ]
    }

    /// Get bounding box size (extents).
    #[must_use]
    pub fn size(&self) -> [f32; 3] {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }
}

impl MbnvChunk {
    /// Get the number of vertices.
    #[must_use]
    pub fn count(&self) -> usize {
        self.vertices.len()
    }

    /// Check if chunk is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

impl MbmiChunk {
    /// Get the number of indices.
    #[must_use]
    pub fn count(&self) -> usize {
        self.indices.len()
    }

    /// Get the number of triangles (indices / 3).
    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Check if chunk is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

// ============================================================================
// Default Implementations
// ============================================================================

impl Default for MbmhChunk {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl Default for MbmhEntry {
    fn default() -> Self {
        Self {
            map_object_id: 0,
            texture_id: 0,
            unknown: 0,
            mbmi_count: 0,
            mbnv_count: 0,
            mbmi_start: 0,
            mbnv_start: 0,
        }
    }
}

impl Default for MbbbChunk {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl Default for MbbbEntry {
    fn default() -> Self {
        Self {
            map_object_id: 0,
            min: [0.0, 0.0, 0.0],
            max: [0.0, 0.0, 0.0],
        }
    }
}

impl Default for MbnvVertex {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0], // Up vector by default
            uv: [0.0, 0.0],
            color: [[0, 0, 0, 0]; 3],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::BinReaderExt;
    use std::io::Cursor;

    // ========================================================================
    // MBMH Tests
    // ========================================================================

    #[test]
    fn test_mbmh_entry_size() {
        assert_eq!(std::mem::size_of::<MbmhEntry>(), 28);
    }

    #[test]
    fn test_mbmh_parse_single() {
        let data: Vec<u8> = vec![
            0x01, 0x00, 0x00, 0x00, // map_object_id: 1
            0x0A, 0x00, 0x00, 0x00, // texture_id: 10
            0x00, 0x00, 0x00, 0x00, // unknown: 0
            0x60, 0x00, 0x00, 0x00, // mbmi_count: 96
            0x40, 0x00, 0x00, 0x00, // mbnv_count: 64
            0x00, 0x00, 0x00, 0x00, // mbmi_start: 0
            0x00, 0x00, 0x00, 0x00, // mbnv_start: 0
        ];

        let mut cursor = Cursor::new(data);
        let mbmh: MbmhChunk = cursor.read_le().unwrap();

        assert_eq!(mbmh.count(), 1);
        assert_eq!(mbmh.entries[0].map_object_id, 1);
        assert_eq!(mbmh.entries[0].texture_id, 10);
        assert_eq!(mbmh.entries[0].mbmi_count, 96);
        assert_eq!(mbmh.entries[0].mbnv_count, 64);
    }

    #[test]
    fn test_mbmh_round_trip() {
        let original = MbmhChunk {
            entries: vec![MbmhEntry {
                map_object_id: 1,
                texture_id: 10,
                unknown: 0,
                mbmi_count: 96,
                mbnv_count: 64,
                mbmi_start: 0,
                mbnv_start: 0,
            }],
        };

        let mut buffer = Cursor::new(Vec::new());
        original.write_le(&mut buffer).unwrap();

        buffer.set_position(0);
        let parsed: MbmhChunk = buffer.read_le().unwrap();

        assert_eq!(parsed, original);
    }

    #[test]
    fn test_mbmh_totals() {
        let mbmh = MbmhChunk {
            entries: vec![
                MbmhEntry {
                    map_object_id: 1,
                    texture_id: 10,
                    unknown: 0,
                    mbmi_count: 96,
                    mbnv_count: 64,
                    mbmi_start: 0,
                    mbnv_start: 0,
                },
                MbmhEntry {
                    map_object_id: 2,
                    texture_id: 11,
                    unknown: 0,
                    mbmi_count: 48,
                    mbnv_count: 32,
                    mbmi_start: 96,
                    mbnv_start: 64,
                },
            ],
        };

        assert_eq!(mbmh.total_vertices(), 96); // 64 + 32
        assert_eq!(mbmh.total_indices(), 144); // 96 + 48
    }

    // ========================================================================
    // MBBB Tests
    // ========================================================================

    #[test]
    fn test_mbbb_entry_size() {
        assert_eq!(std::mem::size_of::<MbbbEntry>(), 28);
    }

    #[test]
    fn test_mbbb_parse_single() {
        let data: Vec<u8> = vec![
            0x01, 0x00, 0x00, 0x00, // map_object_id: 1
            0x00, 0x00, 0x00, 0x00, // min.x: 0.0
            0x00, 0x00, 0x00, 0x00, // min.y: 0.0
            0x00, 0x00, 0x00, 0x00, // min.z: 0.0
            0x00, 0x00, 0x80, 0x3F, // max.x: 1.0
            0x00, 0x00, 0x80, 0x3F, // max.y: 1.0
            0x00, 0x00, 0x80, 0x3F, // max.z: 1.0
        ];

        let mut cursor = Cursor::new(data);
        let mbbb: MbbbChunk = cursor.read_le().unwrap();

        assert_eq!(mbbb.count(), 1);
        assert_eq!(mbbb.entries[0].map_object_id, 1);
        assert_eq!(mbbb.entries[0].min, [0.0, 0.0, 0.0]);
        assert_eq!(mbbb.entries[0].max, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_mbbb_center_size() {
        let entry = MbbbEntry {
            map_object_id: 1,
            min: [0.0, 0.0, 0.0],
            max: [10.0, 20.0, 30.0],
        };

        assert_eq!(entry.center(), [5.0, 10.0, 15.0]);
        assert_eq!(entry.size(), [10.0, 20.0, 30.0]);
    }

    // ========================================================================
    // MBNV Tests
    // ========================================================================

    #[test]
    fn test_mbnv_vertex_size() {
        assert_eq!(std::mem::size_of::<MbnvVertex>(), 44);
    }

    #[test]
    fn test_mbnv_parse_single() {
        let mut data = Vec::new();

        // Position (12 bytes)
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&2.0f32.to_le_bytes());
        data.extend_from_slice(&3.0f32.to_le_bytes());

        // Normal (12 bytes)
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());

        // UV (8 bytes)
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.5f32.to_le_bytes());

        // Color (12 bytes: 3 × RGBA)
        data.extend_from_slice(&[255, 0, 0, 255]); // Red
        data.extend_from_slice(&[0, 255, 0, 255]); // Green
        data.extend_from_slice(&[0, 0, 255, 255]); // Blue

        let mut cursor = Cursor::new(data);
        let mbnv: MbnvChunk = cursor.read_le().unwrap();

        assert_eq!(mbnv.count(), 1);
        assert_eq!(mbnv.vertices[0].position, [1.0, 2.0, 3.0]);
        assert_eq!(mbnv.vertices[0].normal, [0.0, 0.0, 1.0]);
        assert_eq!(mbnv.vertices[0].uv, [0.5, 0.5]);
        assert_eq!(mbnv.vertices[0].color[0], [255, 0, 0, 255]);
    }

    // ========================================================================
    // MBMI Tests
    // ========================================================================

    #[test]
    fn test_mbmi_parse() {
        let data: Vec<u8> = vec![
            0x00, 0x00, // Index 0
            0x01, 0x00, // Index 1
            0x02, 0x00, // Index 2
            0x03, 0x00, // Index 3
            0x04, 0x00, // Index 4
            0x05, 0x00, // Index 5
        ];

        let mut cursor = Cursor::new(data);
        let mbmi: MbmiChunk = cursor.read_le().unwrap();

        assert_eq!(mbmi.count(), 6);
        assert_eq!(mbmi.triangle_count(), 2);
        assert_eq!(mbmi.indices, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_mbmi_round_trip() {
        let original = MbmiChunk {
            indices: vec![0, 1, 2, 3, 4, 5],
        };

        let mut buffer = Cursor::new(Vec::new());
        original.write_le(&mut buffer).unwrap();

        buffer.set_position(0);
        let parsed: MbmiChunk = buffer.read_le().unwrap();

        assert_eq!(parsed, original);
    }
}
