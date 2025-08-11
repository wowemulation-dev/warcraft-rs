//! BZip2 compression and decompression

use crate::Result;
use crate::compression::error_helpers::{compression_error, decompression_error};
use bzip2::Compression;
use bzip2::read::BzDecoder;
use bzip2::write::BzEncoder;
use std::io::{Read, Write};

/// Decompress using BZip2
pub(crate) fn decompress(data: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    let mut decoder = BzDecoder::new(data);
    let mut decompressed = Vec::with_capacity(expected_size);

    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| decompression_error("BZip2", e))?;

    if decompressed.len() != expected_size {
        return Err(decompression_error("BZip2", format!("Decompressed size mismatch: expected {}, got {}", expected_size, decompressed.len())));
    }

    Ok(decompressed)
}

/// Compress using BZip2
pub(crate) fn compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = BzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .map_err(|e| compression_error("BZip2", e))?;

    encoder
        .finish()
        .map_err(|e| compression_error("BZip2", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip() {
        let original = b"Hello, World! This is a test of bzip2 compression in MPQ archives.";

        let compressed = compress(original).expect("Compression failed");

        // Note: Small data might not compress well
        println!(
            "Original size: {}, Compressed size: {}",
            original.len(),
            compressed.len()
        );

        let decompressed = decompress(&compressed, original.len()).expect("Decompression failed");

        assert_eq!(decompressed, original);
    }
}
