//! Integration tests for buffer pool functionality

use wow_mpq::{BufferPool, BufferSize};

#[test]
fn test_buffer_pool_creation() {
    let pool = BufferPool::new();
    let stats = pool.statistics();
    assert_eq!(stats.hit_rate(), 0.0);
}

#[test]
fn test_buffer_pool_basic_operations() {
    let pool = BufferPool::new();

    // Get a buffer
    let mut buffer = pool.get_buffer(BufferSize::Medium);
    buffer.extend_from_slice(&[1, 2, 3, 4, 5]);
    assert_eq!(buffer.len(), 5);

    // Buffer should be cleared when returned to pool
    drop(buffer);

    // Get another buffer - should be reused
    let buffer2 = pool.get_buffer(BufferSize::Medium);
    assert_eq!(buffer2.len(), 0); // Should be cleared

    let stats = pool.statistics();
    assert_eq!(stats.hit_rate(), 0.5); // 1 hit out of 2 requests
}

#[test]
fn test_buffer_size_categories() {
    assert_eq!(BufferSize::for_capacity(1000), BufferSize::Small);
    assert_eq!(BufferSize::for_capacity(50000), BufferSize::Medium);
    assert_eq!(BufferSize::for_capacity(500000), BufferSize::Large);
}

#[test]
fn test_buffer_pool_statistics() {
    let pool = BufferPool::new();
    let stats = pool.statistics();

    // Initial state
    assert_eq!(stats.hit_rate(), 0.0);

    // Make some operations
    {
        let _buf1 = pool.get_buffer(BufferSize::Small);
        let _buf2 = pool.get_buffer(BufferSize::Small);
    }

    // Check statistics updated
    assert!(
        stats.hits.load(std::sync::atomic::Ordering::Relaxed) > 0
            || stats.misses.load(std::sync::atomic::Ordering::Relaxed) > 0
    );
}
