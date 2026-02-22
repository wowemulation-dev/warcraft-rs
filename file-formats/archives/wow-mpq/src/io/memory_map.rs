//! Cross-platform memory-mapped file support with security boundaries
//!
//! Provides memory-mapped access to MPQ archives while
//! maintaining security and cross-platform compatibility.
//!
//! Security features:
//! - Cross-platform safety (Windows, Linux, macOS)
//! - Resource limits to prevent memory exhaustion
//! - Access protection with bounds checking
//! - Graceful fallback to regular I/O

#[cfg(feature = "mmap")]
use crate::security::{SecurityLimits, SessionTracker};
#[cfg(feature = "mmap")]
use crate::{Error, Result};
#[cfg(feature = "mmap")]
use memmap2::{Mmap, MmapOptions};
#[cfg(feature = "mmap")]
use std::fs::File;
#[cfg(feature = "mmap")]
use std::path::Path;
#[cfg(feature = "mmap")]
use std::sync::Arc;

/// Configuration for memory mapping operations
#[derive(Debug, Clone)]
pub struct MemoryMapConfig {
    /// Maximum size for memory mapping (security limit)
    pub max_map_size: u64,
    /// Enable memory mapping (can be disabled for compatibility)
    pub enable_mapping: bool,
    /// Use read-ahead optimization
    pub read_ahead: bool,
    /// Enable advisory locking
    pub advisory_locking: bool,
}

impl Default for MemoryMapConfig {
    fn default() -> Self {
        Self {
            max_map_size: 2 * 1024 * 1024 * 1024, // 2GB limit
            enable_mapping: true,
            read_ahead: true,
            advisory_locking: false,
        }
    }
}

impl MemoryMapConfig {
    /// Create strict memory mapping configuration with lower limits
    pub fn strict() -> Self {
        Self {
            max_map_size: 256 * 1024 * 1024, // 256MB limit
            enable_mapping: true,
            read_ahead: false,
            advisory_locking: true,
        }
    }

    /// Create permissive memory mapping configuration with higher limits
    pub fn permissive() -> Self {
        Self {
            max_map_size: 8 * 1024 * 1024 * 1024, // 8GB limit
            enable_mapping: true,
            read_ahead: true,
            advisory_locking: false,
        }
    }

    /// Disable memory mapping for compatibility mode
    pub fn disabled() -> Self {
        Self {
            max_map_size: 0,
            enable_mapping: false,
            read_ahead: false,
            advisory_locking: false,
        }
    }
}

/// Memory mapping statistics
#[derive(Debug, Clone, Default)]
pub struct MemoryMapStats {
    /// Total bytes mapped
    pub bytes_mapped: u64,
    /// Number of active mappings
    pub active_mappings: usize,
    /// Number of failed mapping attempts
    pub failed_mappings: usize,
    /// Number of fallback operations to regular I/O
    pub fallback_operations: usize,
}

/// Cross-platform memory-mapped file wrapper with security boundaries
#[cfg(feature = "mmap")]
pub struct MemoryMappedArchive {
    /// Memory mapping
    mmap: Mmap,
    /// Configuration
    #[allow(dead_code)] // Used for future functionality
    config: MemoryMapConfig,
    /// Security limits
    security_limits: SecurityLimits,
    /// Session tracker for resource monitoring
    #[allow(dead_code)] // Used for future functionality
    session_tracker: Arc<SessionTracker>,
    /// File size
    file_size: u64,
    /// Statistics
    stats: MemoryMapStats,
}

#[cfg(feature = "mmap")]
impl std::fmt::Debug for MemoryMappedArchive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryMappedArchive")
            .field("file_size", &self.file_size)
            .field("config", &self.config)
            .field("security_limits", &self.security_limits)
            .field("stats", &self.stats)
            .finish()
    }
}

