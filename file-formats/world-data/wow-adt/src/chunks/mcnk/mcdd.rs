use binrw::{BinRead, BinWrite};

/// MCDD chunk - Doodad disable bitmap (WoD+)
///
/// Contains an 8×8 bitmap indicating where detail doodads should be disabled
/// within a terrain chunk. Each bit represents one cell in the chunk grid.
///
/// ## Structure
///
/// ```text
/// struct {
///     uint8_t disable[8][8];  // 64 bytes, 8×8 grid
/// } MCDD;
/// ```
///
/// ## Grid Layout
///
/// The bitmap covers the terrain chunk with 8×8 resolution:
/// - Each byte represents one row (8 cells)
/// - Each bit within a byte represents one column
/// - Bit set to 1 = detail doodads disabled for that cell
/// - Bit set to 0 = detail doodads enabled for that cell
///
/// ## High-Resolution Mode
///
/// wowdev.wiki mentions a potential high-res 16×16 mode (256 bytes),
/// but this is not used in live clients. Only 8×8 (64 bytes) is supported.
///
/// ## Split File Architecture
///
/// MCDD chunks appear in root ADT files (not _tex or _obj):
///
/// **Location**: Root ADT files only
/// **Size**: Fixed 64 bytes (8×8 grid)
/// **Header Offset**: None (discovered via chunk parsing)
///
/// ## Use Cases
///
/// - Suppressing grass/flowers near buildings
/// - Clearing detail doodads from paths
/// - Preventing clutter in specific areas
/// - Performance optimization by disabling detail in dense regions
///
/// ## References
///
/// - **wowdev.wiki**: <https://wowdev.wiki/ADT/v18#MCDD_sub-chunk>
/// - **WoWFormatLib**: Not found in ADT.Struct.cs (implementation gap)
/// - **noggit-red**: Not implemented (WoD+ not supported)
/// - **wow.export**: Not found in ADTLoader.js (implementation gap)
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ❌ Not present
/// - **TBC (2.4.3)**: ❌ Not present
/// - **WotLK (3.3.5a)**: ❌ Not present
/// - **Cataclysm (4.3.4)**: ❌ Not present
/// - **MoP (5.4.8)**: ❌ Not present
/// - **WoD (6.0.1)**: ✅ Introduced
///
/// ## Deviations
///
/// None - straightforward 64-byte bitmap matching wowdev.wiki specification.
///
/// Reference: <https://wowdev.wiki/ADT/v18#MCDD_sub-chunk>
#[derive(Debug, Clone, Copy, BinRead, BinWrite)]
#[brw(little)]
pub struct McddChunk {
    /// Doodad disable bitmap (8×8 grid, 64 bytes)
    ///
    /// Each byte represents one row, each bit represents one column.
    /// 1 = doodads disabled, 0 = doodads enabled.
    pub disable: [u8; 64],
}

impl Default for McddChunk {
    fn default() -> Self {
        Self { disable: [0; 64] }
    }
}

impl McddChunk {
    /// Fixed size in bytes.
    pub const SIZE_BYTES: usize = 64;

    /// Grid resolution (8×8).
    pub const GRID_SIZE: usize = 8;

