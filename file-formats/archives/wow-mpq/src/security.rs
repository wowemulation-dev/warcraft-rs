//! Security validation module for MPQ archives
//!
//! This module provides comprehensive input validation and security controls
//! to prevent common attack vectors when parsing untrusted MPQ archives:
//!
//! - Header field validation to prevent integer overflow
//! - Table size limits to prevent memory exhaustion
//! - File path sanitization to prevent directory traversal
//! - Offset bounds checking to ensure reads stay within archive
//! - Advanced compression bomb detection with adaptive limits
//! - Decompression progress monitoring with resource protection
//! - Pattern analysis for malicious archive structures
//! - Checksum verification where possible
//!
//! All validation functions follow a fail-safe approach: when in doubt, reject.

use crate::{Error, Result};
use std::path::{Component, Path};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Security limits for various MPQ structures
#[derive(Debug, Clone)]
pub struct SecurityLimits {
    /// Maximum allowed archive size (default: 4GB)
    pub max_archive_size: u64,
    /// Maximum allowed hash table entries (default: 1M)
    pub max_hash_entries: u32,
    /// Maximum allowed block table entries (default: 1M)
    pub max_block_entries: u32,
    /// Maximum allowed sector size shift (default: 20 for 512MB sectors)
    pub max_sector_shift: u16,
    /// Maximum allowed file path length (default: 260 chars)
    pub max_path_length: usize,
    /// Maximum allowed compression ratio (default: 1000:1)
    pub max_compression_ratio: u32,
    /// Maximum allowed decompressed size per file (default: 100MB)
    pub max_decompressed_size: u64,
    /// Maximum allowed number of files in archive (default: 100k)
    pub max_file_count: u32,
    /// Maximum total decompressed bytes per session (default: 1GB)
    pub max_session_decompressed: u64,
    /// Maximum decompression time per file (default: 30 seconds)
    pub max_decompression_time: Duration,
    /// Enable pattern-based compression bomb detection (default: true)
    pub enable_pattern_detection: bool,
    /// Adaptive compression ratio limits (default: true)
    pub enable_adaptive_limits: bool,
}

impl Default for SecurityLimits {
    fn default() -> Self {
        Self {
            max_archive_size: 4 * 1024 * 1024 * 1024, // 4GB
            max_hash_entries: 1_000_000,
            max_block_entries: 1_000_000,
            max_sector_shift: 20, // 512MB max sector size
            max_path_length: 260, // Windows MAX_PATH
            max_compression_ratio: 1000,
            max_decompressed_size: 100 * 1024 * 1024, // 100MB
            max_file_count: 100_000,
            max_session_decompressed: 1024 * 1024 * 1024, // 1GB
            max_decompression_time: Duration::from_secs(30),
            enable_pattern_detection: true,
            enable_adaptive_limits: true,
        }
    }
}

/// Session tracker for tracking cumulative decompression across multiple files
#[derive(Debug, Clone)]
pub struct SessionTracker {
    /// Total bytes decompressed in this session
    pub total_decompressed: Arc<AtomicU64>,
    /// Number of files decompressed
    pub files_decompressed: Arc<AtomicUsize>,
    /// Session start time
    pub session_start: Instant,
}

impl Default for SessionTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionTracker {
    /// Create a new session tracker
    pub fn new() -> Self {
        Self {
            total_decompressed: Arc::new(AtomicU64::new(0)),
            files_decompressed: Arc::new(AtomicUsize::new(0)),
            session_start: Instant::now(),
        }
    }

