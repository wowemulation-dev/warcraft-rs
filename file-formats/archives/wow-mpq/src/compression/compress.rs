//! Main compression logic

use super::algorithms;
use super::methods::{CompressionMethod, flags};
use crate::{Error, Result};

/// Compress data using the specified compression method
///
/// This function returns compressed data in the MPQ format, which includes:
/// - For single compression: the compression method byte followed by compressed data
/// - For multiple compression: the combined flags byte followed by compressed data
pub fn compress(data: &[u8], method: u8) -> Result<Vec<u8>> {
    // Check if compression actually reduces size
    let compressed = compress_internal(data, method)?;

    // MPQ format requires that compression saves space
    // Account for the method byte prefix when comparing sizes
    if 1 + compressed.len() >= data.len() {
        // Return uncompressed data (no compression byte prefix)
        Ok(data.to_vec())
    } else {
        // Return compressed data with method byte prefix
        let mut result = Vec::with_capacity(1 + compressed.len());
        result.push(method);
        result.extend_from_slice(&compressed);
        Ok(result)
    }
}

/// Internal compression without the method byte prefix
fn compress_internal(data: &[u8], method: u8) -> Result<Vec<u8>> {
    let compression = CompressionMethod::from_flags(method);

    match compression {
        CompressionMethod::None => Ok(data.to_vec()),
        CompressionMethod::Zlib => algorithms::zlib::compress(data),
        CompressionMethod::BZip2 => algorithms::bzip2::compress(data),
        CompressionMethod::Lzma => algorithms::lzma::compress(data),
        CompressionMethod::Sparse => algorithms::sparse::compress(data),
        CompressionMethod::AdpcmMono => algorithms::adpcm::compress_mono(data, 5), // Default compression level
        CompressionMethod::AdpcmStereo => algorithms::adpcm::compress_stereo(data, 5), // Default compression level
        CompressionMethod::PKWare => algorithms::pkware::compress(data),
        CompressionMethod::Implode => algorithms::implode::compress(data),
        CompressionMethod::Huffman => algorithms::huffman::compress(data),
        CompressionMethod::Multiple(flags) => compress_multiple(data, flags),
    }
}