    /// Check if doodads are disabled at a specific grid position.
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-7)
    /// * `y` - Row index (0-7)
    ///
    /// # Returns
    ///
    /// `true` if doodads are disabled at this position
    #[must_use]
    pub fn is_disabled(&self, x: usize, y: usize) -> bool {
        if x >= Self::GRID_SIZE || y >= Self::GRID_SIZE {
            return false;
        }

        let byte_index = y;
        let bit_index = x;
        let byte = self.disable[byte_index];

        (byte & (1 << bit_index)) != 0
    }

    /// Set whether doodads are disabled at a specific grid position.
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-7)
    /// * `y` - Row index (0-7)
    /// * `disabled` - true to disable doodads, false to enable
    pub fn set_disabled(&mut self, x: usize, y: usize, disabled: bool) {
        if x >= Self::GRID_SIZE || y >= Self::GRID_SIZE {
            return;
        }

        let byte_index = y;
        let bit_index = x;

        if disabled {
            self.disable[byte_index] |= 1 << bit_index;
        } else {
            self.disable[byte_index] &= !(1 << bit_index);
        }
    }

    /// Count total number of cells with disabled doodads.
    #[must_use]
    pub fn disabled_count(&self) -> usize {
        self.disable
            .iter()
            .map(|&byte| byte.count_ones() as usize)
            .sum()
    }

    /// Check if all doodads are enabled (no cells disabled).
    #[must_use]
    pub fn all_enabled(&self) -> bool {
        self.disable.iter().all(|&byte| byte == 0)
    }

    /// Check if all doodads are disabled (all cells disabled).
    #[must_use]
    pub fn all_disabled(&self) -> bool {
        self.disable.iter().all(|&byte| byte == 0xFF)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::{BinReaderExt, BinWriterExt};
    use std::io::Cursor;

    #[test]
    fn test_mcdd_size() {
        assert_eq!(McddChunk::SIZE_BYTES, 64);
        assert_eq!(McddChunk::GRID_SIZE, 8);
    }

    #[test]
    fn test_mcdd_parse() {
        let mut data = vec![0u8; 64];
        data[0] = 0xFF; // Row 0: all disabled
        data[1] = 0x0F; // Row 1: columns 0-3 disabled
        data[7] = 0x80; // Row 7: column 7 disabled

        let mut cursor = Cursor::new(data);
        let mcdd: McddChunk = cursor.read_le().unwrap();

        assert_eq!(mcdd.disable[0], 0xFF);
        assert_eq!(mcdd.disable[1], 0x0F);
        assert_eq!(mcdd.disable[7], 0x80);
    }

    #[test]
    fn test_mcdd_round_trip() {
        let mut original = McddChunk::default();
        original.disable[0] = 0xAA;
        original.disable[7] = 0x55;

        let mut buffer = Cursor::new(Vec::new());
        buffer.write_le(&original).unwrap();
        assert_eq!(buffer.position(), 64);

        let data = buffer.into_inner();
        let mut cursor = Cursor::new(data);
        let parsed: McddChunk = cursor.read_le().unwrap();

        assert_eq!(original.disable, parsed.disable);
    }

    #[test]
    fn test_mcdd_is_disabled() {
        let mut mcdd = McddChunk::default();
        mcdd.disable[0] = 0b0000_0001; // Row 0, col 0 disabled
        mcdd.disable[1] = 0b1000_0000; // Row 1, col 7 disabled
        mcdd.disable[7] = 0b0101_0101; // Row 7, cols 0,2,4,6 disabled

        // Row 0
        assert!(mcdd.is_disabled(0, 0));
        assert!(!mcdd.is_disabled(1, 0));

        // Row 1
        assert!(mcdd.is_disabled(7, 1));
        assert!(!mcdd.is_disabled(0, 1));

        // Row 7
        assert!(mcdd.is_disabled(0, 7));
        assert!(!mcdd.is_disabled(1, 7));
        assert!(mcdd.is_disabled(2, 7));
        assert!(!mcdd.is_disabled(3, 7));

        // Out of bounds
        assert!(!mcdd.is_disabled(8, 0));
        assert!(!mcdd.is_disabled(0, 8));
    }

    #[test]
    fn test_mcdd_set_disabled() {
        let mut mcdd = McddChunk::default();

        // Enable/disable individual cells
        mcdd.set_disabled(0, 0, true);
        assert!(mcdd.is_disabled(0, 0));

        mcdd.set_disabled(7, 7, true);
        assert!(mcdd.is_disabled(7, 7));

        mcdd.set_disabled(0, 0, false);
        assert!(!mcdd.is_disabled(0, 0));

        // Out of bounds (should not panic)
        mcdd.set_disabled(8, 0, true);
        mcdd.set_disabled(0, 8, true);
    }

    #[test]
    fn test_mcdd_disabled_count() {
        let mut mcdd = McddChunk::default();
        assert_eq!(mcdd.disabled_count(), 0);

        mcdd.disable[0] = 0xFF; // 8 disabled
        assert_eq!(mcdd.disabled_count(), 8);

        mcdd.disable[1] = 0x0F; // 4 more disabled
        assert_eq!(mcdd.disabled_count(), 12);

        mcdd.disable[7] = 0x80; // 1 more disabled
        assert_eq!(mcdd.disabled_count(), 13);
    }

    #[test]
    fn test_mcdd_all_enabled() {
        let mcdd1 = McddChunk::default();
        assert!(mcdd1.all_enabled());

        let mut mcdd2 = McddChunk::default();
        mcdd2.disable[0] = 0x01;
        assert!(!mcdd2.all_enabled());
    }

    #[test]
    fn test_mcdd_all_disabled() {
        let mut mcdd1 = McddChunk::default();
        for i in 0..64 {
            mcdd1.disable[i] = 0xFF;
        }
        assert!(mcdd1.all_disabled());

        let mcdd2 = McddChunk::default();
        assert!(!mcdd2.all_disabled());
    }

    #[test]
    fn test_mcdd_default() {
        let mcdd = McddChunk::default();
        assert_eq!(mcdd.disable, [0u8; 64]);
        assert_eq!(mcdd.disabled_count(), 0);
        assert!(mcdd.all_enabled());
    }

    #[test]
    fn test_mcdd_pattern() {
        let mut mcdd = McddChunk::default();

        // Create checkerboard pattern
        for y in 0..8 {
            for x in 0..8 {
                if (x + y) % 2 == 0 {
                    mcdd.set_disabled(x, y, true);
                }
            }
        }

        // Verify pattern
        assert_eq!(mcdd.disabled_count(), 32);
        for y in 0..8 {
            for x in 0..8 {
                if (x + y) % 2 == 0 {
                    assert!(mcdd.is_disabled(x, y));
                } else {
                    assert!(!mcdd.is_disabled(x, y));
                }
            }
        }
    }
}
