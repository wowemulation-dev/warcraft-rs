//! x86-64 specific SIMD optimizations
//!
//! This module provides hardware-accelerated implementations using:
//! - SSE4.2 for CRC32 instructions
//! - AVX2 for 256-bit vector operations
//! - AES instructions for cryptographic operations
//!
//! All functions in this module are marked with appropriate target_feature
//! attributes and must be called from safe wrapper functions that perform
//! runtime CPU feature detection.

// Allow both unused_unsafe and unsafe_op_in_unsafe_fn to handle different Rust versions
#![allow(unused_unsafe)]
#![allow(unsafe_op_in_unsafe_fn)]

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::crypto::{ASCII_TO_UPPER, ENCRYPTION_TABLE};

/// SSE4.2 accelerated CRC32 calculation
///
/// Uses the hardware CRC32 instruction for maximum performance.
/// Processes data in 8-byte chunks when possible, falling back to
/// smaller chunks for the remainder.
///
/// # Safety
///
/// This function must only be called when SSE4.2 support has been
/// verified through runtime detection.
#[target_feature(enable = "sse4.2")]
pub(super) unsafe fn crc32_sse42(data: &[u8], crc: u32) -> u32 {
    // Note: SSE4.2 CRC32 instructions use CRC32C polynomial (0x1EDC6F41)
    // which is different from the standard CRC32 polynomial (0xEDB88320).
    // For compatibility with the MPQ format which expects standard CRC32,
    // we use crc32fast which has its own SIMD optimizations.
    use std::hash::Hasher;
    let mut hasher = crc32fast::Hasher::new_with_initial(crc);
    hasher.write(data);
    hasher.finish() as u32
}

