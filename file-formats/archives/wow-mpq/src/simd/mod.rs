//! SIMD-accelerated operations with runtime CPU detection
//!
//! This module provides hardware-accelerated versions of performance-critical operations
//! with automatic fallback to scalar implementations.
//!
//! ## Features
//!
//! - **Runtime CPU Detection**: Automatic detection of available SIMD instruction sets
//! - **CRC32 Acceleration**: Hardware-accelerated CRC32 using SSE4.2 on x86-64
//! - **Hash Acceleration**: SIMD-optimized hash functions for large-scale operations
//! - **Cross-Platform Support**: Optimized implementations for x86-64 and ARM64
//! - **Safe Fallbacks**: Always provides scalar fallback implementations
//!
//! ## Performance Targets
//!
//! - **CRC32**: 3-5x faster with SSE4.2 hardware acceleration
//! - **Hash Operations**: 2-4x faster with vectorized processing
//! - **Large Archives**: Significant improvements for Cataclysm/MoP size archives
//! - **Bulk Processing**: 20-40% overall improvement for multi-file operations
//!
//! ## Examples
//!
//! ```no_run
//! use wow_mpq::simd::SimdOps;
//!
//! let simd = SimdOps::new();
//!
//! // Hardware-accelerated CRC32
//! let crc = simd.crc32(b"test data", 0);
//!
//! // SIMD-accelerated hash for large batches
//! let hash = simd.hash_string_simd(b"filename.mdx", 0);
//! ```

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "x86_64")]
mod x86_64;

pub mod scalar; // Fallback implementations

/// CPU capabilities detected at runtime
#[derive(Debug, Clone)]
pub struct CpuFeatures {
    /// SSE4.2 support for CRC32 instructions
    pub has_sse42: bool,
    /// AVX2 support for 256-bit vector operations
    pub has_avx2: bool,
    /// AES instructions for cryptographic operations
    pub has_aes: bool,
    /// PCLMULQDQ for carryless multiplication
    pub has_pclmulqdq: bool,
    /// ARM NEON support (ARM64 only)
    #[cfg(target_arch = "aarch64")]
    pub has_neon: bool,
}

impl Default for CpuFeatures {
    fn default() -> Self {
        detect_cpu_features()
    }
}

/// SIMD-optimized operations interface
#[derive(Debug)]
pub struct SimdOps {
    features: CpuFeatures,
}

impl SimdOps {
    /// Create new SIMD operations with runtime CPU detection
    pub fn new() -> Self {
        Self {
            features: detect_cpu_features(),
        }
    }

    /// Get detected CPU features
    pub fn features(&self) -> &CpuFeatures {
        &self.features
    }

    /// Hardware-accelerated CRC32 calculation
    ///
    /// Uses SSE4.2 CRC32 instruction when available, falls back to scalar implementation.
    /// Processes data in 8-byte chunks for maximum efficiency.
    pub fn crc32(&self, data: &[u8], initial: u32) -> u32 {
        #[cfg(target_arch = "x86_64")]
        {
            if self.features.has_sse42 {
                // Use hardware-accelerated CRC32
                return unsafe { x86_64::crc32_sse42(data, initial) };
            }
        }

        // Fall back to scalar implementation
        scalar::crc32_scalar(data, initial)
    }

