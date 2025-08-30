//! Main decompression logic and multi-compression handling
//!
//! This module provides secure decompression with comprehensive protection against
//! compression bomb attacks and resource exhaustion. All decompression operations
//! are monitored for size, time, and pattern-based attack detection.

use super::algorithms;
use super::methods::{CompressionMethod, flags};
use crate::security::{
    DecompressionMonitor, SecurityLimits, SessionTracker, validate_decompression_operation,
};
use crate::{Error, Result};

/// Decompress data using the specified compression method with security monitoring
///
/// This function provides comprehensive protection against compression bomb attacks
/// by monitoring decompression progress, enforcing size limits, and tracking
/// cumulative session usage.
pub fn decompress_secure(
    data: &[u8],
    method: u8,
    decompressed_size: usize,
    file_path: Option<&str>,
    session_tracker: &SessionTracker,
    limits: &SecurityLimits,
) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Err(Error::compression("Empty compressed data"));
    }

    // Validate decompression operation and create monitor
    let monitor = validate_decompression_operation(
        data.len() as u64,
        decompressed_size as u64,
        method,
        file_path,
        session_tracker,
        limits,
    )?;

    // Perform the actual decompression with monitoring
    let result = decompress_with_monitor(data, method, decompressed_size, &monitor)?;

    // Record successful decompression
    session_tracker.record_decompression(result.len() as u64);

    Ok(result)
}

/// Legacy decompress function for backwards compatibility
pub fn decompress(data: &[u8], method: u8, decompressed_size: usize) -> Result<Vec<u8>> {
    // Use default security limits for legacy calls
    let session_tracker = SessionTracker::new();
    let limits = SecurityLimits::default();

    decompress_secure(
        data,
        method,
        decompressed_size,
        None,
        &session_tracker,
        &limits,
    )
}

/// Internal decompression with monitoring support
fn decompress_with_monitor(
    data: &[u8],
    method: u8,
    decompressed_size: usize,
    monitor: &DecompressionMonitor,
) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Err(Error::compression("Empty compressed data"));
    }

    // Check if this is IMPLODE flag rather than COMPRESS
    if method == 0 {
        // No compression
        monitor.check_progress(data.len() as u64)?;
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

    let result = match compression {
        CompressionMethod::None => {
            monitor.check_progress(data.len() as u64)?;
            Ok(data.to_vec())
        }
        CompressionMethod::Zlib => {
            decompress_algorithm_with_monitor(data, decompressed_size, monitor, |d, s| {
                algorithms::zlib::decompress(d, s)
            })
        }
        CompressionMethod::BZip2 => {
            decompress_algorithm_with_monitor(data, decompressed_size, monitor, |d, s| {
                algorithms::bzip2::decompress(d, s)
            })
        }
        CompressionMethod::Lzma => {
            decompress_algorithm_with_monitor(data, decompressed_size, monitor, |d, s| {
                algorithms::lzma::decompress(d, s)
            })
        }
        CompressionMethod::Sparse => {
            decompress_algorithm_with_monitor(data, decompressed_size, monitor, |d, s| {
                algorithms::sparse::decompress(d, s)
            })
        }
        CompressionMethod::Implode => {
            decompress_algorithm_with_monitor(data, decompressed_size, monitor, |d, s| {
                algorithms::implode::decompress(d, s)
            })
        }
        CompressionMethod::PKWare => {
            decompress_algorithm_with_monitor(data, decompressed_size, monitor, |d, s| {
                algorithms::pkware::decompress(d, s)
            })
        }
        CompressionMethod::Huffman => {
            decompress_algorithm_with_monitor(data, decompressed_size, monitor, |d, s| {
                algorithms::huffman::decompress(d, s)
            })
        }
        CompressionMethod::AdpcmMono => {
            decompress_algorithm_with_monitor(data, decompressed_size, monitor, |d, s| {
                algorithms::adpcm::decompress_mono(d, s)
            })
        }
        CompressionMethod::AdpcmStereo => {
            decompress_algorithm_with_monitor(data, decompressed_size, monitor, |d, s| {
                algorithms::adpcm::decompress_stereo(d, s)
            })
        }
        CompressionMethod::Multiple(flags) => {
            log::debug!("Multiple compression with flags 0x{flags:02X}");
            decompress_multiple_with_monitor(data, flags, decompressed_size, monitor)
        }
    }?;

    // Final validation of decompression result
    crate::security::validate_decompression_result(
        decompressed_size as u64,
        result.len() as u64,
        10, // 10% tolerance
    )?;

    Ok(result)
}

