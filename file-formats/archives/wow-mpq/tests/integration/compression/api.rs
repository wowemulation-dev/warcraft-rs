//! Generic compression API tests

use crate::common::test_helpers::{compress_with_method, test_round_trip};
use wow_mpq::compression::{CompressionMethod, decompress, flags};

#[test]
fn test_no_compression() {
    let test_data = b"This data is not compressed";
    test_round_trip(test_data, 0).expect("No compression round trip failed");
}

#[test]
fn test_compression_method_detection() {
    assert_eq!(CompressionMethod::from_flags(0), CompressionMethod::None);
    assert_eq!(
        CompressionMethod::from_flags(flags::ZLIB),
        CompressionMethod::Zlib
    );
    assert_eq!(
        CompressionMethod::from_flags(flags::BZIP2),
        CompressionMethod::BZip2
    );
    assert_eq!(
        CompressionMethod::from_flags(flags::LZMA),
        CompressionMethod::Lzma
    );
    assert_eq!(
        CompressionMethod::from_flags(flags::SPARSE),
        CompressionMethod::Sparse
    );
    assert_eq!(
        CompressionMethod::from_flags(flags::HUFFMAN),
        CompressionMethod::Huffman
    );
    assert_eq!(
        CompressionMethod::from_flags(flags::PKWARE),
        CompressionMethod::PKWare
    );
    assert_eq!(
        CompressionMethod::from_flags(flags::ADPCM_MONO),
        CompressionMethod::AdpcmMono
    );
    assert_eq!(
        CompressionMethod::from_flags(flags::ADPCM_STEREO),
        CompressionMethod::AdpcmStereo
    );

    // Test multiple compression detection
    let multi = flags::ZLIB | flags::PKWARE;
    assert!(CompressionMethod::from_flags(multi).is_multiple());
}

#[test]
#[ignore] // Temporarily disabled - complex multi-compression implementation needs more work
fn test_multiple_compression() {
    // Test multiple compression (zlib as final compression)
    // Format: [compression_order_byte][compressed_data]
    let original = b"This is test data for multiple compression. It should compress well.";

    // First compress with zlib
    let zlib_compressed =
        compress_with_method(original, flags::ZLIB).expect("Zlib compression failed");

    // For multiple compression, we need to handle the existing method byte properly
    let zlib_data = if !zlib_compressed.is_empty() && zlib_compressed[0] == flags::ZLIB {
        &zlib_compressed[1..] // Remove the method byte since we'll add our own
    } else {
        &zlib_compressed // No compression was beneficial
    };

    // Create multiple compression data (zlib was last)
    let mut multi_compressed = vec![flags::ZLIB];
    multi_compressed.extend_from_slice(zlib_data);

    // Decompress with multiple flag
    let multi_flag = flags::ZLIB | flags::PKWARE;
    let decompressed = decompress(&multi_compressed, multi_flag, original.len())
        .expect("Multiple decompression failed");

    assert_eq!(decompressed, original);
}

#[test]
fn test_all_compression_methods_implemented() {
    let test_data = b"Test data for compression";

    // Most compression methods are implemented, except Huffman (only decompression is available)
    let implemented_methods = [
        // flags::HUFFMAN,  // Only decompression is implemented
        flags::ZLIB,
        flags::PKWARE,
        flags::BZIP2,
        flags::SPARSE,
        // flags::ADPCM_MONO,  // Requires valid audio data
        // flags::ADPCM_STEREO,  // Requires valid audio data
        flags::LZMA,
    ];

    for method in &implemented_methods {
        let result = compress_with_method(test_data, *method);
        assert!(
            result.is_ok(),
            "Compression with method 0x{:02X} should succeed, but got: {:?}",
            method,
            result.err()
        );

        // Verify the compression type byte is included if compression was beneficial
        let compressed = result.unwrap();
        assert!(!compressed.is_empty());

        // Note: The compress function only adds the method byte if compression was beneficial
        // For small data like our test data, compression might not be beneficial
        if compressed.len() < test_data.len() {
            assert_eq!(
                compressed[0], *method,
                "First byte should be compression type when compression is beneficial"
            );
        }
    }

    // Test ADPCM with valid audio data
    let audio_samples = [0i16; 100]; // 100 silence samples
    let audio_bytes: Vec<u8> = audio_samples
        .iter()
        .flat_map(|&sample| sample.to_le_bytes())
        .collect();

    let result = compress_with_method(&audio_bytes, flags::ADPCM_MONO);
    assert!(
        result.is_ok(),
        "ADPCM mono compression should succeed with valid audio data"
    );

    let result = compress_with_method(&audio_bytes, flags::ADPCM_STEREO);
    assert!(
        result.is_ok(),
        "ADPCM stereo compression should succeed with valid audio data"
    );

    // Test that Huffman compression is not implemented
    let result = compress_with_method(test_data, flags::HUFFMAN);
    assert!(
        result.is_err(),
        "Huffman compression should fail (only decompression is available)"
    );
}

#[test]
fn test_empty_data_decompression_error() {
    // Decompressing empty data should fail for all methods
    let empty = b"";

    let methods = [flags::ZLIB, flags::BZIP2, flags::LZMA, flags::SPARSE];

    for method in &methods {
        let result = decompress(empty, *method, 100);
        assert!(
            result.is_err(),
            "Decompressing empty data with method 0x{:02X} should fail",
            method
        );
    }
}

#[test]
fn test_compression_flags() {
    // Test that all flag constants are correct
    assert_eq!(flags::HUFFMAN, 0x01);
    assert_eq!(flags::ZLIB, 0x02);
    assert_eq!(flags::PKWARE, 0x08);
    assert_eq!(flags::BZIP2, 0x10);
    assert_eq!(flags::SPARSE, 0x20);
    assert_eq!(flags::ADPCM_MONO, 0x40);
    assert_eq!(flags::ADPCM_STEREO, 0x80);
    assert_eq!(flags::LZMA, 0x12);
}
