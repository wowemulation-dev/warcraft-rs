//! MH2O instance structure and liquid vertex formats.
//!
//! Instances define individual liquid layers with their position, dimensions,
//! and vertex data format. Vertex data follows one of four LVF (Liquid Vertex Format) cases.

use binrw::{BinRead, BinWrite};

/// Liquid Vertex Format (LVF) enumeration.
///
/// Determines the structure of vertex data for a liquid layer.
/// Format can be determined from vertex data size: `size / vertex_count`
///
/// | Multiplier | LVF Case | Components |
/// |------------|----------|------------|
/// | 5          | 0        | Height + Depth |
/// | 8          | 1        | Height + UV |
/// | 1          | 2        | Depth only |
/// | 9          | 3        | Height + UV + Depth |
///
/// Reference: <https://wowdev.wiki/ADT/v18#mh2o_instance>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LiquidVertexFormat {
    /// Height + Depth (5 bytes/vertex: f32 + u8)
    HeightDepth = 0,

    /// Height + UV (8 bytes/vertex: f32 + u16 + u16)
    HeightUv = 1,

    /// Depth only (1 byte/vertex: u8)
    DepthOnly = 2,

    /// Height + UV + Depth (9 bytes/vertex: f32 + u16 + u16 + u8)
    HeightUvDepth = 3,
}

impl LiquidVertexFormat {
    /// Determine LVF from bytes-per-vertex.
    ///
    /// # Arguments
    ///
    /// * `bytes_per_vertex` - Size calculated as `data_size / vertex_count`
    ///
    /// # Returns
    ///
    /// Matching LVF or None if invalid
    pub fn from_bytes_per_vertex(bytes_per_vertex: usize) -> Option<Self> {
        match bytes_per_vertex {
            5 => Some(Self::HeightDepth),
            8 => Some(Self::HeightUv),
            1 => Some(Self::DepthOnly),
            9 => Some(Self::HeightUvDepth),
            _ => None,
        }
    }

    /// Get bytes per vertex for this format.
    pub fn bytes_per_vertex(self) -> usize {
        match self {
            Self::HeightDepth => 5,
            Self::HeightUv => 8,
            Self::DepthOnly => 1,
            Self::HeightUvDepth => 9,
        }
    }
}

/// MH2O instance - defines one liquid layer (WotLK+).
///
/// Contains position, dimensions, and offsets to vertex/bitmap data.
/// Each instance represents a rectangular region of liquid tiles within
/// the 8×8 tile grid.
///
/// # Binary Layout
///
/// ```text
/// Offset | Size | Field                  | Description
/// -------|------|------------------------|----------------------------
/// 0x00   |  2   | liquid_type            | LiquidTypeRec foreign key
/// 0x02   |  2   | liquid_object_or_lvf   | LiquidObjectRec ID or LVF
/// 0x04   |  4   | min_height_level       | Minimum elevation
/// 0x08   |  4   | max_height_level       | Maximum elevation
/// 0x0C   |  1   | x_offset               | Tile X offset (0-7)
/// 0x0D   |  1   | y_offset               | Tile Y offset (0-7)
/// 0x0E   |  1   | width                  | Tile width (1-8)
/// 0x0F   |  1   | height                 | Tile height (1-8)
/// 0x10   |  4   | offset_exists_bitmap   | Render mask offset
/// 0x14   |  4   | offset_vertex_data     | Vertex data offset
/// ```
///
/// Total: 24 bytes
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ❌ Not present
/// - **TBC (2.4.3)**: ❌ Not present
/// - **WotLK (3.3.5a)**: ✅ Introduced
/// - **Cataclysm (4.3.4)**: ✅ Present
/// - **MoP (5.4.8)**: ✅ Present
///
/// Reference: <https://wowdev.wiki/ADT/v18#mh2o_instance>
#[derive(Debug, Clone, Copy, BinRead, BinWrite)]
#[brw(little)]
pub struct Mh2oInstance {
    /// Liquid type ID from LiquidTypeRec.dbc
    pub liquid_type: u16,

    /// LiquidObjectRec ID (if ≥42) or LVF value (WotLK)
    ///
    /// In Cataclysm+, values ≥42 reference LiquidObjectRec which resolves to LVF.
    /// In WotLK, this directly encodes the LVF case (0-3).
    pub liquid_object_or_lvf: u16,

