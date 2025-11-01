use binrw::{BinRead, BinWrite};
use std::io::{Read, Seek};

/// Alpha map compression format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaFormat {
    /// Uncompressed 4096 bytes (8-bit per texel)
    Uncompressed4096,

    /// Uncompressed 2048 bytes (4-bit per texel)
    Uncompressed2048,

    /// RLE compressed (variable size, decompresses to 4096 bytes)
    Compressed,
}

/// Single texture layer alpha map.
///
/// Alpha maps define texture blending for terrain layers. Format depends on
/// WDT flags and MCLY layer flags.
#[derive(Debug, Clone)]
pub struct AlphaMap {
    /// Raw alpha data (size varies by format)
    pub data: Vec<u8>,

    /// Compression format
    pub format: AlphaFormat,
}

impl AlphaMap {
    /// Alpha map resolution (64×64 texels).
    pub const RESOLUTION: usize = 64;

    /// Uncompressed size (8-bit format).
    pub const SIZE_UNCOMPRESSED_8BIT: usize = 4096;

    /// Uncompressed size (4-bit format).
    pub const SIZE_UNCOMPRESSED_4BIT: usize = 2048;

    /// Create from raw data with format.
    pub fn new(data: Vec<u8>, format: AlphaFormat) -> Self {
        Self { data, format }
    }

    /// Decompress alpha map to 64×64 8-bit values.
    ///
    /// Returns 4096 bytes regardless of input format.
    pub fn decompress(&self) -> Result<Vec<u8>, String> {
        match self.format {
            AlphaFormat::Uncompressed4096 => {
                if self.data.len() != Self::SIZE_UNCOMPRESSED_8BIT {
                    return Err(format!(
                        "Invalid uncompressed data size: {}",
                        self.data.len()
                    ));
                }
                Ok(self.data.clone())
            }

            AlphaFormat::Uncompressed2048 => {
                if self.data.len() != Self::SIZE_UNCOMPRESSED_4BIT {
                    return Err(format!("Invalid 4-bit data size: {}", self.data.len()));
                }

                // Expand 4-bit to 8-bit (LSB first)
                let mut output = Vec::with_capacity(Self::SIZE_UNCOMPRESSED_8BIT);
                for &byte in &self.data {
                    let low = byte & 0x0F;
                    let high = (byte >> 4) & 0x0F;
                    // Normalize: (value & 0xF) | (value << 4)
                    output.push(low | (low << 4));
                    output.push(high | (high << 4));
                }
                Ok(output)
            }

            AlphaFormat::Compressed => Self::decompress_rle(&self.data),
        }
    }

    /// Decompress RLE format.
    ///
    /// Control byte: [mode:1bit | count:7bits]
    /// - Mode 0: Copy next `count` bytes
    /// - Mode 1: Fill with next byte `count` times
    fn decompress_rle(data: &[u8]) -> Result<Vec<u8>, String> {
        let mut output = Vec::with_capacity(Self::SIZE_UNCOMPRESSED_8BIT);
        let mut i = 0;

        while i < data.len() && output.len() < Self::SIZE_UNCOMPRESSED_8BIT {
            let control = data[i];
            i += 1;

            let mode = (control & 0x80) != 0; // Bit 7
            let count = (control & 0x7F) as usize; // Bits 0-6

            if mode {
                // Mode 1: Fill
                if i >= data.len() {
                    return Err("RLE: Unexpected EOF in fill mode".to_string());
                }
                let fill_byte = data[i];
                i += 1;

                for _ in 0..count {
                    if output.len() >= Self::SIZE_UNCOMPRESSED_8BIT {
                        break;
                    }
                    output.push(fill_byte);
                }
            } else {
                // Mode 0: Copy
                for _ in 0..count {
                    if i >= data.len() {
                        return Err("RLE: Unexpected EOF in copy mode".to_string());
                    }
                    if output.len() >= Self::SIZE_UNCOMPRESSED_8BIT {
                        break;
                    }
                    output.push(data[i]);
                    i += 1;
                }
            }
        }

        if output.len() != Self::SIZE_UNCOMPRESSED_8BIT {
            return Err(format!(
                "RLE: Output size mismatch: {} (expected 4096)",
                output.len()
            ));
        }

        Ok(output)
    }

