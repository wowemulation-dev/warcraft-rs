//! Tests for digital signature verification

use std::path::Path;
use wow_mpq::{Archive, SignatureStatus};

#[test]
#[ignore = "Requires test archive with valid weak signature"]
fn test_weak_signature_verification() {
    // This test requires a real MPQ archive with a valid weak signature
    // Such archives can be found in older Blizzard games
    let test_archive = "test-data/signed/weak_signature.mpq";

    if !Path::new(test_archive).exists() {
        eprintln!("Skipping test: {test_archive} not found");
        return;
    }

    let mut archive = Archive::open(test_archive).expect("Failed to open test archive");

    let info = archive.get_info().expect("Failed to get archive info");

    assert!(info.has_signature, "Archive should have a signature");
    assert_eq!(
        info.signature_status,
        SignatureStatus::WeakValid,
        "Weak signature should be valid"
    );
}

#[test]
fn test_no_signature() {
    // Create a simple archive without signature
    use tempfile::TempDir;
    use wow_mpq::{ArchiveBuilder, FormatVersion};

    let dir = TempDir::new().unwrap();
    let archive_path = dir.path().join("unsigned.mpq");

    ArchiveBuilder::new()
        .version(FormatVersion::V1)
        .add_file_data(b"test data".to_vec(), "test.txt")
        .build(&archive_path)
        .expect("Failed to create archive");

    let mut archive = Archive::open(&archive_path).expect("Failed to open archive");

    let info = archive.get_info().expect("Failed to get archive info");

    assert!(!info.has_signature, "Archive should not have a signature");
    assert_eq!(
        info.signature_status,
        SignatureStatus::None,
        "Signature status should be None"
    );
}

#[test]
fn test_signature_public_key_parsing() {
    use wow_mpq::crypto::public_keys;

    // Test that we can load the weak public key
    let weak_key = public_keys::weak_public_key().expect("Failed to load weak public key");

    // Verify key properties
    use rsa::traits::PublicKeyParts;
    let n_bytes = weak_key.n().to_bytes_be();
    assert_eq!(n_bytes.len(), 64); // 512 bits / 8

    // Test that we can load the strong public key
    let strong_key = public_keys::strong_public_key().expect("Failed to load strong public key");

    // Verify key properties (may be 255 or 256 bytes depending on leading zeros)
    let n_bytes = strong_key.n().to_bytes_be();
    assert!(n_bytes.len() >= 255 && n_bytes.len() <= 256); // 2048 bits / 8
}

#[test]
fn test_weak_signature_parsing() {
    use wow_mpq::crypto::parse_weak_signature;

    // Create a mock weak signature file (72 bytes total: 8 byte header + 64 byte signature)
    let signature_data = vec![0xAB; 72]; // 72 bytes total as expected

    let parsed = parse_weak_signature(&signature_data).expect("Failed to parse weak signature");

    assert_eq!(parsed.len(), 64);
    // Check that the signature is extracted from offset 8
    assert_eq!(parsed, &signature_data[8..72]);
}

#[test]
fn test_signature_status_enum() {
    // Ensure all SignatureStatus variants are accessible
    let statuses = vec![
        SignatureStatus::None,
        SignatureStatus::WeakValid,
        SignatureStatus::WeakInvalid,
        SignatureStatus::StrongValid,
        SignatureStatus::StrongInvalid,
        SignatureStatus::StrongNoKey,
    ];

    for status in statuses {
        // Just ensure they can be created and compared
        assert_eq!(status, status);
    }
}

#[test]
fn test_stormlib_compatible_hash_calculation() {
    use std::io::Cursor;
    use wow_mpq::crypto::{DIGEST_UNIT_SIZE, SignatureInfo, calculate_mpq_hash_md5};

    // Test that our hash calculation matches StormLib's approach
    // Create test data larger than one chunk to test chunk processing
    let test_data = vec![0x42u8; DIGEST_UNIT_SIZE + 1000]; // 64KB + 1000 bytes
    let mut cursor = Cursor::new(&test_data);

    // Create signature info with no excluded area (simple case)
    let sig_info = SignatureInfo::new_weak(
        0,                      // archive start
        test_data.len() as u64, // archive size
        0,                      // signature position (no exclusion)
        0,                      // signature size (no exclusion)
        vec![],                 // empty signature
    );

    // Calculate hash
    let result = calculate_mpq_hash_md5(&mut cursor, &sig_info);
    assert!(result.is_ok(), "Hash calculation should succeed");

    let hash = result.unwrap();
    assert_eq!(hash.len(), 16, "MD5 hash should be 16 bytes");

    // Hash should not be all zeros (would indicate empty data)
    assert_ne!(hash, [0u8; 16], "Hash should not be all zeros");
}

#[test]
fn test_stormlib_signature_exclusion() {
    use std::io::Cursor;
    use wow_mpq::crypto::{DIGEST_UNIT_SIZE, SignatureInfo, calculate_mpq_hash_md5};

    // Test signature area exclusion by zeroing
    let test_data = vec![0x42u8; DIGEST_UNIT_SIZE]; // Exactly 64KB
    let mut cursor1 = Cursor::new(&test_data);
    let mut cursor2 = Cursor::new(&test_data);

    // Calculate hash without exclusion
    let sig_info_no_exclude = SignatureInfo::new_weak(0, test_data.len() as u64, 0, 0, vec![]);
    let hash_no_exclude = calculate_mpq_hash_md5(&mut cursor1, &sig_info_no_exclude).unwrap();

    // Calculate hash with middle section excluded (should be zeroed)
    let exclude_start = 1000;
    let exclude_size = 72; // Typical weak signature size
    let sig_info_exclude = SignatureInfo::new_weak(
        0,
        test_data.len() as u64,
        exclude_start,
        exclude_size,
        vec![0u8; exclude_size as usize],
    );
    let hash_exclude = calculate_mpq_hash_md5(&mut cursor2, &sig_info_exclude).unwrap();

    // Hashes should be different because exclusion zeros out signature area
    assert_ne!(
        hash_no_exclude, hash_exclude,
        "Hashes should differ when signature area is excluded"
    );
}

#[test]
fn test_zero_signature_validation() {
    use wow_mpq::crypto::parse_weak_signature;

    // Test that zero signatures are properly rejected
    let zero_signature_data = vec![0x00u8; 72]; // All zeros including header
    let result = parse_weak_signature(&zero_signature_data);

    assert!(result.is_err(), "Zero signature should be rejected");
    assert!(
        result.unwrap_err().to_string().contains("all zeros"),
        "Error should mention signature is all zeros"
    );
}

#[test]
fn test_stormlib_digest_unit_size() {
    use wow_mpq::crypto::DIGEST_UNIT_SIZE;

    // Verify we use exactly the same chunk size as StormLib
    assert_eq!(
        DIGEST_UNIT_SIZE, 0x10000,
        "Digest unit size should match StormLib (64KB)"
    );
    assert_eq!(
        DIGEST_UNIT_SIZE, 65536,
        "Digest unit size should be 65536 bytes"
    );
}
