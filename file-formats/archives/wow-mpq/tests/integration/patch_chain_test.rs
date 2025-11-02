// Integration test for patch chain functionality
//
// This test creates synthetic patch files to verify the complete patch chain workflow

use std::path::Path;
use tempfile::TempDir;
use wow_mpq::{ArchiveBuilder, ListfileOption, PatchChain};

/// Helper to create a test MPQ archive
fn create_archive(dir: &Path, name: &str, files: &[(&str, &[u8])]) -> std::path::PathBuf {
    let path = dir.join(name);
    let mut builder = ArchiveBuilder::new().listfile_option(ListfileOption::Generate);

    for (filename, data) in files {
        builder = builder.add_file_data(data.to_vec(), filename);
    }

    builder.build(&path).unwrap();
    path
}

#[test]
fn test_patch_chain_with_regular_files() {
    let temp = TempDir::new().unwrap();

    // Create base archive with SIMPLE filenames (like the working tests)
    let base_files: Vec<(&str, &[u8])> = vec![
        ("file1.txt", b"Base version of file1"),
        ("file2.txt", b"Base version of file2"),
        ("unchanged.txt", b"This file stays the same"),
    ];
    let base_path = create_archive(temp.path(), "base.mpq", &base_files);

    // Create patch archive that overrides some files
    let patch_files: Vec<(&str, &[u8])> = vec![
        ("file1.txt", b"Patched version of file1"),
        ("new.txt", b"New file added in patch"),
    ];
    let patch_path = create_archive(temp.path(), "patch.mpq", &patch_files);

    // Build patch chain
    let mut chain = PatchChain::new();
    chain.add_archive(&base_path, 0).unwrap();
    chain.add_archive(&patch_path, 100).unwrap();

    // Test file priority - patch should override base
    assert_eq!(
        chain.read_file("file1.txt").unwrap(),
        b"Patched version of file1"
    );
    assert_eq!(
        chain.read_file("file2.txt").unwrap(),
        b"Base version of file2"
    );
    assert_eq!(
        chain.read_file("unchanged.txt").unwrap(),
        b"This file stays the same"
    );
    assert_eq!(
        chain.read_file("new.txt").unwrap(),
        b"New file added in patch"
    );

    // Verify chain info
    let chain_info = chain.get_chain_info();
    assert_eq!(chain_info.len(), 2);
    assert_eq!(chain_info[0].priority, 100); // Highest priority first
    assert_eq!(chain_info[1].priority, 0);
}

#[test]
fn test_multiple_patch_chain() {
    let temp = TempDir::new().unwrap();

    // Create base archive
    let base_files: Vec<(&str, &[u8])> = vec![("version.txt", b"1.0.0")];
    let base_path = create_archive(temp.path(), "base.mpq", &base_files);

    // Create multiple patches
    let patch1_files: Vec<(&str, &[u8])> = vec![("version.txt", b"1.1.0")];
    let patch1_path = create_archive(temp.path(), "patch-1.mpq", &patch1_files);

    let patch2_files: Vec<(&str, &[u8])> = vec![("version.txt", b"1.2.0")];
    let patch2_path = create_archive(temp.path(), "patch-2.mpq", &patch2_files);

    let patch3_files: Vec<(&str, &[u8])> = vec![("version.txt", b"1.3.0")];
    let patch3_path = create_archive(temp.path(), "patch-3.mpq", &patch3_files);

    // Build chain with all patches
    let mut chain = PatchChain::new();
    chain.add_archive(&base_path, 0).unwrap();
    chain.add_archive(&patch1_path, 100).unwrap();
    chain.add_archive(&patch2_path, 200).unwrap();
    chain.add_archive(&patch3_path, 300).unwrap();

    // Highest priority patch should win
    assert_eq!(chain.read_file("version.txt").unwrap(), b"1.3.0");

    // Verify chain has correct number of archives
    assert_eq!(chain.archive_count(), 4);
}

#[test]
fn test_parallel_chain_loading() {
    let temp = TempDir::new().unwrap();

    // Create multiple archives
    let mut archives = Vec::new();

    for i in 0..5 {
        let file_content = format!("Content from archive {}", i);
        let unique_name = format!("unique_{}.txt", i);
        let archive_name = format!("archive_{}.mpq", i);
        let files: Vec<(&str, &[u8])> = vec![
            ("common.txt", file_content.as_bytes()),
            (&unique_name, file_content.as_bytes()),
        ];
        let path = create_archive(temp.path(), &archive_name, &files);
        archives.push((path, i * 100));
    }

    // Load in parallel
    let mut chain = PatchChain::from_archives_parallel(archives).unwrap();

    // Highest priority should win for common.txt
    assert_eq!(
        chain.read_file("common.txt").unwrap(),
        b"Content from archive 4"
    );

    // All unique files should be accessible
    for i in 0..5 {
        let filename = format!("unique_{}.txt", i);
        let expected = format!("Content from archive {}", i);
        assert_eq!(chain.read_file(&filename).unwrap(), expected.as_bytes());
    }
}

#[test]
fn test_chain_file_location() {
    let temp = TempDir::new().unwrap();

    let base_files: Vec<(&str, &[u8])> = vec![("base.txt", b"base")];
    let patch_files: Vec<(&str, &[u8])> = vec![("patch.txt", b"patch")];

    let base_path = create_archive(temp.path(), "base.mpq", &base_files);
    let patch_path = create_archive(temp.path(), "patch.mpq", &patch_files);

    let mut chain = PatchChain::new();
    chain.add_archive(&base_path, 0).unwrap();
    chain.add_archive(&patch_path, 100).unwrap();

    // Verify file locations
    assert_eq!(
        chain.find_file_archive("base.txt"),
        Some(base_path.as_path())
    );
    assert_eq!(
        chain.find_file_archive("patch.txt"),
        Some(patch_path.as_path())
    );
    assert_eq!(chain.find_file_archive("nonexistent.txt"), None);

    // Verify contains
    assert!(chain.contains_file("base.txt"));
    assert!(chain.contains_file("patch.txt"));
    assert!(!chain.contains_file("nonexistent.txt"));
}

#[test]
fn test_chain_listing() {
    let temp = TempDir::new().unwrap();

    let base_files: Vec<(&str, &[u8])> = vec![("file1.txt", b"1"), ("file2.txt", b"2")];
    let patch_files: Vec<(&str, &[u8])> = vec![("file2.txt", b"2-patched"), ("file3.txt", b"3")];

    let base_path = create_archive(temp.path(), "base.mpq", &base_files);
    let patch_path = create_archive(temp.path(), "patch.mpq", &patch_files);

    let mut chain = PatchChain::new();
    chain.add_archive(&base_path, 0).unwrap();
    chain.add_archive(&patch_path, 100).unwrap();

    let files = chain.list().unwrap();

    // Filter out listfile
    let user_files: Vec<_> = files
        .into_iter()
        .filter(|f| f.name != "(listfile)")
        .collect();

    // Should have 3 unique files (file1, file2, file3)
    assert_eq!(user_files.len(), 3);

    let names: Vec<_> = user_files.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"file1.txt"));
    assert!(names.contains(&"file2.txt"));
    assert!(names.contains(&"file3.txt"));
}