    /// Get alpha value at texel position (after decompression).
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-63)
    /// * `y` - Row index (0-63)
    pub fn get_alpha(&self, x: usize, y: usize) -> Result<u8, String> {
        if x >= Self::RESOLUTION || y >= Self::RESOLUTION {
            return Err("Coordinates out of bounds".to_string());
        }

        let decompressed = self.decompress()?;
        let index = y * Self::RESOLUTION + x;
        Ok(decompressed[index])
    }

    /// Compress 64×64 8-bit alpha data to RLE format.
    ///
    /// Takes 4096 bytes of uncompressed data and produces RLE-compressed output.
    /// Uses run-length encoding with two modes:
    /// - Fill mode: Repeated byte values
    /// - Copy mode: Unique byte sequences
    ///
    /// # Arguments
    ///
    /// * `data` - Uncompressed 4096-byte alpha map (64×64 8-bit values)
    ///
    /// # Returns
    ///
    /// RLE-compressed data or error if input size invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_adt::chunks::mcnk::mcal::AlphaMap;
    ///
    /// // Create 64×64 alpha map (4096 bytes)
    /// let data = vec![0u8; 4096];
    /// let compressed = AlphaMap::compress_rle(&data).unwrap();
    ///
    /// // Verify round-trip
    /// let alpha_map = AlphaMap::new(compressed, wow_adt::chunks::mcnk::mcal::AlphaFormat::Compressed);
    /// let decompressed = alpha_map.decompress().unwrap();
    /// assert_eq!(decompressed, data);
    /// ```
    pub fn compress_rle(data: &[u8]) -> Result<Vec<u8>, String> {
        if data.len() != Self::SIZE_UNCOMPRESSED_8BIT {
            return Err(format!(
                "Invalid input size for RLE compression: {} (expected 4096)",
                data.len()
            ));
        }

        let mut output = Vec::with_capacity(data.len()); // Worst case: no compression
        let mut i = 0;

        while i < data.len() {
            // Try to find a run of identical bytes (fill mode)
            let fill_byte = data[i];
            let mut run_length = 1;

            while i + run_length < data.len()
                && data[i + run_length] == fill_byte
                && run_length < 127
            {
                run_length += 1;
            }

            // Use fill mode if run is 3+ bytes (saves space: 2 bytes vs 1+N bytes)
            if run_length >= 3 {
                // Fill mode: control byte with bit 7 set + fill value
                output.push(0x80 | (run_length as u8));
                output.push(fill_byte);
                i += run_length;
                continue;
            }

            // Find sequence of unique bytes (copy mode)
            let copy_start = i;
            let mut copy_length = 0;

            while i < data.len() && copy_length < 127 {
                // Check if we're starting a new run of 3+ identical bytes
                let mut lookahead_run = 1;
                while i + lookahead_run < data.len()
                    && data[i + lookahead_run] == data[i]
                    && lookahead_run < 3
                {
                    lookahead_run += 1;
                }

                // If we found a run of 3+, stop copying and handle it as fill mode next iteration
                if lookahead_run >= 3 {
                    break;
                }

                copy_length += 1;
                i += 1;
            }

            if copy_length > 0 {
                // Copy mode: control byte + raw bytes
                output.push(copy_length as u8);
                output.extend_from_slice(&data[copy_start..copy_start + copy_length]);
            }
        }

        Ok(output)
    }

    /// Create compressed AlphaMap from uncompressed 4096-byte data.
    ///
    /// Automatically compresses using RLE algorithm.
    ///
    /// # Arguments
    ///
    /// * `data` - Uncompressed 4096-byte alpha data
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_adt::chunks::mcnk::mcal::AlphaMap;
    ///
    /// let uncompressed = vec![0u8; 4096];
    /// let alpha_map = AlphaMap::from_uncompressed(&uncompressed).unwrap();
    /// assert_eq!(alpha_map.format, wow_adt::chunks::mcnk::mcal::AlphaFormat::Compressed);
    /// ```
    pub fn from_uncompressed(data: &[u8]) -> Result<Self, String> {
        let compressed = Self::compress_rle(data)?;
        Ok(Self::new(compressed, AlphaFormat::Compressed))
    }

    /// Create AlphaMap with optimal format from uncompressed 4096-byte data.
    ///
    /// Automatically chooses the most space-efficient format:
    /// 1. RLE compressed (if smaller than uncompressed)
    /// 2. Uncompressed 2048 bytes (4-bit, if all values are 4-bit safe)
    /// 3. Uncompressed 4096 bytes (fallback)
    ///
    /// # Arguments
    ///
    /// * `data` - Uncompressed 4096-byte alpha data
    ///
    /// # Returns
    ///
    /// AlphaMap with optimal format and potentially compressed data
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_adt::chunks::mcnk::mcal::AlphaMap;
    ///
    /// // All zeros compresses very well
    /// let data = vec![0u8; 4096];
    /// let alpha_map = AlphaMap::with_optimal_format(&data).unwrap();
    /// // Will use Compressed format due to excellent compression ratio
    ///
    /// // Gradient data may not compress well
    /// let gradient: Vec<u8> = (0..4096).map(|i| ((i * 256) / 4096) as u8).collect();
    /// let alpha_map = AlphaMap::with_optimal_format(&gradient).unwrap();
    /// // May use Uncompressed4096 if compression doesn't help
    /// ```
    pub fn with_optimal_format(data: &[u8]) -> Result<Self, String> {
        if data.len() != Self::SIZE_UNCOMPRESSED_8BIT {
            return Err(format!(
                "Invalid input size for optimal format selection: {} (expected 4096)",
                data.len()
            ));
        }

        // Try RLE compression first
        let compressed = Self::compress_rle(data)?;

        // Check if all values are 4-bit safe (value == (value >> 4) | (value << 4))
        // This means the value can be represented with 4 bits without loss
        let can_use_4bit = data.iter().all(|&v| {
            let nibble = v >> 4;
            let expanded = nibble | (nibble << 4);
            v == expanded
        });

        // Size comparison: choose smallest format
        let compressed_size = compressed.len();
        let format_4bit_size = Self::SIZE_UNCOMPRESSED_4BIT; // 2048
        let _format_8bit_size = Self::SIZE_UNCOMPRESSED_8BIT; // 4096

        // Priority: compressed < 4-bit < 8-bit
        if compressed_size < format_4bit_size {
            // RLE compression is smallest
            Ok(Self::new(compressed, AlphaFormat::Compressed))
        } else if can_use_4bit {
            // 4-bit format is smaller than 8-bit and valid
            let mut compressed_4bit = Vec::with_capacity(format_4bit_size);

            // Pack pairs of 8-bit values into 4-bit format
            for chunk in data.chunks(2) {
                let low = chunk[0] >> 4;
                let high = chunk.get(1).map(|&v| v >> 4).unwrap_or(0);
                compressed_4bit.push((high << 4) | low);
            }

            Ok(Self::new(compressed_4bit, AlphaFormat::Uncompressed2048))
        } else {
            // Use uncompressed 8-bit format (no compression benefit)
            Ok(Self::new(data.to_vec(), AlphaFormat::Uncompressed4096))
        }
    }
}

