//! Integration tests for MPQ extraction functionality
//!
//! These tests validate that the MPQ extraction works correctly in practice,
//! testing real-world scenarios with actual WoW MPQ archives.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tempfile::TempDir;
use wow_mpq::{Archive, PatchChain};

/// Test configuration for different MPQ archives
struct TestArchive {
    path: PathBuf,
    name: String,
    expected_min_files: usize,
}

/// Test result tracking
#[derive(Debug)]
struct ExtractionTestResult {
    archive_name: String,
    files_requested: usize,
    files_extracted: usize,
    files_failed: usize,
    total_size: u64,
    duration_ms: u128,
    errors: Vec<String>,
}

/// Get available test archives from the WoW installation directories
fn get_test_archives() -> Vec<TestArchive> {
    let mut archives = Vec::new();

    // WoW 1.12.1 archives
    let wow_1121_data = "/home/danielsreichenbach/Downloads/wow/1.12.1/Data";
    if Path::new(wow_1121_data).exists() {
        for entry in ["base.MPQ", "dbc.MPQ", "interface.MPQ", "patch.MPQ"] {
            let path = Path::new(wow_1121_data).join(entry);
            if path.exists() {
                archives.push(TestArchive {
                    path,
                    name: format!("1.12.1/{}", entry),
                    expected_min_files: match entry {
                        "base.MPQ" => 1000,     // Large archive
                        "dbc.MPQ" => 50,        // Small archive
                        "interface.MPQ" => 100, // Medium archive
                        "patch.MPQ" => 10,      // Tiny archive
                        _ => 1,
                    },
                });
            }
        }
    }

    // WoW 3.3.5a archives
    let wow_335a_data = "/home/danielsreichenbach/Downloads/wow/3.3.5a/Data";
    if Path::new(wow_335a_data).exists() {
        for entry in ["common.MPQ", "expansion.MPQ", "lichking.MPQ", "patch.MPQ"] {
            let path = Path::new(wow_335a_data).join(entry);
            if path.exists() {
                archives.push(TestArchive {
                    path,
                    name: format!("3.3.5a/{}", entry),
                    expected_min_files: match entry {
                        "common.MPQ" => 1000,
                        "expansion.MPQ" => 500,
                        "lichking.MPQ" => 500,
                        "patch.MPQ" => 10,
                        _ => 1,
                    },
                });
            }
        }
    }

    // WoW 4.3.4 archives (English)
    let wow_434_data = "/home/danielsreichenbach/Downloads/wow/4.3.4/4.3.4/Data";
    if Path::new(wow_434_data).exists() {
        for entry in ["world.MPQ", "world2.MPQ"] {
            let path = Path::new(wow_434_data).join(entry);
            if path.exists() {
                archives.push(TestArchive {
                    path,
                    name: format!("4.3.4/{}", entry),
                    expected_min_files: 1000,
                });
            }
        }
    }

    archives
}

