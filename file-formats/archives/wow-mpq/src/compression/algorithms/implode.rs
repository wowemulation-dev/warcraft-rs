//! PKWare Implode compression/decompression implementation
//!
//! This implements the PKWare Implode algorithm using the pklib crate.
//! Based on the StormLib implementation.

use crate::{Error, Result};
use pklib::explode_bytes;

/// PKWare Implode decompression
///
/// This is used specifically for HET/BET table decompression in newer MPQ archives.
/// IMPLODE data in MPQ files is raw compressed data without the header that explode_bytes expects,
/// so we need to prepend the appropriate header based on StormLib's algorithm.
pub(crate) fn decompress(data: &[u8], _expected_size: usize) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    log::debug!(
        "IMPLODE decompress input: {} bytes, first 16 bytes: {:02X?}",
        data.len(),
        &data[..std::cmp::min(16, data.len())]
    );

    // Try all combinations of compression type and dictionary size
    let compression_types = [0u8, 1u8]; // Binary, ASCII
    let dict_sizes = [4u8, 5u8, 6u8]; // 1KB, 2KB, 4KB dictionaries

    for &compression_type in &compression_types {
        for &dict_size_bits in &dict_sizes {
            let mut header_data = Vec::with_capacity(2 + data.len());
            header_data.push(compression_type);
            header_data.push(dict_size_bits);
            header_data.extend_from_slice(data);

            log::debug!(
                "IMPLODE trying header: [0x{:02X}, 0x{:02X}] (mode={}, dict={}KB) + {} bytes",
                compression_type,
                dict_size_bits,
                if compression_type == 0 {
                    "Binary"
                } else {
                    "ASCII"
                },
                1 << (dict_size_bits - 4), // Convert to KB
                data.len()
            );

            // Try this combination
            match explode_bytes(&header_data) {
                Ok(result) => {
                    log::info!(
                        "IMPLODE decompress SUCCESS: mode={}, dict={}KB, output={} bytes",
                        if compression_type == 0 {
                            "Binary"
                        } else {
                            "ASCII"
                        },
                        1 << (dict_size_bits - 4),
                        result.len()
                    );
                    log::debug!(
                        "IMPLODE output first 16 bytes: {:02X?}",
                        &result[..std::cmp::min(16, result.len())]
                    );
                    return Ok(result);
                }
                Err(e) => {
                    log::debug!(
                        "IMPLODE failed with mode={}, dict={}KB: {}",
                        if compression_type == 0 {
                            "Binary"
                        } else {
                            "ASCII"
                        },
                        1 << (dict_size_bits - 4),
                        e
                    );
                    continue;
                }
            }
        }
    }

    // If we get here, all attempts failed
    log::error!("IMPLODE decompression failed with all attempted modes");
    Err(Error::compression(
        "IMPLODE decompression failed with all attempted compression modes",
    ))
}

/// PKWare Implode compression (stub)
pub(crate) fn compress(data: &[u8]) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    // Implode compression is not needed for reading archives
    Err(Error::compression(
        "Implode compression not implemented - only decompression is available",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_implode_empty_data() {
        assert!(decompress(&[], 0).is_ok());
        assert!(compress(&[]).is_ok());
    }

    #[test]
    fn test_implode_compression_not_implemented() {
        let data = b"test data";
        assert!(compress(data).is_err());
    }
}
