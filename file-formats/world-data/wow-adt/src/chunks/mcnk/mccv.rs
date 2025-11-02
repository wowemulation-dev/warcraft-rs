use binrw::{BinRead, BinWrite};

/// Single vertex color (4 bytes BGRA).
///
/// Used for vertex-level color tinting and shading multiplication.
/// Range: 0x00-0xFF per channel, where 0x7F represents neutral (1.0 multiplier).
///
/// Values below 0x7F darken the terrain; values above brighten it.
/// Setting all channels to 0x00 makes the chunk completely black.
#[derive(Debug, Clone, Copy, BinRead, BinWrite, PartialEq)]
#[brw(little)]
pub struct VertexColor {
    /// Blue channel (0x00-0xFF)
    pub b: u8,

    /// Green channel (0x00-0xFF)
    pub g: u8,

    /// Red channel (0x00-0xFF)
    pub r: u8,

    /// Alpha channel (0x00-0xFF)
    pub a: u8,
}

impl Default for VertexColor {
    fn default() -> Self {
        Self {
            b: 0x7F,
            g: 0x7F,
            r: 0x7F,
            a: 0xFF,
        }
    }
}

impl VertexColor {
    /// Create neutral color (no tinting).
    pub const fn neutral() -> Self {
        Self {
            b: 0x7F,
            g: 0x7F,
            r: 0x7F,
            a: 0xFF,
        }
    }

    /// Create from RGB values (alpha = 0xFF).
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { b, g, r, a: 0xFF }
    }

    /// Create from RGBA values.
    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { b, g, r, a }
    }

    /// Convert to normalized float values [R, G, B, A] (0.0-1.0).
    pub fn to_normalized(&self) -> [f32; 4] {
        [
            f32::from(self.r) / 255.0,
            f32::from(self.g) / 255.0,
            f32::from(self.b) / 255.0,
            f32::from(self.a) / 255.0,
        ]
    }

    /// Convert to shader multiplier values [R, G, B] (0.0-2.0).
    ///
    /// 0x7F (127) maps to 1.0 (neutral), 0x00 maps to 0.0 (black), 0xFF maps to ~2.0 (bright).
    pub fn to_multiplier(&self) -> [f32; 3] {
        [
            f32::from(self.r) / 127.0,
            f32::from(self.g) / 127.0,
            f32::from(self.b) / 127.0,
        ]
    }
}

/// MCCV chunk - Vertex colors (WotLK+).
///
/// Contains per-vertex color tinting for terrain shading. Same grid layout as
/// MCVT/MCNR: 9×9 outer vertices + 8×8 inner vertices = 145 total.
///
/// Color values multiply with terrain lighting. 0x7F = neutral (1.0),
/// lower values darken, higher values brighten.
///
/// Only present if McnkHeader.flags.has_mccv() is set.
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ❌ Not present
/// - **TBC (2.4.3)**: ❌ Not present
/// - **WotLK (3.3.5a)**: ✅ Introduced
/// - **Cataclysm (4.3.4)**: ✅ Present
/// - **MoP (5.4.8)**: ✅ Present
///
/// Reference: <https://wowdev.wiki/ADT/v18#MCCV_sub-chunk>
#[derive(Debug, Clone, BinRead, BinWrite)]
#[brw(little)]
pub struct MccvChunk {
    /// Vertex colors (145 BGRA entries)
    #[br(count = 145)]
    pub colors: Vec<VertexColor>,
}

impl Default for MccvChunk {
    fn default() -> Self {
        Self {
            colors: vec![VertexColor::neutral(); 145],
        }
    }
}

impl MccvChunk {
    /// Total number of vertex colors.
    pub const COLOR_COUNT: usize = 145;

