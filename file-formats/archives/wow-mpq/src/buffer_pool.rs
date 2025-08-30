//! Thread-safe buffer pooling for performance optimization
//!
//! This module provides a bounded capacity buffer pool to reduce allocation overhead
//! during file extraction and decompression operations. Buffers are categorized by size
//! to optimize memory usage patterns.

use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

/// Buffer size categories optimized for different MPQ operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferSize {
    /// Small buffers (4KB) - for sector reads and small file operations
    Small = 4096,
    /// Medium buffers (64KB) - for typical file extraction
    Medium = 65536,
    /// Large buffers (1MB) - for bulk operations and large file processing
    Large = 1048576,
}

impl BufferSize {
    /// Get the buffer capacity in bytes
    #[inline]
    pub fn capacity(self) -> usize {
        self as usize
    }

    /// Determine the appropriate buffer size for a given capacity requirement
    pub fn for_capacity(required: usize) -> Self {
        if required <= Self::Small.capacity() {
            Self::Small
        } else if required <= Self::Medium.capacity() {
            Self::Medium
        } else {
            Self::Large
        }
    }
}

/// Statistics for monitoring buffer pool performance
#[derive(Debug, Default)]
pub struct PoolStatistics {
    /// Number of successful buffer retrievals from pool
    pub hits: AtomicU64,
    /// Number of new buffer allocations due to pool miss
    pub misses: AtomicU64,
    /// Number of buffers returned to pool
    pub returns: AtomicU64,
    /// Number of buffers discarded due to pool being full
    pub discards: AtomicU64,
}

impl PoolStatistics {
    /// Calculate hit rate as percentage (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }
}

/// Configuration for buffer pool behavior
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of buffers to keep per size category
    pub max_buffers_per_size: usize,
    /// Whether to enable pool statistics collection
    pub collect_stats: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_buffers_per_size: 16, // Reasonable default to prevent unbounded growth
            collect_stats: true,
        }
    }
}

/// Internal pool state for a single buffer size category
#[derive(Debug)]
struct SizePool {
    buffers: VecDeque<Vec<u8>>,
    max_capacity: usize,
}

impl SizePool {
    fn new(max_capacity: usize) -> Self {
        Self {
            buffers: VecDeque::new(),
            max_capacity,
        }
    }

    /// Try to get a buffer from the pool
    fn get_buffer(&mut self) -> Option<Vec<u8>> {
        self.buffers.pop_front()
    }

    /// Try to return a buffer to the pool
    /// Returns true if accepted, false if pool is full
    fn return_buffer(&mut self, mut buffer: Vec<u8>) -> bool {
        if self.buffers.len() < self.max_capacity {
            buffer.clear(); // Clear data but keep capacity
            self.buffers.push_back(buffer);
            true
        } else {
            false // Pool is full, buffer will be dropped
        }
    }

    /// Get current number of pooled buffers
    fn len(&self) -> usize {
        self.buffers.len()
    }
}

/// Thread-safe buffer pool with bounded capacity and size categorization
#[derive(Debug)]
pub struct BufferPool {
    small_pool: Mutex<SizePool>,
    medium_pool: Mutex<SizePool>,
    large_pool: Mutex<SizePool>,
    stats: PoolStatistics,
    config: PoolConfig,
}

impl BufferPool {
    /// Create a new buffer pool with default configuration
    pub fn new() -> Self {
        Self::with_config(PoolConfig::default())
    }

    /// Create a new buffer pool with custom configuration
    pub fn with_config(config: PoolConfig) -> Self {
        Self {
            small_pool: Mutex::new(SizePool::new(config.max_buffers_per_size)),
            medium_pool: Mutex::new(SizePool::new(config.max_buffers_per_size)),
            large_pool: Mutex::new(SizePool::new(config.max_buffers_per_size)),
            stats: PoolStatistics::default(),
            config,
        }
    }

    /// Get a buffer of the specified size category
    /// Returns a PooledBuffer that automatically returns to pool when dropped
    pub fn get_buffer(&self, size: BufferSize) -> PooledBuffer<'_> {
        let pool = match size {
            BufferSize::Small => &self.small_pool,
            BufferSize::Medium => &self.medium_pool,
            BufferSize::Large => &self.large_pool,
        };

        let buffer = {
            let mut pool_guard = pool.lock();
            pool_guard.get_buffer()
        };

        let buffer = if let Some(mut buf) = buffer {
            // Hit: reuse existing buffer
            if self.config.collect_stats {
                self.stats.hits.fetch_add(1, Ordering::Relaxed);
            }
            // Ensure buffer has correct capacity
            buf.reserve(size.capacity());
            buf
        } else {
            // Miss: allocate new buffer
            if self.config.collect_stats {
                self.stats.misses.fetch_add(1, Ordering::Relaxed);
            }
            Vec::with_capacity(size.capacity())
        };

