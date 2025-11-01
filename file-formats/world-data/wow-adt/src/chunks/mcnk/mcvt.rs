use binrw::{BinRead, BinWrite};

/// MCVT chunk - Vertex height map (145 values, Vanilla+)
///
/// Contains height values for a mixed-resolution terrain grid:
/// - 9×9 outer vertices (81 values): Always rendered
/// - 8×8 inner vertices (64 values): LOD-dependent rendering
///
/// Heights are relative to the world position stored in McnkHeader.position.
/// Total grid points: 145 = (9*9) + (8*8)
///
/// The vertex arrangement follows ROAM (Real-time Optimally Adapting Mesh) principles
/// for efficient terrain LOD transitions.
///
/// ## Binary Format Layout
///
/// The MCVT chunk stores 145 f32 height values in **interleaved row order**:
/// ```text
/// Row  0: 9 values (outer vertices 0-8)
/// Row  1: 8 values (inner vertices 0-7, offset by 0.5 UNITSIZE in X)
/// Row  2: 9 values (outer vertices 9-17)
/// Row  3: 8 values (inner vertices 8-15, offset)
/// ... (pattern continues)
/// Row 16: 9 values (outer vertices 72-80)
/// ```
///
/// This creates a logical 17×17 grid with alternating 9/8 column counts.
///
/// ## Cross-Reference Validation
///
/// This implementation has been validated against wow-map-viewer (proven C++ reference):
///
/// ✅ Vertex count: 145 (9×9 outer + 8×8 inner)
/// ✅ Grid structure: Alternating 9/8 columns across 17 logical rows
/// ✅ Height interpretation: f32 values relative to MCNK position
/// ✅ Binary layout: Interleaved rows (9,8,9,8...9)
/// ✅ Data format: 4-byte IEEE 754 floats (little-endian)
///
/// **Reference:** wow-map-viewer/src/maptile.cpp:366-390
///
/// The wow-map-viewer reads heights sequentially using a 17-row loop:
/// ```cpp
/// for (int j=0; j<17; j++) {
///     for (int i=0; i<((j%2)?8:9); i++) {
///         f.read(&h,4);
///         Vec3D v = Vec3D(xbase+xpos, ybase+h, zbase+zpos);
///     }
/// }
/// ```
///
/// And uses the `indexMapBuf` formula to map logical (x,y) to linear indices:
/// ```cpp
/// int indexMapBuf(int x, int y) {
///     return ((y+1)/2)*9 + (y/2)*8 + x;
/// }
/// ```
///
/// This implementation provides separate accessors for type safety:
/// - `get_outer_height(x, y)`: Access via logical even row mapping
/// - `get_inner_height(x, y)`: Access via logical odd row mapping
///
/// See: `/specs/001-adt-binrw-refactor/CROSS_REFERENCE_MCVT.md` for detailed validation report
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ✅ Introduced
/// - **TBC (2.4.3)**: ✅ Present
/// - **WotLK (3.3.5a)**: ✅ Present
/// - **Cataclysm (4.3.4)**: ✅ Present
/// - **MoP (5.4.8)**: ✅ Present
///
/// Reference: <https://wowdev.wiki/ADT/v18#MCVT_sub-chunk>
#[derive(Debug, Clone, Default, BinRead, BinWrite)]
#[brw(little)]
pub struct McvtChunk {
    /// Height values (145 f32 entries)
    ///
    /// Binary layout (interleaved rows):
    /// - Indices 0-8: Row 0 (outer, 9 vertices)
    /// - Indices 9-16: Row 1 (inner, 8 vertices)
    /// - Indices 17-25: Row 2 (outer, 9 vertices)
    /// - Indices 26-33: Row 3 (inner, 8 vertices)
    /// - ... (pattern continues for 17 rows total)
    ///
    /// Height values are relative offsets from McnkHeader.position[1] (Y coordinate).
    /// Final world Y = McnkHeader.position[1] + heights[index]
    #[br(count = 145)]
    pub heights: Vec<f32>,
}

impl McvtChunk {
    /// Total number of vertices in the grid.
    ///
    /// Cross-reference: Matches wow-map-viewer `mapbufsize = 9*9 + 8*8`
    pub const VERTEX_COUNT: usize = 145;

    /// Number of outer grid vertices (9×9).
    ///
    /// Cross-reference: Matches wow-map-viewer outer vertex count (even rows)
    pub const OUTER_VERTICES: usize = 81;

    /// Number of inner grid vertices (8×8).
    ///
    /// Cross-reference: Matches wow-map-viewer inner vertex count (odd rows)
    pub const INNER_VERTICES: usize = 64;

