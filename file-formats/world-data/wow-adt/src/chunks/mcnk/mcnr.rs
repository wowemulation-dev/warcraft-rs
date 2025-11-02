use binrw::{BinRead, BinWrite};

/// Single vertex normal (3 bytes compressed).
///
/// Normals are stored as signed bytes in X, Z, Y order (note the swap!).
/// To convert to floating point: `component / 127.0`
///
/// The Y and Z components are swapped compared to typical coordinate ordering.
#[derive(Debug, Clone, Copy, BinRead, BinWrite)]
#[brw(little)]
pub struct VertexNormal {
    /// X component (-127 to 127)
    pub x: i8,

    /// Z component (-127 to 127)
    pub z: i8,

    /// Y component (-127 to 127)
    pub y: i8,
}

impl VertexNormal {
    /// Convert to normalized floating point vector [X, Y, Z].
    ///
    /// Returns proper Y-up coordinate order (X, Y, Z) by swapping stored Z/Y.
    pub fn to_normalized(&self) -> [f32; 3] {
        [
            f32::from(self.x) / 127.0,
            f32::from(self.y) / 127.0, // Note: stored in position 2
            f32::from(self.z) / 127.0, // Note: stored in position 1
        ]
    }

    /// Create from normalized floating point vector [X, Y, Z].
    pub fn from_normalized(normal: [f32; 3]) -> Self {
        Self {
            x: (normal[0] * 127.0) as i8,
            z: (normal[2] * 127.0) as i8, // Z stored in position 1
            y: (normal[1] * 127.0) as i8, // Y stored in position 2
        }
    }
}

/// MCNR chunk - Vertex normals (145 entries + padding, Vanilla+)
///
/// Contains surface normals for lighting calculations. Same vertex layout as MCVT:
/// - 9×9 outer grid (81 normals)
/// - 8×8 inner grid (64 normals)
/// - 13 bytes padding at end
///
/// Total size: 448 bytes (145 * 3 + 13)
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ✅ Introduced
/// - **TBC (2.4.3)**: ✅ Present
/// - **WotLK (3.3.5a)**: ✅ Present
/// - **Cataclysm (4.3.4)**: ✅ Present
/// - **MoP (5.4.8)**: ✅ Present
///
/// Reference: <https://wowdev.wiki/ADT/v18#MCNR_sub-chunk>
#[derive(Debug, Clone, BinRead, BinWrite)]
#[brw(little)]
pub struct McnrChunk {
    /// Normal vectors (145 entries)
    #[br(count = 145)]
    pub normals: Vec<VertexNormal>,

    /// Padding bytes (variable length, typically 13 bytes)
    ///
    /// Vanilla/TBC files may have no padding (435 bytes total).
    /// Later expansions typically have 13 bytes padding (448 bytes total).
    #[br(parse_with = binrw::helpers::until_eof)]
    pub padding: Vec<u8>,
}

impl Default for McnrChunk {
    fn default() -> Self {
        Self {
            normals: vec![VertexNormal { x: 0, y: 127, z: 0 }; 145], // Up normals
            padding: vec![0; 13],
        }
    }
}

impl McnrChunk {
    /// Total number of normals in the grid.
    pub const NORMAL_COUNT: usize = 145;

    /// Size in bytes (including padding).
    pub const SIZE_BYTES: usize = 448;