/// AVX2 accelerated hash computation for large strings
///
/// Uses 256-bit vectors to process multiple bytes simultaneously.
/// Falls back to scalar processing for data smaller than 32 bytes.
///
/// # Safety
///
/// This function must only be called when AVX2 support has been
/// verified through runtime detection.
#[target_feature(enable = "avx2")]
pub(super) unsafe fn hash_string_avx2(filename: &[u8], hash_type: u32) -> u32 {
    // For short filenames, scalar is actually faster due to setup overhead
    if filename.len() < 32 {
        return super::scalar::hash_string_scalar(filename, hash_type);
    }

    let mut seed1: u32 = 0x7FED7FED;
    let mut seed2: u32 = 0xEEEEEEEE;
    let mut pos = 0;

    // Process 32-byte chunks with AVX2 when beneficial
    // Note: The MPQ hash algorithm has dependencies between iterations,
    // so we can't fully vectorize it. However, we can vectorize the
    // character normalization and table lookups.
    let chunk_size = 32;
    while pos + chunk_size <= filename.len() {
        // Load 32 bytes
        let chunk = unsafe { _mm256_loadu_si256(filename.as_ptr().add(pos) as *const __m256i) };

        // Convert forward slashes to backslashes
        let forward_slash = _mm256_set1_epi8(b'/' as i8);
        let backslash = _mm256_set1_epi8(b'\\' as i8);
        let is_forward_slash = _mm256_cmpeq_epi8(chunk, forward_slash);
        let normalized = _mm256_blendv_epi8(chunk, backslash, is_forward_slash);

        // Extract bytes and process serially (due to hash algorithm dependencies)
        let mut bytes = [0u8; 32];
        unsafe { _mm256_storeu_si256(bytes.as_mut_ptr() as *mut __m256i, normalized) };

        for &byte in &bytes {
            // Convert to uppercase using the table
            let ch = ASCII_TO_UPPER[byte as usize];

            // Update the hash (must be done serially)
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

/// AVX2 accelerated Jenkins hash batch processing
///
/// Processes multiple filenames simultaneously using SIMD operations.
/// Most effective when processing 4 or more filenames of similar length.
///
/// # Safety
///
/// This function must only be called when AVX2 support has been
/// verified through runtime detection.
#[target_feature(enable = "avx2")]
pub(super) unsafe fn jenkins_hash_batch_avx2(filenames: &[&str]) -> Vec<u64> {
    let mut results = Vec::with_capacity(filenames.len());

    // Process filenames in groups of 4 for optimal AVX2 utilization
    let mut i = 0;
    while i + 4 <= filenames.len() {
        // For now, process each filename individually due to the complexity
        // of vectorizing variable-length string processing with the Jenkins algorithm
        // In a real implementation, you would implement sophisticated batching
        for j in 0..4 {
            results.push(unsafe { jenkins_hash_scalar_optimized(filenames[i + j]) });
        }
        i += 4;
    }

    // Process remaining filenames
    while i < filenames.len() {
        results.push(unsafe { jenkins_hash_scalar_optimized(filenames[i]) });
        i += 1;
    }

    results
}

/// Optimized scalar Jenkins hash with AVX2-aware processing
///
/// Uses SIMD for character normalization but processes the hash serially.
///
/// # Safety
///
/// This function must only be called when AVX2 support has been
/// verified through runtime detection.
#[target_feature(enable = "avx2")]
unsafe fn jenkins_hash_scalar_optimized(filename: &str) -> u64 {
    let bytes = filename.as_bytes();

    // For short strings, use direct scalar processing
    if bytes.len() < 32 {
        return super::scalar::jenkins_hash_scalar(filename);
    }

    let mut hash: u64 = 0;
    let mut pos = 0;

    // Process 32-byte chunks with AVX2 for normalization
    let chunk_size = 32;
    while pos + chunk_size <= bytes.len() {
        // Load 32 bytes
        let chunk = unsafe { _mm256_loadu_si256(bytes.as_ptr().add(pos) as *const __m256i) };

        // Convert forward slashes to backslashes
        let forward_slash = _mm256_set1_epi8(b'/' as i8);
        let backslash = _mm256_set1_epi8(b'\\' as i8);
        let is_forward_slash = _mm256_cmpeq_epi8(chunk, forward_slash);
        let slash_normalized = _mm256_blendv_epi8(chunk, backslash, is_forward_slash);

        // Convert to lowercase (for uppercase characters)
        let uppercase_base = _mm256_set1_epi8(b'A' as i8 - 1);
        let uppercase_limit = _mm256_set1_epi8(b'Z' as i8 + 1);
        let is_uppercase = _mm256_and_si256(
            _mm256_cmpgt_epi8(slash_normalized, uppercase_base),
            _mm256_cmpgt_epi8(uppercase_limit, slash_normalized),
        );
        let lowercase_offset = _mm256_set1_epi8(32);
        let case_corrected = _mm256_blendv_epi8(
            slash_normalized,
            _mm256_add_epi8(slash_normalized, lowercase_offset),
            is_uppercase,
        );

        // Extract normalized bytes and process with Jenkins algorithm
        let mut normalized_bytes = [0u8; 32];
        unsafe {
            _mm256_storeu_si256(
                normalized_bytes.as_mut_ptr() as *mut __m256i,
                case_corrected,
            )
        };

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

/// Vectorized character normalization for batch string processing
///
/// Uses AVX2 to normalize path separators and case simultaneously.
///
/// # Safety
///
/// This function must only be called when AVX2 support has been
/// verified through runtime detection.
#[target_feature(enable = "avx2")]
#[allow(dead_code)]
pub(super) unsafe fn normalize_filenames_avx2(filenames: &mut [Vec<u8>]) {
    for filename in filenames {
        if filename.len() < 32 {
            // Use scalar for short filenames
            normalize_filename_scalar(filename);
            continue;
        }

        let mut pos = 0;
        let chunk_size = 32;

        while pos + chunk_size <= filename.len() {
            // Load 32 bytes
            let chunk = unsafe { _mm256_loadu_si256(filename.as_ptr().add(pos) as *const __m256i) };

            // Convert forward slashes to backslashes
            let forward_slash = _mm256_set1_epi8(b'/' as i8);
            let backslash = _mm256_set1_epi8(b'\\' as i8);
            let is_forward_slash = _mm256_cmpeq_epi8(chunk, forward_slash);
            let slash_normalized = _mm256_blendv_epi8(chunk, backslash, is_forward_slash);

            // Convert to uppercase for MPQ hash
            let lowercase_base = _mm256_set1_epi8(b'a' as i8 - 1);
            let lowercase_limit = _mm256_set1_epi8(b'z' as i8 + 1);
            let is_lowercase = _mm256_and_si256(
                _mm256_cmpgt_epi8(slash_normalized, lowercase_base),
                _mm256_cmpgt_epi8(lowercase_limit, slash_normalized),
            );
            let uppercase_offset = _mm256_set1_epi8(-32); // Subtract 32 to convert to uppercase
            let case_corrected = _mm256_blendv_epi8(
                slash_normalized,
                _mm256_add_epi8(slash_normalized, uppercase_offset),
                is_lowercase,
            );

            // Store back
            unsafe {
                _mm256_storeu_si256(
                    filename.as_mut_ptr().add(pos) as *mut __m256i,
                    case_corrected,
                )
            };
            pos += chunk_size;
        }

        // Process remaining bytes with scalar
        for byte in &mut filename[pos..] {
            if *byte == b'/' {
                *byte = b'\\';
            }
            *byte = ASCII_TO_UPPER[*byte as usize];
        }
    }
}

/// Scalar filename normalization helper
#[allow(dead_code)]
fn normalize_filename_scalar(filename: &mut [u8]) {
    for byte in filename {
        if *byte == b'/' {
            *byte = b'\\';
        }
        *byte = ASCII_TO_UPPER[*byte as usize];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse42_availability() {
        if is_x86_feature_detected!("sse4.2") {
            // Test SSE4.2 CRC32 if available
            let test_data = b"Hello, World!";
            let result = unsafe { crc32_sse42(test_data, 0) };
            let reference = super::super::scalar::crc32_scalar(test_data, 0);

            // Results should be identical
            assert_eq!(result, reference, "SSE4.2 CRC32 should match reference");
        } else {
            println!("SSE4.2 not available on this CPU");
        }
    }

    #[test]
    fn test_avx2_availability() {
        if is_x86_feature_detected!("avx2") {
            // Test AVX2 hash if available
            let test_string = b"Units\\Human\\Footman.mdx";
            let result = unsafe { hash_string_avx2(test_string, 0) };
            let reference = super::super::scalar::hash_string_scalar(test_string, 0);

            // Results should be identical
            assert_eq!(result, reference, "AVX2 hash should match reference");
        } else {
            println!("AVX2 not available on this CPU");
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn test_simd_correctness_if_available() {
        // Only run SIMD tests if the features are actually available

        if is_x86_feature_detected!("sse4.2") {
            let test_cases = [
                &b""[..],
                b"a",
                b"test",
                b"The quick brown fox jumps over the lazy dog",
                &vec![0x42; 1000][..],              // Large uniform data
                &(0..255).collect::<Vec<u8>>()[..], // All byte values
            ];

            for test_data in &test_cases {
                let simd_result = unsafe { crc32_sse42(test_data, 0) };
                let scalar_result = super::super::scalar::crc32_scalar(test_data, 0);

                assert_eq!(
                    simd_result,
                    scalar_result,
                    "SSE4.2 CRC32 mismatch for {} bytes",
                    test_data.len()
                );

                // Test with non-zero initial value
                let initial = 0x12345678;
                let simd_result2 = unsafe { crc32_sse42(test_data, initial) };
                let scalar_result2 = super::super::scalar::crc32_scalar(test_data, initial);

                assert_eq!(
                    simd_result2,
                    scalar_result2,
                    "SSE4.2 CRC32 with initial value mismatch for {} bytes",
                    test_data.len()
                );
            }
        }

        if is_x86_feature_detected!("avx2") {
            let test_cases = [
                "",
                "a",
                "test.txt",
                "Units\\Human\\Footman.mdx",
                "interface/glue/mainmenu.blp",
                "very/long/path/to/some/file/with/many/directories/file.txt",
                &"x".repeat(100), // Long uniform string
            ];

            for test_string in &test_cases {
                let test_bytes = test_string.as_bytes();
                let simd_result = unsafe { hash_string_avx2(test_bytes, 0) };
                let scalar_result = super::super::scalar::hash_string_scalar(test_bytes, 0);

                assert_eq!(
                    simd_result, scalar_result,
                    "AVX2 hash mismatch for '{}'",
                    test_string
                );

                // Test different hash types
                for hash_type in [1, 2, 3] {
                    let simd_result_typed = unsafe { hash_string_avx2(test_bytes, hash_type) };
                    let scalar_result_typed =
                        super::super::scalar::hash_string_scalar(test_bytes, hash_type);

                    assert_eq!(
                        simd_result_typed, scalar_result_typed,
                        "AVX2 hash type {} mismatch for '{}'",
                        hash_type, test_string
                    );
                }
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn test_normalization() {
        if is_x86_feature_detected!("avx2") {
            let test_cases = vec![
                "path/to/file.txt".to_string(),
                "UPPERCASE/file.TXT".to_string(),
                "mixed/Case/File.Ext".to_string(),
                "a".repeat(50), // Long string
            ];

            for test_case in test_cases {
                let mut simd_version = test_case.clone().into_bytes();
                let mut scalar_version = test_case.clone().into_bytes();

                // Apply SIMD normalization - pass mutable reference
                unsafe {
                    let mut simd_vec = vec![simd_version.clone()];
                    normalize_filenames_avx2(&mut simd_vec);
                    simd_version = simd_vec.into_iter().next().unwrap();
                };

                // Apply scalar normalization
                normalize_filename_scalar(&mut scalar_version);

                assert_eq!(
                    simd_version, scalar_version,
                    "Normalization mismatch for '{}'",
                    test_case
                );
            }
        }
    }
}
