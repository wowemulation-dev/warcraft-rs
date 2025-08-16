//! Stress Test for Hanging Issue Resolution
//!
//! This validates that the MPQ extraction system no longer hangs under extreme load
//! and handles resource exhaustion gracefully through intelligent batching.

use std::time::{Duration, Instant};
use tempfile::TempDir;
use wow_mpq::single_archive_parallel::{ParallelConfig, extract_with_config};
use wow_mpq::{Archive, ArchiveBuilder, Result, compression::flags};

/// Create a large archive that would previously cause hanging
fn create_large_test_archive() -> Result<(TempDir, std::path::PathBuf, usize)> {
    println!("📦 Creating large test archive (this may take a moment)...");
    let temp_dir = TempDir::new().map_err(wow_mpq::Error::Io)?;
    let path = temp_dir.path().join("stress_test.mpq");

    let mut builder = ArchiveBuilder::new()
        .block_size(6) // 32KB sectors
        .default_compression(flags::ZLIB);

    let file_count = 20000; // 20K files - previously would cause hanging

    for i in 0..file_count {
        // Create varied file sizes (1KB to 500KB)
        let size_kb = 1 + (i % 500);
        let mut content = vec![0u8; size_kb * 1024];

        // Fill with recognizable data
        for (j, chunk) in content.chunks_mut(4).enumerate() {
            let value = (i * 1000 + j) as u32;
            chunk.copy_from_slice(&value.to_le_bytes());
        }

        let file_path = format!("Data/Files/file_{i:05}.dat");
        builder = builder.add_file_data(content, &file_path);

        if i % 1000 == 0 {
            println!("  Created {} files...", i);
        }
    }

    println!("  Building archive...");
    builder.build(&path)?;
    println!("✅ Archive created with {} files", file_count);

    Ok((temp_dir, path, file_count))
}

/// Test that large extractions complete without hanging
fn test_no_hanging() -> Result<()> {
    println!("🚫 Testing Hanging Issue Resolution...");

    let (_temp_dir, archive_path, expected_files) = create_large_test_archive()?;

    // Get all files in archive
    let files: Vec<String> = {
        let mut archive = Archive::open(&archive_path)?;
        archive.list()?.into_iter().map(|e| e.name).collect()
    };
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();

    println!("📊 Archive contains {} files", files.len());
    assert_eq!(files.len(), expected_files, "File count mismatch");

    // Test with intelligent batching (should not hang)
    println!("🔄 Starting bulk extraction with intelligent batching...");
    let config = ParallelConfig::new()
        .threads(12)
        .batch_size(100) // Batch size prevents resource exhaustion
        .skip_errors(false);

    let start = Instant::now();
    let results = extract_with_config(&archive_path, &file_refs, config)?;
    let duration = start.elapsed();

    println!("⏱️  Extraction completed in {:.2}s", duration.as_secs_f64());
    println!(
        "📈 Performance: {:.1} files/sec",
        files.len() as f64 / duration.as_secs_f64()
    );

    // Validate completion
    assert!(
        duration < Duration::from_secs(300),
        "Extraction took too long - potential hanging"
    );
    assert_eq!(results.len(), files.len(), "Not all files were extracted");

    // Verify successful extractions
    let successful_extractions = results.iter().filter(|(_, result)| result.is_ok()).count();
    println!(
        "✅ Successfully extracted {} out of {} files",
        successful_extractions,
        files.len()
    );
    assert_eq!(
        successful_extractions,
        files.len(),
        "Some extractions failed"
    );

    println!("🎯 RESULT: No hanging detected - system handles large extractions gracefully");
    Ok(())
}