    /// Record a successful decompression
    pub fn record_decompression(&self, bytes: u64) {
        self.total_decompressed.fetch_add(bytes, Ordering::Relaxed);
        self.files_decompressed.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current session statistics
    pub fn get_stats(&self) -> (u64, usize, Duration) {
        (
            self.total_decompressed.load(Ordering::Relaxed),
            self.files_decompressed.load(Ordering::Relaxed),
            self.session_start.elapsed(),
        )
    }

    /// Check if session limits are exceeded
    pub fn check_session_limits(&self, limits: &SecurityLimits) -> Result<()> {
        let total = self.total_decompressed.load(Ordering::Relaxed);
        if total > limits.max_session_decompressed {
            return Err(Error::resource_exhaustion(
                "Session decompression limit exceeded - potential resource exhaustion attack",
            ));
        }
        Ok(())
    }

    /// Check if session limits would be exceeded with additional bytes
    pub fn check_session_limits_with_addition(
        &self,
        additional_bytes: u64,
        limits: &SecurityLimits,
    ) -> Result<()> {
        let current_total = self.total_decompressed.load(Ordering::Relaxed);
        let projected_total = current_total.saturating_add(additional_bytes);
        if projected_total > limits.max_session_decompressed {
            return Err(Error::resource_exhaustion(
                "Session decompression limit would be exceeded - potential resource exhaustion attack",
            ));
        }
        Ok(())
    }
}

/// Decompression monitor for tracking progress during decompression
#[derive(Debug)]
pub struct DecompressionMonitor {
    /// Maximum allowed decompressed size
    pub max_size: u64,
    /// Maximum allowed decompression time
    pub max_time: Duration,
    /// Start time of decompression
    pub start_time: Instant,
    /// Current bytes decompressed (updated during decompression)
    pub bytes_decompressed: Arc<AtomicU64>,
    /// Whether decompression should be cancelled
    pub should_cancel: Arc<AtomicU64>,
}

impl DecompressionMonitor {
    /// Create a new decompression monitor
    pub fn new(max_size: u64, max_time: Duration) -> Self {
        Self {
            max_size,
            max_time,
            start_time: Instant::now(),
            bytes_decompressed: Arc::new(AtomicU64::new(0)),
            should_cancel: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Check if decompression should continue
    pub fn check_progress(&self, current_output_size: u64) -> Result<()> {
        // Check size limits
        if current_output_size > self.max_size {
            return Err(Error::resource_exhaustion(
                "Decompression size limit exceeded - potential compression bomb",
            ));
        }

        // Check time limits
        if self.start_time.elapsed() > self.max_time {
            return Err(Error::resource_exhaustion(
                "Decompression time limit exceeded - potential DoS attack",
            ));
        }

        // Check if cancellation was requested
        if self.should_cancel.load(Ordering::Relaxed) != 0 {
            return Err(Error::resource_exhaustion(
                "Decompression cancelled due to security limits",
            ));
        }

        // Update current progress
        self.bytes_decompressed
            .store(current_output_size, Ordering::Relaxed);

        Ok(())
    }

    /// Request cancellation of decompression
    pub fn request_cancellation(&self) {
        self.should_cancel.store(1, Ordering::Relaxed);
    }

    /// Get current statistics
    pub fn get_stats(&self) -> (u64, Duration) {
        (
            self.bytes_decompressed.load(Ordering::Relaxed),
            self.start_time.elapsed(),
        )
    }
}

/// Adaptive compression ratio calculator based on file characteristics
#[derive(Debug, Clone)]
pub struct AdaptiveCompressionLimits {
    /// Base compression ratio limit
    pub base_limit: u32,
    /// Whether to enable adaptive limits
    pub enabled: bool,
}

impl AdaptiveCompressionLimits {
    /// Create new adaptive limits
    pub fn new(base_limit: u32, enabled: bool) -> Self {
        Self {
            base_limit,
            enabled,
        }
    }

    /// Calculate compression ratio limit based on file characteristics
    pub fn calculate_limit(&self, compressed_size: u64, compression_method: u8) -> u32 {
        if !self.enabled {
            return self.base_limit;
        }

        // Adaptive limits based on compressed file size
        let size_based_limit = match compressed_size {
            // Very small files can have high ratios due to format overhead
            0..=512 => self.base_limit * 10, // Up to 10000:1 for tiny files
            513..=4096 => self.base_limit * 5, // Up to 5000:1 for small files
            4097..=65536 => self.base_limit * 2, // Up to 2000:1 for medium files
            65537..=1048576 => self.base_limit, // Base limit for large files
            _ => self.base_limit / 2,        // Stricter limit for very large files
        };

        // Adjust based on compression method capabilities
        let method_based_limit = match compression_method {
            // Text compression methods can achieve higher ratios legitimately
            0x02 => size_based_limit * 2,        // Zlib - good for text
            0x10 => size_based_limit * 3,        // BZip2 - excellent for text
            0x12 => size_based_limit * 4,        // LZMA - best for text
            0x20 => size_based_limit / 2,        // Sparse - should be moderate
            0x08 => size_based_limit,            // Implode - moderate compression
            0x01 => size_based_limit / 2,        // Huffman - lower ratios expected
            0x40 | 0x80 => size_based_limit * 2, // ADPCM - audio can compress well
            _ => size_based_limit,               // Unknown methods use base calculation
        };

        // Ensure we don't go below a reasonable minimum or above a hard maximum
        method_based_limit.clamp(50, 50000)
    }
}

impl SecurityLimits {
    /// Create new security limits with stricter settings
    pub fn strict() -> Self {
        Self {
            max_archive_size: 1024 * 1024 * 1024, // 1GB
            max_hash_entries: 100_000,
            max_block_entries: 100_000,
            max_sector_shift: 16, // 32MB max sector size
            max_path_length: 128,
            max_compression_ratio: 100,
            max_decompressed_size: 10 * 1024 * 1024, // 10MB
            max_file_count: 10_000,
            max_session_decompressed: 100 * 1024 * 1024, // 100MB
            max_decompression_time: Duration::from_secs(10),
            enable_pattern_detection: true,
            enable_adaptive_limits: true,
        }
    }

    /// Create new security limits with more permissive settings
    pub fn permissive() -> Self {
        Self {
            max_archive_size: 16 * 1024 * 1024 * 1024, // 16GB
            max_hash_entries: 10_000_000,
            max_block_entries: 10_000_000,
            max_sector_shift: 24, // 8GB max sector size
            max_path_length: 1024,
            max_compression_ratio: 10000,
            max_decompressed_size: 1024 * 1024 * 1024, // 1GB
            max_file_count: 1_000_000,
            max_session_decompressed: 16 * 1024 * 1024 * 1024, // 16GB
            max_decompression_time: Duration::from_secs(300),
            enable_pattern_detection: true,
            enable_adaptive_limits: true,
        }
    }
}

/// Validate MPQ header fields for security vulnerabilities
#[allow(clippy::too_many_arguments)]
pub fn validate_header_security(
    signature: u32,
    header_size: u32,
    archive_size: u32,
    format_version: u16,
    sector_shift: u16,
    hash_table_offset: u32,
    block_table_offset: u32,
    hash_table_size: u32,
    block_table_size: u32,
    limits: &SecurityLimits,
) -> Result<()> {
    // Validate signature
    if signature != crate::signatures::MPQ_ARCHIVE {
        return Err(Error::invalid_format(
            "Invalid MPQ signature - not a valid MPQ archive",
        ));
    }

    // Validate header size
    if !(32..=1024).contains(&header_size) {
        return Err(Error::invalid_format(
            "Invalid header size - must be between 32 and 1024 bytes",
        ));
    }

    // Validate archive size
    if archive_size == 0 || archive_size as u64 > limits.max_archive_size {
        return Err(Error::invalid_format(
            "Invalid archive size - too large or zero",
        ));
    }

    // Validate format version
    if format_version > 4 {
        return Err(Error::UnsupportedVersion(format_version));
    }

    // Validate sector shift
    if sector_shift > limits.max_sector_shift {
        return Err(Error::invalid_format(
            "Invalid sector shift - would create excessive sector size",
        ));
    }

    // Validate table offsets are within archive
    if hash_table_offset >= archive_size {
        return Err(Error::invalid_format(
            "Hash table offset exceeds archive size",
        ));
    }

    // For empty archives (especially v4), allow block table offset to equal archive size
    // Empty archives may have no actual block table data
    if block_table_size == 0 && block_table_offset == archive_size {
        // Empty archive - this is valid
    } else if block_table_offset > archive_size {
        return Err(Error::invalid_format(
            "Block table offset exceeds archive size",
        ));
    }

    // Validate table sizes
    if hash_table_size > limits.max_hash_entries {
        return Err(Error::resource_exhaustion(
            "Hash table too large - potential memory exhaustion attack",
        ));
    }

    if block_table_size > limits.max_block_entries {
        return Err(Error::invalid_format(
            "Block table too large - potential memory exhaustion attack",
        ));
    }

    // Check for integer overflow in table calculations
    let hash_table_bytes = hash_table_size
        .checked_mul(16) // Hash entry is 16 bytes
        .ok_or_else(|| Error::invalid_format("Hash table size causes integer overflow"))?;

    let block_table_bytes = block_table_size
        .checked_mul(16) // Block entry is 16 bytes
        .ok_or_else(|| Error::invalid_format("Block table size causes integer overflow"))?;

    // Ensure tables fit within archive with some tolerance for creation-time
    if let Some(end_pos) = hash_table_offset.checked_add(hash_table_bytes) {
        // Allow small tolerance for newly created archives
        if end_pos > archive_size.saturating_add(65536) {
            return Err(Error::invalid_format(
                "Hash table extends beyond archive bounds",
            ));
        }
    } else {
        return Err(Error::invalid_format(
            "Hash table size calculation overflows",
        ));
    }

    // Allow some tolerance for newly created archives where the file size may not match header yet
    if let Some(end_pos) = block_table_offset.checked_add(block_table_bytes) {
        // For new archives, allow the table to extend slightly beyond declared archive_size
        // but not beyond reasonable limits (e.g., 64KB tolerance for headers/metadata)
        if end_pos > archive_size.saturating_add(65536) {
            return Err(Error::invalid_format(
                "Block table extends beyond archive bounds",
            ));
        }
    } else {
        return Err(Error::invalid_format(
            "Block table size calculation overflows",
        ));
    }

    // Ensure hash table size is power of 2 (MPQ requirement)
    if hash_table_size == 0 || !crate::is_power_of_two(hash_table_size) {
        return Err(Error::invalid_format(
            "Hash table size must be a non-zero power of 2",
        ));
    }

    Ok(())
}

/// Validate file path for directory traversal attacks
pub fn validate_file_path(path: &str, limits: &SecurityLimits) -> Result<()> {
    // Check path length
    if path.len() > limits.max_path_length {
        return Err(Error::invalid_format(
            "File path too long - potential buffer overflow",
        ));
    }

    // Check for empty path
    if path.is_empty() {
        return Err(Error::invalid_format("Empty file path not allowed"));
    }

    // Check for null bytes (can cause issues in C FFI)
    if path.contains('\0') {
        return Err(Error::invalid_format(
            "File path contains null bytes - potential security issue",
        ));
    }

    // Normalize and validate path components
    let normalized_path = Path::new(path);

    for component in normalized_path.components() {
        match component {
            // Reject parent directory references
            Component::ParentDir => {
                return Err(Error::directory_traversal(
                    "File path contains parent directory reference",
                ));
            }
            // Reject absolute paths
            Component::RootDir => {
                return Err(Error::invalid_format(
                    "Absolute file paths not allowed in MPQ archives",
                ));
            }
            // Check normal path components
            Component::Normal(name) => {
                let name_str = name.to_string_lossy();

                // Check for Windows reserved names
                let reserved_names = [
                    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6",
                    "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7",
                    "LPT8", "LPT9",
                ];

                let name_upper = name_str.to_uppercase();
                // Check if name matches reserved name exactly or with extension
                for &reserved in &reserved_names {
                    if name_upper == reserved || name_upper.starts_with(&format!("{reserved}.")) {
                        return Err(Error::invalid_format(
                            "File path contains Windows reserved name",
                        ));
                    }
                }

                // Check for dangerous characters
                for ch in name_str.chars() {
                    match ch {
                        // Control characters
                        '\0'..='\x1f' | '\x7f' => {
                            return Err(Error::invalid_format(
                                "File path contains control characters",
                            ));
                        }
                        // Dangerous characters on Windows
                        '<' | '>' | '|' | '"' | '?' | '*' => {
                            return Err(Error::invalid_format(
                                "File path contains dangerous characters",
                            ));
                        }
                        _ => {} // Character is OK
                    }
                }
            }
            _ => {} // Other components (CurDir) are generally OK
        }
    }

    Ok(())
}

/// Validate file offset and size bounds within archive
pub fn validate_file_bounds(
    file_offset: u64,
    file_size: u64,
    compressed_size: u64,
    archive_size: u64,
    limits: &SecurityLimits,
) -> Result<()> {
    // Check for zero sizes (usually invalid)
    if compressed_size == 0 {
        return Err(Error::invalid_format("Compressed file size cannot be zero"));
    }

    // Validate decompressed size limit
    if file_size > limits.max_decompressed_size {
        return Err(Error::resource_exhaustion(
            "File size exceeds maximum allowed limit",
        ));
    }

    // Check file bounds within archive
    let file_end = file_offset
        .checked_add(compressed_size)
        .ok_or_else(|| Error::invalid_format("File offset causes integer overflow"))?;

    if file_end > archive_size {
        return Err(Error::invalid_format(
            "File data extends beyond archive bounds",
        ));
    }

    // Validate compression ratio to detect zip bombs
    if file_size > 0 && compressed_size > 0 {
        let compression_ratio = file_size / compressed_size;
        if compression_ratio > limits.max_compression_ratio as u64 {
            let ratio = file_size / compressed_size;
            return Err(Error::compression_bomb(
                ratio,
                limits.max_compression_ratio as u64,
            ));
        }
    }

    Ok(())
}

/// Validate sector data for consistency
pub fn validate_sector_data(
    sector_index: u32,
    sector_size: u32,
    data_size: usize,
    expected_crc: Option<u32>,
) -> Result<()> {
    // Validate sector size is reasonable
    if sector_size == 0 || sector_size > 16 * 1024 * 1024 {
        return Err(Error::invalid_format(
            "Invalid sector size - must be between 1 byte and 16MB",
        ));
    }

    // Check data size consistency
    if data_size > sector_size as usize {
        return Err(Error::invalid_format(
            "Sector data size exceeds sector size limit",
        ));
    }

    // Validate sector index for reasonable bounds
    if sector_index > 1_000_000 {
        return Err(Error::invalid_format(
            "Sector index too high - potential memory exhaustion",
        ));
    }

    // If CRC is provided, we could validate it here
    // This is placeholder for future CRC validation
    if let Some(_crc) = expected_crc {
        // TODO: Implement CRC validation when sector data is available
    }

    Ok(())
}

/// Validate table entry for security issues
pub fn validate_table_entry(
    entry_index: u32,
    file_offset: u32,
    file_size: u32,
    compressed_size: u32,
    archive_size: u32,
    limits: &SecurityLimits,
) -> Result<()> {
    // Validate entry index
    if entry_index >= limits.max_file_count {
        return Err(Error::invalid_format(
            "Table entry index too high - potential memory exhaustion",
        ));
    }

    // Use the file bounds validation
    validate_file_bounds(
        file_offset as u64,
        file_size as u64,
        compressed_size as u64,
        archive_size as u64,
        limits,
    )?;

    // Additional validation for compressed vs uncompressed size relationship
    if compressed_size > file_size && file_size > 0 {
        // This can happen with small files where compression overhead exceeds savings
        // But if the difference is too large, it might be suspicious
        let size_diff = compressed_size - file_size;
        if size_diff > 1024 && size_diff > file_size {
            return Err(Error::invalid_format(
                "Compressed size significantly larger than uncompressed - suspicious",
            ));
        }
    }

    Ok(())
}

/// Advanced compression bomb detection using pattern analysis
pub fn detect_compression_bomb_patterns(
    compressed_size: u64,
    decompressed_size: u64,
    compression_method: u8,
    file_path: Option<&str>,
    limits: &SecurityLimits,
) -> Result<()> {
    if !limits.enable_pattern_detection {
        return Ok(());
    }

    // Calculate adaptive compression ratio limit
    let adaptive_limits =
        AdaptiveCompressionLimits::new(limits.max_compression_ratio, limits.enable_adaptive_limits);
    let max_ratio = adaptive_limits.calculate_limit(compressed_size, compression_method);

    // Check compression ratio with adaptive limits
    if decompressed_size > 0 && compressed_size > 0 {
        let ratio = decompressed_size / compressed_size;
        if ratio > max_ratio as u64 {
            return Err(Error::compression_bomb(ratio, max_ratio as u64));
        }
    }

    // Pattern 1: Extremely small compressed size with large decompressed size
    if compressed_size < 100 && decompressed_size > 10 * 1024 * 1024 {
        return Err(Error::malicious_content(
            "Suspicious compression pattern: tiny compressed data with huge output",
        ));
    }

    // Pattern 2: Nested archive detection (by file extension)
    if let Some(path) = file_path {
        let path_lower = path.to_lowercase();
        if (path_lower.ends_with(".mpq")
            || path_lower.ends_with(".zip")
            || path_lower.ends_with(".rar")
            || path_lower.ends_with(".7z"))
            && decompressed_size > 50 * 1024 * 1024
        // Large nested archive
        {
            return Err(Error::malicious_content(
                "Suspicious nested archive with large decompressed size",
            ));
        }
    }

    // Pattern 3: Multiple compression methods with suspicious ratios
    if compression_method > 0x80 {
        // Multiple compression flags
        let expected_multi_ratio = max_ratio / 2; // Lower expectation for multi-compression
        if decompressed_size > 0 && compressed_size > 0 {
            let ratio = decompressed_size / compressed_size;
            if ratio > expected_multi_ratio as u64 {
                return Err(Error::compression_bomb(ratio, expected_multi_ratio as u64));
            }
        }
    }

    // Pattern 4: Decompressed size approaching system limits
    if decompressed_size > limits.max_decompressed_size * 3 / 4 {
        log::warn!(
            "Large decompression detected: {} bytes ({}% of limit)",
            decompressed_size,
            (decompressed_size * 100) / limits.max_decompressed_size
        );
    }

    Ok(())
}

/// Validate decompression operation with comprehensive security checks
pub fn validate_decompression_operation(
    compressed_size: u64,
    expected_decompressed_size: u64,
    compression_method: u8,
    file_path: Option<&str>,
    session_tracker: &SessionTracker,
    limits: &SecurityLimits,
) -> Result<DecompressionMonitor> {
    // Check session limits first (current + projected)
    session_tracker.check_session_limits_with_addition(expected_decompressed_size, limits)?;

    // Validate basic file bounds
    validate_file_bounds(
        0, // offset not relevant for this check
        expected_decompressed_size,
        compressed_size,
        u64::MAX, // archive size not relevant for this check
        limits,
    )?;

    // Run pattern-based compression bomb detection
    detect_compression_bomb_patterns(
        compressed_size,
        expected_decompressed_size,
        compression_method,
        file_path,
        limits,
    )?;

    // Create decompression monitor
    let monitor = DecompressionMonitor::new(
        expected_decompressed_size.min(limits.max_decompressed_size),
        limits.max_decompression_time,
    );

    // Log security-relevant decompression attempts
    if expected_decompressed_size > 10 * 1024 * 1024 {
        // Log files > 10MB
        log::info!(
            "Large decompression: {} -> {} bytes ({}:1 ratio) method=0x{:02X} path={}",
            compressed_size,
            expected_decompressed_size,
            if compressed_size > 0 {
                expected_decompressed_size / compressed_size
            } else {
                0
            },
            compression_method,
            file_path.unwrap_or("<unknown>")
        );
    }

    Ok(monitor)
}

/// Check if decompression output size is within expected bounds
pub fn validate_decompression_result(
    expected_size: u64,
    actual_size: u64,
    tolerance_percent: u8,
) -> Result<()> {
    if expected_size == 0 {
        return Ok(()); // Cannot validate if expected size is unknown
    }

    let tolerance = (expected_size * tolerance_percent as u64) / 100;
    let min_size = expected_size.saturating_sub(tolerance);
    let max_size = expected_size.saturating_add(tolerance);

    if actual_size < min_size || actual_size > max_size {
        return Err(Error::compression(format!(
            "Decompression size mismatch: expected {}, got {} (±{}% tolerance)",
            expected_size, actual_size, tolerance_percent
        )));
    }

    Ok(())
}

/// Create a security-aware error for validation failures
pub fn security_error<S: Into<String>>(message: S) -> Error {
    Error::security_violation(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_limits_defaults() {
        let limits = SecurityLimits::default();
        assert_eq!(limits.max_archive_size, 4 * 1024 * 1024 * 1024);
        assert_eq!(limits.max_hash_entries, 1_000_000);
        assert_eq!(limits.max_compression_ratio, 1000);
    }

    #[test]
    fn test_valid_header() {
        let limits = SecurityLimits::default();

        let result = validate_header_security(
            crate::signatures::MPQ_ARCHIVE,
            32,          // header_size
            1024 * 1024, // archive_size (1MB)
            1,           // format_version
            3,           // sector_shift (4KB sectors)
            32,          // hash_table_offset
            512,         // block_table_offset
            16,          // hash_table_size (16 entries, power of 2)
            16,          // block_table_size
            &limits,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_signature() {
        let limits = SecurityLimits::default();

        let result = validate_header_security(
            0x12345678, // Invalid signature
            32,
            1024 * 1024,
            1,
            3,
            32,
            512,
            16,
            16,
            &limits,
        );

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid MPQ signature")
        );
    }

    #[test]
    fn test_oversized_tables() {
        let limits = SecurityLimits::default();

        let result = validate_header_security(
            crate::signatures::MPQ_ARCHIVE,
            32,
            1024 * 1024,
            1,
            3,
            32,
            512,
            limits.max_hash_entries + 1, // Oversized hash table
            16,
            &limits,
        );

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Hash table too large")
        );
    }

    #[test]
    fn test_valid_file_path() {
        let limits = SecurityLimits::default();

        assert!(validate_file_path("data/models/character.m2", &limits).is_ok());
        assert!(validate_file_path("sounds/music/theme.mp3", &limits).is_ok());
        assert!(validate_file_path("world/maps/area.adt", &limits).is_ok());
    }

    #[test]
    fn test_directory_traversal_attack() {
        let limits = SecurityLimits::default();

        assert!(validate_file_path("../../../etc/passwd", &limits).is_err());
        assert!(validate_file_path("data/../../../secret", &limits).is_err());
        assert!(validate_file_path("/absolute/path", &limits).is_err());
    }

    #[test]
    fn test_dangerous_file_names() {
        let limits = SecurityLimits::default();

        assert!(validate_file_path("data/CON", &limits).is_err());
        assert!(validate_file_path("data/PRN.txt", &limits).is_err());
        assert!(validate_file_path("data/file<script>", &limits).is_err());
        assert!(validate_file_path("data/file\x00.txt", &limits).is_err());
    }

    #[test]
    fn test_file_bounds_validation() {
        let limits = SecurityLimits::default();

        // Valid file
        assert!(
            validate_file_bounds(
                1000,   // offset
                2048,   // decompressed size
                1024,   // compressed size
                100000, // archive size
                &limits,
            )
            .is_ok()
        );

        // File extends beyond archive
        assert!(
            validate_file_bounds(
                99000,  // offset
                2048,   // decompressed size
                2000,   // compressed size (would end at 101000)
                100000, // archive size
                &limits,
            )
            .is_err()
        );

        // Potential zip bomb
        assert!(
            validate_file_bounds(
                1000,    // offset
                1000000, // decompressed size (1MB)
                100,     // compressed size (100 bytes = 10000:1 ratio)
                100000,  // archive size
                &limits,
            )
            .is_err()
        );
    }

    #[test]
    fn test_compression_ratio_validation() {
        let limits = SecurityLimits::default();

        // Reasonable compression (10:1)
        assert!(validate_file_bounds(1000, 10240, 1024, 100000, &limits).is_ok());

        // High but acceptable compression (100:1)
        assert!(validate_file_bounds(1000, 102400, 1024, 200000, &limits).is_ok());

        // Excessive compression ratio (10000:1 - zip bomb indicator)
        assert!(validate_file_bounds(1000, 10240000, 1024, 20000000, &limits).is_err());
    }

    #[test]
    fn test_sector_validation() {
        // Valid sector
        assert!(
            validate_sector_data(
                0,    // sector index
                4096, // sector size
                2048, // data size
                None, // no CRC check
            )
            .is_ok()
        );

        // Invalid sector size
        assert!(validate_sector_data(0, 0, 1024, None).is_err());

        // Data larger than sector
        assert!(validate_sector_data(0, 1024, 2048, None).is_err());

        // Excessive sector index
        assert!(validate_sector_data(2_000_000, 4096, 2048, None).is_err());
    }

    #[test]
    fn test_session_tracker() {
        let tracker = SessionTracker::new();
        let limits = SecurityLimits::default();

        // Initial state
        let (total, count, _duration) = tracker.get_stats();
        assert_eq!(total, 0);
        assert_eq!(count, 0);

        // Record some decompressions
        tracker.record_decompression(1024);
        tracker.record_decompression(2048);

        let (total, count, _duration) = tracker.get_stats();
        assert_eq!(total, 3072);
        assert_eq!(count, 2);

        // Session limits should be OK
        assert!(tracker.check_session_limits(&limits).is_ok());
    }

    #[test]
    fn test_session_tracker_limit_exceeded() {
        let tracker = SessionTracker::new();
        let limits = SecurityLimits::strict();

        // Exceed session limit
        tracker.record_decompression(limits.max_session_decompressed + 1);

        // Should fail session limit check
        assert!(tracker.check_session_limits(&limits).is_err());
    }

    #[test]
    fn test_decompression_monitor() {
        let monitor = DecompressionMonitor::new(
            1024 * 1024,            // 1MB limit
            Duration::from_secs(5), // 5 second limit
        );

        // Small decompression should be OK
        assert!(monitor.check_progress(1024).is_ok());

        // Large decompression should fail
        assert!(monitor.check_progress(2 * 1024 * 1024).is_err());

        // Test cancellation
        monitor.request_cancellation();
        assert!(monitor.check_progress(512).is_err());
    }

    #[test]
    fn test_adaptive_compression_limits() {
        let adaptive = AdaptiveCompressionLimits::new(1000, true);

        // Small files should have higher limits
        let small_limit = adaptive.calculate_limit(100, 0x02); // 100 bytes, zlib
        let large_limit = adaptive.calculate_limit(100_000, 0x02); // 100KB, zlib

        assert!(small_limit > large_limit);
        assert!(small_limit >= 1000); // Should be at least base limit

        // Different compression methods should have different limits
        let zlib_limit = adaptive.calculate_limit(1024, 0x02);
        let lzma_limit = adaptive.calculate_limit(1024, 0x12);

        assert!(lzma_limit > zlib_limit); // LZMA should allow higher ratios
    }

    #[test]
    fn test_compression_bomb_pattern_detection() {
        let limits = SecurityLimits::default();

        // Normal compression should pass
        assert!(
            detect_compression_bomb_patterns(1024, 10240, 0x02, Some("data/file.txt"), &limits)
                .is_ok()
        );

        // Extreme ratio should fail
        assert!(
            detect_compression_bomb_patterns(
                100,
                100_000_000,
                0x02,
                Some("data/file.txt"),
                &limits
            )
            .is_err()
        );

        // Tiny compressed with huge output should fail
        assert!(
            detect_compression_bomb_patterns(50, 20_000_000, 0x02, Some("data/file.txt"), &limits)
                .is_err()
        );

        // Nested archive with large size should fail
        assert!(
            detect_compression_bomb_patterns(
                1_000_000,
                100_000_000,
                0x02,
                Some("nested.mpq"),
                &limits
            )
            .is_err()
        );
    }

    #[test]
    fn test_decompression_operation_validation() {
        let session = SessionTracker::new();
        let limits = SecurityLimits::default();

        // Valid decompression should succeed
        let result = validate_decompression_operation(
            1024,
            10240,
            0x02,
            Some("data/file.txt"),
            &session,
            &limits,
        );
        assert!(result.is_ok());

        // Compression bomb should fail
        let result = validate_decompression_operation(
            100,
            100_000_000,
            0x02,
            Some("bomb.txt"),
            &session,
            &limits,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_decompression_result_validation() {
        // Exact match should pass
        assert!(validate_decompression_result(1024, 1024, 5).is_ok());

        // Within tolerance should pass
        assert!(validate_decompression_result(1024, 1000, 5).is_ok()); // ~2.3% diff
        assert!(validate_decompression_result(1024, 1050, 5).is_ok()); // ~2.5% diff

        // Outside tolerance should fail
        assert!(validate_decompression_result(1024, 900, 5).is_err()); // ~12% diff
        assert!(validate_decompression_result(1024, 1150, 5).is_err()); // ~12% diff

        // Unknown expected size should always pass
        assert!(validate_decompression_result(0, 999999, 5).is_ok());
    }

    #[test]
    fn test_security_limits_extended() {
        let default_limits = SecurityLimits::default();
        let strict_limits = SecurityLimits::strict();
        let permissive_limits = SecurityLimits::permissive();

        // Verify session limits are properly set
        assert!(strict_limits.max_session_decompressed < default_limits.max_session_decompressed);
        assert!(
            permissive_limits.max_session_decompressed > default_limits.max_session_decompressed
        );

        // Verify time limits are properly set
        assert!(strict_limits.max_decompression_time < default_limits.max_decompression_time);
        assert!(permissive_limits.max_decompression_time > default_limits.max_decompression_time);

        // Verify pattern detection is enabled by default
        assert!(default_limits.enable_pattern_detection);
        assert!(default_limits.enable_adaptive_limits);
    }
}
