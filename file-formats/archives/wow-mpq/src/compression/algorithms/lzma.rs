//! LZMA compression and decompression

use crate::{Error, Result};
use std::io::{BufReader, Cursor};

/// Decompress using LZMA
pub(crate) fn decompress(data: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    let cursor = Cursor::new(data);
    let mut input = BufReader::new(cursor);
    let mut output = Vec::with_capacity(expected_size);

    // Try LZMA format first
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
                    log::error!("LZMA decompression failed: {:?}", e);
                    log::error!("XZ decompression also failed: {:?}", xz_err);
                    log::debug!(
                        "First 16 bytes of data: {:02X?}",
                        &data[..16.min(data.len())]
                    );
                    Err(Error::compression(format!(
                        "LZMA/XZ decompression failed: LZMA: {:?}, XZ: {:?}",
                        e, xz_err
                    )))
                }
            }
        }
    }
}

/// Compress using LZMA
pub(crate) fn compress(data: &[u8]) -> Result<Vec<u8>> {
    let cursor = Cursor::new(data);
    let mut input = BufReader::new(cursor);
    let mut output = Vec::new();

    // Use LZMA format (not XZ) for MPQ compatibility
    match lzma_rs::lzma_compress(&mut input, &mut output) {
        Ok(()) => Ok(output),
        Err(e) => Err(Error::compression(format!(
            "LZMA compression failed: {:?}",
            e
        ))),
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

        println!(
            "LZMA - Original size: {}, Compressed size: {}",
            original.len(),
            compressed.len()
        );

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