    /// Get color at outer grid position.
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-8)
    /// * `y` - Row index (0-8)
    pub fn get_outer_color(&self, x: usize, y: usize) -> Option<VertexColor> {
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
    pub fn get_inner_color(&self, x: usize, y: usize) -> Option<VertexColor> {
        if x >= 8 || y >= 8 {
            return None;
        }
        let index = 81 + (y * 8 + x);
        self.colors.get(index).copied()
    }

    /// Set all vertices to neutral color.
    pub fn set_all_neutral(&mut self) {
        self.colors.fill(VertexColor::neutral());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::{BinReaderExt, BinWriterExt};
    use std::io::Cursor;

    #[test]
    fn vertex_color_size() {
        assert_eq!(std::mem::size_of::<VertexColor>(), 4);
    }

    #[test]
    fn vertex_color_default() {
        let color = VertexColor::default();
        assert_eq!(color.r, 0x7F);
        assert_eq!(color.g, 0x7F);
        assert_eq!(color.b, 0x7F);
        assert_eq!(color.a, 0xFF);
    }

    #[test]
    fn vertex_color_neutral() {
        let color = VertexColor::neutral();
        assert_eq!(color.r, 0x7F);
        assert_eq!(color.g, 0x7F);
        assert_eq!(color.b, 0x7F);
        assert_eq!(color.a, 0xFF);
    }

    #[test]
    fn vertex_color_from_rgb() {
        let color = VertexColor::from_rgb(0xFF, 0x00, 0x80);
        assert_eq!(color.r, 0xFF);
        assert_eq!(color.g, 0x00);
        assert_eq!(color.b, 0x80);
        assert_eq!(color.a, 0xFF);
    }

    #[test]
    fn vertex_color_from_rgba() {
        let color = VertexColor::from_rgba(0xFF, 0x00, 0x80, 0x40);
        assert_eq!(color.r, 0xFF);
        assert_eq!(color.g, 0x00);
        assert_eq!(color.b, 0x80);
        assert_eq!(color.a, 0x40);
    }

    #[test]
    fn vertex_color_to_normalized() {
        let color = VertexColor::from_rgba(0xFF, 0x7F, 0x00, 0x80);
        let normalized = color.to_normalized();
        assert!((normalized[0] - 1.0).abs() < 0.01);
        assert!((normalized[1] - 0.498).abs() < 0.01);
        assert!(normalized[2].abs() < 0.01);
        assert!((normalized[3] - 0.502).abs() < 0.01);
    }

    #[test]
    fn vertex_color_to_multiplier() {
        let neutral = VertexColor::neutral();
        let multiplier = neutral.to_multiplier();
        assert!((multiplier[0] - 1.0).abs() < 0.01);
        assert!((multiplier[1] - 1.0).abs() < 0.01);
        assert!((multiplier[2] - 1.0).abs() < 0.01);

        let bright = VertexColor::from_rgb(0xFF, 0xFF, 0xFF);
        let bright_mult = bright.to_multiplier();
        assert!((bright_mult[0] - 2.007).abs() < 0.01);
        assert!((bright_mult[1] - 2.007).abs() < 0.01);
        assert!((bright_mult[2] - 2.007).abs() < 0.01);

        let dark = VertexColor::from_rgb(0x00, 0x00, 0x00);
        let dark_mult = dark.to_multiplier();
        assert!(dark_mult[0].abs() < 0.01);
        assert!(dark_mult[1].abs() < 0.01);
        assert!(dark_mult[2].abs() < 0.01);
    }

    #[test]
    fn vertex_color_parse_bgra() {
        let data = [0x80u8, 0x40, 0xFF, 0x7F];
        let mut cursor = Cursor::new(&data);
        let color: VertexColor = cursor.read_le().unwrap();

        assert_eq!(color.b, 0x80);
        assert_eq!(color.g, 0x40);
        assert_eq!(color.r, 0xFF);
        assert_eq!(color.a, 0x7F);
    }

    #[test]
    fn vertex_color_round_trip() {
        let original = VertexColor::from_rgba(0xAB, 0xCD, 0xEF, 0x12);

        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        cursor.write_le(&original).unwrap();

        let mut read_cursor = Cursor::new(&buffer);
        let parsed: VertexColor = read_cursor.read_le().unwrap();

        assert_eq!(parsed, original);
    }

    #[test]
    fn mccv_chunk_default() {
        let chunk = MccvChunk::default();
        assert_eq!(chunk.colors.len(), 145);

        for color in &chunk.colors {
            assert_eq!(*color, VertexColor::neutral());
        }
    }

    #[test]
    fn mccv_chunk_color_count() {
        assert_eq!(MccvChunk::COLOR_COUNT, 145);
    }

    #[test]
    fn mccv_chunk_get_outer_color() {
        let chunk = MccvChunk::default();

        let color = chunk.get_outer_color(0, 0).unwrap();
        assert_eq!(color, VertexColor::neutral());

        let color = chunk.get_outer_color(8, 8).unwrap();
        assert_eq!(color, VertexColor::neutral());

        assert!(chunk.get_outer_color(9, 0).is_none());
        assert!(chunk.get_outer_color(0, 9).is_none());
    }

    #[test]
    fn mccv_chunk_get_inner_color() {
        let chunk = MccvChunk::default();

        let color = chunk.get_inner_color(0, 0).unwrap();
        assert_eq!(color, VertexColor::neutral());

        let color = chunk.get_inner_color(7, 7).unwrap();
        assert_eq!(color, VertexColor::neutral());

        assert!(chunk.get_inner_color(8, 0).is_none());
        assert!(chunk.get_inner_color(0, 8).is_none());
    }

    #[test]
    fn mccv_chunk_outer_inner_indices() {
        let chunk = MccvChunk::default();

        assert_eq!(chunk.get_outer_color(0, 0).unwrap(), chunk.colors[0]);
        assert_eq!(chunk.get_outer_color(8, 0).unwrap(), chunk.colors[8]);
        assert_eq!(chunk.get_outer_color(0, 8).unwrap(), chunk.colors[72]);
        assert_eq!(chunk.get_outer_color(8, 8).unwrap(), chunk.colors[80]);

        assert_eq!(chunk.get_inner_color(0, 0).unwrap(), chunk.colors[81]);
        assert_eq!(chunk.get_inner_color(7, 0).unwrap(), chunk.colors[88]);
        assert_eq!(chunk.get_inner_color(0, 7).unwrap(), chunk.colors[137]);
        assert_eq!(chunk.get_inner_color(7, 7).unwrap(), chunk.colors[144]);
    }

    #[test]
    fn mccv_chunk_set_all_neutral() {
        let mut chunk = MccvChunk {
            colors: vec![VertexColor::from_rgb(0xFF, 0x00, 0x00); 145],
        };

        chunk.set_all_neutral();

        for color in &chunk.colors {
            assert_eq!(*color, VertexColor::neutral());
        }
    }

    #[test]
    fn mccv_chunk_parse() {
        let mut data = Vec::new();

        for i in 0..145 {
            data.push((i % 256) as u8);
            data.push(((i + 1) % 256) as u8);
            data.push(((i + 2) % 256) as u8);
            data.push(((i + 3) % 256) as u8);
        }

        let mut cursor = Cursor::new(&data);
        let chunk: MccvChunk = cursor.read_le().unwrap();

        assert_eq!(chunk.colors.len(), 145);

        for (i, color) in chunk.colors.iter().enumerate() {
            assert_eq!(color.b, (i % 256) as u8);
            assert_eq!(color.g, ((i + 1) % 256) as u8);
            assert_eq!(color.r, ((i + 2) % 256) as u8);
            assert_eq!(color.a, ((i + 3) % 256) as u8);
        }
    }

    #[test]
    fn mccv_chunk_round_trip() {
        let original = MccvChunk {
            colors: (0..145)
                .map(|i| {
                    VertexColor::from_rgba(
                        (i % 256) as u8,
                        ((i + 1) % 256) as u8,
                        ((i + 2) % 256) as u8,
                        ((i + 3) % 256) as u8,
                    )
                })
                .collect(),
        };

        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        cursor.write_le(&original).unwrap();

        let mut read_cursor = Cursor::new(&buffer);
        let parsed: MccvChunk = read_cursor.read_le().unwrap();

        assert_eq!(parsed.colors.len(), original.colors.len());
        for (parsed_color, original_color) in parsed.colors.iter().zip(&original.colors) {
            assert_eq!(parsed_color, original_color);
        }
    }

    #[test]
    fn mccv_chunk_size() {
        let chunk = MccvChunk::default();

        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        cursor.write_le(&chunk).unwrap();

        assert_eq!(buffer.len(), 145 * 4);
    }
}
