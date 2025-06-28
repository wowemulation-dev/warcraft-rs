//! PKWare compression implementation using pklib

use crate::{Error, Result};
use pklib::{CompressionMode, DictionarySize, explode_bytes, implode_bytes};

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

/// Decompress PKWare compressed data
pub(crate) fn decompress(data: &[u8], _expected_size: usize) -> Result<Vec<u8>> {
    // Handle empty data
    if data.is_empty() {
        return Ok(Vec::new());
    }

    log::debug!(
        "PKWare decompress input: {} bytes, first 16 bytes: {:02X?}",
        data.len(),
        &data[..std::cmp::min(16, data.len())]
    );

    explode_bytes(data).map_err(|e| {
        log::error!(
            "PKWare decompression failed with input size {}: {}",
            data.len(),
            e
        );
        Error::compression(format!("PKWare decompression failed: {e}"))
    })
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
