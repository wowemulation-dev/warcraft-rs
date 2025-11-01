//! RLE (Run-Length Encoding) decompression for MPQ patch files
//!
//! This implements the RLE decompression used in BSD0 patch files.
//! Based on the algorithm documented at http://www.zezula.net/en/mpq/patchfiles.html

use crate::error::{Error, Result};

/// Decompress RLE-compressed data
///
/// The RLE format used in MPQ patch files:
/// - If `skip_header` is true: First 4 bytes contain decompressed size (skipped)
/// - Remaining bytes: RLE-encoded data
///
/// Encoding:
/// - If byte has high bit set (0x80): Next (byte & 0x7F) + 1 bytes are literal
/// - If byte has high bit clear: Skip (byte + 1) zero bytes
///
/// # Arguments
/// * `compressed` - RLE-compressed data
/// * `decompressed_size` - Expected size after decompression
/// * `skip_header` - Whether to skip first 4 bytes (size header)
pub fn decompress(compressed: &[u8], decompressed_size: usize, skip_header: bool) -> Result<Vec<u8>> {
    let data = if skip_header {
        if compressed.len() < 4 {
            return Err(Error::compression("RLE data too short for header"));
        }
        // Skip the initial DWORD (decompressed size)
        &compressed[4..]
    } else {
        // No header, use all data
        compressed
    };

    // Pre-fill with zeros
    let mut decompressed = vec![0u8; decompressed_size];

    let mut src_pos = 0;
    let mut dst_pos = 0;

    while src_pos < data.len() && dst_pos < decompressed_size {
        let one_byte = data[src_pos];
        src_pos += 1;

        if one_byte & 0x80 != 0 {
            // High bit set: copy literal bytes
            let repeat_count = ((one_byte & 0x7F) + 1) as usize;

            for _ in 0..repeat_count {
                if dst_pos >= decompressed_size || src_pos >= data.len() {
                    break;
                }

                decompressed[dst_pos] = data[src_pos];
                dst_pos += 1;
                src_pos += 1;
            }
        } else {
            // High bit clear: skip zeros (already filled)
            dst_pos += (one_byte + 1) as usize;
        }
    }

    Ok(decompressed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rle_decompress_with_header() {
        // Test with 4-byte header
        let compressed = vec![
            0x08, 0x00, 0x00, 0x00,  // Decompressed size: 8 bytes
            0x82,                     // Copy 3 bytes
            0x41, 0x42, 0x43,        // ABC
            0x01,                     // Skip 2 zeros
            0x81,                     // Copy 2 bytes
            0x44, 0x45,              // DE
        ];

        let result = decompress(&compressed, 8, true).unwrap();
        assert_eq!(result, vec![0x41, 0x42, 0x43, 0x00, 0x00, 0x44, 0x45, 0x00]);
    }

    #[test]
    fn test_rle_decompress_no_header() {
        // Test without header (sector data)
        let compressed = vec![
            0x82,                     // Copy 3 bytes
            0x41, 0x42, 0x43,        // ABC
            0x01,                     // Skip 2 zeros
            0x81,                     // Copy 2 bytes
            0x44, 0x45,              // DE
        ];

        let result = decompress(&compressed, 8, false).unwrap();
        assert_eq!(result, vec![0x41, 0x42, 0x43, 0x00, 0x00, 0x44, 0x45, 0x00]);
    }
}