/// Test individual file extraction (2-3 files)
#[test]
fn test_individual_file_extraction() -> Result<()> {
    let archives = get_test_archives();
    if archives.is_empty() {
        println!("No test archives found, skipping individual file extraction test");
        return Ok(());
    }

    let test_archive = &archives[0]; // Use first available archive
    let temp_dir = TempDir::new().context("Failed to create temp directory")?;

    println!(
        "Testing individual file extraction with: {}",
        test_archive.name
    );

    // Open the archive and get a list of files
    let mut archive = Archive::open(&test_archive.path).with_context(|| {
        format!(
            "Failed to open test archive: {}",
            test_archive.path.display()
        )
    })?;

    let file_list = archive.list().context("Failed to list archive files")?;

    if file_list.is_empty() {
        return Err(anyhow::anyhow!("Archive contains no files"));
    }

    // Select 2-3 files of different types for testing
    let mut test_files = Vec::new();
    let mut file_types_found = HashMap::new();

    for entry in &file_list {
        let extension = Path::new(&entry.name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !file_types_found.contains_key(&extension) && test_files.len() < 3 {
            file_types_found.insert(extension.clone(), entry.name.clone());
            test_files.push(entry.name.clone());
        }

        if test_files.len() >= 3 {
            break;
        }
    }

    if test_files.is_empty() {
        test_files.push(file_list[0].name.clone());
    }

    println!(
        "Testing extraction of {} files: {:?}",
        test_files.len(),
        test_files
    );

    // Test extraction
    let start_time = Instant::now();
    let mut success_count = 0;
    let mut error_count = 0;
    let mut total_size = 0u64;

    for file_name in &test_files {
        match archive.read_file(file_name) {
            Ok(data) => {
                // Write to temp directory
                let output_path = temp_dir.path().join(sanitize_filename(file_name));
                fs::write(&output_path, &data).with_context(|| {
                    format!("Failed to write extracted file: {}", output_path.display())
                })?;

                // Verify file size
                let written_size = fs::metadata(&output_path)?.len();
                assert_eq!(
                    written_size,
                    data.len() as u64,
                    "Written file size mismatch for {}",
                    file_name
                );

                total_size += data.len() as u64;
                success_count += 1;
                println!("✓ Extracted {}: {} bytes", file_name, data.len());
            }
            Err(e) => {
                error_count += 1;
                println!("✗ Failed to extract {}: {}", file_name, e);
            }
        }
    }

    let duration = start_time.elapsed();

    println!("Individual file extraction results:");
    println!("  Files requested: {}", test_files.len());
    println!("  Files extracted: {}", success_count);
    println!("  Files failed: {}", error_count);
    println!("  Total size: {} bytes", total_size);
    println!("  Duration: {}ms", duration.as_millis());

    // Assert at least some files were extracted successfully
    assert!(success_count > 0, "No files were extracted successfully");
    assert!(
        success_count >= test_files.len() / 2,
        "Too many extraction failures: {}/{}",
        error_count,
        test_files.len()
    );

    Ok(())
}

/// Test medium batch extraction (50-100 files)
#[test]
fn test_medium_batch_extraction() -> Result<()> {
    let archives = get_test_archives();
    if archives.is_empty() {
        println!("No test archives found, skipping medium batch extraction test");
        return Ok(());
    }

    // Find an archive with enough files
    let test_archive = archives
        .iter()
        .find(|a| a.expected_min_files >= 100)
        .unwrap_or(&archives[0]);

    let temp_dir = TempDir::new().context("Failed to create temp directory")?;

    println!(
        "Testing medium batch extraction with: {}",
        test_archive.name
    );

    let result = test_batch_extraction(
        &test_archive.path,
        temp_dir.path(),
        50,    // Target file count
        100,   // Max file count
        false, // Don't preserve paths for this test
    )?;

    println!("Medium batch extraction results:");
    print_test_result(&result);

    // Validate results
    assert!(result.files_extracted > 0, "No files were extracted");
    assert!(
        result.files_extracted >= result.files_requested / 2,
        "Too many extraction failures: {}/{}",
        result.files_failed,
        result.files_requested
    );

    // Performance check - should extract reasonably quickly
    let files_per_second = result.files_extracted as f64 / (result.duration_ms as f64 / 1000.0);
    println!("  Performance: {:.2} files/second", files_per_second);
    assert!(
        files_per_second > 1.0,
        "Extraction too slow: {:.2} files/second",
        files_per_second
    );

    Ok(())
}

/// Test large batch extraction (1000+ files)
#[test]
fn test_large_batch_extraction() -> Result<()> {
    let archives = get_test_archives();
    if archives.is_empty() {
        println!("No test archives found, skipping large batch extraction test");
        return Ok(());
    }

    // Find an archive with many files
    let test_archive = archives
        .iter()
        .find(|a| a.expected_min_files >= 1000)
        .unwrap_or(&archives[0]);

    let temp_dir = TempDir::new().context("Failed to create temp directory")?;

    println!("Testing large batch extraction with: {}", test_archive.name);

    let result = test_batch_extraction(
        &test_archive.path,
        temp_dir.path(),
        500,   // Target file count
        1500,  // Max file count
        false, // Don't preserve paths for this test
    )?;

    println!("Large batch extraction results:");
    print_test_result(&result);

    // Validate results
    assert!(result.files_extracted > 0, "No files were extracted");
    assert!(
        result.files_extracted >= result.files_requested / 2,
        "Too many extraction failures: {}/{}",
        result.files_failed,
        result.files_requested
    );

    // Performance check - batching should maintain good throughput
    let files_per_second = result.files_extracted as f64 / (result.duration_ms as f64 / 1000.0);
    println!("  Performance: {:.2} files/second", files_per_second);
    assert!(
        files_per_second > 5.0,
        "Large extraction too slow: {:.2} files/second",
        files_per_second
    );

    Ok(())
}

/// Test directory structure preservation
#[test]
fn test_preserve_paths() -> Result<()> {
    let archives = get_test_archives();
    if archives.is_empty() {
        println!("No test archives found, skipping preserve paths test");
        return Ok(());
    }

    let test_archive = &archives[0];
    let temp_dir = TempDir::new().context("Failed to create temp directory")?;

    println!(
        "Testing directory structure preservation with: {}",
        test_archive.name
    );

    let result = test_batch_extraction(
        &test_archive.path,
        temp_dir.path(),
        20, // Smaller number for this test
        50,
        true, // Preserve paths
    )?;

    println!("Preserve paths extraction results:");
    print_test_result(&result);

    // Validate that directories were created
    let mut found_subdirs = false;
    for entry in fs::read_dir(temp_dir.path())? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            found_subdirs = true;
            println!(
                "  Found subdirectory: {}",
                entry.file_name().to_string_lossy()
            );
        }
    }

    if result.files_extracted > 0 {
        // We should have at least some directory structure if files were extracted
        // (unless all files were in the root, which is unlikely)
        println!("  Directory structure preserved: {}", found_subdirs);
    }

    Ok(())
}

