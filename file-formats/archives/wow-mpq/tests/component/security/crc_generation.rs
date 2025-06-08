//! Tests for CRC generation during archive creation

use std::fs;
use tempfile::tempdir;
use wow_mpq::{Archive, ArchiveBuilder, ListfileOption};

/// Test that CRC generation works for single-unit files
#[test]
fn test_single_unit_file_crc_generation() {
    let temp_dir = tempdir().unwrap();
    let archive_path = temp_dir.path().join("test_single_crc.mpq");

    // Create test data that's small enough to be single-unit
    let test_data = b"This is a small test file for CRC generation!";
    let test_file = temp_dir.path().join("small.txt");
    fs::write(&test_file, test_data).unwrap();

    // Build archive with CRC generation enabled
    ArchiveBuilder::new()
        .block_size(3) // 4KB sectors
        .generate_crcs(true) // Enable CRC generation
        .add_file(&test_file, "small.txt")
        .listfile_option(ListfileOption::Generate)
        .build(&archive_path)
        .unwrap();

    // Open and verify the archive
    let mut archive = Archive::open(&archive_path).unwrap();

    // Read the file - this should validate the CRC
    let read_data = archive.read_file("small.txt").unwrap();
    assert_eq!(read_data, test_data);

    // Check file info to verify CRC flag is set
    let files = archive.list().unwrap();
    let file_info = files.iter().find(|f| f.name == "small.txt").unwrap();
    assert!(
        file_info.has_sector_crc(),
        "File should have SECTOR_CRC flag set"
    );
}

/// Test that CRC generation works for multi-sector files
#[test]
fn test_multi_sector_file_crc_generation() {
    let temp_dir = tempdir().unwrap();
    let archive_path = temp_dir.path().join("test_multi_crc.mpq");

    // Create test data that spans multiple sectors
    // With block_size=3, sector size is 512 * 2^3 = 4096 bytes
    let sector_size = 4096;
    let test_data = vec![b'A'; sector_size * 3 + 1024]; // 3.25 sectors
    let test_file = temp_dir.path().join("large.bin");
    fs::write(&test_file, &test_data).unwrap();

    // Build archive with CRC generation enabled
    ArchiveBuilder::new()
        .block_size(3) // 4KB sectors
        .generate_crcs(true) // Enable CRC generation
        .add_file(&test_file, "large.bin")
        .listfile_option(ListfileOption::Generate)
        .build(&archive_path)
        .unwrap();

    // Open and verify the archive
    let mut archive = Archive::open(&archive_path).unwrap();

    // Read the file - this should validate the CRCs for all sectors
    let read_data = archive.read_file("large.bin").unwrap();
    assert_eq!(read_data, test_data);

    // Check file info to verify CRC flag is set
    let files = archive.list().unwrap();
    let file_info = files.iter().find(|f| f.name == "large.bin").unwrap();
    assert!(
        file_info.has_sector_crc(),
        "File should have SECTOR_CRC flag set"
    );
}

/// Test that archives without CRC generation work correctly
#[test]
fn test_no_crc_generation() {
    let temp_dir = tempdir().unwrap();
    let archive_path = temp_dir.path().join("test_no_crc.mpq");

    // Create test data
    let test_data = b"Test file without CRC";
    let test_file = temp_dir.path().join("no_crc.txt");
    fs::write(&test_file, test_data).unwrap();

    // Build archive with CRC generation disabled (default)
    ArchiveBuilder::new()
        .add_file(&test_file, "no_crc.txt")
        .listfile_option(ListfileOption::Generate)
        .build(&archive_path)
        .unwrap();

    // Open and verify the archive
    let mut archive = Archive::open(&archive_path).unwrap();

    // Read the file
    let read_data = archive.read_file("no_crc.txt").unwrap();
    assert_eq!(read_data, test_data);

    // Check file info to verify CRC flag is NOT set
    let files = archive.list().unwrap();
    let file_info = files.iter().find(|f| f.name == "no_crc.txt").unwrap();
    assert!(
        !file_info.has_sector_crc(),
        "File should NOT have SECTOR_CRC flag set"
    );
}

/// Test CRC generation with compressed files
#[test]
fn test_crc_generation_with_compression() {
    let temp_dir = tempdir().unwrap();
    let archive_path = temp_dir.path().join("test_compressed_crc.mpq");

    // Create compressible test data
    let test_data = "Hello World! ".repeat(1000).into_bytes();
    let test_file = temp_dir.path().join("compressible.txt");
    fs::write(&test_file, &test_data).unwrap();

    // Build archive with CRC generation and compression enabled
    ArchiveBuilder::new()
        .block_size(3) // 4KB sectors
        .generate_crcs(true) // Enable CRC generation
        .default_compression(wow_mpq::compression::flags::ZLIB)
        .add_file(&test_file, "compressible.txt")
        .listfile_option(ListfileOption::Generate)
        .build(&archive_path)
        .unwrap();

    // Open and verify the archive
    let mut archive = Archive::open(&archive_path).unwrap();

    // Read the file - this should decompress and validate CRC
    let read_data = archive.read_file("compressible.txt").unwrap();
    assert_eq!(read_data, test_data);

    // Check file info
    let files = archive.list().unwrap();
    let file_info = files.iter().find(|f| f.name == "compressible.txt").unwrap();
    assert!(
        file_info.has_sector_crc(),
        "File should have SECTOR_CRC flag set"
    );
    assert!(file_info.is_compressed(), "File should be compressed");
}

/// Test CRC generation with encrypted files
#[test]
fn test_crc_generation_with_encryption() {
    let temp_dir = tempdir().unwrap();
    let archive_path = temp_dir.path().join("test_encrypted_crc.mpq");

    // Create test data
    let test_data = b"Secret data with CRC protection";
    let test_file = temp_dir.path().join("secret.dat");
    fs::write(&test_file, test_data).unwrap();

    // Build archive with CRC generation and encryption enabled
    ArchiveBuilder::new()
        .generate_crcs(true) // Enable CRC generation
        .add_file_with_encryption(
            &test_file,
            "secret.dat",
            0,     // No compression
            false, // No FIX_KEY
            0,     // Default locale
        )
        .listfile_option(ListfileOption::Generate)
        .build(&archive_path)
        .unwrap();

    // Open and verify the archive
    let mut archive = Archive::open(&archive_path).unwrap();

    // Read the file - this should decrypt and validate CRC
    let read_data = archive.read_file("secret.dat").unwrap();
    assert_eq!(read_data, test_data);

    // Check file info
    let files = archive.list().unwrap();
    let file_info = files.iter().find(|f| f.name == "secret.dat").unwrap();
    assert!(
        file_info.has_sector_crc(),
        "File should have SECTOR_CRC flag set"
    );
    assert!(file_info.is_encrypted(), "File should be encrypted");
}