/// Helper function to wrap algorithm calls with monitoring
fn decompress_algorithm_with_monitor<F>(
    data: &[u8],
    expected_size: usize,
    monitor: &DecompressionMonitor,
    algorithm: F,
) -> Result<Vec<u8>>
where
    F: FnOnce(&[u8], usize) -> Result<Vec<u8>>,
{
    // Check progress before decompression
    monitor.check_progress(0)?;

    // Perform decompression
    let result = algorithm(data, expected_size)?;

    // Check progress after decompression
    monitor.check_progress(result.len() as u64)?;

    Ok(result)
}

/// Handle multiple compression methods with monitoring
fn decompress_multiple_with_monitor(
    data: &[u8],
    flags: u8,
    expected_size: usize,
    monitor: &DecompressionMonitor,
) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Err(Error::compression("Empty compressed data"));
    }

    log::debug!("Decompressing multiple compression with flags 0x{flags:02X}");

    // Initial progress check
    monitor.check_progress(0)?;

    let result = decompress_multiple_internal(data, flags, expected_size, monitor)?;

    // Final progress check
    monitor.check_progress(result.len() as u64)?;

    Ok(result)
}

/// Handle multiple compression methods (legacy function)
#[allow(dead_code)] // Used by legacy decompress function
fn decompress_multiple(data: &[u8], flags: u8, expected_size: usize) -> Result<Vec<u8>> {
    let session_tracker = SessionTracker::new();
    let limits = SecurityLimits::default();
    let monitor = validate_decompression_operation(
        data.len() as u64,
        expected_size as u64,
        flags,
        None,
        &session_tracker,
        &limits,
    )?;

    decompress_multiple_internal(data, flags, expected_size, &monitor)
}

