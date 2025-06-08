//! Round-trip compression tests for all algorithms

use crate::common::test_helpers::test_round_trip;
use wow_mpq::compression::flags;

#[test]
fn test_zlib_round_trip() {
    let test_cases = vec![
        b"Hello, World!".to_vec(),
        b"A".repeat(1000),               // Highly compressible
        vec![0u8; 100],                  // All zeros
        (0u8..255).collect::<Vec<u8>>(), // All byte values
    ];

    for original in test_cases {
        test_round_trip(&original, flags::ZLIB).expect("Zlib round trip failed");
    }
}

#[test]
fn test_bzip2_round_trip() {
    let test_cases = vec![
        b"Hello, World!".to_vec(),
        b"B".repeat(1000), // Highly compressible
        vec![0u8; 100],    // All zeros
    ];

    for original in test_cases {
        test_round_trip(&original, flags::BZIP2).expect("Bzip2 round trip failed");
    }
}

#[test]
fn test_lzma_round_trip() {
    let test_cases = vec![
        b"Hello, World!".to_vec(),
        b"C".repeat(1000), // Highly compressible
        vec![0u8; 100],    // All zeros
        b"The quick brown fox jumps over the lazy dog".to_vec(),
    ];

    for original in test_cases {
        test_round_trip(&original, flags::LZMA).expect("LZMA round trip failed");
    }
}

#[test]
fn test_sparse_round_trip() {
    let test_cases = vec![
        b"Hello\0\0\0World".to_vec(),
        vec![0u8; 1000], // All zeros - should compress very well
        b"No zeros here!".to_vec(),
        vec![1, 2, 3, 0, 0, 0, 0, 0, 4, 5, 6], // Mixed data
    ];

    for original in test_cases {
        test_round_trip(&original, flags::SPARSE).expect("Sparse round trip failed");
    }
}

#[test]
fn test_no_compression() {
    let original = b"This is uncompressed data";
    test_round_trip(original, 0).expect("No compression round trip failed");
}