/// MCAL chunk - Alpha maps for texture blending (Vanilla+)
///
/// Contains alpha maps for texture layers defined in MCLY. Each layer (except
/// the base layer 0) has an alpha map defining blend weights.
///
/// Alpha maps are stored consecutively with offsets specified in MCLY.offset_in_mcal.
/// Format varies based on WDT flags and MCLY layer flags.
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ✅ Introduced
/// - **TBC (2.4.3)**: ✅ Present
/// - **WotLK (3.3.5a)**: ✅ Present
/// - **Cataclysm (4.3.4)**: ✅ Present
/// - **MoP (5.4.8)**: ✅ Present
///
/// Reference: <https://wowdev.wiki/ADT/v18#MCAL_sub-chunk>
#[derive(Debug, Clone, Default)]
pub struct McalChunk {
    /// Raw alpha map data (contains all layer alpha maps)
    pub data: Vec<u8>,
}

impl McalChunk {
    /// Create from raw data.
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Extract alpha map for a specific layer.
    ///
    /// # Arguments
    ///
    /// * `offset` - Byte offset from MCLY.offset_in_mcal
    /// * `size` - Alpha map size in bytes
    /// * `format` - Compression format
    ///
    /// # Returns
    ///
    /// Extracted AlphaMap or error if bounds invalid
    pub fn get_layer_alpha(
        &self,
        offset: usize,
        size: usize,
        format: AlphaFormat,
    ) -> Result<AlphaMap, String> {
        if offset + size > self.data.len() {
            return Err(format!(
                "Alpha map out of bounds: offset={}, size={}, data_len={}",
                offset,
                size,
                self.data.len()
            ));
        }

        let layer_data = self.data[offset..offset + size].to_vec();
        Ok(AlphaMap::new(layer_data, format))
    }

    /// Get total data size.
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

impl BinRead for McalChunk {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Ok(Self { data })
    }
}