#[cfg(feature = "mmap")]
impl MemoryMappedArchive {
    /// Create a new memory-mapped archive from a file
    pub fn new<P: AsRef<Path>>(
        path: P,
        config: MemoryMapConfig,
        security_limits: SecurityLimits,
        session_tracker: Arc<SessionTracker>,
    ) -> Result<Self> {
        // Check if memory mapping is enabled
        if !config.enable_mapping {
            return Err(Error::unsupported_feature(
                "Memory mapping is disabled in configuration",
            ));
        }

        // Open the file
        let file = File::open(&path).map_err(|e| {
            Error::io_error(format!("Failed to open file for memory mapping: {}", e))
        })?;

        // Get file size
        let file_size = file
            .metadata()
            .map_err(|e| Error::io_error(format!("Failed to get file metadata: {}", e)))?
            .len();

        // Validate file size against security limits
        Self::validate_file_size(file_size, &config, &security_limits)?;

        // Create memory mapping with security options
        let mmap = Self::create_secure_mmap(&file, file_size, &config)?;

        // Create statistics with proper initialization
        let stats = MemoryMapStats {
            bytes_mapped: file_size,
            active_mappings: 1,
            ..Default::default()
        };

        Ok(Self {
            mmap,
            config,
            security_limits,
            session_tracker,
            file_size,
            stats,
        })
    }

    /// Create a memory-mapped archive from an existing file handle
    pub fn from_file(
        file: File,
        config: MemoryMapConfig,
        security_limits: SecurityLimits,
        session_tracker: Arc<SessionTracker>,
    ) -> Result<Self> {
        if !config.enable_mapping {
            return Err(Error::unsupported_feature(
                "Memory mapping is disabled in configuration",
            ));
        }

        let file_size = file
            .metadata()
            .map_err(|e| Error::io_error(format!("Failed to get file metadata: {}", e)))?
            .len();

        Self::validate_file_size(file_size, &config, &security_limits)?;

        let mmap = Self::create_secure_mmap(&file, file_size, &config)?;

        let stats = MemoryMapStats {
            bytes_mapped: file_size,
            active_mappings: 1,
            ..Default::default()
        };

        Ok(Self {
            mmap,
            config,
            security_limits,
            session_tracker,
            file_size,
            stats,
        })
    }

    /// Validate file size against security limits
    fn validate_file_size(
        file_size: u64,
        config: &MemoryMapConfig,
        security_limits: &SecurityLimits,
    ) -> Result<()> {
        if file_size == 0 {
            return Err(Error::invalid_format("Cannot memory map empty file"));
        }

        if file_size > config.max_map_size {
            return Err(Error::resource_exhaustion(format!(
                "File size {} exceeds memory mapping limit {}",
                file_size, config.max_map_size
            )));
        }

        if file_size > security_limits.max_archive_size {
            return Err(Error::resource_exhaustion(format!(
                "File size {} exceeds security archive size limit {}",
                file_size, security_limits.max_archive_size
            )));
        }

        Ok(())
    }

    /// Create secure memory mapping with platform-specific optimizations
    fn create_secure_mmap(file: &File, file_size: u64, config: &MemoryMapConfig) -> Result<Mmap> {
        let mut mmap_options = MmapOptions::new();

        // Configure read-ahead if enabled
        if config.read_ahead {
            // Platform-specific read-ahead hints will be applied by memmap2
        }

        // Create the memory mapping
        let mmap = unsafe {
            mmap_options
                .len(file_size as usize)
                .map(file)
                .map_err(|e| Error::io_error(format!("Failed to create memory mapping: {}", e)))?
        };

        // Apply platform-specific optimizations
        #[cfg(unix)]
        {
            Self::apply_unix_optimizations(&mmap, config)?;
        }

        #[cfg(windows)]
        {
            Self::apply_windows_optimizations(&mmap, config)?;
        }

        Ok(mmap)
    }

    /// Apply Unix-specific optimizations
    #[cfg(unix)]
    fn apply_unix_optimizations(mmap: &Mmap, config: &MemoryMapConfig) -> Result<()> {
        // Apply madvise hints for better performance
        // Set appropriate madvise flags based on configuration
        let advice = if config.read_ahead {
            libc::MADV_SEQUENTIAL | libc::MADV_WILLNEED
        } else {
            libc::MADV_RANDOM
        };

        unsafe {
            let result = libc::madvise(mmap.as_ptr() as *mut libc::c_void, mmap.len(), advice);

            if result != 0 {
                log::warn!(
                    "Failed to apply madvise optimization: {}",
                    std::io::Error::last_os_error()
                );
                // Don't fail on madvise errors - they're just hints
            }
        }

        Ok(())
    }

    /// Apply Windows-specific optimizations
    #[cfg(windows)]
    fn apply_windows_optimizations(_mmap: &Mmap, _config: &MemoryMapConfig) -> Result<()> {
        // Windows-specific memory mapping optimizations could be added here
        // For now, we rely on memmap2's cross-platform defaults
        Ok(())
    }

