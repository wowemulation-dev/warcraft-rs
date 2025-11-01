use binrw::{BinRead, BinWrite};

/// MCLV chunk - Vertex lighting (Cataclysm+)
///
/// Contains baked omni-light color data for terrain vertices. Each value is an
/// ARGB color that lightens (not just colors) the corresponding vertex.
///
/// Same vertex layout as MCVT and MCNR:
/// - 9×9 outer grid (81 values)
/// - 8×8 inner grid (64 values)
/// - Total: 145 ARGB values
///
/// ## Structure
///
/// ```text
/// struct {
///     CArgb values[145];  // 4 bytes each (ARGB u32)
/// } MCLV;
/// ```
///
/// Size: 580 bytes (145 × 4)
///
/// ## Usage
///
/// MCLV represents baked lighting from level designer-placed omni-lights.
/// Unlike MCCV (which only colors vertices), MCLV actively lightens the
/// terrain, providing localized illumination effects. Heavily used in zones
/// like Deepholm where dramatic lighting effects are needed.
///
/// The alpha channel is apparently ignored by the client.
///
/// ## References
///
/// - **wowdev.wiki**: <https://wowdev.wiki/ADT/v18#MCLV_sub-chunk>
/// - **WoWFormatLib**: `WoWFormatLib/Structs/ADT.Struct.cs:1084-1088` (struct MCLV)
/// - **noggit-red**: Not implemented (Cataclysm+ not supported)
/// - **wow.export**: Not found in ADTLoader.js
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ❌ Not present
/// - **TBC (2.4.3)**: ❌ Not present
/// - **WotLK (3.3.5a)**: ❌ Not present
/// - **Cataclysm (4.3.4)**: ✅ Introduced
/// - **MoP (5.4.8)**: ✅ Present
///
/// ## Deviations from WoWFormatLib
///
/// **CRITICAL**: WoWFormatLib incorrectly defines MCLV.color as `ushort[]` (u16 array).
/// The actual format is `CArgb` (u32 ARGB values). This is confirmed by:
/// - wowdev.wiki: "CArgb values[145]"
/// - Size calculation: 580 bytes = 145 × 4 bytes (not 145 × 2)
/// - Usage context: "color" field suggests RGB+Alpha, requiring 4 bytes
///
/// Reference: <https://wowdev.wiki/ADT/v18#MCLV_sub-chunk>
#[derive(Debug, Clone, Default, BinRead, BinWrite)]
#[brw(little)]
pub struct MclvChunk {
    /// Vertex lighting colors (145 ARGB u32 entries)
    ///
    /// Layout: First 81 values are outer 9×9 grid (row-major order),
    /// followed by 64 inner 8×8 grid values.
    ///
    /// Format: ARGB (Alpha | Red | Green | Blue)
    /// - Alpha channel is ignored by client
    /// - RGB channels lighten the vertex color
    #[br(count = 145)]
    pub colors: Vec<u32>,
}

impl MclvChunk {
    /// Total number of vertices in the grid.
    pub const VERTEX_COUNT: usize = 145;

    /// Number of outer grid vertices (9×9).
    pub const OUTER_VERTICES: usize = 81;

    /// Number of inner grid vertices (8×8).
    pub const INNER_VERTICES: usize = 64;

    /// Size in bytes.
    pub const SIZE_BYTES: usize = 580;

