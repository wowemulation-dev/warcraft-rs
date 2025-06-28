//! Thread safety tests for MPQ archive operations
//!
//! These tests verify that parallel operations work correctly and safely.

use std::thread;
use tempfile::TempDir;
use wow_mpq::{Archive, ArchiveBuilder, parallel};

fn create_test_archive(temp_dir: &TempDir, name: &str, file_count: usize) -> std::path::PathBuf {
    let path = temp_dir.path().join(name);
    let mut builder = ArchiveBuilder::new();

    for i in 0..file_count {
        let content = format!("File {i} content from {name}");
        builder = builder.add_file_data(content.into_bytes(), &format!("file_{i:03}.txt"));
    }

    builder.build(&path).unwrap();
    path
}

#[test]
fn test_concurrent_archive_opening() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = create_test_archive(&temp_dir, "test.mpq", 10);

    let handles: Vec<_> = (0..8)
        .map(|i| {
            let path = archive_path.clone();
            thread::spawn(move || {
                let mut archive = Archive::open(&path).unwrap();
                let data = archive.read_file("file_005.txt").unwrap();
                let content = String::from_utf8(data).unwrap();
                assert!(content.contains("File 5 content"));
                println!("Thread {i} successfully read file");
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_parallel_multi_archive_consistency() {
    let temp_dir = TempDir::new().unwrap();
    let mut archives = Vec::new();

    // Create 10 test archives
    for i in 0..10 {
        archives.push(create_test_archive(
            &temp_dir,
            &format!("archive_{i}.mpq"),
            20,
        ));
    }

    // Extract the same file from all archives in parallel
    let results = parallel::extract_from_multiple_archives(&archives, "file_010.txt").unwrap();

    // Verify all results are correct
    assert_eq!(results.len(), 10);
    for (i, (path, data)) in results.iter().enumerate() {
        assert_eq!(path, &archives[i]);
        let content = String::from_utf8_lossy(data);
        assert!(content.contains("File 10 content"));
        assert!(content.contains(&format!("archive_{i}.mpq")));
    }
}

#[test]
fn test_parallel_search_correctness() {
    let temp_dir = TempDir::new().unwrap();
    let mut archives = Vec::new();

    // Create archives with different file sets
    for i in 0..5 {
        let path = temp_dir.path().join(format!("search_{i}.mpq"));
        let mut builder = ArchiveBuilder::new();

        // Each archive has unique files plus some common ones
        for j in 0..10 {
            builder = builder.add_file_data(
                format!("Common file {j}").into_bytes(),
                &format!("common/file_{j:02}.txt"),
            );
        }

        // Unique files for each archive
        for j in 0..5 {
            builder = builder.add_file_data(
                format!("Unique to archive {i}").into_bytes(),
                &format!("unique/archive_{i}/file_{j}.txt"),
            );
        }

        builder.build(&path).unwrap();
        archives.push(path);
    }

    // Search for unique files
    let results = parallel::search_in_multiple_archives(&archives, "unique").unwrap();

    assert_eq!(results.len(), 5);

    // Verify each archive has its unique files
    for (i, (_path, matches)) in results.iter().enumerate() {
        // MPQ uses backslashes for paths
        let expected_pattern = format!("unique\\archive_{i}\\");

        // Filter to just the unique files for this archive
        let unique_files: Vec<_> = matches
            .iter()
            .filter(|name| name.contains(&expected_pattern))
            .collect();

        assert_eq!(
            unique_files.len(),
            5,
            "Archive {i} should have 5 unique files matching pattern '{expected_pattern}', found: {unique_files:?}"
        );
    }
}

#[test]
fn test_parallel_extraction_data_integrity() {
    use std::collections::HashMap;

    let temp_dir = TempDir::new().unwrap();
    let mut archives = Vec::new();
    let mut expected_data = HashMap::new();

    // Create archives with known content
    for i in 0..4 {
        let path = temp_dir.path().join(format!("integrity_{i}.mpq"));
        let mut builder = ArchiveBuilder::new();

        let files_to_extract = vec!["test1.bin", "test2.bin", "test3.bin"];

        for &file_name in &files_to_extract {
            // Create deterministic content based on archive index and file name
            let content: Vec<u8> = (0..1000).map(|j| ((i * 1000 + j) % 256) as u8).collect();

            let key = (i, file_name.to_string());
            expected_data.insert(key, content.clone());

            builder = builder.add_file_data(content, file_name);
        }

        builder.build(&path).unwrap();
        archives.push(path);
    }

    // Extract files in parallel
    let files_to_extract = vec!["test1.bin", "test2.bin", "test3.bin"];
    let results =
        parallel::extract_multiple_from_multiple_archives(&archives, &files_to_extract).unwrap();

    // Verify data integrity
    assert_eq!(results.len(), 4);
    for (i, (_path, file_results)) in results.iter().enumerate() {
        assert_eq!(file_results.len(), 3);

        for (file_name, data) in file_results {
            let key = (i, file_name.clone());
            let expected = expected_data.get(&key).unwrap();
            assert_eq!(
                data, expected,
                "Data mismatch for archive {i} file {file_name}"
            );
        }
    }
}

#[test]
fn test_parallel_processing_with_errors() {
    let temp_dir = TempDir::new().unwrap();
    let mut archives = Vec::new();

    // Create some valid archives
    for i in 0..3 {
        archives.push(create_test_archive(&temp_dir, &format!("valid_{i}.mpq"), 5));
    }

    // Add a non-existent archive path
    archives.push(temp_dir.path().join("nonexistent.mpq"));

    // Try to extract from all archives (including the invalid one)
    let result = parallel::extract_from_multiple_archives(&archives, "file_001.txt");

    // Should fail because one archive doesn't exist
    assert!(result.is_err());
}

#[test]
fn test_rayon_thread_pool_sizes() {
    use rayon::ThreadPoolBuilder;

    let temp_dir = TempDir::new().unwrap();
    let archives: Vec<_> = (0..16)
        .map(|i| create_test_archive(&temp_dir, &format!("pool_{i}.mpq"), 5))
        .collect();

    // Test with different thread pool sizes
    for num_threads in [1, 2, 4, 8] {
        let pool = ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .unwrap();

        let results =
            pool.install(|| parallel::extract_from_multiple_archives(&archives, "file_002.txt"));

        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 16);

        // Verify all extractions succeeded
        for (_path, data) in results {
            assert!(!data.is_empty());
            let content = String::from_utf8_lossy(&data);
            assert!(content.contains("File 2 content"));
        }
    }
}

#[test]
fn test_custom_processor_thread_safety() {
    let temp_dir = TempDir::new().unwrap();
    let archives: Vec<_> = (0..8)
        .map(|i| create_test_archive(&temp_dir, &format!("custom_{i}.mpq"), 10))
        .collect();

    // Use a custom processor that does multiple operations
    let results = parallel::process_archives_parallel(&archives, |mut archive| {
        let mut total_size = 0u64;
        let files = archive.list()?;

        for entry in files {
            if let Some(info) = archive.find_file(&entry.name)? {
                total_size += info.file_size;
            }
        }

        Ok(total_size)
    })
    .unwrap();

    assert_eq!(results.len(), 8);

    // All archives should have the same total size since they have identical content structure
    let first_size = results[0];
    for &size in &results[1..] {
        assert_eq!(size, first_size);
    }
}