    /// Safely read data from memory mapping with bounds checking
    pub fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<()> {
        self.validate_read_bounds(offset, buf.len())?;

        let start = offset as usize;
        let end = start + buf.len();

        // Safe bounds-checked copy
        buf.copy_from_slice(&self.mmap[start..end]);

        Ok(())
    }

    /// Get a safe slice from memory mapping with bounds checking
    pub fn get_slice(&self, offset: u64, len: usize) -> Result<&[u8]> {
        self.validate_read_bounds(offset, len)?;

        let start = offset as usize;
        let end = start + len;

        Ok(&self.mmap[start..end])
    }

    /// Validate read bounds against memory mapping size
    fn validate_read_bounds(&self, offset: u64, len: usize) -> Result<()> {
        let start = offset;
        let end = offset.saturating_add(len as u64);

        if start >= self.file_size {
            return Err(Error::invalid_bounds(format!(
                "Read offset {} beyond file size {}",
                start, self.file_size
            )));
        }

        if end > self.file_size {
            return Err(Error::invalid_bounds(format!(
                "Read end {} beyond file size {}",
                end, self.file_size
            )));
        }

        if len > self.security_limits.max_decompressed_size as usize {
            return Err(Error::resource_exhaustion(format!(
                "Read length {} exceeds security limit {}",
                len, self.security_limits.max_decompressed_size
            )));
        }

        Ok(())
    }

    /// Get file size
    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    /// Get memory mapping statistics
    pub fn stats(&self) -> &MemoryMapStats {
        &self.stats
    }

    /// Check if memory mapping is active and healthy
    pub fn is_healthy(&self) -> bool {
        self.mmap.len() == self.file_size as usize
    }

    /// Synchronize memory mapping with underlying file (if writable)
    pub fn sync(&self) -> Result<()> {
        // For read-only mappings, this is a no-op
        // Future versions could support writable mappings
        Ok(())
    }

    /// Get raw memory mapping for advanced use cases
    pub fn as_slice(&self) -> &[u8] {
        &self.mmap
    }
}

#[cfg(feature = "mmap")]
impl Drop for MemoryMappedArchive {
    fn drop(&mut self) {
        // Update statistics
        self.stats.active_mappings = 0;

        // Memory mapping will be automatically unmapped by memmap2
        log::debug!("Unmapping memory-mapped archive ({} bytes)", self.file_size);
    }
}

/// Memory mapping manager for handling multiple archives
#[cfg(feature = "mmap")]
#[derive(Debug)]
pub struct MemoryMapManager {
    config: MemoryMapConfig,
    security_limits: SecurityLimits,
    session_tracker: Arc<SessionTracker>,
    global_stats: MemoryMapStats,
}

#[cfg(feature = "mmap")]
impl MemoryMapManager {
    /// Create a new memory mapping manager
    pub fn new(
        config: MemoryMapConfig,
        security_limits: SecurityLimits,
        session_tracker: Arc<SessionTracker>,
    ) -> Self {
        Self {
            config,
            security_limits,
            session_tracker,
            global_stats: MemoryMapStats::default(),
        }
    }

    /// Create a memory-mapped archive
    pub fn create_mapping<P: AsRef<Path>>(&mut self, path: P) -> Result<MemoryMappedArchive> {
        match MemoryMappedArchive::new(
            path,
            self.config.clone(),
            self.security_limits.clone(),
            self.session_tracker.clone(),
        ) {
            Ok(mmap) => {
                self.global_stats.bytes_mapped += mmap.file_size();
                self.global_stats.active_mappings += 1;
                Ok(mmap)
            }
            Err(e) => {
                self.global_stats.failed_mappings += 1;
                Err(e)
            }
        }
    }

    /// Record a fallback operation
    pub fn record_fallback(&mut self) {
        self.global_stats.fallback_operations += 1;
    }

    /// Get global statistics
    pub fn global_stats(&self) -> &MemoryMapStats {
        &self.global_stats
    }

