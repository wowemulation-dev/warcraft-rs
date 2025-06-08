//! BZip2 compression tests

use crate::common::test_helpers::{compress_with_method, test_round_trip};
use wow_mpq::compression::flags;

#[test]
fn test_bzip2_compression() {
    let test_data = b"This is a test string that should compress well because it has repeated patterns. \
                      This is a test string that should compress well because it has repeated patterns. \
                      This is a test string that should compress well because it has repeated patterns.";

    // Compress
    let compressed = compress_with_method(test_data, flags::BZIP2).expect("Compression failed");

    // Check if compression was beneficial
    if !compressed.is_empty() && compressed[0] == flags::BZIP2 {
        // Should be smaller than original (including the method byte)
        assert!(compressed.len() < test_data.len());
        println!(
            "BZip2 compression ratio: {:.1}%",
            100.0 * compressed.len() as f64 / test_data.len() as f64
        );
    }

    // Test round trip
    test_round_trip(test_data, flags::BZIP2).expect("Round trip failed");
}

#[test]
fn test_bzip2_large_data() {
    // Test with larger data (32KB of repeated pattern to keep test fast)
    // BZip2 should compress repeated patterns very well
    let pattern = b"The quick brown fox jumps over the lazy dog. ";
    let mut large_data = Vec::new();
    for _ in 0..(32 * 1024 / pattern.len()) {
        large_data.extend_from_slice(pattern);
    }

    let compressed = compress_with_method(&large_data, flags::BZIP2).expect("Compression failed");

    // Check if compression was beneficial
    if !compressed.is_empty() && compressed[0] == flags::BZIP2 {
        println!(
            "Large data bzip2 ratio: {:.1}%",
            100.0 * compressed.len() as f64 / large_data.len() as f64
        );
        // Should achieve good compression
        assert!(compressed.len() < large_data.len() / 2); // Should compress to less than 50%
    }

    // Test round trip
    test_round_trip(&large_data, flags::BZIP2).expect("Large data round trip failed");
}