impl BinWrite for McalChunk {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + Seek>(
        &self,
        writer: &mut W,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        writer.write_all(&self.data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn uncompressed_4096_format() {
        let data: Vec<u8> = (0..4096).map(|i| (i % 256) as u8).collect();
        let alpha_map = AlphaMap::new(data.clone(), AlphaFormat::Uncompressed4096);

        let decompressed = alpha_map.decompress().unwrap();
        assert_eq!(decompressed.len(), 4096);
        assert_eq!(decompressed, data);
    }

    #[test]
    fn uncompressed_4096_invalid_size() {
        let data = vec![0u8; 2048];
        let alpha_map = AlphaMap::new(data, AlphaFormat::Uncompressed4096);

        let result = alpha_map.decompress();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Invalid uncompressed data size")
        );
    }

    #[test]
    fn uncompressed_2048_format_normalization() {
        // Test 4-bit normalization: 0x0F should expand to 0xFF
        let mut data = vec![0u8; 2048];
        data[0] = 0xF0; // high=0xF, low=0x0
        data[1] = 0x0F; // high=0x0, low=0xF
        data[2] = 0x8A; // high=0x8, low=0xA

        let alpha_map = AlphaMap::new(data, AlphaFormat::Uncompressed2048);
        let decompressed = alpha_map.decompress().unwrap();

        assert_eq!(decompressed.len(), 4096);

        // Byte 0: 0xF0 -> low=0x0 (0x00), high=0xF (0xFF)
        assert_eq!(decompressed[0], 0x00);
        assert_eq!(decompressed[1], 0xFF);

        // Byte 1: 0x0F -> low=0xF (0xFF), high=0x0 (0x00)
        assert_eq!(decompressed[2], 0xFF);
        assert_eq!(decompressed[3], 0x00);

        // Byte 2: 0x8A -> low=0xA (0xAA), high=0x8 (0x88)
        assert_eq!(decompressed[4], 0xAA);
        assert_eq!(decompressed[5], 0x88);
    }

    #[test]
    fn uncompressed_2048_invalid_size() {
        let data = vec![0u8; 1024];
        let alpha_map = AlphaMap::new(data, AlphaFormat::Uncompressed2048);

        let result = alpha_map.decompress();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid 4-bit data size"));
    }

    #[test]
    fn rle_copy_mode() {
        let mut data = Vec::new();

        // Copy 5 bytes
        data.push(0x05);
        data.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);

        // Fill remaining 4091 bytes with 0x00
        // Need multiple fill commands since max count is 127
        let mut remaining = 4091;
        while remaining > 0 {
            let chunk_size = remaining.min(127);
            data.push(0x80 | chunk_size as u8);
            data.push(0x00);
            remaining -= chunk_size;
        }

        let alpha_map = AlphaMap::new(data, AlphaFormat::Compressed);
        let decompressed = alpha_map.decompress().unwrap();

        assert_eq!(decompressed.len(), 4096);
        assert_eq!(decompressed[0], 0xAA);
        assert_eq!(decompressed[1], 0xBB);
        assert_eq!(decompressed[2], 0xCC);
        assert_eq!(decompressed[3], 0xDD);
        assert_eq!(decompressed[4], 0xEE);
        assert_eq!(decompressed[5], 0x00);
    }

    #[test]
    fn rle_fill_mode() {
        let mut data = Vec::new();

        // Fill 4 bytes with 0xFF
        data.push(0x84);
        data.push(0xFF);

        // Fill remaining 4092 bytes with 0x00
        let mut remaining = 4092;
        while remaining > 0 {
            let chunk_size = remaining.min(127);
            data.push(0x80 | chunk_size as u8);
            data.push(0x00);
            remaining -= chunk_size;
        }

        let alpha_map = AlphaMap::new(data, AlphaFormat::Compressed);
        let decompressed = alpha_map.decompress().unwrap();

        assert_eq!(decompressed.len(), 4096);
        assert_eq!(decompressed[0], 0xFF);
        assert_eq!(decompressed[1], 0xFF);
        assert_eq!(decompressed[2], 0xFF);
        assert_eq!(decompressed[3], 0xFF);
        assert_eq!(decompressed[4], 0x00);
    }

    #[test]
    fn rle_mixed_modes() {
        let mut data = Vec::new();

        // Copy 3 bytes
        data.push(0x03);
        data.extend_from_slice(&[0xAA, 0xBB, 0xCC]);

        // Fill 5 bytes with 0xFF
        data.push(0x85);
        data.push(0xFF);

        // Fill remaining 4088 bytes with 0x00
        let mut remaining = 4088;
        while remaining > 0 {
            let chunk_size = remaining.min(127);
            data.push(0x80 | chunk_size as u8);
            data.push(0x00);
            remaining -= chunk_size;
        }

        let alpha_map = AlphaMap::new(data, AlphaFormat::Compressed);
        let decompressed = alpha_map.decompress().unwrap();

        assert_eq!(decompressed.len(), 4096);
        assert_eq!(decompressed[0], 0xAA);
        assert_eq!(decompressed[1], 0xBB);
        assert_eq!(decompressed[2], 0xCC);
        assert_eq!(decompressed[3], 0xFF);
        assert_eq!(decompressed[4], 0xFF);
        assert_eq!(decompressed[5], 0xFF);
        assert_eq!(decompressed[6], 0xFF);
        assert_eq!(decompressed[7], 0xFF);
        assert_eq!(decompressed[8], 0x00);
    }

