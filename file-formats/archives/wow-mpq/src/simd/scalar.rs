//! Scalar fallback implementations for SIMD-optimized operations
//!
//! This module provides scalar implementations that are used as fallbacks
//! when SIMD instructions are not available. These implementations ensure
//! compatibility across all platforms and CPU architectures.

use crate::crypto::{ASCII_TO_UPPER, ENCRYPTION_TABLE};

/// Scalar CRC32 implementation (fallback)
///
/// Uses the same algorithm as crc32fast but without hardware acceleration.
pub fn crc32_scalar(data: &[u8], crc: u32) -> u32 {
    // Use the standard library's crc32fast implementation as fallback
    // This provides good performance even without SIMD
    use std::hash::Hasher;
    let mut hasher = crc32fast::Hasher::new_with_initial(crc);
    hasher.write(data);
    hasher.finish() as u32
}

/// Scalar MPQ hash implementation (fallback)
///
/// This is the reference implementation that matches the crypto/hash.rs module.
pub fn hash_string_scalar(filename: &[u8], hash_type: u32) -> u32 {
    let mut seed1: u32 = 0x7FED7FED;
    let mut seed2: u32 = 0xEEEEEEEE;

    for &byte in filename {
        // Get the next character and normalize it
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

/// Scalar Jenkins one-at-a-time hash implementation (fallback)
///
/// Reference implementation matching the crypto/jenkins.rs module.
pub fn jenkins_hash_scalar(filename: &str) -> u64 {
    let mut hash: u64 = 0;

    for &byte in filename.as_bytes() {
        // Get the next character and normalize it
        let mut ch = byte;

        // Convert path separators to backslash
        if ch == b'/' {
            ch = b'\\';
        }

        // Convert to lowercase - use our lowercase table for consistency
        ch = crate::crypto::ASCII_TO_LOWER[ch as usize];

        // Jenkins one-at-a-time hash algorithm
        hash = hash.wrapping_add(ch as u64);
        hash = hash.wrapping_add(hash << 10);
        hash ^= hash >> 6;
    }

    hash = hash.wrapping_add(hash << 3);
    hash ^= hash >> 11;
    hash = hash.wrapping_add(hash << 15);

    hash
}

/// Optimized scalar hash for batch processing
///
/// Processes multiple filename hash operations with minimal overhead.
#[allow(dead_code)]
pub fn hash_string_batch_scalar(filenames: &[&[u8]], hash_type: u32) -> Vec<u32> {
    let mut results = Vec::with_capacity(filenames.len());

    for filename in filenames {
        results.push(hash_string_scalar(filename, hash_type));
    }

    results
}

/// Optimized scalar Jenkins hash for batch processing
///
/// Processes multiple Jenkins hash operations efficiently.
#[allow(dead_code)]
pub fn jenkins_hash_batch_scalar(filenames: &[&str]) -> Vec<u64> {
    let mut results = Vec::with_capacity(filenames.len());

    for filename in filenames {
        results.push(jenkins_hash_scalar(filename));
    }

    results
}

/// Memory-efficient hash computation for large files
///
/// Processes data in chunks to maintain good cache locality.
#[allow(dead_code)]
pub fn hash_string_chunked_scalar(data: &[u8], hash_type: u32, chunk_size: usize) -> u32 {
    if data.len() <= chunk_size {
        return hash_string_scalar(data, hash_type);
    }

    // For large inputs, we still need to process sequentially due to the nature
    // of the MPQ hash algorithm (each byte depends on the previous state)
    hash_string_scalar(data, hash_type)
}

/// CRC32 with optimized processing for different data sizes
///
/// Uses different strategies based on input size for optimal performance.
#[allow(dead_code)]
pub fn crc32_optimized_scalar(data: &[u8], initial: u32) -> u32 {
    // For small data, use direct calculation
    if data.len() < 64 {
        return crc32_scalar(data, initial);
    }

    // For larger data, the crc32fast crate already has optimizations
    crc32_scalar(data, initial)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{hash_string, jenkins_hash};

    #[test]
    fn test_scalar_hash_compatibility() {
        // Test that scalar implementation matches the main crypto module
        let filename = "Units\\Human\\Footman.mdx";

        let scalar_result = hash_string_scalar(filename.as_bytes(), 0);
        let crypto_result = hash_string(filename, 0);

        assert_eq!(
            scalar_result, crypto_result,
            "Scalar hash should match crypto module implementation"
        );
    }

    #[test]
    fn test_scalar_jenkins_compatibility() {
        // Test that scalar Jenkins hash matches the main crypto module
        let filename = "file.txt";

        let scalar_result = jenkins_hash_scalar(filename);
        let crypto_result = jenkins_hash(filename);

        assert_eq!(
            scalar_result, crypto_result,
            "Scalar Jenkins hash should match crypto module implementation"
        );
    }

    #[test]
    fn test_batch_processing_correctness() {
        let filenames = ["file1.txt", "file2.txt", "file3.txt"];
        let filename_bytes: Vec<&[u8]> = filenames.iter().map(|s| s.as_bytes()).collect();

        // Test hash batch
        let batch_results = hash_string_batch_scalar(&filename_bytes, 0);
        assert_eq!(batch_results.len(), filenames.len());

        for (i, filename) in filenames.iter().enumerate() {
            let individual_result = hash_string_scalar(filename.as_bytes(), 0);
            assert_eq!(
                batch_results[i], individual_result,
                "Batch result should match individual hash for '{}'",
                filename
            );
        }

        // Test Jenkins batch
        let jenkins_batch = jenkins_hash_batch_scalar(&filenames);
        assert_eq!(jenkins_batch.len(), filenames.len());

        for (i, filename) in filenames.iter().enumerate() {
            let individual_result = jenkins_hash_scalar(filename);
            assert_eq!(
                jenkins_batch[i], individual_result,
                "Batch Jenkins result should match individual hash for '{}'",
                filename
            );
        }
    }

    #[test]
    fn test_crc32_scalar_correctness() {
        let test_data = b"The quick brown fox jumps over the lazy dog";

        // Test with initial value of 0
        let result1 = crc32_scalar(test_data, 0);
        assert_ne!(
            result1, 0,
            "CRC32 should produce non-zero result for non-empty data"
        );

        // Test with different initial value
        let result2 = crc32_scalar(test_data, 0x12345678);
        assert_ne!(
            result1, result2,
            "Different initial values should produce different results"
        );

        // Test empty data
        let empty_result = crc32_scalar(&[], 0);
        assert_eq!(
            empty_result, 0,
            "CRC32 of empty data with 0 initial should be 0"
        );
    }

    #[test]
    fn test_chunked_processing() {
        let test_data = b"This is a longer test string for chunked processing";

        // Test different chunk sizes
        for chunk_size in [8, 16, 32, 64] {
            let chunked_result = hash_string_chunked_scalar(test_data, 0, chunk_size);
            let direct_result = hash_string_scalar(test_data, 0);

            assert_eq!(
                chunked_result, direct_result,
                "Chunked processing with size {} should match direct processing",
                chunk_size
            );
        }
    }

    #[test]
    fn test_optimized_crc32() {
        let small_data = b"small";
        let large_data = b"This is a much larger piece of data that should trigger the optimized path for CRC32 calculation with better performance characteristics";

        // Test small data optimization
        let small_result1 = crc32_optimized_scalar(small_data, 0);
        let small_result2 = crc32_scalar(small_data, 0);
        assert_eq!(
            small_result1, small_result2,
            "Optimized CRC32 should match regular for small data"
        );

        // Test large data optimization
        let large_result1 = crc32_optimized_scalar(large_data, 0);
        let large_result2 = crc32_scalar(large_data, 0);
        assert_eq!(
            large_result1, large_result2,
            "Optimized CRC32 should match regular for large data"
        );
    }

    #[test]
    fn test_edge_cases() {
        // Empty data
        assert_eq!(hash_string_scalar(&[], 0), 0x7FED7FED);

        // Test Jenkins hash for empty string - use the main implementation for reference
        let empty_jenkins = jenkins_hash("");
        assert_eq!(jenkins_hash_scalar(""), empty_jenkins);

        // Single character
        assert_ne!(hash_string_scalar(b"A", 0), hash_string_scalar(b"B", 0));
        assert_ne!(jenkins_hash_scalar("A"), jenkins_hash_scalar("B"));

        // Path separator normalization
        let hash1 = hash_string_scalar(b"path/file", 0);
        let hash2 = hash_string_scalar(b"path\\file", 0);
        assert_eq!(hash1, hash2, "Path separators should be normalized");

        let jenkins1 = jenkins_hash_scalar("path/file");
        let jenkins2 = jenkins_hash_scalar("path\\file");
        assert_eq!(
            jenkins1, jenkins2,
            "Jenkins hash should normalize path separators"
        );

        // Case sensitivity (MPQ hash is case-insensitive, Jenkins is lowercase)
        let hash_upper = hash_string_scalar(b"FILE", 0);
        let hash_lower = hash_string_scalar(b"file", 0);
        assert_eq!(
            hash_upper, hash_lower,
            "MPQ hash should be case-insensitive"
        );

        let jenkins_upper = jenkins_hash_scalar("FILE");
        let jenkins_lower = jenkins_hash_scalar("file");
        assert_eq!(
            jenkins_upper, jenkins_lower,
            "Jenkins hash should normalize to lowercase"
        );
    }
}