/// Test error handling with missing files
#[test]
fn test_error_handling_missing_files() -> Result<()> {
    let archives = get_test_archives();
    if archives.is_empty() {
        println!("No test archives found, skipping error handling test");
        return Ok(());
    }

    let test_archive = &archives[0];
    let _temp_dir = TempDir::new().context("Failed to create temp directory")?;

    println!(
        "Testing error handling with missing files: {}",
        test_archive.name
    );

    let mut archive = Archive::open(&test_archive.path).with_context(|| {
        format!(
            "Failed to open test archive: {}",
            test_archive.path.display()
        )
    })?;

    // Try to extract files that don't exist
    let nonexistent_files = vec![
        "does_not_exist.txt",
        "fake/path/file.dat",
        "another_missing_file.bin",
    ];

    let mut error_count = 0;

    for file_name in &nonexistent_files {
        match archive.read_file(file_name) {
            Ok(_) => {
                println!("  Unexpected success for: {}", file_name);
            }
            Err(e) => {
                error_count += 1;
                println!("  ✓ Expected error for {}: {}", file_name, e);
            }
        }
    }

    // All files should have failed
    assert_eq!(
        error_count,
        nonexistent_files.len(),
        "Expected all nonexistent files to fail"
    );

    println!(
        "Error handling test passed: {}/{} expected failures",
        error_count,
        nonexistent_files.len()
    );

    Ok(())
}

/// Test data integrity verification
#[test]
fn test_data_integrity() -> Result<()> {
    let archives = get_test_archives();
    if archives.is_empty() {
        println!("No test archives found, skipping data integrity test");
        return Ok(());
    }

    let test_archive = &archives[0];
    let temp_dir = TempDir::new().context("Failed to create temp directory")?;

    println!("Testing data integrity with: {}", test_archive.name);

    let mut archive = Archive::open(&test_archive.path).with_context(|| {
        format!(
            "Failed to open test archive: {}",
            test_archive.path.display()
        )
    })?;

    let file_list = archive.list().context("Failed to list archive files")?;

    if file_list.is_empty() {
        println!("  Archive contains no files, skipping integrity test");
        return Ok(());
    }

    // Test up to 10 files for integrity
    let test_count = std::cmp::min(10, file_list.len());
    let mut integrity_checks = 0;

    for entry in file_list.iter().take(test_count) {
        match archive.read_file(&entry.name) {
            Ok(data) => {
                // Verify the data size matches the expected size
                assert_eq!(
                    data.len() as u64,
                    entry.size,
                    "Data size mismatch for {}: got {} bytes, expected {} bytes",
                    entry.name,
                    data.len(),
                    entry.size
                );

                // Write and re-read to verify data integrity
                let output_path = temp_dir.path().join(sanitize_filename(&entry.name));
                fs::write(&output_path, &data)
                    .with_context(|| format!("Failed to write file: {}", output_path.display()))?;

                let written_data = fs::read(&output_path).with_context(|| {
                    format!("Failed to read back file: {}", output_path.display())
                })?;

                assert_eq!(
                    data, written_data,
                    "Data integrity check failed for {}",
                    entry.name
                );

                integrity_checks += 1;
                println!(
                    "  ✓ Integrity verified for {}: {} bytes",
                    entry.name,
                    data.len()
                );
            }
            Err(e) => {
                println!("  ⚠ Failed to read {}: {}", entry.name, e);
            }
        }
    }

    assert!(integrity_checks > 0, "No files passed integrity check");
    println!(
        "Data integrity test passed: {}/{} files verified",
        integrity_checks, test_count
    );

    Ok(())
}

// Helper functions