    #[test]
    fn rle_unexpected_eof_fill() {
        let data = vec![0x84]; // Fill mode but no fill byte
        let alpha_map = AlphaMap::new(data, AlphaFormat::Compressed);

        let result = alpha_map.decompress();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unexpected EOF in fill mode"));
    }

    #[test]
    fn rle_unexpected_eof_copy() {
        let data = vec![0x05, 0xAA, 0xBB]; // Copy 5 but only 2 bytes available
        let alpha_map = AlphaMap::new(data, AlphaFormat::Compressed);

        let result = alpha_map.decompress();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unexpected EOF in copy mode"));
    }

    #[test]
    fn rle_output_size_mismatch() {
        let mut data = vec![0x05, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE];
        data.push(0x85);
        data.push(0xFF);

        let alpha_map = AlphaMap::new(data, AlphaFormat::Compressed);
        let result = alpha_map.decompress();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Output size mismatch"));
    }

    #[test]
    fn get_alpha_at_coordinates() {
        let mut data = vec![0u8; 4096];
        data[0] = 0xFF; // (0,0)
        data[63] = 0xAA; // (63,0)
        data[64] = 0xBB; // (0,1)
        data[64 * 63 + 63] = 0xCC; // (63,63)

        let alpha_map = AlphaMap::new(data, AlphaFormat::Uncompressed4096);

        assert_eq!(alpha_map.get_alpha(0, 0).unwrap(), 0xFF);
        assert_eq!(alpha_map.get_alpha(63, 0).unwrap(), 0xAA);
        assert_eq!(alpha_map.get_alpha(0, 1).unwrap(), 0xBB);
        assert_eq!(alpha_map.get_alpha(63, 63).unwrap(), 0xCC);
    }

    #[test]
    fn get_alpha_out_of_bounds() {
        let data = vec![0u8; 4096];
        let alpha_map = AlphaMap::new(data, AlphaFormat::Uncompressed4096);

        assert!(alpha_map.get_alpha(64, 0).is_err());
        assert!(alpha_map.get_alpha(0, 64).is_err());
        assert!(alpha_map.get_alpha(64, 64).is_err());
    }

    #[test]
    fn mcal_multiple_layers() {
        let mut mcal_data = Vec::new();

        // Layer 1: 4096 bytes at offset 0
        let layer1_data: Vec<u8> = (0..4096).map(|i| (i % 256) as u8).collect();
        mcal_data.extend_from_slice(&layer1_data);

        // Layer 2: 2048 bytes at offset 4096
        let layer2_data = vec![0xFFu8; 2048];
        mcal_data.extend_from_slice(&layer2_data);

        let mcal = McalChunk::new(mcal_data);

        assert_eq!(mcal.size(), 6144);

        // Extract layer 1
        let alpha1 = mcal
            .get_layer_alpha(0, 4096, AlphaFormat::Uncompressed4096)
            .unwrap();
        let decompressed1 = alpha1.decompress().unwrap();
        assert_eq!(decompressed1.len(), 4096);
        assert_eq!(decompressed1[0], 0);
        assert_eq!(decompressed1[255], 255);

        // Extract layer 2
        let alpha2 = mcal
            .get_layer_alpha(4096, 2048, AlphaFormat::Uncompressed2048)
            .unwrap();
        let decompressed2 = alpha2.decompress().unwrap();
        assert_eq!(decompressed2.len(), 4096);
        assert!(decompressed2.iter().all(|&b| b == 0xFF));
    }