    /// Minimum height level
    ///
    /// Used as default height if no vertex heightmap present.
    pub min_height_level: f32,

    /// Maximum height level
    ///
    /// Used as default height if no vertex heightmap present.
    pub max_height_level: f32,

    /// X offset within 8×8 tile grid (0-7)
    pub x_offset: u8,

    /// Y offset within 8×8 tile grid (0-7)
    pub y_offset: u8,

    /// Width in tiles (1-8)
    pub width: u8,

    /// Height in tiles (1-8)
    pub height: u8,

    /// Offset to exists bitmap (relative to MH2O data start)
    ///
    /// Bitmap size: `(width * height + 7) / 8` bytes.
    /// Each bit indicates whether that tile quad should render.
    pub offset_exists_bitmap: u32,

    /// Offset to vertex data (relative to MH2O data start)
    ///
    /// Vertex count: `(width + 1) * (height + 1)`.
    /// Format depends on LVF case (0-3).
    /// If 0, all heights default to min_height_level/max_height_level.
    pub offset_vertex_data: u32,
}

impl Default for Mh2oInstance {
    fn default() -> Self {
        Self {
            liquid_type: 0,
            liquid_object_or_lvf: 0,
            min_height_level: 0.0,
            max_height_level: 0.0,
            x_offset: 0,
            y_offset: 0,
            width: 1,
            height: 1,
            offset_exists_bitmap: 0,
            offset_vertex_data: 0,
        }
    }
}

impl Mh2oInstance {
    /// Instance structure size in bytes.
    pub const SIZE: usize = 24;

    /// Maximum tile grid size.
    pub const MAX_TILE_SIZE: usize = 8;

    /// Check if instance has vertex data.
    pub fn has_vertex_data(&self) -> bool {
        self.offset_vertex_data != 0
    }

    /// Check if instance has exists bitmap.
    pub fn has_exists_bitmap(&self) -> bool {
        self.offset_exists_bitmap != 0
    }

    /// Calculate vertex count.
    ///
    /// Returns `(width + 1) * (height + 1)` for vertex grid.
    pub fn vertex_count(&self) -> usize {
        (self.width as usize + 1) * (self.height as usize + 1)
    }

    /// Calculate tile quad count.
    ///
    /// Returns `width * height` for renderable tile quads.
    pub fn tile_count(&self) -> usize {
        self.width as usize * self.height as usize
    }

    /// Calculate exists bitmap size in bytes.
    ///
    /// Returns `(width * height + 7) / 8` bytes.
    pub fn bitmap_size(&self) -> usize {
        self.tile_count().div_ceil(8)
    }

