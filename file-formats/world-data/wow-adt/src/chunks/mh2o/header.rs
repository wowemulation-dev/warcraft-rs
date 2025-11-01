//! MH2O header and instance structures for WotLK+ liquid data.
//!
//! MH2O replaces the legacy MCLQ chunk with a more flexible system supporting
//! multiple liquid layers, partial tile coverage, and various vertex formats.

use binrw::{BinRead, BinWrite};

/// MH2O header entry for one map chunk (1/256 of ADT, WotLK+).
///
/// The MH2O chunk contains 256 of these entries (16×16 grid), one per map chunk.
/// Each entry points to liquid instances and attributes for that chunk.
///
/// # Binary Layout
///
/// ```text
/// Offset | Size | Field              | Description
/// -------|------|--------------------|---------------------------------
/// 0x00   |  4   | offset_instances   | Offset to instance array
/// 0x04   |  4   | layer_count        | Number of liquid layers (0 = none)
/// 0x08   |  4   | offset_attributes  | Offset to attributes (optional)
/// ```
///
/// All offsets are relative to the start of MH2O chunk data.
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ❌ Not present
/// - **TBC (2.4.3)**: ❌ Not present
/// - **WotLK (3.3.5a)**: ✅ Introduced
/// - **Cataclysm (4.3.4)**: ✅ Present
/// - **MoP (5.4.8)**: ✅ Present
///
/// Reference: <https://wowdev.wiki/ADT/v18#MH2O_chunk>
#[derive(Debug, Clone, Copy, Default, BinRead, BinWrite)]
#[brw(little)]
pub struct Mh2oHeader {
    /// Offset to instance array (relative to MH2O data start)
    ///
    /// Points to an array of `layer_count` Mh2oInstance entries.
    /// If 0, no liquid data exists for this chunk.
    pub offset_instances: u32,

    /// Number of liquid layers (0 = no liquids)
    ///
    /// Most chunks have 0-1 layers, but multiple layers are possible
    /// (e.g., water surface above lava).
    pub layer_count: u32,

    /// Offset to attributes (relative to MH2O data start)
    ///
    /// Points to Mh2oAttributes structure. If 0, no attributes present.
    pub offset_attributes: u32,
}

impl Mh2oHeader {
    /// Grid size: 16×16 headers per ADT.
    pub const GRID_SIZE: usize = 16;

    /// Total header count per ADT.
    pub const TOTAL_COUNT: usize = Self::GRID_SIZE * Self::GRID_SIZE;

    /// Check if chunk has liquid data.
    pub fn has_liquid(&self) -> bool {
        self.layer_count > 0 && self.offset_instances != 0
    }

    /// Check if chunk has attributes.
    pub fn has_attributes(&self) -> bool {
        self.offset_attributes != 0
    }
}

/// MH2O attributes for visibility and gameplay (WotLK+).
///
/// Provides 8×8 bitmaps for fishable areas and fatigue (deep water) zones.
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ❌ Not present
/// - **TBC (2.4.3)**: ❌ Not present
/// - **WotLK (3.3.5a)**: ✅ Introduced
/// - **Cataclysm (4.3.4)**: ✅ Present
/// - **MoP (5.4.8)**: ✅ Present
///
/// # Binary Layout
///
/// ```text
/// Offset | Size | Field    | Description
/// -------|------|----------|----------------------------------
/// 0x00   |  8   | fishable | Fishable area bitmap (8×8 bits)
/// 0x08   |  8   | deep     | Fatigue area bitmap (8×8 bits)
/// ```
///
/// Reference: <https://wowdev.wiki/ADT/v18#mh2o_chunk_attributes>
#[derive(Debug, Clone, Copy, Default, BinRead, BinWrite)]
#[brw(little)]
pub struct Mh2oAttributes {
    /// Fishable area bitmap (8×8 = 64 bits)
    ///
    /// Bit set = area is fishable. Access: `(fishable >> (row * 8 + col)) & 1`
    pub fishable: u64,

    /// Deep water/fatigue area bitmap (8×8 = 64 bits)
    ///
    /// Bit set = area causes fatigue. Access: `(deep >> (row * 8 + col)) & 1`
    pub deep: u64,
}

impl Mh2oAttributes {
    /// Tile grid resolution (8×8 tiles).
    pub const TILE_SIZE: usize = 8;