/// Extract a batch of files and return test results
fn test_batch_extraction(
    archive_path: &Path,
    output_dir: &Path,
    target_files: usize,
    max_files: usize,
    preserve_paths: bool,
) -> Result<ExtractionTestResult> {
    let mut archive = Archive::open(archive_path)
        .with_context(|| format!("Failed to open archive: {}", archive_path.display()))?;

    let file_list = archive.list().context("Failed to list archive files")?;

    if file_list.is_empty() {
        return Err(anyhow::anyhow!("Archive contains no files"));
    }

    // Select files to extract
    let files_to_extract: Vec<String> = file_list
        .iter()
        .take(std::cmp::min(
            max_files,
            std::cmp::max(target_files, file_list.len()),
        ))
        .map(|e| e.name.clone())
        .collect();

    let files_requested = files_to_extract.len();

    println!("  Extracting {} files...", files_requested);

    let start_time = Instant::now();
    let mut files_extracted = 0;
    let mut files_failed = 0;
    let mut total_size = 0u64;
    let mut errors = Vec::new();

    for file_name in &files_to_extract {
        match archive.read_file(file_name) {
            Ok(data) => {
                let output_path = if preserve_paths {
                    // Convert MPQ path to system path and preserve directory structure
                    let system_path = file_name.replace('\\', "/");
                    output_dir.join(&system_path)
                } else {
                    // Extract just the filename
                    output_dir.join(sanitize_filename(file_name))
                };

                // Create parent directories if needed
                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent).with_context(|| {
                        format!("Failed to create directory: {}", parent.display())
                    })?;
                }

                match fs::write(&output_path, &data) {
                    Ok(()) => {
                        total_size += data.len() as u64;
                        files_extracted += 1;
                    }
                    Err(e) => {
                        files_failed += 1;
                        errors.push(format!("Failed to write {}: {}", file_name, e));
                    }
                }
            }
            Err(e) => {
                files_failed += 1;
                errors.push(format!("Failed to read {}: {}", file_name, e));
            }
        }
    }

    let duration = start_time.elapsed();

    Ok(ExtractionTestResult {
        archive_name: archive_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        files_requested,
        files_extracted,
        files_failed,
        total_size,
        duration_ms: duration.as_millis(),
        errors,
    })
}

/// Print formatted test results
fn print_test_result(result: &ExtractionTestResult) {
    println!("  Archive: {}", result.archive_name);
    println!("  Files requested: {}", result.files_requested);
    println!("  Files extracted: {}", result.files_extracted);
    println!("  Files failed: {}", result.files_failed);
    println!(
        "  Total size: {} bytes ({:.2} MB)",
        result.total_size,
        result.total_size as f64 / 1024.0 / 1024.0
    );
    println!("  Duration: {}ms", result.duration_ms);

    if !result.errors.is_empty() {
        println!("  Errors (showing first 5):");
        for error in result.errors.iter().take(5) {
            println!("    - {}", error);
        }
        if result.errors.len() > 5 {
            println!("    ... and {} more errors", result.errors.len() - 5);
        }
    }
}

/// Sanitize filename for safe filesystem operations
fn sanitize_filename(filename: &str) -> String {
    // Convert MPQ path to just the filename, replacing invalid characters
    let name = filename.split('\\').last().unwrap_or(filename);
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
}

/// Performance benchmark for large extractions
#[test]
fn test_extraction_performance() -> Result<()> {
    let archives = get_test_archives();
    if archives.is_empty() {
        println!("No test archives found, skipping performance test");
        return Ok(());
    }

    // Find the largest available archive
    let test_archive = archives
        .iter()
        .max_by_key(|a| a.expected_min_files)
        .unwrap();

    let temp_dir = TempDir::new().context("Failed to create temp directory")?;

    println!("Performance testing with: {}", test_archive.name);

    // Test different batch sizes
    let batch_sizes = [10, 50, 100, 200];

    for batch_size in batch_sizes {
        let result = test_batch_extraction(
            &test_archive.path,
            temp_dir.path(),
            batch_size,
            batch_size,
            false,
        )?;

        if result.files_extracted > 0 {
            let files_per_second =
                result.files_extracted as f64 / (result.duration_ms as f64 / 1000.0);
            let mb_per_second =
                (result.total_size as f64 / 1024.0 / 1024.0) / (result.duration_ms as f64 / 1000.0);

            println!(
                "  Batch size {}: {:.2} files/sec, {:.2} MB/sec",
                batch_size, files_per_second, mb_per_second
            );
        }

        // Clean up between tests
        for entry in fs::read_dir(temp_dir.path())? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                fs::remove_file(entry.path())?;
            } else if entry.file_type()?.is_dir() {
                fs::remove_dir_all(entry.path())?;
            }
        }
    }

    Ok(())
}