    #[test]
    fn mcal_layer_out_of_bounds() {
        let mcal = McalChunk::new(vec![0u8; 4096]);

        let result = mcal.get_layer_alpha(4000, 200, AlphaFormat::Uncompressed4096);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("out of bounds"));
    }

    #[test]
    fn mcal_binrw_roundtrip() {
        let original_data = vec![0xAAu8; 8192];
        let mcal = McalChunk::new(original_data.clone());

        let mut buffer = Cursor::new(Vec::new());
        mcal.write_le(&mut buffer).unwrap();

        buffer.set_position(0);
        let decoded: McalChunk = BinRead::read_le(&mut buffer).unwrap();

        assert_eq!(decoded.data, original_data);
        assert_eq!(decoded.size(), 8192);
    }

    #[test]
    fn mcal_default() {
        let mcal = McalChunk::default();
        assert_eq!(mcal.size(), 0);
        assert!(mcal.data.is_empty());
    }

    #[test]
    fn alpha_format_equality() {
        assert_eq!(AlphaFormat::Uncompressed4096, AlphaFormat::Uncompressed4096);
        assert_ne!(AlphaFormat::Uncompressed4096, AlphaFormat::Compressed);
    }

    // ========== RLE Compression Tests ==========

    #[test]
    fn compress_rle_all_zeros() {
        let data = vec![0u8; 4096];
        let compressed = AlphaMap::compress_rle(&data).unwrap();

        // Should compress very well (all identical bytes)
        // Max run length is 127, so need multiple fill commands
        // 4096 / 127 = 32.25... = 33 fill commands (32 * 127 + 32)
        // Each fill command is 2 bytes: control byte + fill value
        assert!(compressed.len() < 100); // Should be much smaller than 4096

        // Verify round-trip
        let alpha_map = AlphaMap::new(compressed, AlphaFormat::Compressed);
        let decompressed = alpha_map.decompress().unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn compress_rle_all_same_value() {
        let data = vec![0xFFu8; 4096];
        let compressed = AlphaMap::compress_rle(&data).unwrap();

        // Should compress very well
        assert!(compressed.len() < 100);

        // Verify round-trip
        let alpha_map = AlphaMap::new(compressed, AlphaFormat::Compressed);
        let decompressed = alpha_map.decompress().unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn compress_rle_alternating_pattern() {
        // Worst case: alternating values (no runs possible)
        let data: Vec<u8> = (0..4096).map(|i| if i % 2 == 0 { 0xAA } else { 0x55 }).collect();
        let compressed = AlphaMap::compress_rle(&data).unwrap();

        // Should not compress well (all copy mode)
        // Each copy command can handle max 127 bytes: 1 control + 127 data = 128 bytes
        // 4096 / 127 = 32.25... = 33 copy commands
        // 33 * 128 = 4224 bytes (slightly larger than original due to control bytes)
        assert!(compressed.len() >= data.len());

        // Verify round-trip
        let alpha_map = AlphaMap::new(compressed, AlphaFormat::Compressed);
        let decompressed = alpha_map.decompress().unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn compress_rle_mixed_runs() {
        let mut data = Vec::with_capacity(4096);

        // Pattern: 100 zeros, 50 unique bytes, 200 0xFFs, rest zeros
        data.extend(vec![0u8; 100]);
        data.extend((0..50).map(|i| i as u8));
        data.extend(vec![0xFFu8; 200]);
        data.extend(vec![0u8; 4096 - data.len()]);

        assert_eq!(data.len(), 4096);

        let compressed = AlphaMap::compress_rle(&data).unwrap();

        // Should compress reasonably well (mix of runs and unique data)
        assert!(compressed.len() < data.len());

        // Verify round-trip
        let alpha_map = AlphaMap::new(compressed, AlphaFormat::Compressed);
        let decompressed = alpha_map.decompress().unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn compress_rle_gradient() {
        // Gradient pattern: smooth transition from 0 to 255
        let data: Vec<u8> = (0..4096).map(|i| ((i * 256) / 4096) as u8).collect();
        let compressed = AlphaMap::compress_rle(&data).unwrap();

        // Gradient has some runs but mostly unique values
        // Compression ratio depends on how many duplicate values occur

        // Verify round-trip
        let alpha_map = AlphaMap::new(compressed, AlphaFormat::Compressed);
        let decompressed = alpha_map.decompress().unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn compress_rle_short_runs() {
        // Pattern with runs of length 3 (minimum for fill mode efficiency)
        let mut data = Vec::with_capacity(4096);
        for _ in 0..(4096 / 3) {
            data.extend(vec![0xAAu8; 3]);
        }
        data.resize(4096, 0);

        let compressed = AlphaMap::compress_rle(&data).unwrap();

        // Should compress (runs of 3 are handled by fill mode)
        assert!(compressed.len() < data.len());

        // Verify round-trip
        let alpha_map = AlphaMap::new(compressed, AlphaFormat::Compressed);
        let decompressed = alpha_map.decompress().unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn compress_rle_long_runs() {
        // Pattern with very long runs (longer than 127)
        let mut data = Vec::with_capacity(4096);
        data.extend(vec![0xAAu8; 500]); // Long run: ceil(500/127)=4 commands = 8 bytes
        data.extend(vec![0xBBu8; 500]); // Long run: ceil(500/127)=4 commands = 8 bytes
        data.resize(4096, 0xCC); // Long run: ceil(3096/127)=25 commands = 50 bytes

        let compressed = AlphaMap::compress_rle(&data).unwrap();

        // Should compress very well (long identical runs)
        // Expected: 8 + 8 + 50 = 66 bytes (vs 4096 uncompressed)
        assert!(compressed.len() < 70); // Much smaller than 4096

        // Verify round-trip
        let alpha_map = AlphaMap::new(compressed, AlphaFormat::Compressed);
        let decompressed = alpha_map.decompress().unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn compress_rle_invalid_size() {
        let data = vec![0u8; 2048];
        let result = AlphaMap::compress_rle(&data);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid input size"));
    }

    #[test]
    fn compress_rle_random_data_round_trip() {
        // Use deterministic "random" pattern
        let data: Vec<u8> = (0..4096).map(|i| ((i * 17 + 42) % 256) as u8).collect();
        let compressed = AlphaMap::compress_rle(&data).unwrap();

        // Verify round-trip (most important test)
        let alpha_map = AlphaMap::new(compressed, AlphaFormat::Compressed);
        let decompressed = alpha_map.decompress().unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn from_uncompressed_creates_compressed() {
        let data = vec![0u8; 4096];
        let alpha_map = AlphaMap::from_uncompressed(&data).unwrap();

        assert_eq!(alpha_map.format, AlphaFormat::Compressed);
        assert!(alpha_map.data.len() < data.len()); // Should be compressed

        // Verify decompression works
        let decompressed = alpha_map.decompress().unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn from_uncompressed_invalid_size() {
        let data = vec![0u8; 1024];
        let result = AlphaMap::from_uncompressed(&data);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid input size"));
    }

    #[test]
    fn compress_decompress_complex_pattern() {
        // Complex pattern combining all features:
        // - Long runs of identical values
        // - Short runs (exactly 3 bytes)
        // - Unique sequences
        // - Edge cases at boundaries
        let mut data = Vec::with_capacity(4096);

        data.extend(vec![0x00u8; 256]); // Long run
        data.extend((0..127).map(|i| i as u8)); // Unique sequence (max copy length)
        data.extend(vec![0xFFu8; 3]); // Short run (minimum fill)
        data.extend(vec![0xAA, 0xBB]); // 2-byte unique (becomes copy mode)
        data.extend(vec![0xCCu8; 127]); // Exactly max run length
        data.extend(vec![0xDD, 0xEE, 0xFF]); // 3-byte unique at end of run
        data.resize(4096, 0x77); // Fill rest with pattern

        assert_eq!(data.len(), 4096);

        let compressed = AlphaMap::compress_rle(&data).unwrap();
        let alpha_map = AlphaMap::new(compressed, AlphaFormat::Compressed);
        let decompressed = alpha_map.decompress().unwrap();

        assert_eq!(decompressed, data);
    }

    #[test]
    fn compress_rle_boundary_conditions() {
        // Test edge cases:
        // - Exactly 127-byte runs (max for single control byte)
        // - Runs longer than 127 (require multiple commands)
        // - Transitions between modes at boundaries

        let mut data = Vec::with_capacity(4096);

        // Exactly 127-byte run
        data.extend(vec![0xAAu8; 127]);

        // 128-byte run (requires 2 fill commands: 127 + 1)
        data.extend(vec![0xBBu8; 128]);

        // 254-byte run (requires 2 fill commands: 127 + 127)
        data.extend(vec![0xCCu8; 254]);

        // Fill rest
        data.resize(4096, 0xDD);

        let compressed = AlphaMap::compress_rle(&data).unwrap();
        let alpha_map = AlphaMap::new(compressed, AlphaFormat::Compressed);
        let decompressed = alpha_map.decompress().unwrap();

        assert_eq!(decompressed, data);
        assert_eq!(decompressed.len(), 4096);
    }

    // ========== Optimal Format Selection Tests ==========

    #[test]
    fn optimal_format_chooses_compressed_for_zeros() {
        let data = vec![0u8; 4096];
        let alpha_map = AlphaMap::with_optimal_format(&data).unwrap();

        // All zeros should compress very well
        assert_eq!(alpha_map.format, AlphaFormat::Compressed);
        assert!(alpha_map.data.len() < 100);

        // Verify round-trip
        let decompressed = alpha_map.decompress().unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn optimal_format_chooses_compressed_for_runs() {
        let mut data = Vec::with_capacity(4096);
        data.extend(vec![0xFFu8; 2048]); // Long run
        data.extend(vec![0x00u8; 2048]); // Another long run

        let alpha_map = AlphaMap::with_optimal_format(&data).unwrap();

        // Long runs should compress well
        assert_eq!(alpha_map.format, AlphaFormat::Compressed);
        assert!(alpha_map.data.len() < 2048); // Better than 4-bit format

        // Verify round-trip
        let decompressed = alpha_map.decompress().unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn optimal_format_chooses_4bit_for_nibble_safe_data() {
        // Create data that is 4-bit safe but doesn't compress well with RLE
        // Use alternating pattern of 4-bit safe values
        let data: Vec<u8> = (0..4096)
            .map(|i| {
                let nibble = (i % 16) as u8;
                nibble | (nibble << 4) // 4-bit safe: 0x00, 0x11, 0x22, ..., 0xFF
            })
            .collect();

        let alpha_map = AlphaMap::with_optimal_format(&data).unwrap();

        // Should choose 4-bit format (2048 bytes) over uncompressed (4096 bytes)
        // RLE won't compress well due to alternating pattern
        assert_eq!(alpha_map.format, AlphaFormat::Uncompressed2048);
        assert_eq!(alpha_map.data.len(), 2048);

        // Verify round-trip
        let decompressed = alpha_map.decompress().unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn optimal_format_chooses_8bit_for_complex_data() {
        // Create data that is NOT 4-bit safe and doesn't compress well
        // Use values that require full 8-bit precision
        let data: Vec<u8> = (0..4096).map(|i| ((i * 37 + 13) % 256) as u8).collect();

        let alpha_map = AlphaMap::with_optimal_format(&data).unwrap();

        // Should choose uncompressed 8-bit format
        // Not 4-bit safe and RLE won't help with pseudo-random data
        assert_eq!(alpha_map.format, AlphaFormat::Uncompressed4096);
        assert_eq!(alpha_map.data.len(), 4096);

        // Verify round-trip
        let decompressed = alpha_map.decompress().unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn optimal_format_preserves_gradient() {
        // Smooth gradient from 0 to 255
        let data: Vec<u8> = (0..4096).map(|i| ((i * 256) / 4096) as u8).collect();

        let alpha_map = AlphaMap::with_optimal_format(&data).unwrap();

        // Gradient may compress or may not, but round-trip must work
        let decompressed = alpha_map.decompress().unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn optimal_format_invalid_size() {
        let data = vec![0u8; 1024];
        let result = AlphaMap::with_optimal_format(&data);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Invalid input size for optimal format selection"));
    }

    #[test]
    fn optimal_format_4bit_safe_all_zeros() {
        let data = vec![0x00u8; 4096]; // All zeros are 4-bit safe
        let alpha_map = AlphaMap::with_optimal_format(&data).unwrap();

        // Compression should win over 4-bit (much smaller)
        assert_eq!(alpha_map.format, AlphaFormat::Compressed);
        assert!(alpha_map.data.len() < 100);

        // Verify round-trip
        let decompressed = alpha_map.decompress().unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn optimal_format_4bit_safe_all_ff() {
        let data = vec![0xFFu8; 4096]; // All 0xFF are 4-bit safe
        let alpha_map = AlphaMap::with_optimal_format(&data).unwrap();

        // Compression should win over 4-bit (much smaller)
        assert_eq!(alpha_map.format, AlphaFormat::Compressed);
        assert!(alpha_map.data.len() < 100);

        // Verify round-trip
        let decompressed = alpha_map.decompress().unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn optimal_format_mixed_4bit_safe() {
        // Mix of 4-bit safe values that alternate frequently
        let data: Vec<u8> = (0..4096)
            .map(|i| if i % 2 == 0 { 0x00 } else { 0xFF })
            .collect();

        let alpha_map = AlphaMap::with_optimal_format(&data).unwrap();

        // Alternating pattern won't compress well with RLE (expands due to control bytes)
        // Should choose 4-bit format (2048 bytes) over 8-bit (4096 bytes)
        assert_eq!(alpha_map.format, AlphaFormat::Uncompressed2048);
        assert_eq!(alpha_map.data.len(), 2048);

        // Verify round-trip
        let decompressed = alpha_map.decompress().unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn optimal_format_not_4bit_safe() {
        // Values that require full 8-bit precision (not 4-bit safe)
        let data: Vec<u8> = (0..4096).map(|i| (i % 256) as u8).collect();

        let alpha_map = AlphaMap::with_optimal_format(&data).unwrap();

        // Not 4-bit safe, so must use 8-bit or compressed
        // Verify it's not 4-bit format
        assert_ne!(alpha_map.format, AlphaFormat::Uncompressed2048);

        // Verify round-trip
        let decompressed = alpha_map.decompress().unwrap();
        assert_eq!(decompressed, data);
    }
}