    /// Get color at outer grid position.
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-8)
    /// * `y` - Row index (0-8)
    ///
    /// # Returns
    ///
    /// ARGB color value or None if indices out of bounds
    pub fn get_outer_color(&self, x: usize, y: usize) -> Option<u32> {
        if x >= 9 || y >= 9 {
            return None;
        }
        let index = y * 9 + x;
        self.colors.get(index).copied()
    }

    /// Get color at inner grid position.
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-7)
    /// * `y` - Row index (0-7)
    ///
    /// # Returns
    ///
    /// ARGB color value or None if indices out of bounds
    pub fn get_inner_color(&self, x: usize, y: usize) -> Option<u32> {
        if x >= 8 || y >= 8 {
            return None;
        }
        let index = 81 + (y * 8 + x);
        self.colors.get(index).copied()
    }

    /// Extract RGB components from ARGB color.
    ///
    /// # Arguments
    ///
    /// * `argb` - ARGB color value
    ///
    /// # Returns
    ///
    /// Tuple of (red, green, blue) as u8 values
    pub fn extract_rgb(argb: u32) -> (u8, u8, u8) {
        let r = ((argb >> 16) & 0xFF) as u8;
        let g = ((argb >> 8) & 0xFF) as u8;
        let b = (argb & 0xFF) as u8;
        (r, g, b)
    }

    /// Create ARGB color from RGB components.
    ///
    /// Alpha is set to 0xFF (255) as it's ignored by the client.
    ///
    /// # Arguments
    ///
    /// * `r` - Red component (0-255)
    /// * `g` - Green component (0-255)
    /// * `b` - Blue component (0-255)
    ///
    /// # Returns
    ///
    /// ARGB color value
    pub fn create_argb(r: u8, g: u8, b: u8) -> u32 {
        0xFF00_0000 | (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::{BinReaderExt, BinWriterExt};
    use std::io::Cursor;

    #[test]
    fn test_mclv_size() {
        assert_eq!(MclvChunk::SIZE_BYTES, 580);
        assert_eq!(MclvChunk::VERTEX_COUNT, 145);
        assert_eq!(MclvChunk::VERTEX_COUNT * 4, MclvChunk::SIZE_BYTES);
    }

    #[test]
    fn test_mclv_parse() {
        // Create test data: 145 ARGB values
        let mut data = Vec::new();
        for i in 0..145u32 {
            data.extend_from_slice(&i.to_le_bytes());
        }

        let mut cursor = Cursor::new(data);
        let mclv: MclvChunk = cursor.read_le().unwrap();

        assert_eq!(mclv.colors.len(), 145);
        assert_eq!(mclv.colors[0], 0);
        assert_eq!(mclv.colors[144], 144);
    }

    #[test]
    fn test_mclv_round_trip() {
        let mut colors = Vec::with_capacity(145);
        for i in 0..145 {
            colors.push(MclvChunk::create_argb(
                i as u8,
                (i * 2) as u8,
                (i * 3) as u8,
            ));
        }
        let original = MclvChunk { colors };

        let mut buffer = Cursor::new(Vec::new());
        buffer.write_le(&original).unwrap();
        assert_eq!(buffer.position(), 580);

        let data = buffer.into_inner();
        let mut cursor = Cursor::new(data);
        let parsed: MclvChunk = cursor.read_le().unwrap();

        assert_eq!(original.colors, parsed.colors);
    }

    #[test]
    fn test_mclv_get_outer_color() {
        let mut colors = vec![0u32; 145];
        colors[0] = 0xFF112233; // Top-left
        colors[80] = 0xFF445566; // Bottom-right of outer grid
        let mclv = MclvChunk { colors };

        assert_eq!(mclv.get_outer_color(0, 0), Some(0xFF112233));
        assert_eq!(mclv.get_outer_color(8, 8), Some(0xFF445566));
        assert_eq!(mclv.get_outer_color(9, 0), None); // Out of bounds
    }

    #[test]
    fn test_mclv_get_inner_color() {
        let mut colors = vec![0u32; 145];
        colors[81] = 0xFF778899; // First inner vertex
        colors[144] = 0xFFAABBCC; // Last inner vertex
        let mclv = MclvChunk { colors };

        assert_eq!(mclv.get_inner_color(0, 0), Some(0xFF778899));
        assert_eq!(mclv.get_inner_color(7, 7), Some(0xFFAABBCC));
        assert_eq!(mclv.get_inner_color(8, 0), None); // Out of bounds
    }

    #[test]
    fn test_extract_rgb() {
        let argb = 0xFF112233;
        let (r, g, b) = MclvChunk::extract_rgb(argb);
        assert_eq!(r, 0x11);
        assert_eq!(g, 0x22);
        assert_eq!(b, 0x33);
    }

    #[test]
    fn test_create_argb() {
        let argb = MclvChunk::create_argb(0xAA, 0xBB, 0xCC);
        assert_eq!(argb, 0xFFAABBCC);
        assert_eq!(argb >> 24, 0xFF); // Alpha should be 0xFF
    }

    #[test]
    fn test_default() {
        let mclv = MclvChunk::default();
        assert_eq!(mclv.colors.len(), 0);
    }
}
