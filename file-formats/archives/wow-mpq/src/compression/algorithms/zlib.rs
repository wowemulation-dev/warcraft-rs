//! Zlib compression and decompression

use crate::Result;
use crate::compression::error_helpers::{compression_error, decompression_error};
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use std::io::{Read, Write};

/// Decompress using zlib/deflate
pub(crate) fn decompress(data: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    // Some MPQ implementations use raw deflate without zlib headers
    // Standard zlib header starts with 0x78 (deflate with 32K window)
    let has_zlib_header = !data.is_empty() && data[0] == 0x78;

    if !data.is_empty() && !has_zlib_header {
        // Only log at trace level since this is common in some MPQ files
        log::trace!(
            "Data lacks standard zlib header (starts with 0x{:02X}), attempting raw deflate",
            data[0]
        );
    }

    // ZlibDecoder can handle both zlib-wrapped and raw deflate data
    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::with_capacity(expected_size);

    match decoder.read_to_end(&mut decompressed) {
        Ok(_) => {
            if decompressed.len() != expected_size {
                log::debug!(
                    "Decompressed size mismatch: expected {}, got {} (this is common in some MPQ files)",
                    expected_size,
                    decompressed.len()
                );
                // Some MPQ files have incorrect size info, so we'll allow this
            }
            Ok(decompressed)
        }
        Err(e) => {
            log::debug!("Zlib decompression failed: {e}");
            log::trace!(
                "First 16 bytes of data: {:02X?}",
                &data[..16.min(data.len())]
            );
            if data.len() <= 64 {
                log::trace!("Full data ({} bytes): {:02X?}", data.len(), data);
            }
            Err(decompression_error("Zlib", e))
        }
    }
}

/// Compress using zlib/deflate
pub(crate) fn compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .map_err(|e| compression_error("Zlib", e))?;

    encoder
        .finish()
        .map_err(|e| compression_error("Zlib", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip() {
        let original = b"Hello, World! This is a test of zlib compression in MPQ archives.";

        let compressed = compress(original).expect("Compression failed");

        // Note: Small data might not compress well due to compression headers
        println!(
            "Original size: {}, Compressed size: {}",
            original.len(),
            compressed.len()
        );

        let decompressed = decompress(&compressed, original.len()).expect("Decompression failed");

        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_compression_efficiency() {
        // Create highly compressible data
        let original: Vec<u8> = "A".repeat(1000).into_bytes();

        let compressed = compress(&original).expect("Compression failed");

        // This highly repetitive data should compress well
        assert!(
            compressed.len() < original.len() / 2,
            "Highly repetitive data should compress to less than 50% of original size"
        );

        let decompressed = decompress(&compressed, original.len()).expect("Decompression failed");

        assert_eq!(decompressed, original);
    }
}
