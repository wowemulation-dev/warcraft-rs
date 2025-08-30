//! Optional async I/O support with resource protection
//!
//! Provides non-blocking archive operations while maintaining security boundaries
//! and preventing resource exhaustion in async contexts.

#[cfg(feature = "async")]
use crate::security::{SecurityLimits, SessionTracker};
#[cfg(feature = "async")]
use crate::{Error, Result};
#[cfg(feature = "async")]
use std::sync::Arc;
#[cfg(feature = "async")]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(feature = "async")]
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt};
#[cfg(feature = "async")]
use tokio::sync::{Mutex, Semaphore};
#[cfg(feature = "async")]
use tokio::time::{Duration, Instant, timeout};

/// Configuration for async I/O operations with security limits
#[cfg(feature = "async")]
#[derive(Debug, Clone)]
pub struct AsyncConfig {
    /// Maximum concurrent operations per session
    pub max_concurrent_ops: usize,
    /// Timeout for individual I/O operations
    pub operation_timeout: Duration,
    /// Maximum memory usage for async buffers
    pub max_async_memory: usize,
    /// Enable detailed async metrics collection
    pub collect_metrics: bool,
    /// Maximum number of files that can be extracted concurrently
    pub max_concurrent_extractions: usize,
    /// Buffer size for async operations
    pub buffer_size: usize,
}

#[cfg(feature = "async")]
impl Default for AsyncConfig {
    fn default() -> Self {
        Self {
            max_concurrent_ops: 10,
            operation_timeout: Duration::from_secs(30),
            max_async_memory: 64 * 1024 * 1024, // 64MB
            collect_metrics: false,
            max_concurrent_extractions: 5,
            buffer_size: 64 * 1024, // 64KB
        }
    }
}

/// Metrics for async operations
#[cfg(feature = "async")]
#[derive(Debug, Default)]
pub struct AsyncMetrics {
    /// Total number of operations started
    pub total_operations: AtomicU64,
    /// Number of completed operations
    pub completed_operations: AtomicU64,
    /// Number of cancelled operations
    pub cancelled_operations: AtomicU64,
    /// Number of timeout operations
    pub timeout_operations: AtomicU64,
    /// Peak memory usage during async operations
    pub peak_memory_usage: AtomicU64,
    /// Current active operations
    pub active_operations: AtomicU64,
    /// Total bytes read asynchronously
    pub total_bytes_read: AtomicU64,
}

#[cfg(feature = "async")]
impl AsyncMetrics {
    /// Create new async metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record the start of an operation
    pub fn record_operation_start(&self) {
        self.total_operations.fetch_add(1, Ordering::Relaxed);
        self.active_operations.fetch_add(1, Ordering::Relaxed);
    }

    /// Record operation completion
    pub fn record_operation_complete(&self, bytes_read: u64) {
        self.completed_operations.fetch_add(1, Ordering::Relaxed);
        self.active_operations.fetch_sub(1, Ordering::Relaxed);
        self.total_bytes_read
            .fetch_add(bytes_read, Ordering::Relaxed);
    }

    /// Record operation cancellation
    pub fn record_operation_cancelled(&self) {
        self.cancelled_operations.fetch_add(1, Ordering::Relaxed);
        self.active_operations.fetch_sub(1, Ordering::Relaxed);
    }

    /// Record operation timeout
    pub fn record_operation_timeout(&self) {
        self.timeout_operations.fetch_add(1, Ordering::Relaxed);
        self.active_operations.fetch_sub(1, Ordering::Relaxed);
    }

    /// Update peak memory usage
    pub fn update_peak_memory(&self, current_usage: u64) {
        let mut current_peak = self.peak_memory_usage.load(Ordering::Relaxed);
        loop {
            if current_usage <= current_peak {
                break;
            }
            match self.peak_memory_usage.compare_exchange_weak(
                current_peak,
                current_usage,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_peak = actual,
            }
        }
    }

    /// Get current statistics
    pub fn get_stats(&self) -> AsyncOperationStats {
        AsyncOperationStats {
            total_operations: self.total_operations.load(Ordering::Relaxed),
            completed_operations: self.completed_operations.load(Ordering::Relaxed),
            cancelled_operations: self.cancelled_operations.load(Ordering::Relaxed),
            timeout_operations: self.timeout_operations.load(Ordering::Relaxed),
            peak_memory_usage: self.peak_memory_usage.load(Ordering::Relaxed),
            active_operations: self.active_operations.load(Ordering::Relaxed),
            total_bytes_read: self.total_bytes_read.load(Ordering::Relaxed),
        }
    }
}

