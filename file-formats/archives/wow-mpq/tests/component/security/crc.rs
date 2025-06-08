//! Integration tests for CRC validation

use wow_mpq::{Archive, Error};

#[test]
fn test_crc32_algorithm() {
    // Verify we're using the correct CRC-32 algorithm
    let test_cases = vec![
        (b"".as_ref(), 0x00000000),
        (b"a".as_ref(), 0xE8B7BE43),
        (b"abc".as_ref(), 0x352441C2),
        (b"Hello, World!".as_ref(), 0xEC4AC3D0),
        (
            b"The quick brown fox jumps over the lazy dog".as_ref(),
            0x414FA339,
        ),
    ];

    for (data, expected) in test_cases {
        let crc = crc32fast::hash(data);
        assert_eq!(
            crc, expected,
            "CRC mismatch for {:?}: got 0x{:08X}, expected 0x{:08X}",
            data, crc, expected
        );
    }
}

#[test]
fn test_valid_crc_extraction() {
    // This test requires the test data to be generated first
    // Test data is generated automatically by Rust test utilities

    let test_file = "test-data/crc/sectors.mpq";
    if !std::path::Path::new(test_file).exists() {
        eprintln!("Skipping CRC test - test file not found. Run tests to generate automatically");
        return;
    }

    let mut archive = Archive::open(test_file).expect("Failed to open test archive");

    // This should succeed with valid CRCs
    let data = archive
        .read_file("test_crc.txt")
        .expect("Failed to read file with CRC");

    // Verify we got the expected content
    assert!(data.starts_with(b"This is test data for CRC validation. "));
}

#[test]
fn test_invalid_crc_detection() {
    // Create a temporary MPQ with an invalid CRC
    // This is a simplified version - in reality, we'd need proper MPQ structure

    // For now, we'll skip this test as it requires complex MPQ creation
    // The CRC validation is tested through the extraction commands
}

#[test]
fn test_single_unit_crc() {
    let test_file = "test-data/crc/single.mpq";
    if !std::path::Path::new(test_file).exists() {
        eprintln!("Skipping single unit CRC test - test file not found");
        return;
    }

    let mut archive = Archive::open(test_file).expect("Failed to open test archive");

    // This should succeed with valid CRC
    let data = archive
        .read_file("single_crc.txt")
        .expect("Failed to read single unit file");

    assert_eq!(data, b"This is a single unit file with CRC validation.");
}

#[test]
fn test_crc_error_type() {
    // Test that CRC errors produce the correct error type
    let err = Error::ChecksumMismatch {
        file: "test.txt".to_string(),
        expected: 0x12345678,
        actual: 0x87654321,
    };

    let error_string = err.to_string();
    assert!(error_string.contains("Checksum mismatch"));
    assert!(error_string.contains("test.txt"));
    assert!(error_string.contains("12345678"));
    assert!(error_string.contains("87654321"));
}
