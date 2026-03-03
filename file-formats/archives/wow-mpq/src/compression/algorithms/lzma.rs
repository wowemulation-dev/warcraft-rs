//! LZMA compression and decompression

use crate::Result;
use crate::compression::error_helpers::{compression_error, decompression_error};
use std::io::{BufReader, Cursor};

/// MPQ LZMA header size: 1 (filter) + 5 (props) + 8 (uncompressed size) = 14 bytes
const MPQ_LZMA_HEADER_SIZE: usize = 14;

/// Decompress using LZMA
///
/// MPQ archives use a custom LZMA header format (from StormLib):
/// - Byte 0: "useFilter" flag (must be 0x00 for Blizzard MPQs)
/// - Bytes 1-5: LZMA properties (5 bytes, standard LZMA_PROPS_SIZE)
/// - Bytes 6-13: Uncompressed size (little-endian u64)
/// - Bytes 14+: Raw LZMA compressed data
///
/// Standard LZMA format expected by lzma_rs is: props(5) + size(8) + data.
/// So we strip the leading filter byte to convert MPQ format to standard format.
pub(crate) fn decompress(data: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    // Check if this looks like MPQ LZMA format (filter byte + standard LZMA)
    // MPQ LZMA has a leading filter byte (0x00) before the standard LZMA header
    if data.len() > MPQ_LZMA_HEADER_SIZE && data[0] == 0x00 && is_valid_lzma_props(data[1]) {
        log::debug!(
            "Detected MPQ LZMA format: filter=0x{:02X}, props=0x{:02X}, header size={}",
            data[0],
            data[1],
            MPQ_LZMA_HEADER_SIZE
        );

        // Skip the filter byte — bytes 1..end are standard LZMA format
        let standard_lzma = &data[1..];
        let cursor = Cursor::new(standard_lzma);
        let mut input = BufReader::new(cursor);
        let mut output = Vec::with_capacity(expected_size);

        match lzma_rs::lzma_decompress(&mut input, &mut output) {
            Ok(()) => {
                if expected_size > 0 && output.len() != expected_size {
                    log::warn!(
                        "LZMA decompressed size mismatch: expected {}, got {}",
                        expected_size,
                        output.len()
                    );
                }
                return Ok(output);
            }
            Err(e) => {
                log::debug!("MPQ LZMA decompression failed, trying raw: {e:?}");
                // Fall through to try raw LZMA
            }
        }
    }

    // Try standard LZMA format (data starts with props directly)
    let cursor = Cursor::new(data);
    let mut input = BufReader::new(cursor);
    let mut output = Vec::with_capacity(expected_size);

    match lzma_rs::lzma_decompress(&mut input, &mut output) {
        Ok(()) => {
            if expected_size > 0 && output.len() != expected_size {
                log::warn!(
                    "LZMA decompressed size mismatch: expected {}, got {}",
                    expected_size,
                    output.len()
                );
            }
            Ok(output)
        }
        Err(e) => {
            // If LZMA fails, try XZ format
            let cursor = Cursor::new(data);
            let mut input = BufReader::new(cursor);
            let mut output = Vec::with_capacity(expected_size);

            match lzma_rs::xz_decompress(&mut input, &mut output) {
                Ok(()) => Ok(output),
                Err(xz_err) => {
                    log::error!("LZMA decompression failed: {e:?}");
                    log::error!("XZ decompression also failed: {xz_err:?}");
                    log::debug!(
                        "First 16 bytes of data: {:02X?}",
                        &data[..16.min(data.len())]
                    );
                    Err(decompression_error(
                        "LZMA/XZ",
                        format!("LZMA: {e:?}, XZ: {xz_err:?}"),
                    ))
                }
            }
        }
    }
}

/// Check if a byte is a plausible LZMA properties byte.
///
/// LZMA props encode lc, lp, pb where: lc in 0..8, lp in 0..4, pb in 0..4.
/// The byte value is: pb * 45 + lp * 9 + lc. Max valid value: 4*45 + 4*9 + 8 = 224.
/// Common values: 0x5D (lc=3, lp=0, pb=2) is the most frequent in MPQ files.
fn is_valid_lzma_props(byte: u8) -> bool {
    byte <= 224
}

/// Compress using LZMA in MPQ format
///
/// Produces the MPQ LZMA header: filter byte (0x00) + standard LZMA stream.
/// This matches StormLib's `Compress_LZMA` output format.
pub(crate) fn compress(data: &[u8]) -> Result<Vec<u8>> {
    let cursor = Cursor::new(data);
    let mut input = BufReader::new(cursor);
    let mut lzma_data = Vec::new();

    match lzma_rs::lzma_compress(&mut input, &mut lzma_data) {
        Ok(()) => {
            // Prepend the MPQ filter byte (0x00 = no filter)
            let mut output = Vec::with_capacity(1 + lzma_data.len());
            output.push(0x00);
            output.extend_from_slice(&lzma_data);
            Ok(output)
        }
        Err(e) => Err(compression_error("LZMA", format!("{e:?}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip() {
        let original = b"Hello, World! This is a test of LZMA compression in MPQ archives. \
                     LZMA should provide good compression ratios.";

        let compressed = compress(original).expect("Compression failed");

        #[cfg(debug_assertions)]
        println!(
            "LZMA - Original size: {}, Compressed size: {}",
            original.len(),
            compressed.len()
        );

        let decompressed = decompress(&compressed, original.len()).expect("Decompression failed");

        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_mpq_lzma_header_format() {
        let original = b"Test data for MPQ LZMA header verification";

        let compressed = compress(original).expect("Compression failed");

        // Verify MPQ format: filter byte (0x00) + standard LZMA stream
        assert_eq!(compressed[0], 0x00, "First byte should be filter=0x00");
        assert!(
            is_valid_lzma_props(compressed[1]),
            "Second byte should be valid LZMA props"
        );
        assert!(
            compressed.len() > MPQ_LZMA_HEADER_SIZE,
            "Compressed data should be larger than header"
        );

        // Should decompress correctly
        let decompressed = decompress(&compressed, original.len()).expect("Decompression failed");
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_xz_format() {
        let original = b"Test data for XZ format";

        // Test XZ compression/decompression directly
        let cursor = Cursor::new(original);
        let mut input = BufReader::new(cursor);
        let mut compressed = Vec::new();

        lzma_rs::xz_compress(&mut input, &mut compressed).expect("XZ compression failed");

        // Our decompress function should handle XZ format as fallback
        let decompressed = decompress(&compressed, original.len()).expect("Decompression failed");

        assert_eq!(decompressed, original);
    }
}
