//! Thread safety tests for single archive parallel operations

use std::sync::{Arc, Barrier};
use std::thread;
use tempfile::TempDir;
use wow_mpq::single_archive_parallel::{ParallelArchive, ParallelConfig, extract_with_config};
use wow_mpq::{ArchiveBuilder, compression::flags};

/// Create a test archive with multiple files for testing
fn create_large_test_archive() -> (TempDir, std::path::PathBuf) {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test.mpq");

    let mut builder = ArchiveBuilder::new()
        .block_size(7) // 64KB sectors
        .default_compression(flags::ZLIB);

    // Add 100 files with varying sizes
    for i in 0..100 {
        let content = format!("File {} content: ", i);
        let content = content.repeat(100 + i); // Varying sizes
        builder = builder.add_file_data(content.into_bytes(), &format!("test/file_{:03}.txt", i));
    }

    // Add some larger files
    for i in 0..10 {
        let large_content = format!("Large file {} with lots of data\n", i).repeat(1000);
        builder = builder.add_file_data(
            large_content.into_bytes(),
            &format!("test/large_{:02}.dat", i),
        );
    }

    builder.build(&path).unwrap();
    (temp, path)
}

#[test]
fn test_concurrent_reads_same_files() {
    let (_temp, archive_path) = create_large_test_archive();
    let archive = Arc::new(ParallelArchive::open(&archive_path).unwrap());

    let num_threads = 8;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = vec![];

    // All threads try to read the same files
    let target_files = vec![
        "test/file_050.txt",
        "test/large_05.dat",
        "test/file_099.txt",
    ];

    for thread_id in 0..num_threads {
        let archive = Arc::clone(&archive);
        let barrier = Arc::clone(&barrier);
        let files = target_files.clone();

        let handle = thread::spawn(move || {
            // Synchronize all threads to start at the same time
            barrier.wait();

            // Extract files
            let results = archive.extract_files_parallel(&files).unwrap();

            // Verify results
            assert_eq!(results.len(), files.len());
            for (filename, data) in &results {
                assert!(!data.is_empty());
                assert!(files.contains(&filename.as_str()));
            }

            println!(
                "Thread {} successfully read {} files",
                thread_id,
                results.len()
            );
            results
        });

        handles.push(handle);
    }

    // Wait for all threads and verify they got the same data
    let mut all_results = Vec::new();
    for handle in handles {
        all_results.push(handle.join().unwrap());
    }

    // Verify all threads got identical data
    let reference = &all_results[0];
    for results in &all_results[1..] {
        assert_eq!(results.len(), reference.len());
        for i in 0..results.len() {
            assert_eq!(results[i].0, reference[i].0); // Same filename
            assert_eq!(results[i].1, reference[i].1); // Same data
        }
    }
}

#[test]
fn test_concurrent_reads_different_files() {
    let (_temp, archive_path) = create_large_test_archive();
    let archive = Arc::new(ParallelArchive::open(&archive_path).unwrap());

    let num_threads = 10;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let archive = Arc::clone(&archive);
        let barrier = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            // Each thread reads different files
            let start = thread_id * 10;
            let files: Vec<&str> = (start..start + 10)
                .map(|i| Box::leak(format!("test/file_{:03}.txt", i).into_boxed_str()) as &str)
                .collect();

            // Synchronize start
            barrier.wait();

            // Extract files
            let results = archive.extract_files_parallel(&files).unwrap();

            assert_eq!(results.len(), 10);
            for (_filename, data) in &results {
                assert!(!data.is_empty());
            }

            println!(
                "Thread {} extracted files {} to {}",
                thread_id,
                start,
                start + 9
            );
            results
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        let results = handle.join().unwrap();
        assert_eq!(results.len(), 10);
    }
}

#[test]
fn test_pattern_matching_parallel() {
    let (_temp, archive_path) = create_large_test_archive();
    let archive = ParallelArchive::open(&archive_path).unwrap();

    // Extract all files containing "5" in the name
    let results = archive
        .extract_matching_parallel(|name| name.contains("5"))
        .unwrap();

    // Should match file_005, file_015, ..., file_095, file_050-059, large_05
    assert!(results.len() > 10);

    for (filename, data) in &results {
        assert!(filename.contains("5"));
        assert!(!data.is_empty());
    }
}

#[test]
fn test_batched_extraction_performance() {
    let (_temp, archive_path) = create_large_test_archive();
    let archive = ParallelArchive::open(&archive_path).unwrap();

    // Extract many small files
    let files: Vec<&str> = (0..50)
        .map(|i| Box::leak(format!("test/file_{:03}.txt", i).into_boxed_str()) as &str)
        .collect();

    // Test different batch sizes
    for batch_size in &[1, 5, 10, 20] {
        let results = archive.extract_files_batched(&files, *batch_size).unwrap();
        assert_eq!(results.len(), 50);

        // Verify all files extracted correctly
        for (i, (filename, data)) in results.iter().enumerate() {
            assert_eq!(filename, &format!("test/file_{:03}.txt", i));
            assert!(!data.is_empty());
        }
    }
}