    /// Check if memory mapping should be attempted based on current state
    pub fn should_attempt_mapping(&self, file_size: u64) -> bool {
        if !self.config.enable_mapping {
            return false;
        }

        if file_size > self.config.max_map_size {
            return false;
        }

        if file_size > self.security_limits.max_archive_size {
            return false;
        }

        // Consider fallback rate - if too many mappings are failing,
        // temporarily disable to avoid overhead
        let total_attempts = self.global_stats.active_mappings + self.global_stats.failed_mappings;
        if total_attempts > 10 {
            let failure_rate = self.global_stats.failed_mappings as f64 / total_attempts as f64;
            if failure_rate > 0.5 {
                log::warn!(
                    "High memory mapping failure rate ({:.1}%), temporarily disabling",
                    failure_rate * 100.0
                );
                return false;
            }
        }

        true
    }
}

// Provide stub implementations when mmap feature is disabled
/// Stub implementation of memory-mapped archive when feature is disabled
///
/// This implementation always returns an error indicating that memory mapping
/// support was not compiled in and the feature needs to be enabled.
#[cfg(not(feature = "mmap"))]
#[derive(Debug)]
pub struct MemoryMappedArchive;

#[cfg(not(feature = "mmap"))]
impl MemoryMappedArchive {
    /// Create a new memory-mapped archive - stub implementation
    ///
    /// # Errors
    /// Always returns an error indicating memory mapping is not available
    pub fn new<P: AsRef<std::path::Path>>(
        _path: P,
        _config: MemoryMapConfig,
        _security_limits: crate::security::SecurityLimits,
        _session_tracker: std::sync::Arc<crate::security::SessionTracker>,
    ) -> crate::Result<Self> {
        Err(crate::Error::unsupported_feature(
            "Memory mapping support not compiled in - enable 'mmap' feature",
        ))
    }
}

/// Stub implementation of memory mapping manager when feature is disabled
///
/// This provides a no-op implementation for compatibility when memory mapping
/// is not available.
#[cfg(not(feature = "mmap"))]
#[derive(Debug)]
pub struct MemoryMapManager;