    /// SIMD-accelerated hash computation for file lookups
    ///
    /// Optimizes hash computation for batch processing of multiple filenames.
    /// Uses AVX2 on x86-64 or NEON on ARM64 when available.
    pub fn hash_string_simd(&self, data: &[u8], hash_type: u32) -> u32 {
        #[cfg(target_arch = "x86_64")]
        {
            if self.features.has_avx2 && data.len() >= 32 {
                // Use AVX2 for large strings
                return unsafe { x86_64::hash_string_avx2(data, hash_type) };
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            if self.features.has_neon && data.len() >= 16 {
                // Use NEON for ARM64
                return unsafe { aarch64::hash_string_neon(data, hash_type) };
            }
        }

        // Fall back to scalar implementation
        scalar::hash_string_scalar(data, hash_type)
    }

    /// SIMD-accelerated Jenkins hash for batch processing
    ///
    /// Optimizes Jenkins one-at-a-time hash for processing multiple files.
    pub fn jenkins_hash_batch(&self, filenames: &[&str]) -> Vec<u64> {
        let mut results = Vec::with_capacity(filenames.len());

        #[cfg(target_arch = "x86_64")]
        {
            if self.features.has_avx2 && filenames.len() >= 4 {
                // Process 4 filenames at once with AVX2
                return unsafe { x86_64::jenkins_hash_batch_avx2(filenames) };
            }
        }

        // Process one by one with scalar fallback
        for filename in filenames {
            results.push(scalar::jenkins_hash_scalar(filename));
        }

        results
    }

    /// Check if any SIMD optimizations are available
    pub fn has_simd_support(&self) -> bool {
        self.features.has_sse42
            || self.features.has_avx2
            || self.features.has_aes
            || self.features.has_pclmulqdq
            || {
                #[cfg(target_arch = "aarch64")]
                {
                    self.features.has_neon
                }
                #[cfg(not(target_arch = "aarch64"))]
                {
                    false
                }
            }
    }
}

impl Default for SimdOps {
    fn default() -> Self {
        Self::new()
    }
}

/// Runtime CPU feature detection
fn detect_cpu_features() -> CpuFeatures {
    #[cfg(target_arch = "x86_64")]
    {
        CpuFeatures {
            has_sse42: is_x86_feature_detected!("sse4.2"),
            has_avx2: is_x86_feature_detected!("avx2"),
            has_aes: is_x86_feature_detected!("aes"),
            has_pclmulqdq: is_x86_feature_detected!("pclmulqdq"),
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        CpuFeatures {
            has_sse42: false,
            has_avx2: false,
            has_aes: is_aarch64_feature_detected!("aes"),
            has_pclmulqdq: is_aarch64_feature_detected!("pmull"),
            has_neon: is_aarch64_feature_detected!("neon"),
        }
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        CpuFeatures {
            has_sse42: false,
            has_avx2: false,
            has_aes: false,
            has_pclmulqdq: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_ops_creation() {
        let simd = SimdOps::new();
        let features = simd.features();

        // Should successfully detect features without crashing
        println!("Detected SIMD features:");
        println!("  SSE4.2: {}", features.has_sse42);
        println!("  AVX2: {}", features.has_avx2);
        println!("  AES: {}", features.has_aes);
        println!("  PCLMULQDQ: {}", features.has_pclmulqdq);

        #[cfg(target_arch = "aarch64")]
        println!("  NEON: {}", features.has_neon);

        // Should indicate if any SIMD support is available
        let has_simd = simd.has_simd_support();
        println!("  SIMD supported: {}", has_simd);
    }

    #[test]
    fn test_crc32_simd_correctness() {
        let simd = SimdOps::new();
        let test_data = b"The quick brown fox jumps over the lazy dog";

        let simd_result = simd.crc32(test_data, 0);
        let scalar_result = scalar::crc32_scalar(test_data, 0);

        // SIMD and scalar results should match
        assert_eq!(
            simd_result, scalar_result,
            "SIMD CRC32 should match scalar implementation"
        );

        // Test with different initial values
        let simd_result2 = simd.crc32(test_data, 0x12345678);
        let scalar_result2 = scalar::crc32_scalar(test_data, 0x12345678);

        assert_eq!(
            simd_result2, scalar_result2,
            "SIMD CRC32 with initial value should match scalar"
        );
    }

    #[test]
    fn test_hash_string_simd_correctness() {
        let simd = SimdOps::new();
        let test_string = b"Units\\Human\\Footman.mdx";

        let simd_result = simd.hash_string_simd(test_string, 0);
        let scalar_result = scalar::hash_string_scalar(test_string, 0);

        // SIMD and scalar results should match
        assert_eq!(
            simd_result, scalar_result,
            "SIMD hash should match scalar implementation"
        );

        // Test with different hash types
        for hash_type in [0, 1, 2, 3] {
            let simd_result = simd.hash_string_simd(test_string, hash_type);
            let scalar_result = scalar::hash_string_scalar(test_string, hash_type);

            assert_eq!(
                simd_result, scalar_result,
                "SIMD hash type {} should match scalar",
                hash_type
            );
        }
    }

    #[test]
    fn test_jenkins_hash_batch() {
        let simd = SimdOps::new();
        let filenames = [
            "file1.txt",
            "file2.txt",
            "file3.txt",
            "file4.txt",
            "file5.txt",
        ];

        let batch_result = simd.jenkins_hash_batch(&filenames);
        assert_eq!(batch_result.len(), filenames.len());

        // Verify each result matches scalar implementation
        for (i, filename) in filenames.iter().enumerate() {
            let scalar_result = scalar::jenkins_hash_scalar(filename);
            assert_eq!(
                batch_result[i], scalar_result,
                "Batch Jenkins hash for '{}' should match scalar",
                filename
            );
        }
    }

    #[test]
    fn test_empty_input_handling() {
        let simd = SimdOps::new();

        // Empty data should work without crashing
        let empty_crc = simd.crc32(&[], 0);
        let scalar_empty_crc = scalar::crc32_scalar(&[], 0);
        assert_eq!(empty_crc, scalar_empty_crc);

        let empty_hash = simd.hash_string_simd(&[], 0);
        let scalar_empty_hash = scalar::hash_string_scalar(&[], 0);
        assert_eq!(empty_hash, scalar_empty_hash);

        // Empty batch should return empty results
        let empty_batch = simd.jenkins_hash_batch(&[]);
        assert!(empty_batch.is_empty());
    }

    #[test]
    fn test_feature_detection_stability() {
        // Feature detection should be stable across multiple calls
        let features1 = detect_cpu_features();
        let features2 = detect_cpu_features();

        assert_eq!(features1.has_sse42, features2.has_sse42);
        assert_eq!(features1.has_avx2, features2.has_avx2);
        assert_eq!(features1.has_aes, features2.has_aes);
        assert_eq!(features1.has_pclmulqdq, features2.has_pclmulqdq);

        #[cfg(target_arch = "aarch64")]
        assert_eq!(features1.has_neon, features2.has_neon);
    }
}