#[test]
fn test_custom_processing() {
    let (_temp, archive_path) = create_large_test_archive();
    let archive = ParallelArchive::open(&archive_path).unwrap();

    let files = vec![
        "test/large_00.dat",
        "test/large_01.dat",
        "test/large_02.dat",
    ];

    // Process files to get their sizes and first few bytes
    let results = archive
        .process_files_parallel(&files, |filename, data| {
            Ok((
                filename.to_string(),
                data.len(),
                data[..10.min(data.len())].to_vec(),
            ))
        })
        .unwrap();

    assert_eq!(results.len(), 3);
    for (filename, size, preview) in &results {
        assert!(filename.starts_with("test/large_"));
        assert!(*size > 1000); // Large files
        assert_eq!(preview.len(), 10);
    }
}

#[test]
fn test_error_handling() {
    let (_temp, archive_path) = create_large_test_archive();
    let archive = ParallelArchive::open(&archive_path).unwrap();

    // Mix valid and invalid files
    let files = vec![
        "test/file_000.txt",
        "nonexistent.txt",
        "test/file_001.txt",
        "also/missing.dat",
    ];

    // Without skip_errors, should fail
    let result = archive.extract_files_parallel(&files);
    assert!(result.is_err());

    // With skip_errors config
    let config = ParallelConfig::new().skip_errors(true);
    let results = extract_with_config(&archive_path, &files, config).unwrap();

    assert_eq!(results.len(), 4);
    assert!(results[0].1.is_ok()); // file_000.txt exists
    assert!(results[1].1.is_err()); // nonexistent.txt
    assert!(results[2].1.is_ok()); // file_001.txt exists
    assert!(results[3].1.is_err()); // also/missing.dat
}

#[test]
fn test_custom_thread_pool() {
    let (_temp, archive_path) = create_large_test_archive();

    // Test with different thread counts
    for num_threads in &[1, 2, 4, 8] {
        let config = ParallelConfig::new().threads(*num_threads).batch_size(5);

        let files: Vec<&str> = (0..20)
            .map(|i| Box::leak(format!("test/file_{:03}.txt", i).into_boxed_str()) as &str)
            .collect();

        let results = extract_with_config(&archive_path, &files, config).unwrap();
        assert_eq!(results.len(), 20);

        for result in &results {
            assert!(result.1.is_ok());
        }
    }
}

#[test]
fn test_stress_many_concurrent_operations() {
    let (_temp, archive_path) = create_large_test_archive();
    let archive = Arc::new(ParallelArchive::open(&archive_path).unwrap());

    let num_operations = 50;
    let barrier = Arc::new(Barrier::new(num_operations));
    let mut handles = vec![];

    // Launch many operations with different patterns
    for op_id in 0..num_operations {
        let archive = Arc::clone(&archive);
        let barrier = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            barrier.wait();

            match op_id % 5 {
                0 => {
                    // Extract specific files
                    let files = vec!["test/file_000.txt", "test/file_050.txt"];
                    archive.extract_files_parallel(&files).unwrap()
                }
                1 => {
                    // Extract by pattern
                    let results = archive
                        .extract_matching_parallel(|name| name.ends_with(".dat"))
                        .unwrap();
                    results.into_iter().take(3).collect()
                }
                2 => {
                    // Batched extraction
                    let files: Vec<&str> = vec!["test/file_010.txt", "test/file_020.txt"];
                    archive.extract_files_batched(&files, 2).unwrap()
                }
                3 => {
                    // Custom processing
                    let files = vec!["test/large_00.dat"];
                    archive
                        .process_files_parallel(&files, |name, data| Ok((name.to_string(), data)))
                        .unwrap()
                }
                _ => {
                    // Single file
                    let file = format!("test/file_{:03}.txt", op_id % 100);
                    vec![(
                        file.clone(),
                        archive.read_file_with_new_handle(&file).unwrap(),
                    )]
                }
            }
        });

        handles.push(handle);
    }

    // All operations should complete successfully
    for handle in handles {
        let results = handle.join().unwrap();
        assert!(!results.is_empty());
    }
}

#[test]
fn test_archive_path_not_found() {
    let result = ParallelArchive::open("/nonexistent/path/archive.mpq");
    assert!(result.is_err());
}

#[test]
fn test_empty_file_list() {
    let (_temp, archive_path) = create_large_test_archive();
    let archive = ParallelArchive::open(&archive_path).unwrap();

    let files: Vec<&str> = vec![];
    let results = archive.extract_files_parallel(&files).unwrap();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_thread_count() {
    let (_temp, archive_path) = create_large_test_archive();
    let archive = ParallelArchive::open(&archive_path).unwrap();

    let thread_count = archive.thread_count();
    assert!(thread_count > 0);
    println!("Current thread count: {}", thread_count);
}
