//! MCSH chunk - Shadow map (64×64 1-bit map, Vanilla+).
//!
//! Contains pre-baked shadow data for terrain. Each bit represents one texel
//! in the 64×64 shadow texture.
//!
//! Format:
//! - 64 rows × 8 bytes per row = 512 bytes total
//! - LSB-first bit ordering within bytes
//! - Bit set = shadowed texel
//!
//! This chunk is optional and only present if McnkHeader.flags.has_mcsh() is true.
//!
//! Reference: <https://wowdev.wiki/ADT/v18#MCSH_sub-chunk>

use binrw::{BinRead, BinWrite};

/// MCSH chunk - Shadow map (64×64 1-bit map, Vanilla+).
///
/// Contains pre-baked shadow data for terrain. Each bit represents one texel
/// in the 64×64 shadow texture.
///
/// # Format
///
/// - 64 rows × 8 bytes per row = 512 bytes total
/// - LSB-first bit ordering within bytes
/// - Bit set = shadowed texel
///
/// This chunk is optional and only present if McnkFlags.has_mcsh() is true.
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ✅ Introduced
/// - **TBC (2.4.3)**: ✅ Present
/// - **WotLK (3.3.5a)**: ✅ Present
/// - **Cataclysm (4.3.4)**: ✅ Present
/// - **MoP (5.4.8)**: ✅ Present
///
/// # Binary Layout
///
/// ```text
/// Offset | Size | Field       | Description
/// -------|------|-------------|----------------------------------
/// 0x000  | 512  | shadow_map  | 64×64 1-bit shadow map
/// ```
///
/// Reference: <https://wowdev.wiki/ADT/v18#MCSH_sub-chunk>
#[derive(Debug, Clone, BinRead, BinWrite)]
#[brw(little)]
pub struct McshChunk {
    /// Shadow map data (512 bytes = 64 rows * 8 bytes/row)
    ///
    /// Each row is 8 bytes (64 bits) representing one row of the 64×64 map.
    /// Bit ordering: LSB-first within each byte.
    #[br(count = 512)]
    pub shadow_map: Vec<u8>,
}

impl Default for McshChunk {
    fn default() -> Self {
        Self {
            shadow_map: vec![0; 512], // All unshadowed
        }
    }
}

impl McshChunk {
    /// Shadow map resolution (64×64 texels).
    pub const RESOLUTION: usize = 64;

    /// Size in bytes (512 = 64 * 8).
    pub const SIZE_BYTES: usize = 512;