    /// Get height at outer grid position.
    ///
    /// Maps to wow-map-viewer logical grid position (x, y*2) where y*2 is an even row.
    ///
    /// Uses indexMapBuf formula: `((y*2+1)/2)*9 + ((y*2)/2)*8 + x` = `y*9 + y*8 + x`
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-8)
    /// * `y` - Row index in outer grid (0-8)
    ///
    /// # Returns
    ///
    /// Height value or None if indices out of bounds
    ///
    /// # Example
    ///
    /// ```
    /// # use wow_adt::chunks::mcnk::McvtChunk;
    /// let mut heights = vec![0.0; 145];
    /// heights[0] = 10.0;  // Row 0, col 0
    /// heights[17] = 25.0; // Row 2, col 0
    ///
    /// let chunk = McvtChunk { heights };
    /// assert_eq!(chunk.get_outer_height(0, 0), Some(10.0));  // Logical row 0
    /// assert_eq!(chunk.get_outer_height(0, 1), Some(25.0));  // Logical row 2
    /// ```
    pub fn get_outer_height(&self, x: usize, y: usize) -> Option<f32> {
        if x >= 9 || y >= 9 {
            return None;
        }
        // Map to interleaved storage: even row y*2 using indexMapBuf formula
        let logical_row = y * 2;
        let index = ((logical_row + 1) / 2) * 9 + (logical_row / 2) * 8 + x;
        self.heights.get(index).copied()
    }

    /// Get height at inner grid position.
    ///
    /// Maps to wow-map-viewer logical grid position (x, y*2+1) where y*2+1 is an odd row.
    ///
    /// Uses indexMapBuf formula: `((y*2+1+1)/2)*9 + ((y*2+1)/2)*8 + x` = `(y+1)*9 + y*8 + x`
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-7)
    /// * `y` - Row index in inner grid (0-7)
    ///
    /// # Returns
    ///
    /// Height value or None if indices out of bounds
    ///
    /// # Example
    ///
    /// ```
    /// # use wow_adt::chunks::mcnk::McvtChunk;
    /// let mut heights = vec![0.0; 145];
    /// heights[9] = 15.0;   // Row 1, col 0
    /// heights[26] = 30.0;  // Row 3, col 0
    ///
    /// let chunk = McvtChunk { heights };
    /// assert_eq!(chunk.get_inner_height(0, 0), Some(15.0));  // Logical row 1
    /// assert_eq!(chunk.get_inner_height(0, 1), Some(30.0));  // Logical row 3
    /// ```
    pub fn get_inner_height(&self, x: usize, y: usize) -> Option<f32> {
        if x >= 8 || y >= 8 {
            return None;
        }
        // Map to interleaved storage: odd row y*2+1 using indexMapBuf formula
        let logical_row = y * 2 + 1;
        let index = ((logical_row + 1) / 2) * 9 + (logical_row / 2) * 8 + x;
        self.heights.get(index).copied()
    }

    /// Find minimum height value.
    pub fn min_height(&self) -> Option<f32> {
        self.heights
            .iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
    }

    /// Find maximum height value.
    pub fn max_height(&self) -> Option<f32> {
        self.heights
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::{BinRead, BinWrite};
    use std::io::Cursor;

    #[test]
    fn test_vertex_count_constants() {
        assert_eq!(McvtChunk::VERTEX_COUNT, 145);
        assert_eq!(McvtChunk::OUTER_VERTICES, 81);
        assert_eq!(McvtChunk::INNER_VERTICES, 64);
        assert_eq!(
            McvtChunk::OUTER_VERTICES + McvtChunk::INNER_VERTICES,
            McvtChunk::VERTEX_COUNT
        );
    }

    #[test]
    fn test_parse_145_heights() {
        let mut data = Vec::new();
        for i in 0..145 {
            data.extend_from_slice(&(i as f32).to_le_bytes());
        }

        let mut cursor = Cursor::new(data);
        let chunk = McvtChunk::read(&mut cursor).expect("Failed to parse MCVT chunk");

        assert_eq!(chunk.heights.len(), 145);
        for (i, &height) in chunk.heights.iter().enumerate() {
            assert_eq!(height, i as f32);
        }
    }

    #[test]
    fn test_outer_grid_accessor() {
        let mut heights = vec![0.0; 145];
        // Set up interleaved row pattern
        // Row 0 (indices 0-8): outer row 0
        heights[0] = 100.0;  // (0,0)
        heights[8] = 108.0;  // (8,0)

        // Row 2 (indices 17-25): outer row 1
        heights[17] = 117.0; // (0,1)
        heights[25] = 125.0; // (8,1)

        let chunk = McvtChunk { heights };

        assert_eq!(chunk.get_outer_height(0, 0), Some(100.0));
        assert_eq!(chunk.get_outer_height(8, 0), Some(108.0));
        assert_eq!(chunk.get_outer_height(0, 1), Some(117.0));
        assert_eq!(chunk.get_outer_height(8, 1), Some(125.0));
    }

    #[test]
    fn test_outer_grid_bounds_checking() {
        let chunk = McvtChunk {
            heights: vec![0.0; 145],
        };

        assert_eq!(chunk.get_outer_height(9, 0), None);
        assert_eq!(chunk.get_outer_height(0, 9), None);
        assert_eq!(chunk.get_outer_height(9, 9), None);
        assert_eq!(chunk.get_outer_height(100, 100), None);
    }

    #[test]
    fn test_inner_grid_accessor() {
        let mut heights = vec![0.0; 145];
        // Set up interleaved row pattern
        // Row 1 (indices 9-16): inner row 0
        heights[9] = 209.0;   // (0,0)
        heights[16] = 216.0;  // (7,0)

        // Row 3 (indices 26-33): inner row 1
        heights[26] = 226.0;  // (0,1)
        heights[33] = 233.0;  // (7,1)

        let chunk = McvtChunk { heights };

        assert_eq!(chunk.get_inner_height(0, 0), Some(209.0));
        assert_eq!(chunk.get_inner_height(7, 0), Some(216.0));
        assert_eq!(chunk.get_inner_height(0, 1), Some(226.0));
        assert_eq!(chunk.get_inner_height(7, 1), Some(233.0));
    }

    #[test]
    fn test_inner_grid_bounds_checking() {
        let chunk = McvtChunk {
            heights: vec![0.0; 145],
        };

        assert_eq!(chunk.get_inner_height(8, 0), None);
        assert_eq!(chunk.get_inner_height(0, 8), None);
        assert_eq!(chunk.get_inner_height(8, 8), None);
        assert_eq!(chunk.get_inner_height(100, 100), None);
    }

    #[test]
    fn test_min_max_height() {
        let heights = vec![
            10.0, 20.0, 5.0, 30.0, 15.0, 25.0, 8.0, 35.0, 12.0, 18.0, 22.0, 28.0, 14.0, 32.0, 16.0,
            24.0, 26.0, 11.0, 19.0, 21.0, 27.0, 13.0, 29.0, 17.0, 23.0, 31.0, 9.0, 33.0, 7.0, 34.0,
            6.0, 36.0, 4.0, 37.0, 3.0, 38.0, 2.0, 39.0, 1.0, 40.0, 0.0, 41.0, 42.0, 43.0, 44.0,
            45.0, 46.0, 47.0, 48.0, 49.0, 50.0, 51.0, 52.0, 53.0, 54.0, 55.0, 56.0, 57.0, 58.0,
            59.0, 60.0, 61.0, 62.0, 63.0, 64.0, 65.0, 66.0, 67.0, 68.0, 69.0, 70.0, 71.0, 72.0,
            73.0, 74.0, 75.0, 76.0, 77.0, 78.0, 79.0, 80.0, 81.0, 82.0, 83.0, 84.0, 85.0, 86.0,
            87.0, 88.0, 89.0, 90.0, 91.0, 92.0, 93.0, 94.0, 95.0, 96.0, 97.0, 98.0, 99.0, 100.0,
            101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0, 111.0, 112.0,
            113.0, 114.0, 115.0, 116.0, 117.0, 118.0, 119.0, 120.0, 121.0, 122.0, 123.0, 124.0,
            125.0, 126.0, 127.0, 128.0, 129.0, 130.0, 131.0, 132.0, 133.0, 134.0, 135.0, 136.0,
            137.0, 138.0, 139.0, 140.0, 141.0, 142.0, 143.0, 144.0,
        ];
        let chunk = McvtChunk { heights };

        assert_eq!(chunk.min_height(), Some(0.0));
        assert_eq!(chunk.max_height(), Some(144.0));
    }

    #[test]
    fn test_min_max_height_negative_values() {
        let heights = vec![
            -50.0, -20.0, -100.0, 30.0, 0.0, 25.0, -75.0, 40.0, 10.0, -30.0, 20.0, 50.0, -10.0,
            60.0, 5.0, 15.0, 35.0, -5.0, 45.0, 55.0, 70.0, -15.0, 65.0, 12.0, 22.0, 75.0, -25.0,
            80.0, -35.0, 85.0, -40.0, 90.0, -45.0, 95.0, -55.0, 100.0, -60.0, 105.0, -65.0, 110.0,
            -70.0, 115.0, 120.0, 125.0, 130.0, 135.0, 140.0, 145.0, 150.0, 155.0, 160.0, 165.0,
            170.0, 175.0, 180.0, 185.0, 190.0, 195.0, 200.0, 205.0, 210.0, 215.0, 220.0, 225.0,
            230.0, 235.0, 240.0, 245.0, 250.0, 255.0, 260.0, 265.0, 270.0, 275.0, 280.0, 285.0,
            290.0, 295.0, 300.0, 305.0, 310.0, 315.0, 320.0, 325.0, 330.0, 335.0, 340.0, 345.0,
            350.0, 355.0, 360.0, 365.0, 370.0, 375.0, 380.0, 385.0, 390.0, 395.0, 400.0, 405.0,
            410.0, 415.0, 420.0, 425.0, 430.0, 435.0, 440.0, 445.0, 450.0, 455.0, 460.0, 465.0,
            470.0, 475.0, 480.0, 485.0, 490.0, 495.0, 500.0, 505.0, 510.0, 515.0, 520.0, 525.0,
            530.0, 535.0, 540.0, 545.0, 550.0, 555.0, 560.0, 565.0, 570.0, 575.0, 580.0, 585.0,
            590.0, 595.0, 600.0, 605.0, 610.0, 615.0, 620.0,
        ];
        let chunk = McvtChunk { heights };

        assert_eq!(chunk.min_height(), Some(-100.0));
        assert_eq!(chunk.max_height(), Some(620.0));
    }

    #[test]
    fn test_min_max_empty_heights() {
        let chunk = McvtChunk {
            heights: Vec::new(),
        };

        assert_eq!(chunk.min_height(), None);
        assert_eq!(chunk.max_height(), None);
    }

    #[test]
    fn test_round_trip_serialization() {
        let original_heights: Vec<f32> = (0..145).map(|i| i as f32 * 2.5).collect();
        let original = McvtChunk {
            heights: original_heights.clone(),
        };

        let mut buffer = Cursor::new(Vec::new());
        original
            .write(&mut buffer)
            .expect("Failed to write MCVT chunk");

        buffer.set_position(0);
        let parsed = McvtChunk::read(&mut buffer).expect("Failed to read MCVT chunk");

        assert_eq!(parsed.heights.len(), 145);
        for (i, (&original_height, &parsed_height)) in original
            .heights
            .iter()
            .zip(parsed.heights.iter())
            .enumerate()
        {
            assert_eq!(original_height, parsed_height, "Mismatch at index {}", i);
        }
    }

    #[test]
    fn test_default_construction() {
        let chunk = McvtChunk::default();
        assert_eq!(chunk.heights.len(), 0);
    }

    /// Test cross-reference validation with wow-map-viewer indexMapBuf formula
    ///
    /// The wow-map-viewer uses the formula: `indexMapBuf(x, y) = ((y+1)/2)*9 + (y/2)*8 + x`
    /// This test verifies our accessor methods correctly map to the interleaved storage.
    #[test]
    fn test_wow_map_viewer_index_equivalence() {
        let mut heights = Vec::new();
        for i in 0..145 {
            heights.push(i as f32);
        }
        let chunk = McvtChunk { heights };

        // Helper to simulate wow-map-viewer's indexMapBuf formula
        fn index_map_buf(x: usize, y: usize) -> usize {
            ((y + 1) / 2) * 9 + (y / 2) * 8 + x
        }

        // Test outer vertices (even logical rows)
        for outer_y in 0..9 {
            for x in 0..9 {
                let logical_y = outer_y * 2; // Even row (0,2,4...16)
                let expected_index = index_map_buf(x, logical_y);
                let warcraft_rs_height = chunk.get_outer_height(x, outer_y).unwrap();
                let expected_height = chunk.heights[expected_index];
                assert_eq!(
                    warcraft_rs_height, expected_height,
                    "Outer vertex ({}, {}) mismatch at logical row {}: got index {} but expected {}",
                    x, outer_y, logical_y, warcraft_rs_height as usize, expected_index
                );
            }
        }

        // Test inner vertices (odd logical rows)
        for inner_y in 0..8 {
            for x in 0..8 {
                let logical_y = inner_y * 2 + 1; // Odd row (1,3,5...15)
                let expected_index = index_map_buf(x, logical_y);
                let warcraft_rs_height = chunk.get_inner_height(x, inner_y).unwrap();
                let expected_height = chunk.heights[expected_index];
                assert_eq!(
                    warcraft_rs_height, expected_height,
                    "Inner vertex ({}, {}) mismatch at logical row {}: got index {} but expected {}",
                    x, inner_y, logical_y, warcraft_rs_height as usize, expected_index
                );
            }
        }
    }
}
