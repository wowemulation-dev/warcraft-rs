//! Demonstrates single archive parallel processing capabilities
//!
//! This example shows how to use the ParallelArchive API to extract
//! multiple files from a single MPQ archive concurrently.

use std::env;
use std::time::Instant;
use wow_mpq::single_archive_parallel::{ParallelArchive, ParallelConfig, extract_with_config};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <archive.mpq> [pattern]", args[0]);
        eprintln!("Example: {} patch-3.MPQ Interface", args[0]);
        return Ok(());
    }

    let archive_path = &args[1];
    let pattern = args.get(2).map(|s| s.as_str());

    println!("Opening archive: {}", archive_path);
    let archive = ParallelArchive::open(archive_path)?;

    // Example 1: Extract specific files
    println!("\n=== Example 1: Extract Specific Files ===");
    let specific_files = vec!["(listfile)", "(attributes)"];

    match archive.extract_files_parallel(&specific_files) {
        Ok(results) => {
            for (filename, data) in results {
                println!("Extracted {}: {} bytes", filename, data.len());
            }
        }
        Err(e) => {
            println!("Note: Some files may not exist in this archive: {}", e);
        }
    }

    // Example 2: Extract by pattern
    if let Some(pattern) = pattern {
        println!("\n=== Example 2: Extract Files Matching '{}' ===", pattern);
        let start = Instant::now();

        let results = archive.extract_matching_parallel(|name| name.contains(pattern))?;

        let elapsed = start.elapsed();
        let total_size: usize = results.iter().map(|(_, data)| data.len()).sum();

        println!(
            "Extracted {} files matching '{}' in {:?}",
            results.len(),
            pattern,
            elapsed
        );
        println!("Total size: {} MB", total_size / 1024 / 1024);

        // Show first few files
        for (filename, data) in results.iter().take(5) {
            println!("  - {}: {} bytes", filename, data.len());
        }
        if results.len() > 5 {
            println!("  ... and {} more files", results.len() - 5);
        }
    }

    // Example 3: Custom processing (calculate checksums)
    println!("\n=== Example 3: Calculate File Checksums ===");
    let files_to_checksum = vec!["(listfile)", "(attributes)"];

    let checksums = archive.process_files_parallel(&files_to_checksum, |name, data| {
        // Simple checksum: sum of all bytes
        let checksum: u64 = data.iter().map(|&b| b as u64).sum();
        Ok((name.to_string(), checksum))
    });

    match checksums {
        Ok(results) => {
            for (name, checksum) in results {
                println!("{}: checksum = {}", name, checksum);
            }
        }
        Err(_) => {
            println!("Note: Some files may not exist for checksum calculation");
        }
    }

    // Example 4: Benchmark sequential vs parallel
    println!("\n=== Example 4: Performance Comparison ===");

    // Get list of files to extract
    let all_files = archive.list_files();
    let test_files: Vec<&str> = all_files
        .iter()
        .take(50) // Test with first 50 files
        .map(|s| s.as_str())
        .collect();

    if !test_files.is_empty() {
        println!("Comparing extraction of {} files...", test_files.len());

        // Sequential extraction
        let start = Instant::now();
        let mut sequential_results = Vec::new();
        for &file in &test_files {
            if let Ok(data) = archive.read_file_with_new_handle(file) {
                sequential_results.push((file, data));
            }
        }
        let sequential_time = start.elapsed();

        // Parallel extraction
        let start = Instant::now();
        let parallel_results = archive.extract_files_parallel(&test_files)?;
        let parallel_time = start.elapsed();

        println!(
            "Sequential: {:?} ({} files)",
            sequential_time,
            sequential_results.len()
        );
        println!(
            "Parallel:   {:?} ({} files)",
            parallel_time,
            parallel_results.len()
        );
        println!(
            "Speedup:    {:.2}x",
            sequential_time.as_secs_f64() / parallel_time.as_secs_f64()
        );
    }

    // Example 5: Error handling with mixed files
    println!("\n=== Example 5: Error Handling ===");
    let mixed_files = vec![
        "(listfile)",
        "nonexistent_file.txt",
        "(attributes)",
        "another_missing.dat",
    ];

    let config = ParallelConfig::new().skip_errors(true).threads(4);

    let results = extract_with_config(archive_path, &mixed_files, config)?;

    for (filename, result) in results {
        match result {
            Ok(data) => println!("{}: SUCCESS ({} bytes)", filename, data.len()),
            Err(e) => println!("{}: ERROR - {}", filename, e),
        }
    }

    // Show thread count
    println!("\n=== System Information ===");
    println!("Available threads: {}", archive.thread_count());

    Ok(())
}