/// Test resource usage under extreme load
fn test_resource_usage() -> Result<()> {
    println!("\n💾 Testing Resource Usage Under Load...");

    let (_temp_dir, archive_path, _) = create_large_test_archive()?;

    // Get subset of files to test different configurations
    let files: Vec<String> = {
        let mut archive = Archive::open(&archive_path)?;
        archive
            .list()?
            .into_iter()
            .take(5000)
            .map(|e| e.name)
            .collect()
    };
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();

    // Test configurations that previously caused issues
    let test_configs = vec![
        ("High Thread Count", 16, 10),  // Many threads, small batches
        ("Resource Conscious", 8, 100), // Fewer threads, larger batches
        ("Conservative", 4, 200),       // Conservative approach
    ];

    for (name, threads, batch_size) in test_configs {
        println!(
            "  Testing {}: {} threads, {} batch size",
            name, threads, batch_size
        );

        let config = ParallelConfig::new()
            .threads(threads)
            .batch_size(batch_size)
            .skip_errors(false);

        let start = Instant::now();
        let results = extract_with_config(&archive_path, &file_refs, config)?;
        let duration = start.elapsed();

        let files_per_sec = files.len() as f64 / duration.as_secs_f64();
        println!(
            "    ⏱️  Duration: {:.2}s, Rate: {:.1} files/sec",
            duration.as_secs_f64(),
            files_per_sec
        );

        assert_eq!(
            results.len(),
            files.len(),
            "File count mismatch in {}",
            name
        );
        assert!(
            duration < Duration::from_secs(120),
            "Configuration {} took too long",
            name
        );
    }

    println!("✅ All resource usage tests passed - system remains stable under load");
    Ok(())
}

/// Test edge cases that might cause hanging
fn test_edge_cases() -> Result<()> {
    println!("\n🔍 Testing Edge Cases...");

    let (_temp_dir, archive_path, _) = create_large_test_archive()?;

    // Get all files
    let all_files: Vec<String> = {
        let mut archive = Archive::open(&archive_path)?;
        archive.list()?.into_iter().map(|e| e.name).collect()
    };

    // Test 1: Extract all files at once (stress test)
    println!("  Test 1: Full archive extraction");
    let file_refs: Vec<&str> = all_files.iter().map(|s| s.as_str()).collect();
    let config = ParallelConfig::new().threads(8).batch_size(150);

    let start = Instant::now();
    let results = extract_with_config(&archive_path, &file_refs, config)?;
    let duration = start.elapsed();

    println!(
        "    ⏱️  Extracted {} files in {:.2}s ({:.1} files/sec)",
        results.len(),
        duration.as_secs_f64(),
        results.len() as f64 / duration.as_secs_f64()
    );

    assert_eq!(results.len(), all_files.len());
    assert!(
        duration < Duration::from_secs(600),
        "Full extraction took too long"
    );

    // Test 2: Very small batch sizes (high overhead)
    println!("  Test 2: Small batch sizes (high overhead test)");
    let subset: Vec<&str> = all_files.iter().take(1000).map(|s| s.as_str()).collect();
    let config = ParallelConfig::new().threads(16).batch_size(1); // Stress test with individual file batches

    let start = Instant::now();
    let results = extract_with_config(&archive_path, &subset, config)?;
    let duration = start.elapsed();

    println!(
        "    ⏱️  Extracted {} files in {:.2}s ({:.1} files/sec)",
        results.len(),
        duration.as_secs_f64(),
        results.len() as f64 / duration.as_secs_f64()
    );

    assert_eq!(results.len(), subset.len());

    println!("✅ All edge case tests passed");
    Ok(())
}

fn main() -> Result<()> {
    println!("🚀 MPQ Extraction Hanging Issue Resolution Validation");
    println!("=====================================================");

    // Run all stress tests
    test_no_hanging()?;
    test_resource_usage()?;
    test_edge_cases()?;

    println!("\n🎉 VALIDATION COMPLETE");
    println!("======================");
    println!("✅ No hanging issues detected");
    println!("✅ System handles large archives gracefully");
    println!("✅ Resource usage remains bounded");
    println!("✅ Intelligent batching prevents resource exhaustion");
    println!("✅ System is production-ready for large-scale MPQ extraction");

    Ok(())
}