/// Handle multiple compression methods
fn compress_multiple(data: &[u8], flags: u8) -> Result<Vec<u8>> {
    // Check which compressions are requested
    let has_adpcm_mono = (flags & flags::ADPCM_MONO) != 0;
    let has_adpcm_stereo = (flags & flags::ADPCM_STEREO) != 0;
    let has_huffman = (flags & flags::HUFFMAN) != 0;
    let has_zlib = (flags & flags::ZLIB) != 0;
    let has_pkware = (flags & flags::PKWARE) != 0;
    let has_bzip2 = (flags & flags::BZIP2) != 0;
    let has_sparse = (flags & flags::SPARSE) != 0;

    // We apply compressions in order: ADPCM, then others
    let mut current_data = data.to_vec();

    // Apply ADPCM first if requested
    if has_adpcm_mono {
        current_data = algorithms::adpcm::compress_mono(&current_data, 5)?;
    } else if has_adpcm_stereo {
        current_data = algorithms::adpcm::compress_stereo(&current_data, 5)?;
    }

    // Count remaining compressions
    let remaining_count = [has_huffman, has_zlib, has_pkware, has_bzip2, has_sparse]
        .iter()
        .filter(|&&x| x)
        .count();

    if remaining_count == 0 {
        // Only ADPCM was requested, but it was flagged as multiple
        // This shouldn't happen, but handle it gracefully
        return Ok(current_data);
    }

    if remaining_count > 1 {
        // Multiple non-ADPCM compressions - not supported yet
        return Err(Error::compression(
            "Multiple compression methods beyond ADPCM not yet supported",
        ));
    }

    // Apply the single remaining compression
    if has_huffman {
        current_data = algorithms::huffman::compress(&current_data)?;
    } else if has_zlib {
        current_data = algorithms::zlib::compress(&current_data)?;
    } else if has_bzip2 {
        current_data = algorithms::bzip2::compress(&current_data)?;
    } else if has_sparse {
        current_data = algorithms::sparse::compress(&current_data)?;
    } else if has_pkware {
        current_data = algorithms::pkware::compress(&current_data)?;
    }

    Ok(current_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::flags;

    #[test]
    fn test_compress_api_small_data() {
        // Test that the public compress API works with small data
        let original = b"Small data";

        // Test uncompressed
        let result = compress(original, 0).expect("Compression failed");
        assert_eq!(result, original);

        // Test zlib - small data might not compress well
        let compressed = compress(original, flags::ZLIB).expect("Compression failed");

        // If compression didn't help, we get back the original
        if compressed == original {
            // No compression was beneficial
            assert_eq!(compressed, original);
        } else {
            // Should have compression byte prefix
            assert_eq!(compressed[0], flags::ZLIB);

            // Extract compressed data and decompress
            let compressed_data = &compressed[1..];
            let decompressed =
                super::super::decompress::decompress(compressed_data, flags::ZLIB, original.len())
                    .expect("Decompression failed");
            assert_eq!(decompressed, original);
        }
    }

    #[test]
    fn test_lzma_api() {
        let original = b"Test data for LZMA compression through the public API";

        // Test through our wrapper API
        let compressed = compress(original, flags::LZMA).expect("Compression failed");

        #[cfg(debug_assertions)]
        println!("Original size: {}", original.len());
        #[cfg(debug_assertions)]
        println!("Compressed size: {}", compressed.len());
        #[cfg(debug_assertions)]
        println!(
            "First few bytes: {:02X?}",
            &compressed[..10.min(compressed.len())]
        );

        // Check if compression was beneficial
        if compressed == original {
            // No compression applied
            #[cfg(debug_assertions)]
            println!("Compression not beneficial, data unchanged");
        } else {
            // Should have compression byte prefix
            assert_eq!(
                compressed[0],
                flags::LZMA,
                "Expected LZMA compression byte 0x{:02X}, got 0x{:02X}",
                flags::LZMA,
                compressed[0]
            );

            // Extract compressed data and decompress
            let compressed_data = &compressed[1..];
            let decompressed =
                super::super::decompress::decompress(compressed_data, flags::LZMA, original.len())
                    .expect("Decompression failed");

            assert_eq!(decompressed, original);
        }
    }

    #[test]
    fn test_adpcm_zlib_multi_compression() {
        // Create audio-like data
        let mut original = Vec::new();
        for i in 0..100 {
            let sample = ((i as f32 * 0.1).sin() * 10000.0) as i16;
            original.extend_from_slice(&sample.to_le_bytes());
        }

        // Test ADPCM + ZLIB multi-compression
        let multi_flags = flags::ADPCM_MONO | flags::ZLIB;
        let compressed = compress(&original, multi_flags).expect("Multi-compression failed");

        // Should be smaller than original
        assert!(compressed.len() < original.len());

        // Should have compression byte prefix
        assert_eq!(compressed[0], multi_flags);

        // Extract compressed data and decompress
        let compressed_data = &compressed[1..];
        let decompressed =
            super::super::decompress::decompress(compressed_data, multi_flags, original.len())
                .expect("Multi-decompression failed");

        // ADPCM is lossy, so we check samples are close
        assert_eq!(decompressed.len(), original.len());
        for i in 0..100 {
            let orig_sample = i16::from_le_bytes([original[i * 2], original[i * 2 + 1]]);
            let dec_sample = i16::from_le_bytes([decompressed[i * 2], decompressed[i * 2 + 1]]);
            let diff = (orig_sample - dec_sample).abs();
            assert!(diff < 2000, "Sample {i} error too large: {diff}");
        }
    }
}
