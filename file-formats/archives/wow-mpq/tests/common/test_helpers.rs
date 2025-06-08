//! Helper functions for compression tests

use wow_mpq::compression::{compress, decompress, flags};

/// Compress data and verify it includes the method byte (if compression was beneficial)
pub fn compress_with_method(data: &[u8], method: u8) -> Result<Vec<u8>, wow_mpq::Error> {
    compress(data, method)
}

/// Decompress data that was compressed with the high-level compress API
/// This handles extracting the method byte if present
pub fn decompress_from_compress_api(
    compressed: &[u8],
    expected_method: u8,
    expected_size: usize,
) -> Result<Vec<u8>, wow_mpq::Error> {
    if compressed.is_empty() {
        // Empty data should remain empty
        return Ok(vec![]);
    }

    // Check if the first byte is the expected compression method
    if compressed[0] == expected_method {
        // Has compression byte prefix, extract and decompress
        decompress(&compressed[1..], expected_method, expected_size)
    } else {
        // No compression byte, data wasn't compressed (wasn't beneficial)
        // Return as-is
        Ok(compressed.to_vec())
    }
}

/// Round-trip test helper
pub fn test_round_trip(data: &[u8], method: u8) -> Result<Vec<u8>, wow_mpq::Error> {
    let compressed = compress_with_method(data, method)?;
    let decompressed = decompress_from_compress_api(&compressed, method, data.len())?;

    if decompressed != data {
        return Err(wow_mpq::Error::compression(format!(
            "Round trip failed: original {} bytes, decompressed {} bytes",
            data.len(),
            decompressed.len()
        )));
    }

    Ok(decompressed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_with_compressible_data() {
        let data = vec![0u8; 1000]; // Highly compressible
        let result = test_round_trip(&data, flags::ZLIB).expect("Round trip should succeed");
        assert_eq!(result, data);
    }

    #[test]
    fn test_helper_with_incompressible_data() {
        let data = b"abc"; // Too small to compress
        let result = test_round_trip(data, flags::ZLIB).expect("Round trip should succeed");
        assert_eq!(result, data);
    }
}