    /// Get normal at outer grid position.
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-8)
    /// * `y` - Row index (0-8)
    pub fn get_outer_normal(&self, x: usize, y: usize) -> Option<VertexNormal> {
        if x >= 9 || y >= 9 {
            return None;
        }
        let index = y * 9 + x;
        self.normals.get(index).copied()
    }

    /// Get normal at inner grid position.
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-7)
    /// * `y` - Row index (0-7)
    pub fn get_inner_normal(&self, x: usize, y: usize) -> Option<VertexNormal> {
        if x >= 8 || y >= 8 {
            return None;
        }
        let index = 81 + (y * 8 + x);
        self.normals.get(index).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::{BinReaderExt, BinWriterExt};
    use std::io::Cursor;

    #[test]
    fn vertex_normal_to_normalized() {
        let normal = VertexNormal {
            x: 127,
            y: 127,
            z: 0,
        };
        let normalized = normal.to_normalized();
        assert!((normalized[0] - 1.0).abs() < 0.01);
        assert!((normalized[1] - 1.0).abs() < 0.01);
        assert!(normalized[2].abs() < 0.01);
    }

    #[test]
    fn vertex_normal_from_normalized() {
        let normal = VertexNormal::from_normalized([1.0, 0.0, -1.0]);
        assert_eq!(normal.x, 127);
        assert_eq!(normal.y, 0);
        assert_eq!(normal.z, -127);
    }

    #[test]
    fn vertex_normal_round_trip() {
        let original = [0.5, -0.5, 0.707];
        let normal = VertexNormal::from_normalized(original);
        let normalized = normal.to_normalized();

        assert!((normalized[0] - original[0]).abs() < 0.02);
        assert!((normalized[1] - original[1]).abs() < 0.02);
        assert!((normalized[2] - original[2]).abs() < 0.02);
    }

    #[test]
    fn vertex_normal_coordinate_swap() {
        // Verify Y and Z are swapped in storage
        let normal = VertexNormal {
            x: 10,
            z: 20,
            y: 30,
        };
        let normalized = normal.to_normalized();

        // X stays in position 0
        assert_eq!((normalized[0] * 127.0) as i8, 10);
        // Y (stored in field 'y') goes to position 1
        assert_eq!((normalized[1] * 127.0) as i8, 30);
        // Z (stored in field 'z') goes to position 2
        assert_eq!((normalized[2] * 127.0) as i8, 20);
    }

    #[test]
    fn parse_mcnr_chunk() {
        let mut data = Vec::new();

        // 145 normals (up-facing)
        for _ in 0..145 {
            data.extend_from_slice(&[0i8 as u8, 0, 127]); // x=0, z=0, y=127
        }

        // 13 padding bytes
        data.extend_from_slice(&[0u8; 13]);

        assert_eq!(data.len(), 448);

        let mut cursor = Cursor::new(&data);
        let chunk: McnrChunk = cursor.read_le().unwrap();

        assert_eq!(chunk.normals.len(), 145);
        assert_eq!(chunk.padding.len(), 13);

        // Verify first normal is up-facing
        let first = chunk.normals[0];
        assert_eq!(first.x, 0);
        assert_eq!(first.z, 0);
        assert_eq!(first.y, 127);
    }

    #[test]
    fn get_outer_normal() {
        let chunk = McnrChunk::default();

        // Valid access
        let normal = chunk.get_outer_normal(0, 0);
        assert!(normal.is_some());

        // Corner cases
        assert!(chunk.get_outer_normal(8, 8).is_some()); // Last valid position
        assert!(chunk.get_outer_normal(9, 0).is_none()); // Out of bounds
        assert!(chunk.get_outer_normal(0, 9).is_none()); // Out of bounds
    }

    #[test]
    fn get_inner_normal() {
        let chunk = McnrChunk::default();

        // Valid access
        let normal = chunk.get_inner_normal(0, 0);
        assert!(normal.is_some());

        // Verify correct offset (should access index 81)
        let first_inner = chunk.normals[81];
        let accessed = chunk.get_inner_normal(0, 0).unwrap();
        assert_eq!(accessed.x, first_inner.x);
        assert_eq!(accessed.y, first_inner.y);
        assert_eq!(accessed.z, first_inner.z);

        // Corner cases
        assert!(chunk.get_inner_normal(7, 7).is_some()); // Last valid position
        assert!(chunk.get_inner_normal(8, 0).is_none()); // Out of bounds
        assert!(chunk.get_inner_normal(0, 8).is_none()); // Out of bounds
    }

    #[test]
    fn outer_grid_indexing() {
        let chunk = McnrChunk::default();

        // Verify 9×9 grid indexing
        assert_eq!(chunk.normals[0].x, chunk.get_outer_normal(0, 0).unwrap().x);
        assert_eq!(chunk.normals[8].x, chunk.get_outer_normal(8, 0).unwrap().x);
        assert_eq!(chunk.normals[9].x, chunk.get_outer_normal(0, 1).unwrap().x);
        assert_eq!(chunk.normals[80].x, chunk.get_outer_normal(8, 8).unwrap().x);
    }

    #[test]
    fn inner_grid_indexing() {
        let chunk = McnrChunk::default();

        // Verify 8×8 grid indexing starting at offset 81
        assert_eq!(chunk.normals[81].x, chunk.get_inner_normal(0, 0).unwrap().x);
        assert_eq!(chunk.normals[88].x, chunk.get_inner_normal(7, 0).unwrap().x);
        assert_eq!(chunk.normals[89].x, chunk.get_inner_normal(0, 1).unwrap().x);
        assert_eq!(
            chunk.normals[144].x,
            chunk.get_inner_normal(7, 7).unwrap().x
        );
    }

    #[test]
    fn default_chunk() {
        let chunk = McnrChunk::default();

        assert_eq!(chunk.normals.len(), 145);
        assert_eq!(chunk.padding.len(), 13);

        // All normals should be up-facing by default
        for normal in &chunk.normals {
            assert_eq!(normal.x, 0);
            assert_eq!(normal.z, 0);
            assert_eq!(normal.y, 127);
        }

        // Padding should be zeros
        assert!(chunk.padding.iter().all(|&b| b == 0));
    }

    #[test]
    fn round_trip_serialization() {
        let mut original = McnrChunk::default();

        // Modify some normals
        original.normals[0] = VertexNormal {
            x: 10,
            y: 20,
            z: 30,
        };
        original.normals[80] = VertexNormal {
            x: -50,
            y: 60,
            z: -70,
        };
        original.normals[144] = VertexNormal {
            x: 100,
            y: -100,
            z: 50,
        };

        // Serialize
        let mut buffer = Cursor::new(Vec::new());
        buffer.write_le(&original).unwrap();

        // Deserialize
        buffer.set_position(0);
        let parsed: McnrChunk = buffer.read_le().unwrap();

        // Verify
        assert_eq!(parsed.normals.len(), original.normals.len());
        assert_eq!(parsed.padding.len(), original.padding.len());

        for (i, (orig, parsed)) in original
            .normals
            .iter()
            .zip(parsed.normals.iter())
            .enumerate()
        {
            assert_eq!(orig.x, parsed.x, "Mismatch at index {i}");
            assert_eq!(orig.y, parsed.y, "Mismatch at index {i}");
            assert_eq!(orig.z, parsed.z, "Mismatch at index {i}");
        }
    }

    #[test]
    fn size_validation() {
        let chunk = McnrChunk::default();

        // Serialize and check size
        let mut buffer = Cursor::new(Vec::new());
        buffer.write_le(&chunk).unwrap();

        assert_eq!(buffer.get_ref().len(), McnrChunk::SIZE_BYTES);
        assert_eq!(buffer.get_ref().len(), 448);
    }

    #[test]
    fn constants() {
        assert_eq!(McnrChunk::NORMAL_COUNT, 145);
        assert_eq!(McnrChunk::SIZE_BYTES, 448);
        assert_eq!(McnrChunk::SIZE_BYTES, 145 * 3 + 13);
    }
}
