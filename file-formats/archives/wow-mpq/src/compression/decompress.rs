//! Main decompression logic and multi-compression handling

use super::algorithms;
use super::methods::{CompressionMethod, flags};
use crate::{Error, Result};

/// Decompress data using the specified compression method
pub fn decompress(data: &[u8], method: u8, decompressed_size: usize) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Err(Error::compression("Empty compressed data"));
    }

    // Check if this is IMPLODE flag rather than COMPRESS
    if method == 0 {
        // No compression
        return Ok(data.to_vec());
    }

    // Log what we're trying to decompress for debugging
    log::debug!(
        "Decompressing {} bytes to {} bytes with method 0x{:02X}",
        data.len(),
        decompressed_size,
        method
    );

    let compression = CompressionMethod::from_flags(method);

    match compression {
        CompressionMethod::None => Ok(data.to_vec()),
        CompressionMethod::Zlib => algorithms::zlib::decompress(data, decompressed_size),
        CompressionMethod::BZip2 => algorithms::bzip2::decompress(data, decompressed_size),
        CompressionMethod::Lzma => algorithms::lzma::decompress(data, decompressed_size),
        CompressionMethod::Sparse => algorithms::sparse::decompress(data, decompressed_size),
        CompressionMethod::Implode => algorithms::implode::decompress(data, decompressed_size),
        CompressionMethod::PKWare => algorithms::pkware::decompress(data, decompressed_size),
        CompressionMethod::Huffman => algorithms::huffman::decompress(data, decompressed_size),
        CompressionMethod::AdpcmMono => algorithms::adpcm::decompress_mono(data, decompressed_size),
        CompressionMethod::AdpcmStereo => {
            algorithms::adpcm::decompress_stereo(data, decompressed_size)
        }
        CompressionMethod::Multiple(flags) => {
            log::debug!("Multiple compression with flags 0x{:02X}", flags);
            decompress_multiple(data, flags, decompressed_size)
        }
    }
}

/// Handle multiple compression methods
fn decompress_multiple(data: &[u8], flags: u8, expected_size: usize) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Err(Error::compression("Empty compressed data"));
    }

    log::debug!(
        "Decompressing multiple compression with flags 0x{:02X}",
        flags
    );

    // Check which compression methods are present
    let has_huffman = (flags & flags::HUFFMAN) != 0;
    let has_zlib = (flags & flags::ZLIB) != 0;
    let has_implode = (flags & flags::IMPLODE) != 0;
    let has_pkware = (flags & flags::PKWARE) != 0;
    let has_bzip2 = (flags & flags::BZIP2) != 0;
    let has_sparse = (flags & flags::SPARSE) != 0;
    let has_adpcm_mono = (flags & flags::ADPCM_MONO) != 0;
    let has_adpcm_stereo = (flags & flags::ADPCM_STEREO) != 0;

    // Handle the special case where both ADPCM_MONO and ADPCM_STEREO are set
    // This appears in WoW 4.3.4 with flag 0xC9 = HUFFMAN + PKWARE + ADPCM_MONO + ADPCM_STEREO
    let has_adpcm = has_adpcm_mono || has_adpcm_stereo;
    let adpcm_type = if has_adpcm_mono && has_adpcm_stereo {
        // Both flags set - this is unusual but appears in WoW 4.3.4
        // We'll assume stereo since it's more complex
        log::debug!("Both ADPCM_MONO and ADPCM_STEREO flags set, assuming stereo");
        Some("stereo")
    } else if has_adpcm_stereo {
        Some("stereo")
    } else if has_adpcm_mono {
        Some("mono")
    } else {
        None
    };

    // Multi-compression decompression order for StormLib compatibility:
    // 1. First decompress the outermost compression (usually Huffman/Zlib/etc.)
    // 2. Then decompress PKWare if present
    // 3. Finally decompress ADPCM if present (ADPCM is applied first during compression)

    // Pre-allocate buffer with worst-case size to avoid reallocations
    let buffer_size = std::cmp::max(expected_size * 4, data.len() * 4);
    let mut current_data = Vec::with_capacity(buffer_size);
    current_data.extend_from_slice(data);

    // Step 1: Decompress the primary compression method
    if has_huffman {
        log::debug!("Decompressing Huffman");
        // For Huffman, we don't know the intermediate size, so we estimate conservatively
        let huffman_output_size = std::cmp::max(expected_size * 2, current_data.len() * 2);
        current_data = algorithms::huffman::decompress(&current_data, huffman_output_size)?;
    } else if has_zlib {
        log::debug!("Decompressing Zlib");
        current_data = algorithms::zlib::decompress(&current_data, expected_size * 4)?;
    } else if has_bzip2 {
        log::debug!("Decompressing BZip2");
        current_data = algorithms::bzip2::decompress(&current_data, expected_size * 4)?;
    } else if has_sparse {
        log::debug!("Decompressing Sparse");
        current_data = algorithms::sparse::decompress(&current_data, expected_size * 4)?;
    } else if has_implode {
        log::debug!("Decompressing Implode");
        current_data = algorithms::implode::decompress(&current_data, expected_size * 4)?;
    }

    // Step 2: Decompress PKWare if present
    if has_pkware {
        log::debug!("Decompressing PKWare");
        // PKWare expected size should be estimated based on current data size
        let pkware_output_size = std::cmp::max(expected_size, current_data.len() * 2);
        current_data = algorithms::pkware::decompress(&current_data, pkware_output_size)?;
    }

    // Step 3: Decompress ADPCM if present (applied last since it was first during compression)
    if let Some(adpcm_type) = adpcm_type {
        log::debug!("Decompressing ADPCM {}", adpcm_type);
        current_data = match adpcm_type {
            "mono" => algorithms::adpcm::decompress_mono(&current_data, expected_size)?,
            "stereo" => algorithms::adpcm::decompress_stereo(&current_data, expected_size)?,
            _ => return Err(Error::compression("Unknown ADPCM type")),
        };
    }

    // If we only have single ADPCM compression, handle it directly
    if flags == flags::ADPCM_MONO {
        return algorithms::adpcm::decompress_mono(data, expected_size);
    } else if flags == flags::ADPCM_STEREO {
        return algorithms::adpcm::decompress_stereo(data, expected_size);
    }

    // If no multi-compression was detected, try single method decompression
    if !has_adpcm && !has_pkware && (has_huffman || has_zlib || has_bzip2 || has_sparse) {
        // Single compression method detected - no need for intermediate data
        if has_huffman {
            return algorithms::huffman::decompress(data, expected_size);
        } else if has_zlib {
            return algorithms::zlib::decompress(data, expected_size);
        } else if has_bzip2 {
            return algorithms::bzip2::decompress(data, expected_size);
        } else if has_sparse {
            return algorithms::sparse::decompress(data, expected_size);
        }
    }

    log::debug!(
        "Multi-compression decompression complete, output size: {}",
        current_data.len()
    );
    Ok(current_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompress_api() {
        let original = b"Test data for compression";

        // Test uncompressed
        let result = decompress(original, 0, original.len()).expect("Decompression failed");
        assert_eq!(result, original);

        // Test zlib
        let compressed = algorithms::zlib::compress(original).expect("Compression failed");
        let result =
            decompress(&compressed, flags::ZLIB, original.len()).expect("Decompression failed");
        assert_eq!(result, original);
    }
}