/// Statistics snapshot for async operations
#[cfg(feature = "async")]
#[derive(Debug, Clone)]
pub struct AsyncOperationStats {
    /// Total number of operations started
    pub total_operations: u64,
    /// Number of completed operations
    pub completed_operations: u64,
    /// Number of cancelled operations  
    pub cancelled_operations: u64,
    /// Number of timeout operations
    pub timeout_operations: u64,
    /// Peak memory usage during operations
    pub peak_memory_usage: u64,
    /// Current active operations
    pub active_operations: u64,
    /// Total bytes read across all operations
    pub total_bytes_read: u64,
}

/// Async-aware MPQ archive reader with resource protection
#[cfg(feature = "async")]
#[derive(Debug)]
pub struct AsyncArchiveReader<R> {
    reader: Arc<Mutex<R>>,
    config: AsyncConfig,
    session_tracker: Arc<SessionTracker>,
    active_operations: Arc<Semaphore>,
    extraction_semaphore: Arc<Semaphore>,
    metrics: Arc<AsyncMetrics>,
    security_limits: SecurityLimits,
}

#[cfg(feature = "async")]
impl<R: AsyncRead + AsyncSeek + Unpin + Send + 'static> AsyncArchiveReader<R> {
    /// Create a new async archive reader with default configuration
    pub fn new(reader: R, session_tracker: Arc<SessionTracker>) -> Self {
        Self::with_config(reader, AsyncConfig::default(), session_tracker)
    }

    /// Create a new async archive reader with custom configuration
    pub fn with_config(
        reader: R,
        config: AsyncConfig,
        session_tracker: Arc<SessionTracker>,
    ) -> Self {
        let active_operations = Arc::new(Semaphore::new(config.max_concurrent_ops));
        let extraction_semaphore = Arc::new(Semaphore::new(config.max_concurrent_extractions));
        let metrics = if config.collect_metrics {
            Arc::new(AsyncMetrics::new())
        } else {
            Arc::new(AsyncMetrics::default())
        };

        Self {
            reader: Arc::new(Mutex::new(reader)),
            config,
            session_tracker,
            active_operations,
            extraction_semaphore,
            metrics,
            security_limits: SecurityLimits::default(),
        }
    }

    /// Create with custom security limits
    pub fn with_security_limits(
        reader: R,
        config: AsyncConfig,
        session_tracker: Arc<SessionTracker>,
        security_limits: SecurityLimits,
    ) -> Self {
        let mut async_reader = Self::with_config(reader, config, session_tracker);
        async_reader.security_limits = security_limits;
        async_reader
    }

    /// Read data at a specific offset with timeout and resource protection
    pub async fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        // Acquire operation permit
        let _permit = self.active_operations.acquire().await.map_err(|_| {
            self.metrics.record_operation_cancelled();
            Error::resource_exhaustion("Failed to acquire operation permit - system overloaded")
        })?;

        self.metrics.record_operation_start();

        // Apply timeout to the entire operation
        let result = timeout(self.config.operation_timeout, async {
            // Validate buffer size against security limits
            if buffer.len() > self.config.max_async_memory {
                return Err(Error::resource_exhaustion(
                    "Read buffer exceeds maximum allowed size for async operations",
                ));
            }

            // Update memory usage tracking
            self.metrics.update_peak_memory(buffer.len() as u64);

            // Perform the actual read
            let mut reader = self.reader.lock().await;
            reader.seek(std::io::SeekFrom::Start(offset)).await?;
            let bytes_read = reader.read(buffer).await?;

            Ok(bytes_read)
        })
        .await;

        match result {
            Ok(Ok(bytes_read)) => {
                self.metrics.record_operation_complete(bytes_read as u64);
                Ok(bytes_read)
            }
            Ok(Err(e)) => {
                self.metrics.record_operation_cancelled();
                Err(e)
            }
            Err(_) => {
                self.metrics.record_operation_timeout();
                Err(Error::resource_exhaustion(
                    "Async read operation timed out - potential DoS protection activated",
                ))
            }
        }
    }

    /// Read an exact number of bytes at a specific offset with security validation
    pub async fn read_exact_at(&self, offset: u64, buffer: &mut [u8]) -> Result<()> {
        let mut total_read = 0;
        let mut current_offset = offset;

        while total_read < buffer.len() {
            let bytes_read = self
                .read_at(current_offset, &mut buffer[total_read..])
                .await?;

            if bytes_read == 0 {
                return Err(Error::invalid_format(
                    "Unexpected end of file during async read operation",
                ));
            }

            total_read += bytes_read;
            current_offset += bytes_read as u64;
        }

        Ok(())
    }

    /// Perform multiple file extractions concurrently with bounded parallelism
    pub async fn extract_files_concurrent(
        &self,
        file_requests: Vec<(String, u64, u64)>, // (filename, offset, size)
    ) -> Result<Vec<(String, Vec<u8>)>> {
        if file_requests.len() > self.config.max_concurrent_extractions * 2 {
            return Err(Error::resource_exhaustion(
                "Too many concurrent file extraction requests - potential resource exhaustion",
            ));
        }

        // Check session limits for all files combined
        let total_bytes: u64 = file_requests.iter().map(|(_, _, size)| *size).sum();
        self.session_tracker
            .check_session_limits_with_addition(total_bytes, &self.security_limits)?;

        let mut handles = Vec::new();

        for (filename, offset, size) in file_requests {
            // Validate individual file size
            if size > self.security_limits.max_decompressed_size {
                return Err(Error::resource_exhaustion(format!(
                    "File '{}' exceeds maximum size limit for async extraction",
                    filename
                )));
            }

            let reader = Arc::clone(&self.reader);
            let extraction_permit = Arc::clone(&self.extraction_semaphore);
            let metrics = Arc::clone(&self.metrics);
            let config = self.config.clone();

            let handle = tokio::spawn(async move {
                let _permit = extraction_permit.acquire().await.map_err(|_| {
                    Error::resource_exhaustion("Failed to acquire extraction permit")
                })?;

                metrics.record_operation_start();

                let result = timeout(config.operation_timeout, async {
                    let mut buffer = vec![0u8; size as usize];
                    let mut reader = reader.lock().await;
                    reader.seek(std::io::SeekFrom::Start(offset)).await?;
                    reader.read_exact(&mut buffer).await?;
                    Ok((filename.clone(), buffer))
                })
                .await;

                match result {
                    Ok(Ok((filename, data))) => {
                        metrics.record_operation_complete(size);
                        Ok((filename, data))
                    }
                    Ok(Err(e)) => {
                        metrics.record_operation_cancelled();
                        Err(e)
                    }
                    Err(_) => {
                        metrics.record_operation_timeout();
                        Err(Error::resource_exhaustion(format!(
                            "Extraction of '{}' timed out - potential DoS protection activated",
                            filename
                        )))
                    }
                }
            });

            handles.push(handle);
        }

        // Collect all results
        let mut results = Vec::new();
        for handle in handles {
            let result = handle
                .await
                .map_err(|e| Error::resource_exhaustion(format!("Async task failed: {}", e)))??;
            results.push(result);
        }

        // Update session tracker with total bytes extracted
        self.session_tracker.record_decompression(total_bytes);

        Ok(results)
    }

    /// Create a decompression monitor for async operations
    pub fn create_decompression_monitor(
        &self,
        expected_size: u64,
        compression_method: u8,
        file_path: Option<&str>,
    ) -> Result<Arc<AsyncDecompressionMonitor>> {
        // Validate decompression request
        crate::security::validate_decompression_operation(
            0, // Compressed size not known at this point
            expected_size,
            compression_method,
            file_path,
            &self.session_tracker,
            &self.security_limits,
        )?;

        Ok(Arc::new(AsyncDecompressionMonitor::new(
            expected_size.min(self.security_limits.max_decompressed_size),
            self.security_limits.max_decompression_time,
            self.config.buffer_size,
        )))
    }

    /// Get current async operation statistics
    pub fn get_stats(&self) -> AsyncOperationStats {
        self.metrics.get_stats()
    }

    /// Check if the reader is under resource pressure
    pub fn is_under_pressure(&self) -> bool {
        let available_ops = self.active_operations.available_permits();
        let available_extractions = self.extraction_semaphore.available_permits();

        // Consider under pressure if less than 20% permits available
        // Use max(1, x/5) to ensure we detect pressure even with small limits
        let ops_threshold = std::cmp::max(1, self.config.max_concurrent_ops / 5);
        let extraction_threshold = std::cmp::max(1, self.config.max_concurrent_extractions / 5);

        available_ops < ops_threshold || available_extractions < extraction_threshold
    }

    /// Force cancellation of all pending operations (for cleanup)
    pub async fn shutdown(&self) -> Result<()> {
        // Close semaphores to prevent new operations
        self.active_operations.close();
        self.extraction_semaphore.close();

        // Give existing operations a moment to complete
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(())
    }
}

