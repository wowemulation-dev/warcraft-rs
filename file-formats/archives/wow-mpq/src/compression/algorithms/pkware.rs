//! PKWare compression implementation using implode crate for MPQ archives

use crate::{Error, Result};
use implode::exploder::Exploder;
use implode::symbol::DEFAULT_CODE_TABLE;
use pklib::{CompressionMode, DictionarySize, implode_bytes};

/// Compress data using PKWare DCL algorithm
pub(crate) fn compress(data: &[u8]) -> Result<Vec<u8>> {
    // Handle empty data
    if data.is_empty() {
        return Ok(Vec::new());
    }

    // Use ASCII mode with 2KB dictionary as default for MPQ archives
    // This provides good compression ratio for most data types
    implode_bytes(data, CompressionMode::ASCII, DictionarySize::Size2K)
        .map_err(|e| Error::compression(format!("PKWare compression failed: {e}")))
}

/// Decompress PKWare compressed data using implode crate (for MPQ archives)
pub(crate) fn decompress(data: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    // Handle empty data
    if data.is_empty() {
        return Ok(Vec::new());
    }

    log::debug!(
        "PKWare decompress input: {} bytes, expected output: {} bytes, first 16 bytes: {:02X?}",
        data.len(),
        expected_size,
        &data[..std::cmp::min(16, data.len())]
    );

    // Use the implode crate for PKWare decompression in MPQ archives
    // Based on the working implementation in msierks/mpq-rust
    let mut exploder = Exploder::new(&DEFAULT_CODE_TABLE);
    let mut output = Vec::with_capacity(expected_size);
    let mut input_pos = 0;
    let mut total_output = 0;

    while !exploder.ended && input_pos < data.len() && total_output < expected_size {
        let remaining_input = &data[input_pos..];

        match exploder.explode_block(remaining_input) {
            Ok((consumed, output_block)) => {
                input_pos += consumed;

                // Copy output block to our buffer
                let copy_len = std::cmp::min(output_block.len(), expected_size - total_output);
                output.extend_from_slice(&output_block[..copy_len]);
                total_output += copy_len;

                log::debug!(
                    "PKWare explode block: consumed {consumed} bytes, produced {copy_len} bytes, total output: {total_output}"
                );
            }
            Err(e) => {
                log::error!("PKWare decompression failed at input position {input_pos}: {e:?}");
                return Err(Error::compression(format!(
                    "PKWare decompression failed: {e:?}"
                )));
            }
        }
    }

    log::debug!(
        "PKWare decompress complete: input {} bytes -> output {} bytes (expected {})",
        data.len(),
        output.len(),
        expected_size
    );

    Ok(output)
}

/// Compress data using PKWare DCL algorithm with specific parameters
#[allow(dead_code)]
pub(crate) fn compress_with_options(
    data: &[u8],
    mode: CompressionMode,
    dict_size: DictionarySize,
) -> Result<Vec<u8>> {
    // Handle empty data
    if data.is_empty() {
        return Ok(Vec::new());
    }

    implode_bytes(data, mode, dict_size)
        .map_err(|e| Error::compression(format!("PKWare compression failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkware_roundtrip() {
        let original = b"This is a test of PKWare compression and decompression.";

        let compressed = compress(original).expect("Compression failed");
        let decompressed = decompress(&compressed, original.len()).expect("Decompression failed");

        assert_eq!(decompressed, original);
        assert!(
            compressed.len() < original.len(),
            "Compression should reduce size"
        );
    }

    #[test]
    fn test_pkware_with_options() {
        let original = b"Testing PKWare with binary mode and different dictionary sizes.";

        // Test binary mode with 4KB dictionary
        let compressed =
            compress_with_options(original, CompressionMode::Binary, DictionarySize::Size4K)
                .expect("Compression failed");
        let decompressed = decompress(&compressed, original.len()).expect("Decompression failed");

        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_pkware_empty_data() {
        let original = b"";

        let compressed = compress(original).expect("Compression should handle empty data");
        let decompressed = decompress(&compressed, original.len()).expect("Decompression failed");

        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_pkware_large_data() {
        // Create a simple repeating pattern that compresses well
        let pattern = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let mut original = Vec::new();
        for _ in 0..10 {
            original.extend_from_slice(pattern);
        }

        let compressed = compress(&original).expect("Compression failed");
        let decompressed = decompress(&compressed, original.len()).expect("Decompression failed");

        assert_eq!(decompressed, original);
        assert!(
            compressed.len() < original.len(),
            "Repeating pattern should compress well"
        );
    }
}
