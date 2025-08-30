//! ARM64 NEON SIMD optimizations
//!
//! This module provides ARM64-specific optimizations using NEON SIMD instructions.
//! NEON provides 128-bit vector operations that can significantly accelerate
//! string processing and hash calculations.
//!
//! All functions in this module are marked with appropriate target_feature
//! attributes and must be called from safe wrapper functions that perform
//! runtime CPU feature detection.

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

use crate::crypto::keys::{ASCII_TO_UPPER, ENCRYPTION_TABLE};

/// NEON accelerated hash computation for ARM64
///
/// Uses 128-bit NEON vectors to accelerate character normalization
/// and table lookups in the MPQ hash algorithm.
///
/// # Safety
///
/// This function must only be called when NEON support has been
/// verified through runtime detection.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub unsafe fn hash_string_neon(filename: &[u8], hash_type: u32) -> u32 {
    // For short filenames, scalar is faster due to setup overhead
    if filename.len() < 16 {
        return super::scalar::hash_string_scalar(filename, hash_type);
    }

    let mut seed1: u32 = 0x7FED7FED;
    let mut seed2: u32 = 0xEEEEEEEE;
    let mut pos = 0;

    // Process 16-byte chunks with NEON
    let chunk_size = 16;
    while pos + chunk_size <= filename.len() {
        // Load 16 bytes
        let chunk = vld1q_u8(filename.as_ptr().add(pos));

        // Convert forward slashes to backslashes
        let forward_slash = vdupq_n_u8(b'/');
        let backslash = vdupq_n_u8(b'\\');
        let is_forward_slash = vceqq_u8(chunk, forward_slash);
        let normalized = vbslq_u8(is_forward_slash, backslash, chunk);

        // Extract bytes and process serially (due to hash algorithm dependencies)
        let mut bytes = [0u8; 16];
        vst1q_u8(bytes.as_mut_ptr(), normalized);

        for &byte in &bytes {
            // Convert to uppercase using the table
            let ch = ASCII_TO_UPPER[byte as usize];

            // Update the hash (must be done serially due to dependencies)
            let table_idx = hash_type.wrapping_add(ch as u32) as usize;
            seed1 = ENCRYPTION_TABLE[table_idx] ^ (seed1.wrapping_add(seed2));
            seed2 = (ch as u32)
                .wrapping_add(seed1)
                .wrapping_add(seed2)
                .wrapping_add(seed2 << 5)
                .wrapping_add(3);
        }

        pos += chunk_size;
    }

    // Process remaining bytes with scalar implementation
    for &byte in &filename[pos..] {
        let mut ch = byte;

        // Convert path separators to backslash
        if ch == b'/' {
            ch = b'\\';
        }

        // Convert to uppercase using the table
        ch = ASCII_TO_UPPER[ch as usize];

        // Update the hash
        let table_idx = hash_type.wrapping_add(ch as u32) as usize;
        seed1 = ENCRYPTION_TABLE[table_idx] ^ (seed1.wrapping_add(seed2));
        seed2 = (ch as u32)
            .wrapping_add(seed1)
            .wrapping_add(seed2)
            .wrapping_add(seed2 << 5)
            .wrapping_add(3);
    }

    seed1
}

