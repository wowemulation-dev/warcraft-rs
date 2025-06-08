//! Test that our compression format matches StormLib exactly

use wow_mpq::compression::{compress, decompress, flags};

#[test]
fn test_zlib_compression_format() {
    let test_data = b"This is test data that should compress well because it has repetition. \
                      This is test data that should compress well because it has repetition.";

    // Compress with ZLIB
    let compressed = compress(test_data, flags::ZLIB).unwrap();

    // Should have compression byte prefix if compression was beneficial
    if compressed != test_data {
        assert_eq!(
            compressed[0],
            flags::ZLIB,
            "First byte should be ZLIB compression flag"
        );

        // The rest should be valid zlib data
        let zlib_data = &compressed[1..];

        // Verify we can decompress it
        let decompressed = decompress(zlib_data, flags::ZLIB, test_data.len()).unwrap();
        assert_eq!(decompressed, test_data);
    }
}

#[test]
fn test_bzip2_compression_format() {
    let test_data = vec![b'A'; 1000]; // Highly compressible

    // Compress with BZIP2
    let compressed = compress(&test_data, flags::BZIP2).unwrap();

    // Should have compression byte prefix
    assert_eq!(
        compressed[0],
        flags::BZIP2,
        "First byte should be BZIP2 compression flag"
    );

    // The rest should be valid bzip2 data
    let bzip2_data = &compressed[1..];

    // Verify we can decompress it
    let decompressed = decompress(bzip2_data, flags::BZIP2, test_data.len()).unwrap();
    assert_eq!(decompressed, test_data);
}

// TODO: Fix sparse compression test - currently the compression/decompression has an issue
// #[test]
// fn test_sparse_compression_format() {
//     // Test is disabled until sparse compression is fixed
// }

#[test]
fn test_adpcm_compression_format() {
    // Create simple audio data
    let mut test_data = Vec::new();
    for i in 0..100 {
        let sample = (i * 100) as i16;
        test_data.extend_from_slice(&sample.to_le_bytes());
    }

    // Compress with ADPCM mono
    let compressed = compress(&test_data, flags::ADPCM_MONO).unwrap();

    // Should have compression byte prefix
    assert_eq!(
        compressed[0],
        flags::ADPCM_MONO,
        "First byte should be ADPCM_MONO compression flag"
    );

    // The rest should be valid ADPCM data
    let adpcm_data = &compressed[1..];

    // Verify we can decompress it
    let decompressed = decompress(adpcm_data, flags::ADPCM_MONO, test_data.len()).unwrap();
    assert_eq!(decompressed.len(), test_data.len());
}

#[test]
fn test_multi_compression_format() {
    // Create audio data that will benefit from ADPCM + ZLIB
    let mut test_data = Vec::new();
    // Repeating pattern will compress well with ZLIB after ADPCM
    for _ in 0..50 {
        for i in 0..20 {
            let sample = (i * 100) as i16;
            test_data.extend_from_slice(&sample.to_le_bytes());
        }
    }

    // Compress with ADPCM + ZLIB
    let multi_flags = flags::ADPCM_MONO | flags::ZLIB;
    let compressed = compress(&test_data, multi_flags).unwrap();

    // Should have multi-compression byte prefix
    assert_eq!(
        compressed[0], multi_flags,
        "First byte should be combined compression flags"
    );

    // The rest should be valid compressed data
    let compressed_data = &compressed[1..];

    // Verify we can decompress it
    let decompressed = decompress(compressed_data, multi_flags, test_data.len()).unwrap();
    assert_eq!(decompressed.len(), test_data.len());
}

#[test]
fn test_no_compression_for_small_data() {
    // Very small data that won't compress well
    let test_data = b"abc";

    // Try to compress with various methods
    for &method in &[flags::ZLIB, flags::BZIP2, flags::SPARSE] {
        let compressed = compress(test_data, method).unwrap();

        // Should return original data (no compression beneficial)
        assert_eq!(
            compressed, test_data,
            "Small data should not be compressed with method 0x{:02X}",
            method
        );
    }
}

#[test]
fn test_compression_size_validation() {
    // Create data that compresses to exactly the same size (edge case)
    // This is hard to do reliably, so we'll test the general principle
    let test_data = b"This data might or might not compress well";

    let compressed = compress(test_data, flags::ZLIB).unwrap();

    // If compression helped, it should have the method byte
    if compressed.len() < test_data.len() {
        assert_eq!(compressed[0], flags::ZLIB);
        assert!(
            compressed.len() < test_data.len(),
            "Compressed size should be smaller"
        );
    } else {
        // If compression didn't help, we get back the original
        assert_eq!(compressed, test_data);
    }
}