    /// Get shadow value at texel position.
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-63)
    /// * `y` - Row index (0-63)
    ///
    /// # Returns
    ///
    /// `true` if texel is shadowed, `false` if lit or indices out of bounds
    pub fn is_shadowed(&self, x: usize, y: usize) -> bool {
        if x >= Self::RESOLUTION || y >= Self::RESOLUTION {
            return false;
        }

        let byte_index = y * 8 + (x / 8);
        let bit_index = x % 8;

        if let Some(&byte) = self.shadow_map.get(byte_index) {
            (byte >> bit_index) & 1 != 0
        } else {
            false
        }
    }

    /// Set shadow value at texel position.
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-63)
    /// * `y` - Row index (0-63)
    /// * `shadowed` - `true` to mark texel as shadowed
    pub fn set_shadow(&mut self, x: usize, y: usize, shadowed: bool) {
        if x >= Self::RESOLUTION || y >= Self::RESOLUTION {
            return;
        }

        let byte_index = y * 8 + (x / 8);
        let bit_index = x % 8;

        if let Some(byte) = self.shadow_map.get_mut(byte_index) {
            if shadowed {
                *byte |= 1 << bit_index;
            } else {
                *byte &= !(1 << bit_index);
            }
        }
    }

    /// Count number of shadowed texels.
    pub fn shadowed_count(&self) -> usize {
        self.shadow_map
            .iter()
            .map(|&byte| byte.count_ones() as usize)
            .sum()
    }

    /// Get percentage of shadowed texels (0.0 to 1.0).
    pub fn shadow_ratio(&self) -> f32 {
        let total_texels = Self::RESOLUTION * Self::RESOLUTION;
        self.shadowed_count() as f32 / total_texels as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_mcsh_chunk_size() {
        let chunk = McshChunk::default();
        assert_eq!(chunk.shadow_map.len(), 512);
        assert_eq!(McshChunk::SIZE_BYTES, 512);
        assert_eq!(McshChunk::RESOLUTION, 64);
    }

    #[test]
    fn test_mcsh_parse_512_bytes() {
        let data = vec![0u8; 512];
        let mut cursor = Cursor::new(data);
        let chunk = McshChunk::read_le(&mut cursor).unwrap();

        assert_eq!(chunk.shadow_map.len(), 512);
        assert_eq!(chunk.shadowed_count(), 0);
    }

    #[test]
    fn test_mcsh_default_all_unshadowed() {
        let chunk = McshChunk::default();

        for y in 0..64 {
            for x in 0..64 {
                assert!(!chunk.is_shadowed(x, y));
            }
        }

        assert_eq!(chunk.shadowed_count(), 0);
        assert_eq!(chunk.shadow_ratio(), 0.0);
    }

    #[test]
    fn test_mcsh_shadow_bit_access() {
        let mut chunk = McshChunk::default();

        // Set some shadow bits
        chunk.set_shadow(0, 0, true);
        chunk.set_shadow(7, 0, true);
        chunk.set_shadow(31, 15, true);
        chunk.set_shadow(63, 63, true);

        assert!(chunk.is_shadowed(0, 0));
        assert!(chunk.is_shadowed(7, 0));
        assert!(chunk.is_shadowed(31, 15));
        assert!(chunk.is_shadowed(63, 63));

        assert!(!chunk.is_shadowed(1, 0));
        assert!(!chunk.is_shadowed(0, 1));

        // Clear a bit
        chunk.set_shadow(0, 0, false);
        assert!(!chunk.is_shadowed(0, 0));
    }

    #[test]
    fn test_mcsh_lsb_first_bit_ordering() {
        let mut chunk = McshChunk::default();

        // Set first 8 bits in row 0 (byte 0)
        for x in 0..8 {
            chunk.set_shadow(x, 0, true);
        }

        // Verify byte 0 is 0xFF (all bits set, LSB-first)
        assert_eq!(chunk.shadow_map[0], 0xFF);

        // Set alternating bits in row 1 (byte 8)
        chunk.set_shadow(0, 1, true); // bit 0
        chunk.set_shadow(2, 1, true); // bit 2
        chunk.set_shadow(4, 1, true); // bit 4
        chunk.set_shadow(6, 1, true); // bit 6

        // Verify byte 8 is 0b01010101 = 0x55
        assert_eq!(chunk.shadow_map[8], 0b0101_0101);

        // Verify individual bit reads
        assert!(chunk.is_shadowed(0, 1));
        assert!(!chunk.is_shadowed(1, 1));
        assert!(chunk.is_shadowed(2, 1));
        assert!(!chunk.is_shadowed(3, 1));
    }

    #[test]
    fn test_mcsh_bounds_checking() {
        let mut chunk = McshChunk::default();

        // Out of bounds get
        assert!(!chunk.is_shadowed(64, 0));
        assert!(!chunk.is_shadowed(0, 64));
        assert!(!chunk.is_shadowed(100, 100));

        // Out of bounds set (should not panic)
        chunk.set_shadow(64, 0, true);
        chunk.set_shadow(0, 64, true);
        chunk.set_shadow(100, 100, true);

        // Verify no state changed
        assert_eq!(chunk.shadowed_count(), 0);
    }

    #[test]
    fn test_mcsh_shadow_count() {
        let mut chunk = McshChunk::default();

        assert_eq!(chunk.shadowed_count(), 0);

        // Set 10 random shadow bits
        chunk.set_shadow(0, 0, true);
        chunk.set_shadow(10, 5, true);
        chunk.set_shadow(20, 10, true);
        chunk.set_shadow(30, 15, true);
        chunk.set_shadow(40, 20, true);
        chunk.set_shadow(50, 25, true);
        chunk.set_shadow(15, 30, true);
        chunk.set_shadow(25, 35, true);
        chunk.set_shadow(35, 40, true);
        chunk.set_shadow(45, 45, true);

        assert_eq!(chunk.shadowed_count(), 10);
    }

    #[test]
    fn test_mcsh_shadow_ratio() {
        let mut chunk = McshChunk::default();

        // No shadows
        assert_eq!(chunk.shadow_ratio(), 0.0);

        // Set all bits in first row (64 bits)
        for x in 0..64 {
            chunk.set_shadow(x, 0, true);
        }

        let expected_ratio = 64.0 / 4096.0; // 64 out of 64×64
        assert!((chunk.shadow_ratio() - expected_ratio).abs() < 0.0001);

        // Set all bits
        for y in 0..64 {
            for x in 0..64 {
                chunk.set_shadow(x, y, true);
            }
        }

        assert_eq!(chunk.shadow_ratio(), 1.0);
        assert_eq!(chunk.shadowed_count(), 4096);
    }

    #[test]
    fn test_mcsh_round_trip() {
        let mut original = McshChunk::default();

        // Create a pattern: diagonal shadow
        for i in 0..64 {
            original.set_shadow(i, i, true);
        }

        assert_eq!(original.shadowed_count(), 64);

        // Serialize
        let mut buffer = Cursor::new(Vec::new());
        original.write_le(&mut buffer).unwrap();

        let data = buffer.into_inner();
        assert_eq!(data.len(), 512);

        // Deserialize
        let mut cursor = Cursor::new(data);
        let parsed = McshChunk::read_le(&mut cursor).unwrap();

        // Verify
        assert_eq!(parsed.shadowed_count(), 64);
        for i in 0..64 {
            assert!(parsed.is_shadowed(i, i));
        }
    }

    #[test]
    fn test_mcsh_byte_indexing() {
        let mut chunk = McshChunk::default();

        // Row 0, column 0-7 -> byte 0
        chunk.set_shadow(0, 0, true);
        assert_eq!(chunk.shadow_map[0], 0b0000_0001);

        chunk.set_shadow(7, 0, true);
        assert_eq!(chunk.shadow_map[0], 0b1000_0001);

        // Row 0, column 8-15 -> byte 1
        chunk.set_shadow(8, 0, true);
        assert_eq!(chunk.shadow_map[1], 0b0000_0001);

        // Row 1, column 0-7 -> byte 8
        chunk.set_shadow(0, 1, true);
        assert_eq!(chunk.shadow_map[8], 0b0000_0001);

        // Row 63, column 56-63 -> byte 511
        chunk.set_shadow(63, 63, true);
        assert_eq!(chunk.shadow_map[511], 0b1000_0000);
    }

    #[test]
    fn test_mcsh_all_bytes_used() {
        let mut chunk = McshChunk::default();

        // Set first bit of every byte
        for byte_index in 0..512 {
            let y = byte_index / 8;
            let x = (byte_index % 8) * 8;
            chunk.set_shadow(x, y, true);
        }

        // Verify all 512 bytes have bit 0 set
        for byte_index in 0..512 {
            assert_eq!(chunk.shadow_map[byte_index] & 1, 1);
        }

        assert_eq!(chunk.shadowed_count(), 512);
    }

    #[test]
    fn test_mcsh_full_coverage() {
        let mut chunk = McshChunk::default();

        // Set every texel
        for y in 0..64 {
            for x in 0..64 {
                chunk.set_shadow(x, y, true);
            }
        }

        // Verify all bytes are 0xFF
        for &byte in &chunk.shadow_map {
            assert_eq!(byte, 0xFF);
        }

        assert_eq!(chunk.shadowed_count(), 4096);
        assert_eq!(chunk.shadow_ratio(), 1.0);
    }
}