/// Handle multiple compression methods (internal implementation)
fn decompress_multiple_internal(
    data: &[u8],
    flags: u8,
    expected_size: usize,
    monitor: &DecompressionMonitor,
) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Err(Error::compression("Empty compressed data"));
    }

    log::debug!("Decompressing multiple compression with flags 0x{flags:02X}");

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

    // Check progress after initial setup
    monitor.check_progress(current_data.len() as u64)?;

    // Step 1: Decompress the primary compression method
    if has_huffman {
        log::debug!("Decompressing Huffman in multi-compression mode");
        log::debug!(
            "Current data size: {}, first few bytes: {:02X?}",
            current_data.len(),
            &current_data[..current_data.len().min(16)]
        );
        // For Huffman, we don't know the intermediate size, so we estimate conservatively
        let huffman_output_size = std::cmp::max(expected_size * 2, current_data.len() * 2);
        current_data = algorithms::huffman::decompress(&current_data, huffman_output_size)?;
        monitor.check_progress(current_data.len() as u64)?;
        log::debug!("After Huffman: {} bytes", current_data.len());
    } else if has_zlib {
        log::debug!("Decompressing Zlib");
        current_data = algorithms::zlib::decompress(&current_data, expected_size * 4)?;
        monitor.check_progress(current_data.len() as u64)?;
    } else if has_bzip2 {
        log::debug!("Decompressing BZip2");
        current_data = algorithms::bzip2::decompress(&current_data, expected_size * 4)?;
        monitor.check_progress(current_data.len() as u64)?;
    } else if has_sparse {
        log::debug!("Decompressing Sparse");
        current_data = algorithms::sparse::decompress(&current_data, expected_size * 4)?;
        monitor.check_progress(current_data.len() as u64)?;
    } else if has_implode {
        log::debug!("Decompressing Implode");
        current_data = algorithms::implode::decompress(&current_data, expected_size * 4)?;
        monitor.check_progress(current_data.len() as u64)?;
    }

    // Step 2: Decompress PKWare if present
    if has_pkware {
        log::debug!("Decompressing PKWare");
        // PKWare expected size should be estimated based on current data size
        let pkware_output_size = std::cmp::max(expected_size, current_data.len() * 2);
        current_data = algorithms::pkware::decompress(&current_data, pkware_output_size)?;
        monitor.check_progress(current_data.len() as u64)?;
    }

    // Step 3: Decompress ADPCM if present (applied last since it was first during compression)
    if let Some(adpcm_type) = adpcm_type {
        log::debug!("Decompressing ADPCM {adpcm_type}");
        current_data = match adpcm_type {
            "mono" => algorithms::adpcm::decompress_mono(&current_data, expected_size)?,
            "stereo" => algorithms::adpcm::decompress_stereo(&current_data, expected_size)?,
            _ => return Err(Error::compression("Unknown ADPCM type")),
        };
        monitor.check_progress(current_data.len() as u64)?;
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

    #[test]
    fn test_decompress_secure_api() {
        let original = b"Test data for secure compression";
        let session = SessionTracker::new();
        let limits = SecurityLimits::default();

        // Test uncompressed with security monitoring
        let result = decompress_secure(
            original,
            0,
            original.len(),
            Some("test.txt"),
            &session,
            &limits,
        )
        .expect("Secure decompression failed");
        assert_eq!(result, original);

        // Verify session tracking
        let (total_bytes, file_count, _) = session.get_stats();
        assert_eq!(total_bytes, original.len() as u64);
        assert_eq!(file_count, 1);
    }

    #[test]
    fn test_compression_bomb_protection() {
        let session = SessionTracker::new();
        let limits = SecurityLimits::strict();

        // Try to decompress with suspicious ratio
        let tiny_data = b"x";
        let huge_expected_size = 100_000_000; // 100MB from 1 byte

        let result = decompress_secure(
            tiny_data,
            flags::ZLIB,
            huge_expected_size,
            Some("bomb.txt"),
            &session,
            &limits,
        );

        // Should be rejected as compression bomb
        assert!(result.is_err());

        // Verify no bytes were recorded (failed before decompression)
        let (total_bytes, file_count, _) = session.get_stats();
        assert_eq!(total_bytes, 0);
        assert_eq!(file_count, 0);
    }

    #[test]
    fn test_session_limits() {
        let session = SessionTracker::new();
        let mut limits = SecurityLimits::strict();
        limits.max_session_decompressed = 150; // Very small limit

        let data = vec![0u8; 100];

        // First decompression should succeed
        let result = decompress_secure(&data, 0, data.len(), Some("file1.txt"), &session, &limits);
        assert!(result.is_ok());

        // Verify session tracker recorded the decompression
        let (total_bytes, _, _) = session.get_stats();
        assert_eq!(total_bytes, 100);

        // Second decompression should exceed session limit
        // (100 already recorded + 100 new = 200 > 150 limit)
        let second_data = vec![0u8; 100];
        let result = decompress_secure(
            &second_data,
            0,
            second_data.len(),
            Some("file2.txt"),
            &session,
            &limits,
        );

        // Should fail due to session limit check during validation
        assert!(result.is_err());
    }

    #[test]
    fn test_decompression_monitor() {
        let monitor = DecompressionMonitor::new(
            1024,                                  // 1KB limit
            std::time::Duration::from_millis(100), // 100ms limit
        );

        // Small progress should be OK
        assert!(monitor.check_progress(512).is_ok());

        // Exceeding size limit should fail
        assert!(monitor.check_progress(2048).is_err());

        // Test cancellation
        monitor.request_cancellation();
        assert!(monitor.check_progress(256).is_err());
    }
}