    /// Get LVF from liquid_object_or_lvf field (WotLK only).
    ///
    /// In WotLK, values <42 directly encode LVF (0-3).
    /// In Cataclysm+, this requires database lookup.
    pub fn get_lvf_wotlk(&self) -> Option<LiquidVertexFormat> {
        if self.liquid_object_or_lvf < 42 {
            match self.liquid_object_or_lvf {
                0 => Some(LiquidVertexFormat::HeightDepth),
                1 => Some(LiquidVertexFormat::HeightUv),
                2 => Some(LiquidVertexFormat::DepthOnly),
                3 => Some(LiquidVertexFormat::HeightUvDepth),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Calculate expected vertex data size for given LVF.
    ///
    /// # Arguments
    ///
    /// * `lvf` - Liquid vertex format
    ///
    /// # Returns
    ///
    /// Expected size in bytes
    pub fn expected_vertex_size(&self, lvf: LiquidVertexFormat) -> usize {
        self.vertex_count() * lvf.bytes_per_vertex()
    }

    /// Validate instance dimensions.
    pub fn validate_dimensions(&self) -> bool {
        self.width >= 1
            && self.width <= Self::MAX_TILE_SIZE as u8
            && self.height >= 1
            && self.height <= Self::MAX_TILE_SIZE as u8
            && self.x_offset <= (Self::MAX_TILE_SIZE - self.width as usize) as u8
            && self.y_offset <= (Self::MAX_TILE_SIZE - self.height as usize) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_lvf_from_bytes_per_vertex() {
        assert_eq!(
            LiquidVertexFormat::from_bytes_per_vertex(5),
            Some(LiquidVertexFormat::HeightDepth)
        );
        assert_eq!(
            LiquidVertexFormat::from_bytes_per_vertex(8),
            Some(LiquidVertexFormat::HeightUv)
        );
        assert_eq!(
            LiquidVertexFormat::from_bytes_per_vertex(1),
            Some(LiquidVertexFormat::DepthOnly)
        );
        assert_eq!(
            LiquidVertexFormat::from_bytes_per_vertex(9),
            Some(LiquidVertexFormat::HeightUvDepth)
        );
        assert_eq!(LiquidVertexFormat::from_bytes_per_vertex(7), None);
    }

    #[test]
    fn test_lvf_bytes_per_vertex() {
        assert_eq!(LiquidVertexFormat::HeightDepth.bytes_per_vertex(), 5);
        assert_eq!(LiquidVertexFormat::HeightUv.bytes_per_vertex(), 8);
        assert_eq!(LiquidVertexFormat::DepthOnly.bytes_per_vertex(), 1);
        assert_eq!(LiquidVertexFormat::HeightUvDepth.bytes_per_vertex(), 9);
    }

    #[test]
    fn test_mh2o_instance_size() {
        assert_eq!(std::mem::size_of::<Mh2oInstance>(), 24);
        assert_eq!(Mh2oInstance::SIZE, 24);
    }

    #[test]
    fn test_mh2o_instance_parse() {
        let data = [
            0x01, 0x00, // liquid_type: 1
            0x00, 0x00, // liquid_object_or_lvf: 0 (HeightDepth)
            0x00, 0x00, 0x80, 0x3F, // min_height: 1.0
            0x00, 0x00, 0x00, 0x40, // max_height: 2.0
            0x02, // x_offset: 2
            0x03, // y_offset: 3
            0x04, // width: 4
            0x05, // height: 5
            0x10, 0x00, 0x00, 0x00, // offset_exists_bitmap: 16
            0x20, 0x00, 0x00, 0x00, // offset_vertex_data: 32
        ];

        let mut cursor = Cursor::new(&data);
        let instance = Mh2oInstance::read_le(&mut cursor).unwrap();

        assert_eq!(instance.liquid_type, 1);
        assert_eq!(instance.liquid_object_or_lvf, 0);
        assert_eq!(instance.min_height_level, 1.0);
        assert_eq!(instance.max_height_level, 2.0);
        assert_eq!(instance.x_offset, 2);
        assert_eq!(instance.y_offset, 3);
        assert_eq!(instance.width, 4);
        assert_eq!(instance.height, 5);
        assert_eq!(instance.offset_exists_bitmap, 16);
        assert_eq!(instance.offset_vertex_data, 32);
    }

    #[test]
    fn test_mh2o_instance_vertex_count() {
        let instance = Mh2oInstance {
            width: 4,
            height: 5,
            ..Default::default()
        };

        assert_eq!(instance.vertex_count(), 30); // (4+1) * (5+1) = 5 * 6 = 30
    }

    #[test]
    fn test_mh2o_instance_tile_count() {
        let instance = Mh2oInstance {
            width: 4,
            height: 5,
            ..Default::default()
        };

        assert_eq!(instance.tile_count(), 20); // 4 * 5
    }

    #[test]
    fn test_mh2o_instance_bitmap_size() {
        // width=4, height=5 -> 20 tiles -> ceil(20/8) = 3 bytes
        let instance = Mh2oInstance {
            width: 4,
            height: 5,
            ..Default::default()
        };
        assert_eq!(instance.bitmap_size(), 3);

        // width=8, height=8 -> 64 tiles -> ceil(64/8) = 8 bytes
        let instance2 = Mh2oInstance {
            width: 8,
            height: 8,
            ..Default::default()
        };
        assert_eq!(instance2.bitmap_size(), 8);

        // width=1, height=1 -> 1 tile -> ceil(1/8) = 1 byte
        let instance3 = Mh2oInstance {
            width: 1,
            height: 1,
            ..Default::default()
        };
        assert_eq!(instance3.bitmap_size(), 1);
    }

    #[test]
    fn test_mh2o_instance_get_lvf_wotlk() {
        assert_eq!(
            Mh2oInstance {
                liquid_object_or_lvf: 0,
                ..Default::default()
            }
            .get_lvf_wotlk(),
            Some(LiquidVertexFormat::HeightDepth)
        );

        assert_eq!(
            Mh2oInstance {
                liquid_object_or_lvf: 1,
                ..Default::default()
            }
            .get_lvf_wotlk(),
            Some(LiquidVertexFormat::HeightUv)
        );

        assert_eq!(
            Mh2oInstance {
                liquid_object_or_lvf: 2,
                ..Default::default()
            }
            .get_lvf_wotlk(),
            Some(LiquidVertexFormat::DepthOnly)
        );

        assert_eq!(
            Mh2oInstance {
                liquid_object_or_lvf: 3,
                ..Default::default()
            }
            .get_lvf_wotlk(),
            Some(LiquidVertexFormat::HeightUvDepth)
        );

        // LiquidObject reference (Cata+)
        assert_eq!(
            Mh2oInstance {
                liquid_object_or_lvf: 42,
                ..Default::default()
            }
            .get_lvf_wotlk(),
            None
        );
    }

    #[test]
    fn test_mh2o_instance_expected_vertex_size() {
        let instance = Mh2oInstance {
            width: 4,
            height: 5,
            ..Default::default()
        };

        // vertex_count = (4+1) * (5+1) = 30
        assert_eq!(
            instance.expected_vertex_size(LiquidVertexFormat::HeightDepth),
            150
        ); // 30 * 5
        assert_eq!(
            instance.expected_vertex_size(LiquidVertexFormat::HeightUv),
            240
        ); // 30 * 8
        assert_eq!(
            instance.expected_vertex_size(LiquidVertexFormat::DepthOnly),
            30
        ); // 30 * 1
        assert_eq!(
            instance.expected_vertex_size(LiquidVertexFormat::HeightUvDepth),
            270
        ); // 30 * 9
    }

    #[test]
    fn test_mh2o_instance_validate_dimensions() {
        // Valid dimensions
        assert!(
            Mh2oInstance {
                x_offset: 0,
                y_offset: 0,
                width: 4,
                height: 4,
                ..Default::default()
            }
            .validate_dimensions()
        );

        // Maximum valid dimensions
        assert!(
            Mh2oInstance {
                x_offset: 0,
                y_offset: 0,
                width: 8,
                height: 8,
                ..Default::default()
            }
            .validate_dimensions()
        );

        // Invalid: width = 0
        assert!(
            !Mh2oInstance {
                width: 0,
                height: 4,
                ..Default::default()
            }
            .validate_dimensions()
        );

        // Invalid: width + x_offset > 8
        assert!(
            !Mh2oInstance {
                x_offset: 5,
                width: 4,
                ..Default::default()
            }
            .validate_dimensions()
        );

        // Invalid: height + y_offset > 8
        assert!(
            !Mh2oInstance {
                y_offset: 7,
                height: 2,
                ..Default::default()
            }
            .validate_dimensions()
        );
    }

    #[test]
    fn test_mh2o_instance_has_checks() {
        let instance = Mh2oInstance {
            offset_exists_bitmap: 10,
            offset_vertex_data: 20,
            ..Default::default()
        };

        assert!(instance.has_exists_bitmap());
        assert!(instance.has_vertex_data());

        let empty_instance = Mh2oInstance::default();
        assert!(!empty_instance.has_exists_bitmap());
        assert!(!empty_instance.has_vertex_data());
    }

    #[test]
    fn test_mh2o_instance_round_trip() {
        let original = Mh2oInstance {
            liquid_type: 5,
            liquid_object_or_lvf: 1,
            min_height_level: 10.5,
            max_height_level: 15.75,
            x_offset: 3,
            y_offset: 2,
            width: 5,
            height: 6,
            offset_exists_bitmap: 100,
            offset_vertex_data: 200,
        };

        let mut buffer = Cursor::new(Vec::new());
        original.write_le(&mut buffer).unwrap();

        let data = buffer.into_inner();
        let mut cursor = Cursor::new(data);
        let parsed = Mh2oInstance::read_le(&mut cursor).unwrap();

        assert_eq!(original.liquid_type, parsed.liquid_type);
        assert_eq!(original.width, parsed.width);
        assert_eq!(original.height, parsed.height);
        assert_eq!(original.min_height_level, parsed.min_height_level);
    }
}