    /// Check if tile is fishable.
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-7)
    /// * `y` - Row index (0-7)
    pub fn is_fishable(&self, x: usize, y: usize) -> bool {
        if x >= Self::TILE_SIZE || y >= Self::TILE_SIZE {
            return false;
        }
        let bit_index = y * Self::TILE_SIZE + x;
        (self.fishable >> bit_index) & 1 != 0
    }

    /// Check if tile is deep (causes fatigue).
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-7)
    /// * `y` - Row index (0-7)
    pub fn is_deep(&self, x: usize, y: usize) -> bool {
        if x >= Self::TILE_SIZE || y >= Self::TILE_SIZE {
            return false;
        }
        let bit_index = y * Self::TILE_SIZE + x;
        (self.deep >> bit_index) & 1 != 0
    }

    /// Count fishable tiles.
    pub fn fishable_count(&self) -> u32 {
        self.fishable.count_ones()
    }

    /// Count deep tiles.
    pub fn deep_count(&self) -> u32 {
        self.deep.count_ones()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_mh2o_header_size() {
        assert_eq!(std::mem::size_of::<Mh2oHeader>(), 12);
    }

    #[test]
    fn test_mh2o_header_constants() {
        assert_eq!(Mh2oHeader::GRID_SIZE, 16);
        assert_eq!(Mh2oHeader::TOTAL_COUNT, 256);
    }

    #[test]
    fn test_mh2o_header_parse() {
        let data = [
            0x10, 0x00, 0x00, 0x00, // offset_instances: 16
            0x02, 0x00, 0x00, 0x00, // layer_count: 2
            0x20, 0x00, 0x00, 0x00, // offset_attributes: 32
        ];

        let mut cursor = Cursor::new(&data);
        let header = Mh2oHeader::read_le(&mut cursor).unwrap();

        assert_eq!(header.offset_instances, 16);
        assert_eq!(header.layer_count, 2);
        assert_eq!(header.offset_attributes, 32);
        assert!(header.has_liquid());
        assert!(header.has_attributes());
    }

    #[test]
    fn test_mh2o_header_no_liquid() {
        let header = Mh2oHeader {
            offset_instances: 0,
            layer_count: 0,
            offset_attributes: 0,
        };

        assert!(!header.has_liquid());
        assert!(!header.has_attributes());
    }

    #[test]
    fn test_mh2o_header_round_trip() {
        let original = Mh2oHeader {
            offset_instances: 100,
            layer_count: 3,
            offset_attributes: 200,
        };

        let mut buffer = Cursor::new(Vec::new());
        original.write_le(&mut buffer).unwrap();

        let data = buffer.into_inner();
        let mut cursor = Cursor::new(data);
        let parsed = Mh2oHeader::read_le(&mut cursor).unwrap();

        assert_eq!(original.offset_instances, parsed.offset_instances);
        assert_eq!(original.layer_count, parsed.layer_count);
        assert_eq!(original.offset_attributes, parsed.offset_attributes);
    }

    #[test]
    fn test_mh2o_attributes_size() {
        assert_eq!(std::mem::size_of::<Mh2oAttributes>(), 16);
    }

    #[test]
    fn test_mh2o_attributes_parse() {
        let data = [
            0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // fishable: first 8 bits set
            0x00, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // deep: second 8 bits set
        ];

        let mut cursor = Cursor::new(&data);
        let attrs = Mh2oAttributes::read_le(&mut cursor).unwrap();

        assert_eq!(attrs.fishable, 0xFF);
        assert_eq!(attrs.deep, 0xFF00);
    }

    #[test]
    fn test_mh2o_attributes_fishable() {
        let attrs = Mh2oAttributes {
            fishable: 0xAAAA, // Alternating pattern in first two rows (bits 0-15)
            deep: 0,
        };

        // Row 0 (bits 0-7): 0xAA = 0b10101010
        assert!(!attrs.is_fishable(0, 0)); // Bit 0 = 0
        assert!(attrs.is_fishable(1, 0)); // Bit 1 = 1
        assert!(!attrs.is_fishable(2, 0)); // Bit 2 = 0
        assert!(attrs.is_fishable(3, 0)); // Bit 3 = 1

        // Row 1 (bits 8-15): 0xAA = 0b10101010
        assert!(!attrs.is_fishable(0, 1)); // Bit 8 = 0
        assert!(attrs.is_fishable(1, 1)); // Bit 9 = 1
    }

    #[test]
    fn test_mh2o_attributes_deep() {
        let attrs = Mh2oAttributes {
            fishable: 0,
            deep: 0x00_00_00_FF_00_00_00_00, // Row 4 all deep (bits 32-39)
        };

        // Row 4 (bits 32-39) should all be deep
        for x in 0..8 {
            assert!(attrs.is_deep(x, 4), "Failed for x={}", x);
        }

        // Other rows should not be deep
        for x in 0..8 {
            assert!(!attrs.is_deep(x, 0), "Row 0, x={} should not be deep", x);
            assert!(!attrs.is_deep(x, 1), "Row 1, x={} should not be deep", x);
        }
    }

    #[test]
    fn test_mh2o_attributes_bounds_checking() {
        let attrs = Mh2oAttributes {
            fishable: u64::MAX,
            deep: u64::MAX,
        };

        // Out of bounds should return false
        assert!(!attrs.is_fishable(8, 0));
        assert!(!attrs.is_fishable(0, 8));
        assert!(!attrs.is_deep(8, 0));
        assert!(!attrs.is_deep(0, 8));
    }

    #[test]
    fn test_mh2o_attributes_count() {
        let attrs = Mh2oAttributes {
            fishable: 0xFF, // 8 bits set
            deep: 0xFFFF,   // 16 bits set
        };

        assert_eq!(attrs.fishable_count(), 8);
        assert_eq!(attrs.deep_count(), 16);
    }

    #[test]
    fn test_mh2o_attributes_round_trip() {
        let original = Mh2oAttributes {
            fishable: 0x123456789ABCDEF0,
            deep: 0xFEDCBA9876543210,
        };

        let mut buffer = Cursor::new(Vec::new());
        original.write_le(&mut buffer).unwrap();

        let data = buffer.into_inner();
        let mut cursor = Cursor::new(data);
        let parsed = Mh2oAttributes::read_le(&mut cursor).unwrap();

        assert_eq!(original.fishable, parsed.fishable);
        assert_eq!(original.deep, parsed.deep);
    }
}