/// Async-aware decompression monitor with progress tracking
#[cfg(feature = "async")]
#[derive(Debug)]
pub struct AsyncDecompressionMonitor {
    max_size: u64,
    max_time: Duration,
    buffer_size: usize,
    start_time: Instant,
    bytes_decompressed: AtomicU64,
    should_cancel: AtomicU64,
}

#[cfg(feature = "async")]
impl AsyncDecompressionMonitor {
    /// Create a new async decompression monitor
    pub fn new(max_size: u64, max_time: Duration, buffer_size: usize) -> Self {
        Self {
            max_size,
            max_time,
            buffer_size,
            start_time: Instant::now(),
            bytes_decompressed: AtomicU64::new(0),
            should_cancel: AtomicU64::new(0),
        }
    }

    /// Check if decompression should continue (async-safe)
    pub async fn check_progress(&self, current_output_size: u64) -> Result<()> {
        // Check size limits
        if current_output_size > self.max_size {
            return Err(Error::resource_exhaustion(
                "Async decompression size limit exceeded - potential compression bomb",
            ));
        }

        // Check time limits
        if self.start_time.elapsed() > self.max_time {
            return Err(Error::resource_exhaustion(
                "Async decompression time limit exceeded - potential DoS attack",
            ));
        }

        // Check if cancellation was requested
        if self.should_cancel.load(Ordering::Relaxed) != 0 {
            return Err(Error::resource_exhaustion(
                "Async decompression cancelled due to security limits",
            ));
        }

        // Update current progress
        self.bytes_decompressed
            .store(current_output_size, Ordering::Relaxed);

        // Yield control to allow other tasks to run
        tokio::task::yield_now().await;

        Ok(())
    }