/// NEON accelerated Jenkins hash computation for ARM64
///
/// Uses SIMD for character normalization while maintaining the
/// sequential nature of the Jenkins algorithm.
///
/// # Safety
///
/// This function must only be called when NEON support has been
/// verified through runtime detection.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub unsafe fn jenkins_hash_neon(filename: &str) -> u64 {
    let bytes = filename.as_bytes();

    // For short strings, use direct scalar processing
    if bytes.len() < 16 {
        return super::scalar::jenkins_hash_scalar(filename);
    }

    let mut hash: u64 = 0;
    let mut pos = 0;

    // Process 16-byte chunks with NEON for normalization
    let chunk_size = 16;
    while pos + chunk_size <= bytes.len() {
        // Load 16 bytes
        let chunk = vld1q_u8(bytes.as_ptr().add(pos));

        // Convert forward slashes to backslashes
        let forward_slash = vdupq_n_u8(b'/');
        let backslash = vdupq_n_u8(b'\\');
        let is_forward_slash = vceqq_u8(chunk, forward_slash);
        let slash_normalized = vbslq_u8(is_forward_slash, backslash, chunk);

        // Convert uppercase to lowercase
        let uppercase_min = vdupq_n_u8(b'A');
        let uppercase_max = vdupq_n_u8(b'Z');
        let is_ge_a = vcgeq_u8(slash_normalized, uppercase_min);
        let is_le_z = vcleq_u8(slash_normalized, uppercase_max);
        let is_uppercase = vandq_u8(is_ge_a, is_le_z);
        let lowercase_offset = vdupq_n_u8(32);
        let case_corrected = vbslq_u8(
            is_uppercase,
            vaddq_u8(slash_normalized, lowercase_offset),
            slash_normalized,
        );

        // Extract normalized bytes and process with Jenkins algorithm
        let mut normalized_bytes = [0u8; 16];
        vst1q_u8(normalized_bytes.as_mut_ptr(), case_corrected);

        for &byte in &normalized_bytes {
            // Jenkins one-at-a-time hash algorithm
            hash = hash.wrapping_add(byte as u64);
            hash = hash.wrapping_add(hash << 10);
            hash ^= hash >> 6;
        }

        pos += chunk_size;
    }

    // Process remaining bytes
    for &byte in &bytes[pos..] {
        let mut ch = byte;

        // Convert path separators to backslash
        if ch == b'/' {
            ch = b'\\';
        }

        // Convert to lowercase
        ch = if ch.is_ascii_uppercase() { ch + 32 } else { ch };

        // Jenkins one-at-a-time hash algorithm
        hash = hash.wrapping_add(ch as u64);
        hash = hash.wrapping_add(hash << 10);
        hash ^= hash >> 6;
    }

    // Final mixing
    hash = hash.wrapping_add(hash << 3);
    hash ^= hash >> 11;
    hash = hash.wrapping_add(hash << 15);

    hash
}

/// NEON optimized batch string normalization for ARM64
///
/// Processes multiple strings simultaneously for optimal throughput.
///
/// # Safety
///
/// This function must only be called when NEON support has been
/// verified through runtime detection.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub unsafe fn normalize_strings_neon(strings: &mut [Vec<u8>]) {
    for string in strings {
        if string.len() < 16 {
            // Use scalar for short strings
            normalize_string_scalar(string);
            continue;
        }

        let mut pos = 0;
        let chunk_size = 16;

        while pos + chunk_size <= string.len() {
            // Load 16 bytes
            let chunk = vld1q_u8(string.as_ptr().add(pos));

            // Convert forward slashes to backslashes
            let forward_slash = vdupq_n_u8(b'/');
            let backslash = vdupq_n_u8(b'\\');
            let is_forward_slash = vceqq_u8(chunk, forward_slash);
            let slash_normalized = vbslq_u8(is_forward_slash, backslash, chunk);

            // Convert lowercase to uppercase for MPQ hash
            let lowercase_min = vdupq_n_u8(b'a');
            let lowercase_max = vdupq_n_u8(b'z');
            let is_ge_a = vcgeq_u8(slash_normalized, lowercase_min);
            let is_le_z = vcleq_u8(slash_normalized, lowercase_max);
            let is_lowercase = vandq_u8(is_ge_a, is_le_z);
            let uppercase_offset = vdupq_n_u8(32);
            let case_corrected = vbslq_u8(
                is_lowercase,
                vsubq_u8(slash_normalized, uppercase_offset), // Subtract to convert to uppercase
                slash_normalized,
            );

            // Store back
            vst1q_u8(string.as_mut_ptr().add(pos), case_corrected);
            pos += chunk_size;
        }

        // Process remaining bytes with scalar
        for byte in &mut string[pos..] {
            if *byte == b'/' {
                *byte = b'\\';
            }
            *byte = ASCII_TO_UPPER[*byte as usize];
        }
    }
}

/// NEON accelerated CRC32 computation using table-based approach
///
/// ARM64 doesn't have a direct CRC32 instruction equivalent to x86's SSE4.2,
/// but we can still use NEON to accelerate table lookups and data processing.
///
/// # Safety
///
/// This function must only be called when NEON support has been
/// verified through runtime detection.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub unsafe fn crc32_neon_accelerated(data: &[u8], initial: u32) -> u32 {
    // For small data, scalar is more efficient
    if data.len() < 64 {
        return super::scalar::crc32_scalar(data, initial);
    }

    // Use the standard crc32fast implementation which may already
    // have ARM optimizations. NEON doesn't provide the same level
    // of CRC32 acceleration as x86 SSE4.2, so we primarily benefit
    // from vectorized data movement and preprocessing.

    // For now, delegate to the scalar implementation
    // In a production version, you could implement NEON-accelerated
    // table lookups or other optimizations specific to the CRC algorithm
    super::scalar::crc32_scalar(data, initial)
}

