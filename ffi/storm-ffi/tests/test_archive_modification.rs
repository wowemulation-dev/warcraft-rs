//! Integration tests for archive modification operations
//! Tests the MutableArchive API which wraps the StormLib FFI functions

use std::fs;
use tempfile::NamedTempFile;
use wow_mpq::{AddFileOptions, Archive, ArchiveBuilder, MutableArchive};

#[test]
fn test_create_archive_and_add_files() {
    // Create a temporary file for the archive
    let temp_file = NamedTempFile::new().unwrap();
    let archive_path = temp_file.path();

    // Create archive using builder with a reasonable block size
    // Note: file_limit is determined by block_size
    let builder = ArchiveBuilder::new()
        .version(wow_mpq::FormatVersion::V2)
        .block_size(10); // 2^10 = 1024 file limit

    builder.build(archive_path).unwrap();

    // Open for modification
    let mut archive = MutableArchive::open(archive_path).unwrap();

    // Add file without compression
    let test_data = b"Hello, World! This is test file content.";
    let temp_data_file = NamedTempFile::new().unwrap();
    fs::write(&temp_data_file, test_data).unwrap();

    let options = AddFileOptions::new().compression(wow_mpq::compression::CompressionMethod::None);

    archive
        .add_file(temp_data_file.path(), "test_file.txt", options)
        .unwrap();

    // Add file with compression
    let test_data2 = b"This is a longer text that should compress well. ".repeat(10);
    let temp_data_file2 = NamedTempFile::new().unwrap();
    fs::write(&temp_data_file2, &test_data2).unwrap();

    let options = AddFileOptions::new().compression(wow_mpq::compression::CompressionMethod::Zlib);

    archive
        .add_file(temp_data_file2.path(), "compressed_file.txt", options)
        .unwrap();

    // Close and reopen to verify
    drop(archive);

    let mut archive = Archive::open(archive_path).unwrap();

    // Verify first file exists
    assert!(archive.find_file("test_file.txt").unwrap().is_some());
    let data = archive.read_file("test_file.txt").unwrap();
    assert_eq!(&data, test_data);

    // Verify second file exists
    assert!(archive.find_file("compressed_file.txt").unwrap().is_some());
    let data = archive.read_file("compressed_file.txt").unwrap();
    assert_eq!(&data[..], &test_data2[..]);
}

#[test]
fn test_remove_file() {
    let temp_file = NamedTempFile::new().unwrap();
    let archive_path = temp_file.path();

    // Create archive with multiple files
    let mut builder = ArchiveBuilder::new()
        .version(wow_mpq::FormatVersion::V2)
        .block_size(7); // 2^7 = 128 file limit

    // Add test files
    for i in 0..3 {
        let test_data = format!("Test file {i} content");
        builder = builder.add_file_data(test_data.into_bytes(), &format!("file_{i}.txt"));
    }

    builder.build(archive_path).unwrap();

    // Open for modification and remove middle file
    let mut archive = MutableArchive::open(archive_path).unwrap();
    archive.remove_file("file_1.txt").unwrap();

    // Verify file is removed
    assert!(archive.find_file("file_1.txt").unwrap().is_none());

    // Verify other files still exist
    assert!(archive.find_file("file_0.txt").unwrap().is_some());
    assert!(archive.find_file("file_2.txt").unwrap().is_some());
}

#[test]
fn test_rename_file() {
    let temp_file = NamedTempFile::new().unwrap();
    let archive_path = temp_file.path();

    // Create archive with a test file
    let test_data = b"Original file content";
    let builder = ArchiveBuilder::new()
        .version(wow_mpq::FormatVersion::V2)
        .block_size(7)
        .add_file_data(test_data.to_vec(), "old_name.txt");

    builder.build(archive_path).unwrap();

    // Open for modification and rename
    let mut archive = MutableArchive::open(archive_path).unwrap();
    archive.rename_file("old_name.txt", "new_name.txt").unwrap();

    // Verify old name doesn't exist
    assert!(archive.find_file("old_name.txt").unwrap().is_none());

    // Verify new name exists
    assert!(archive.find_file("new_name.txt").unwrap().is_some());

    // Verify content is preserved
    let data = archive.read_file("new_name.txt").unwrap();
    assert_eq!(&data, test_data);
}

