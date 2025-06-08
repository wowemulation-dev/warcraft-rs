//! Zlib compression tests

use crate::common::test_helpers::{
    compress_with_method, decompress_from_compress_api, test_round_trip,
};
use wow_mpq::compression::flags;

#[test]
fn test_zlib_compression() {
    let test_data = include_bytes!("../../../../../../README.md");

    // Compress
    let compressed = compress_with_method(test_data, flags::ZLIB).expect("Compression failed");

    // Check if compression was beneficial
    if !compressed.is_empty() && compressed[0] == flags::ZLIB {
        // Should be smaller than original (including the method byte)
        assert!(compressed.len() < test_data.len());
        println!(
            "Zlib compression ratio: {:.1}%",
            100.0 * compressed.len() as f64 / test_data.len() as f64
        );
    }

    // Test round trip
    test_round_trip(test_data, flags::ZLIB).expect("Round trip failed");
}

#[test]
fn test_zlib_empty_data() {
    // Test empty data handling
    let empty: &[u8] = &[];

    // Test round trip with empty data
    test_round_trip(empty, flags::ZLIB).expect("Empty data round trip failed");
}

#[test]
fn test_zlib_invalid_data() {
    // Try to decompress random data that isn't valid compressed data
    let invalid_data = vec![0xFF, 0xDE, 0xAD, 0xBE, 0xEF];

    // Use low-level decompress API directly for this test
    use wow_mpq::compression::decompress;
    let result = decompress(&invalid_data, flags::ZLIB, 100);
    assert!(result.is_err(), "Decompressing invalid data should fail");
}

#[test]
fn test_zlib_size_mismatch() {
    let test_data = b"Test data";
    let compressed = compress_with_method(test_data, flags::ZLIB).expect("Compression failed");

    // Try to decompress with wrong expected size (larger than actual)
    // This should succeed - expected_size is just a hint for buffer allocation
    let result = decompress_from_compress_api(&compressed, flags::ZLIB, 1000);

    assert!(
        result.is_ok(),
        "Decompression should succeed even with wrong expected size"
    );
    let decompressed = result.unwrap();
    assert_eq!(
        decompressed, test_data,
        "Decompressed data should match original"
    );
    assert_eq!(
        decompressed.len(),
        test_data.len(),
        "Actual size is {} not the expected 1000",
        decompressed.len()
    );
}

#[test]
fn test_zlib_size_too_small() {
    let test_data = b"This is a longer test string that will compress";
    let compressed = compress_with_method(test_data, flags::ZLIB).expect("Compression failed");

    // Try to decompress with a smaller expected size
    // The implementation will still decompress all the data
    let result = decompress_from_compress_api(&compressed, flags::ZLIB, 5);

    match result {
        Ok(decompressed) => {
            // The decompression succeeds and returns all the data
            // even though we only "expected" 5 bytes
            assert_eq!(
                decompressed, test_data,
                "Should decompress all data regardless of expected size"
            );
        }
        Err(_) => {
            // Some implementations might fail if expected size is too small
            // This is also acceptable behavior
        }
    }
}

#[test]
fn test_zlib_pre_compressed_data() {
    // Test with mixed binary data (simulating compressed data)
    let binary_data: Vec<u8> = (0..=255).cycle().take(2048).collect();
    test_round_trip(&binary_data, flags::ZLIB).expect("Pre-compressed data round trip failed");
}

#[test]
fn test_zlib_large_data() {
    // Test with large repetitive data that should compress well
    let data = vec![b'A'; 100_000];
    let compressed = compress_with_method(&data, flags::ZLIB).expect("Compression failed");

    // Check if compression was beneficial
    if !compressed.is_empty() && compressed[0] == flags::ZLIB {
        // Should achieve good compression
        println!(
            "Large data zlib ratio: {:.1}%",
            100.0 * compressed.len() as f64 / data.len() as f64
        );
        assert!(compressed.len() < data.len() / 10); // Should compress to less than 10%
    }

    test_round_trip(&data, flags::ZLIB).expect("Large data round trip failed");
}

#[test]
fn test_zlib_binary_data() {
    // Test with all possible byte values
    let data: Vec<u8> = (0..=255).cycle().take(1024).collect();
    test_round_trip(&data, flags::ZLIB).expect("Binary data round trip failed");
}

#[test]
fn test_zlib_empty_buffer() {
    // Test empty buffer compression
    let empty = Vec::<u8>::new();
    test_round_trip(&empty, flags::ZLIB).expect("Empty buffer round trip failed");
}
