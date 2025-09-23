//! Tests for path separator handling in MPQ archives

use std::fs;
use tempfile::TempDir;
use wow_mpq::{Archive, ArchiveBuilder};

#[test]
fn test_path_normalization_in_archive() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("test.mpq");

    // Create test files
    let test_data = b"Hello, World!";
    let temp_file = temp_dir.path().join("test.txt");
    fs::write(&temp_file, test_data).unwrap();

    // Create archive with files using different path separators
    ArchiveBuilder::new()
        .add_file(&temp_file, "dir/subdir/file1.txt") // Forward slashes
        .add_file(&temp_file, "dir\\subdir\\file2.txt") // Backslashes
        .add_file(&temp_file, "dir/subdir\\file3.txt") // Mixed
        .build(&archive_path)
        .unwrap();

    // Open archive and verify all files can be found
    let archive = Archive::open(&archive_path).unwrap();

    // All these lookups should work regardless of separator used
    assert!(archive.read_file("dir/subdir/file1.txt").is_ok());
    assert!(archive.read_file("dir\\subdir\\file1.txt").is_ok());

    assert!(archive.read_file("dir/subdir/file2.txt").is_ok());
    assert!(archive.read_file("dir\\subdir\\file2.txt").is_ok());

    assert!(archive.read_file("dir/subdir/file3.txt").is_ok());
    assert!(archive.read_file("dir\\subdir\\file3.txt").is_ok());
}

#[test]
fn test_listfile_path_separators() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("test_list.mpq");

    // Create test files
    let test_data = b"Test content";
    let temp_file = temp_dir.path().join("test.txt");
    fs::write(&temp_file, test_data).unwrap();

    // Create archive with various path formats
    ArchiveBuilder::new()
        .add_file(&temp_file, "root.txt")
        .add_file(&temp_file, "folder/file.txt")
        .add_file(&temp_file, "deep/path/to/file.txt")
        .add_file(&temp_file, "mixed\\path/separators\\file.txt")
        .listfile_option(wow_mpq::ListfileOption::Generate)
        .build(&archive_path)
        .unwrap();

    // Open and list files
    let archive = Archive::open(&archive_path).unwrap();
    let entries = archive.list().unwrap();

    // Should have 5 files (4 added + listfile)
    assert_eq!(entries.len(), 5);

    // All paths should be normalized to backslashes internally
    let names: Vec<String> = entries.iter().map(|e| e.name.clone()).collect();
    assert!(names.contains(&"(listfile)".to_string()));
    assert!(names.contains(&"root.txt".to_string()));
    assert!(names.contains(&"folder\\file.txt".to_string()));
    assert!(names.contains(&"deep\\path\\to\\file.txt".to_string()));
    assert!(names.contains(&"mixed\\path\\separators\\file.txt".to_string()));
}

#[test]
fn test_extraction_path_conversion() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("test_extract.mpq");
    let extract_dir = temp_dir.path().join("extracted");
    fs::create_dir_all(&extract_dir).unwrap();

    // Create test data
    let test_data = b"Extracted content";
    let temp_file = temp_dir.path().join("source.txt");
    fs::write(&temp_file, test_data).unwrap();

    // Create archive with nested path
    ArchiveBuilder::new()
        .add_file(&temp_file, "data/config/settings.ini")
        .build(&archive_path)
        .unwrap();

    // Extract file
    let archive = Archive::open(&archive_path).unwrap();
    let file_data = archive.read_file("data/config/settings.ini").unwrap();

    // When extracting with preserved paths, system separators should be used
    let output_path = extract_dir.join(wow_mpq::path::mpq_path_to_system(
        "data\\config\\settings.ini",
    ));

    // Create parent directories
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    // Write extracted file
    fs::write(&output_path, file_data).unwrap();

    // Verify file exists at correct location
    assert!(output_path.exists());
    assert_eq!(fs::read(&output_path).unwrap(), test_data);
}

#[test]
fn test_hash_consistency_with_separators() {
    // Test that the hash function produces consistent results
    use wow_mpq::crypto::{hash_string, hash_type};

    let path1 = "interface/glue/mainmenu.blp";
    let path2 = "interface\\glue\\mainmenu.blp";

    // Both paths should produce identical hashes
    assert_eq!(
        hash_string(path1, hash_type::TABLE_OFFSET),
        hash_string(path2, hash_type::TABLE_OFFSET)
    );

    assert_eq!(
        hash_string(path1, hash_type::NAME_A),
        hash_string(path2, hash_type::NAME_A)
    );

    assert_eq!(
        hash_string(path1, hash_type::NAME_B),
        hash_string(path2, hash_type::NAME_B)
    );
}

#[test]
fn test_jenkins_hash_consistency() {
    // Test Jenkins hash (used for HET tables) also handles separators
    use wow_mpq::crypto::jenkins_hash;

    let path1 = "units/human/footman.mdx";
    let path2 = "units\\human\\footman.mdx";

    // Jenkins hash should also normalize paths
    assert_eq!(jenkins_hash(path1), jenkins_hash(path2));
}