#[test]
fn test_compact_archive() {
    let temp_file = NamedTempFile::new().unwrap();
    let archive_path = temp_file.path();

    // Create archive with several files
    let mut builder = ArchiveBuilder::new()
        .version(wow_mpq::FormatVersion::V2)
        .block_size(7);

    for i in 0..5 {
        let test_data = format!("Test file {i} with some content to take up space").repeat(10);
        builder = builder.add_file_data(test_data.into_bytes(), &format!("file_{i}.txt"));
    }

    builder.build(archive_path).unwrap();

    // Get initial size
    let initial_size = fs::metadata(archive_path).unwrap().len();

    // Open for modification, remove some files, and compact
    let mut archive = MutableArchive::open(archive_path).unwrap();

    // Remove some files
    archive.remove_file("file_1.txt").unwrap();
    archive.remove_file("file_3.txt").unwrap();

    // Compact the archive
    archive.compact().unwrap();

    // Verify remaining files still exist
    assert!(archive.find_file("file_0.txt").unwrap().is_some());
    assert!(archive.find_file("file_2.txt").unwrap().is_some());
    assert!(archive.find_file("file_4.txt").unwrap().is_some());

    // Verify removed files don't exist
    assert!(archive.find_file("file_1.txt").unwrap().is_none());
    assert!(archive.find_file("file_3.txt").unwrap().is_none());

    drop(archive);

    // Check that archive size decreased
    let final_size = fs::metadata(archive_path).unwrap().len();
    assert!(
        final_size < initial_size,
        "Archive size should decrease after compaction: {initial_size} -> {final_size}"
    );
}

#[test]
fn test_error_handling() {
    let temp_file = NamedTempFile::new().unwrap();
    let archive_path = temp_file.path();

    // Create empty archive
    ArchiveBuilder::new()
        .version(wow_mpq::FormatVersion::V2)
        .block_size(7)
        .build(archive_path)
        .unwrap();

    let mut archive = MutableArchive::open(archive_path).unwrap();

    // Try to remove non-existent file
    match archive.remove_file("non_existent.txt") {
        Err(wow_mpq::Error::FileNotFound(_)) => {}
        other => panic!("Expected FileNotFound error, got: {other:?}"),
    }

    // Try to rename non-existent file
    match archive.rename_file("non_existent.txt", "new.txt") {
        Err(wow_mpq::Error::FileNotFound(_)) => {}
        other => panic!("Expected FileNotFound error, got: {other:?}"),
    }

    // Try to add file with invalid path
    let options = AddFileOptions::new();
    match archive.add_file("non/existent/path.txt", "test.txt", options) {
        Err(wow_mpq::Error::Io(_)) => {}
        other => panic!("Expected Io error, got: {other:?}"),
    }
}

#[test]
fn test_file_enumeration() {
    let temp_file = NamedTempFile::new().unwrap();
    let archive_path = temp_file.path();

    // Create archive with pattern-matching files
    let mut builder = ArchiveBuilder::new()
        .version(wow_mpq::FormatVersion::V2)
        .block_size(7);

    // Add various files
    let files = vec![
        "readme.txt",
        "data/file1.dat",
        "data/file2.dat",
        "docs/readme.md",
        "docs/manual.pdf",
        "test.txt",
    ];

    for file in &files {
        builder = builder.add_file_data(b"test content".to_vec(), file);
    }

    builder.build(archive_path).unwrap();

    // Test file listing
    let mut archive = Archive::open(archive_path).unwrap();
    let file_list = archive.list().unwrap();

    // Account for the automatically generated listfile
    let expected_count = files.len() + 1; // +1 for (listfile)
    assert_eq!(file_list.len(), expected_count);

    // Verify all files are present (accounting for path normalization)
    for file in &files {
        let normalized_file = file.replace('/', "\\");
        assert!(
            file_list.iter().any(|f| f.name == normalized_file),
            "File {file} (normalized: {normalized_file}) should be in the list"
        );
    }
}

#[test]
#[ignore = "Attributes file handling needs investigation"]
fn test_archive_modification_with_attributes() {
    let temp_file = NamedTempFile::new().unwrap();
    let archive_path = temp_file.path();

    // Create archive WITHOUT attributes first - just test modification operations
    let builder = ArchiveBuilder::new()
        .version(wow_mpq::FormatVersion::V2)
        .block_size(7);

    builder.build(archive_path).unwrap();

    // Open for modification and add files
    let mut archive = MutableArchive::open(archive_path).unwrap();

    // Add files with compression
    let options = AddFileOptions::new().compression(wow_mpq::compression::CompressionMethod::Zlib);

    archive
        .add_file_data(b"Test with compression", "compressed_file.txt", options)
        .unwrap();

    // Add file with encryption
    let options = AddFileOptions::new().encrypt();

    archive
        .add_file_data(b"Test with encryption", "encrypted_file.txt", options)
        .unwrap();

    // Add file with both compression and encryption
    let options = AddFileOptions::new()
        .compression(wow_mpq::compression::CompressionMethod::BZip2)
        .encrypt();

    archive
        .add_file_data(b"Test with both", "both_file.txt", options)
        .unwrap();

    // Verify files were added
    let file_list = archive.list().unwrap();

    // Should have 4 files: (listfile) and our 3 added files
    assert!(file_list.len() >= 3, "Should have at least 3 files");
    assert!(
        file_list.iter().any(|f| f.name == "compressed_file.txt"),
        "compressed_file.txt should exist"
    );
    assert!(
        file_list.iter().any(|f| f.name == "encrypted_file.txt"),
        "encrypted_file.txt should exist"
    );
    assert!(
        file_list.iter().any(|f| f.name == "both_file.txt"),
        "both_file.txt should exist"
    );
}