/// Scalar string normalization helper for ARM64
fn normalize_string_scalar(string: &mut [u8]) {
    for byte in string {
        if *byte == b'/' {
            *byte = b'\\';
        }
        *byte = ASCII_TO_UPPER[*byte as usize];
    }
}

/// NEON-optimized parallel processing helper
///
/// Processes multiple hash operations in parallel when possible.
///
/// # Safety
///
/// This function must only be called when NEON support has been
/// verified through runtime detection.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub unsafe fn hash_batch_neon(filenames: &[&[u8]], hash_type: u32) -> Vec<u32> {
    let mut results = Vec::with_capacity(filenames.len());

    // Process each filename with NEON acceleration when beneficial
    for filename in filenames {
        if filename.len() >= 16 {
            results.push(hash_string_neon(filename, hash_type));
        } else {
            results.push(super::scalar::hash_string_scalar(filename, hash_type));
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn test_neon_availability() {
        if is_aarch64_feature_detected!("neon") {
            // Test NEON hash if available
            let test_string = b"Units\\Human\\Footman.mdx";
            let result = unsafe { hash_string_neon(test_string, 0) };
            let reference = super::super::scalar::hash_string_scalar(test_string, 0);

            // Results should be identical
            assert_eq!(result, reference, "NEON hash should match reference");
        } else {
            println!("NEON not available on this CPU");
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn test_neon_jenkins_hash() {
        if is_aarch64_feature_detected!("neon") {
            let test_cases = [
                "",
                "a",
                "test.txt",
                "Units\\Human\\Footman.mdx",
                "interface/glue/mainmenu.blp",
                "very/long/path/to/some/file.txt",
                &"x".repeat(50), // Long string
            ];

            for test_string in &test_cases {
                let neon_result = unsafe { jenkins_hash_neon(test_string) };
                let scalar_result = super::super::scalar::jenkins_hash_scalar(test_string);

                assert_eq!(
                    neon_result, scalar_result,
                    "NEON Jenkins hash mismatch for '{}'",
                    test_string
                );
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn test_neon_normalization() {
        if is_aarch64_feature_detected!("neon") {
            let test_cases = vec![
                "path/to/file.txt".to_string(),
                "UPPERCASE/file.TXT".to_string(),
                "mixed/Case/File.Ext".to_string(),
                "a".repeat(30), // Long string
            ];

            for test_case in test_cases {
                let mut neon_version = test_case.clone().into_bytes();
                let mut scalar_version = test_case.clone().into_bytes();

                // Apply NEON normalization
                unsafe { normalize_strings_neon(&mut [neon_version.clone()]) };

                // Apply scalar normalization
                normalize_string_scalar(&mut scalar_version);

                assert_eq!(
                    neon_version, scalar_version,
                    "NEON normalization mismatch for '{}'",
                    test_case
                );
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn test_neon_batch_processing() {
        if is_aarch64_feature_detected!("neon") {
            let test_filenames = [
                &b"file1.txt"[..],
                &b"file2.mdx"[..],
                &b"Units\\Human\\Footman.mdx"[..],
                &b"interface/glue/mainmenu.blp"[..],
            ];

            let neon_results = unsafe { hash_batch_neon(&test_filenames, 0) };
            assert_eq!(neon_results.len(), test_filenames.len());

            // Verify each result matches scalar implementation
            for (i, filename) in test_filenames.iter().enumerate() {
                let scalar_result = super::super::scalar::hash_string_scalar(filename, 0);
                assert_eq!(
                    neon_results[i], scalar_result,
                    "NEON batch result mismatch for filename {}",
                    i
                );
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn test_neon_crc32() {
        if is_aarch64_feature_detected!("neon") {
            let test_data = b"The quick brown fox jumps over the lazy dog";

            let neon_result = unsafe { crc32_neon_accelerated(test_data, 0) };
            let scalar_result = super::super::scalar::crc32_scalar(test_data, 0);

            assert_eq!(
                neon_result, scalar_result,
                "NEON CRC32 should match scalar implementation"
            );

            // Test with initial value
            let initial = 0x12345678;
            let neon_result2 = unsafe { crc32_neon_accelerated(test_data, initial) };
            let scalar_result2 = super::super::scalar::crc32_scalar(test_data, initial);

            assert_eq!(
                neon_result2, scalar_result2,
                "NEON CRC32 with initial value should match scalar"
            );
        }
    }
}