        PooledBuffer {
            buffer: Some(buffer),
            size,
            pool: self,
        }
    }

    /// Return a buffer to the appropriate pool
    fn return_buffer(&self, buffer: Vec<u8>, size: BufferSize) {
        if self.config.collect_stats {
            self.stats.returns.fetch_add(1, Ordering::Relaxed);
        }

        let pool = match size {
            BufferSize::Small => &self.small_pool,
            BufferSize::Medium => &self.medium_pool,
            BufferSize::Large => &self.large_pool,
        };

        let accepted = {
            let mut pool_guard = pool.lock();
            pool_guard.return_buffer(buffer)
        };

        if !accepted && self.config.collect_stats {
            self.stats.discards.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get current statistics for monitoring
    pub fn statistics(&self) -> &PoolStatistics {
        &self.stats
    }

    /// Get pool configuration
    pub fn config(&self) -> &PoolConfig {
        &self.config
    }

    /// Get current pool sizes for each category
    pub fn pool_sizes(&self) -> (usize, usize, usize) {
        (
            self.small_pool.lock().len(),
            self.medium_pool.lock().len(),
            self.large_pool.lock().len(),
        )
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Smart pointer that automatically returns buffer to pool when dropped
/// Provides exclusive access to the underlying `Vec<u8>`
#[derive(Debug)]
pub struct PooledBuffer<'a> {
    buffer: Option<Vec<u8>>,
    size: BufferSize,
    pool: &'a BufferPool,
}

impl PooledBuffer<'_> {
    /// Get mutable access to the underlying buffer
    pub fn get_mut(&mut self) -> &mut Vec<u8> {
        self.buffer.as_mut().expect("PooledBuffer buffer was taken")
    }

    /// Get immutable access to the underlying buffer
    pub fn get_ref(&self) -> &Vec<u8> {
        self.buffer.as_ref().expect("PooledBuffer buffer was taken")
    }

    /// Get the buffer size category
    pub fn size(&self) -> BufferSize {
        self.size
    }

    /// Take ownership of the buffer, preventing automatic return to pool
    /// Useful when the buffer needs to outlive the PooledBuffer
    pub fn take(mut self) -> Vec<u8> {
        self.buffer
            .take()
            .expect("PooledBuffer buffer was already taken")
    }
}

impl std::ops::Deref for PooledBuffer<'_> {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        self.get_ref()
    }
}

impl std::ops::DerefMut for PooledBuffer<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

impl Drop for PooledBuffer<'_> {
    fn drop(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            self.pool.return_buffer(buffer, self.size);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_buffer_size_for_capacity() {
        assert_eq!(BufferSize::for_capacity(1024), BufferSize::Small);
        assert_eq!(BufferSize::for_capacity(4096), BufferSize::Small);
        assert_eq!(BufferSize::for_capacity(8192), BufferSize::Medium);
        assert_eq!(BufferSize::for_capacity(65536), BufferSize::Medium);
        assert_eq!(BufferSize::for_capacity(131072), BufferSize::Large);
    }

    #[test]
    fn test_buffer_pool_basic_operation() {
        let pool = BufferPool::new();

        // Get a buffer
        let mut buffer = pool.get_buffer(BufferSize::Medium);
        buffer.extend_from_slice(&[1, 2, 3, 4, 5]);
        assert_eq!(buffer.len(), 5);

        // Buffer should return to pool when dropped
        drop(buffer);

        // Verify statistics
        let stats = pool.statistics();
        assert_eq!(stats.misses.load(Ordering::Relaxed), 1); // First allocation
        assert_eq!(stats.returns.load(Ordering::Relaxed), 1); // Buffer returned
    }

    #[test]
    fn test_buffer_reuse() {
        let pool = BufferPool::new();

        // Get and return a buffer
        {
            let mut buffer = pool.get_buffer(BufferSize::Small);
            buffer.extend_from_slice(&[1, 2, 3]);
        }

        // Get another buffer of same size - should reuse
        {
            let buffer = pool.get_buffer(BufferSize::Small);
            assert_eq!(buffer.len(), 0); // Should be cleared
        }

        let stats = pool.statistics();
        assert_eq!(stats.hits.load(Ordering::Relaxed), 1);
        assert_eq!(stats.misses.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_bounded_capacity() {
        let config = PoolConfig {
            max_buffers_per_size: 2,
            collect_stats: true,
        };
        let pool = BufferPool::with_config(config);

        // Create and return buffers to fill the pool to capacity
        {
            let _buf1 = pool.get_buffer(BufferSize::Small);
            let _buf2 = pool.get_buffer(BufferSize::Small);
        } // buf1 and buf2 are returned here, pool now has 2 buffers

        // Try to add a third buffer - should be accepted from pool
        {
            let _buf3 = pool.get_buffer(BufferSize::Small); // Gets buf1 or buf2 from pool
            let _buf4 = pool.get_buffer(BufferSize::Small); // Gets the other from pool
            let _buf5 = pool.get_buffer(BufferSize::Small); // New allocation
        } // All three are returned, but pool can only accept 2, so one is discarded

        let stats = pool.statistics();
        assert_eq!(stats.discards.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_thread_safety() {
        let pool = Arc::new(BufferPool::new());
        let mut handles = vec![];

        // Spawn multiple threads using the pool
        for _ in 0..4 {
            let pool_clone = Arc::clone(&pool);
            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    let mut buffer = pool_clone.get_buffer(BufferSize::Medium);
                    buffer.extend_from_slice(&[42; 100]);
                    // Buffer automatically returns to pool
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify no panics occurred and stats look reasonable
        let stats = pool.statistics();
        assert!(stats.hits.load(Ordering::Relaxed) + stats.misses.load(Ordering::Relaxed) == 40);
    }

    #[test]
    fn test_hit_rate_calculation() {
        let pool = BufferPool::new();

        // Initial hit rate should be 0
        assert_eq!(pool.statistics().hit_rate(), 0.0);

        // Generate some hits and misses
        {
            let _buf1 = pool.get_buffer(BufferSize::Small); // miss
            let _buf2 = pool.get_buffer(BufferSize::Small); // miss
        }
        {
            let _buf3 = pool.get_buffer(BufferSize::Small); // hit
            let _buf4 = pool.get_buffer(BufferSize::Small); // hit
        }

        // Should have 50% hit rate (2 hits out of 4 requests)
        assert_eq!(pool.statistics().hit_rate(), 0.5);
    }
}