    /// Request cancellation of decompression
    pub fn request_cancellation(&self) {
        self.should_cancel.store(1, Ordering::Relaxed);
    }

    /// Get recommended buffer size for async operations
    pub fn get_buffer_size(&self) -> usize {
        self.buffer_size
    }

    /// Get current statistics
    pub fn get_stats(&self) -> (u64, Duration) {
        (
            self.bytes_decompressed.load(Ordering::Relaxed),
            self.start_time.elapsed(),
        )
    }
}

#[cfg(test)]
#[cfg(feature = "async")]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_async_config_default() {
        let config = AsyncConfig::default();
        assert_eq!(config.max_concurrent_ops, 10);
        assert_eq!(config.operation_timeout, Duration::from_secs(30));
        assert_eq!(config.max_async_memory, 64 * 1024 * 1024);
        assert!(!config.collect_metrics);
        assert_eq!(config.max_concurrent_extractions, 5);
        assert_eq!(config.buffer_size, 64 * 1024);
    }

    #[tokio::test]
    async fn test_async_metrics() {
        let metrics = AsyncMetrics::new();

        metrics.record_operation_start();
        metrics.record_operation_complete(1024);

        let stats = metrics.get_stats();
        assert_eq!(stats.total_operations, 1);
        assert_eq!(stats.completed_operations, 1);
        assert_eq!(stats.total_bytes_read, 1024);
        assert_eq!(stats.active_operations, 0);
    }

    #[tokio::test]
    async fn test_async_reader_creation() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let cursor = Cursor::new(data);
        let session = Arc::new(SessionTracker::new());

        let reader = AsyncArchiveReader::new(cursor, session);
        assert!(!reader.is_under_pressure());
    }

    #[tokio::test]
    async fn test_async_read_at() {
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let cursor = Cursor::new(data);
        let session = Arc::new(SessionTracker::new());

        let reader = AsyncArchiveReader::new(cursor, session);

        let mut buffer = [0u8; 4];
        let bytes_read = reader.read_at(5, &mut buffer).await.unwrap();
        assert_eq!(bytes_read, 4);
        assert_eq!(buffer, [5, 6, 7, 8]);
    }

    #[tokio::test]
    async fn test_async_read_exact_at() {
        let data = vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
        let cursor = Cursor::new(data);
        let session = Arc::new(SessionTracker::new());

        let reader = AsyncArchiveReader::new(cursor, session);

        let mut buffer = [0u8; 3];
        reader.read_exact_at(2, &mut buffer).await.unwrap();
        assert_eq!(buffer, [30, 40, 50]);
    }

    #[tokio::test]
    async fn test_async_read_oversized_buffer() {
        let data = vec![1, 2, 3];
        let cursor = Cursor::new(data);
        let session = Arc::new(SessionTracker::new());

        let config = AsyncConfig {
            max_async_memory: 2, // Very small limit
            ..Default::default()
        };

        let reader = AsyncArchiveReader::with_config(cursor, config, session);

        let mut buffer = [0u8; 5]; // Exceeds limit
        let result = reader.read_at(0, &mut buffer).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("exceeds maximum allowed size")
        );
    }

    #[tokio::test]
    async fn test_concurrent_file_extraction() {
        let data = vec![0u8; 1000]; // 1KB of zeros
        let cursor = Cursor::new(data);
        let session = Arc::new(SessionTracker::new());

        let reader = AsyncArchiveReader::new(cursor, session);

        let requests = vec![
            ("file1.txt".to_string(), 0, 100),
            ("file2.txt".to_string(), 100, 100),
            ("file3.txt".to_string(), 200, 100),
        ];

        let results = reader.extract_files_concurrent(requests).await.unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, "file1.txt");
        assert_eq!(results[0].1.len(), 100);
    }

    #[tokio::test]
    async fn test_too_many_concurrent_extractions() {
        let data = vec![0u8; 1000];
        let cursor = Cursor::new(data);
        let session = Arc::new(SessionTracker::new());

        let config = AsyncConfig {
            max_concurrent_extractions: 2,
            ..Default::default()
        };

        let reader = AsyncArchiveReader::with_config(cursor, config, session);

        // Request more than max_concurrent_extractions * 2
        let requests = (0..6)
            .map(|i| (format!("file{}.txt", i), i * 100, 50))
            .collect();

        let result = reader.extract_files_concurrent(requests).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Too many concurrent")
        );
    }

    #[tokio::test]
    async fn test_decompression_monitor() {
        let monitor = AsyncDecompressionMonitor::new(1024, Duration::from_millis(100), 64);

        // Normal operation should succeed
        monitor.check_progress(512).await.unwrap();

        // Exceeding size limit should fail
        let result = monitor.check_progress(2048).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("size limit exceeded")
        );
    }

    #[tokio::test]
    async fn test_decompression_monitor_cancellation() {
        let monitor = AsyncDecompressionMonitor::new(1024, Duration::from_secs(10), 64);

        monitor.request_cancellation();

        let result = monitor.check_progress(512).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cancelled"));
    }

    #[tokio::test]
    async fn test_reader_shutdown() {
        let data = vec![1, 2, 3, 4, 5];
        let cursor = Cursor::new(data);
        let session = Arc::new(SessionTracker::new());

        let reader = AsyncArchiveReader::new(cursor, session);

        // Should shutdown cleanly
        reader.shutdown().await.unwrap();

        // New operations should fail after shutdown
        let mut buffer = [0u8; 2];
        let result = reader.read_at(0, &mut buffer).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_resource_pressure_detection() {
        let data = vec![1, 2, 3, 4, 5];
        let cursor = Cursor::new(data);
        let session = Arc::new(SessionTracker::new());

        let config = AsyncConfig {
            max_concurrent_ops: 1, // Very limited
            ..Default::default()
        };

        let reader = AsyncArchiveReader::with_config(cursor, config, session);

        // Initially should not be under pressure
        assert!(!reader.is_under_pressure());

        // Start an operation that holds the permit
        let _permit = reader.active_operations.acquire().await.unwrap();

        // Now should be under pressure
        assert!(reader.is_under_pressure());
    }
}