#[cfg(not(feature = "mmap"))]
impl MemoryMapManager {
    /// Create a new memory mapping manager - stub implementation
    pub fn new(
        _config: MemoryMapConfig,
        _security_limits: crate::security::SecurityLimits,
        _session_tracker: std::sync::Arc<crate::security::SessionTracker>,
    ) -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "mmap")]
    use crate::security::SecurityLimits;
    #[cfg(feature = "mmap")]
    use std::io::Write;
    #[cfg(feature = "mmap")]
    use tempfile::NamedTempFile;

    #[test]
    fn test_memory_map_config_defaults() {
        let config = MemoryMapConfig::default();
        assert!(config.enable_mapping);
        assert!(config.read_ahead);
        assert!(!config.advisory_locking);
        assert_eq!(config.max_map_size, 2 * 1024 * 1024 * 1024);
    }

    #[test]
    fn test_memory_map_config_variants() {
        let strict = MemoryMapConfig::strict();
        let permissive = MemoryMapConfig::permissive();
        let disabled = MemoryMapConfig::disabled();

        assert!(strict.max_map_size < permissive.max_map_size);
        assert!(strict.advisory_locking);
        assert!(!strict.read_ahead);

        assert!(!disabled.enable_mapping);
        assert_eq!(disabled.max_map_size, 0);
    }

    #[test]
    fn test_memory_map_stats_default() {
        let stats = MemoryMapStats::default();
        assert_eq!(stats.bytes_mapped, 0);
        assert_eq!(stats.active_mappings, 0);
        assert_eq!(stats.failed_mappings, 0);
        assert_eq!(stats.fallback_operations, 0);
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn test_memory_mapped_archive_creation() -> Result<()> {
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Hello, memory mapped world! This is test data for validation.";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();

        let config = MemoryMapConfig::default();
        let security_limits = SecurityLimits::default();
        let session_tracker = Arc::new(crate::security::SessionTracker::new());

        let mmap_archive =
            MemoryMappedArchive::new(temp_file.path(), config, security_limits, session_tracker)?;

        assert_eq!(mmap_archive.file_size(), test_data.len() as u64);
        assert!(mmap_archive.is_healthy());
        assert_eq!(mmap_archive.stats().bytes_mapped, test_data.len() as u64);
        assert_eq!(mmap_archive.stats().active_mappings, 1);

        Ok(())
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn test_memory_mapped_archive_read_at() -> Result<()> {
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();

        let config = MemoryMapConfig::default();
        let security_limits = SecurityLimits::default();
        let session_tracker = Arc::new(crate::security::SessionTracker::new());

        let mmap_archive =
            MemoryMappedArchive::new(temp_file.path(), config, security_limits, session_tracker)?;

        // Test reading from different positions
        let mut buf = [0u8; 5];
        mmap_archive.read_at(0, &mut buf)?;
        assert_eq!(&buf, b"01234");

        mmap_archive.read_at(10, &mut buf)?;
        assert_eq!(&buf, b"ABCDE");

        mmap_archive.read_at(30, &mut buf)?;
        assert_eq!(&buf, b"UVWXY");

        Ok(())
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn test_memory_mapped_archive_bounds_checking() -> Result<()> {
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Short data";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();

        let config = MemoryMapConfig::default();
        let security_limits = SecurityLimits::default();
        let session_tracker = Arc::new(crate::security::SessionTracker::new());

        let mmap_archive =
            MemoryMappedArchive::new(temp_file.path(), config, security_limits, session_tracker)?;

        // Test reading beyond file size
        let mut buf = [0u8; 5];
        let result = mmap_archive.read_at(test_data.len() as u64, &mut buf);
        assert!(result.is_err());

        // Test reading past end
        let result = mmap_archive.read_at(8, &mut buf);
        assert!(result.is_err());

        Ok(())
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn test_memory_mapped_archive_get_slice() -> Result<()> {
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Memory mapped slice test data";
        temp_file.write_all(test_data).unwrap();
        temp_file.flush().unwrap();

        let config = MemoryMapConfig::default();
        let security_limits = SecurityLimits::default();
        let session_tracker = Arc::new(crate::security::SessionTracker::new());

        let mmap_archive =
            MemoryMappedArchive::new(temp_file.path(), config, security_limits, session_tracker)?;

        // Test getting slices
        let slice = mmap_archive.get_slice(0, 6)?;
        assert_eq!(slice, b"Memory");

        let slice = mmap_archive.get_slice(7, 6)?;
        assert_eq!(slice, b"mapped");

        let slice = mmap_archive.get_slice(14, 5)?;
        assert_eq!(slice, b"slice");

        Ok(())
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn test_file_size_validation() {
        let config = MemoryMapConfig::strict(); // 256MB limit
        let security_limits = SecurityLimits::strict(); // 1GB limit

        // Valid size
        let result =
            MemoryMappedArchive::validate_file_size(100 * 1024 * 1024, &config, &security_limits);
        assert!(result.is_ok());

        // Exceeds config limit
        let result =
            MemoryMappedArchive::validate_file_size(300 * 1024 * 1024, &config, &security_limits);
        assert!(result.is_err());

        // Exceeds security limit
        let large_config = MemoryMapConfig::permissive(); // 8GB limit
        let result = MemoryMappedArchive::validate_file_size(
            2 * 1024 * 1024 * 1024,
            &large_config,
            &security_limits,
        );
        assert!(result.is_err());

        // Empty file
        let result = MemoryMappedArchive::validate_file_size(0, &config, &security_limits);
        assert!(result.is_err());
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn test_memory_map_manager() -> Result<()> {
        let config = MemoryMapConfig::default();
        let security_limits = SecurityLimits::default();
        let session_tracker = Arc::new(crate::security::SessionTracker::new());

        let mut manager = MemoryMapManager::new(config, security_limits, session_tracker);

        // Test should_attempt_mapping
        assert!(manager.should_attempt_mapping(1024 * 1024)); // 1MB
        assert!(!manager.should_attempt_mapping(10 * 1024 * 1024 * 1024)); // 10GB

        // Test fallback recording
        manager.record_fallback();
        assert_eq!(manager.global_stats().fallback_operations, 1);

        Ok(())
    }

    #[test]
    fn test_disabled_feature_stubs() {
        #[cfg(not(feature = "mmap"))]
        {
            let config = MemoryMapConfig::default();
            let security_limits = crate::security::SecurityLimits::default();
            let session_tracker = std::sync::Arc::new(crate::security::SessionTracker::new());

            let result = MemoryMappedArchive::new(
                "/nonexistent/path",
                config.clone(),
                security_limits.clone(),
                session_tracker.clone(),
            );
            assert!(result.is_err());

            let _manager = MemoryMapManager::new(config, security_limits, session_tracker);
        }
    }
}
