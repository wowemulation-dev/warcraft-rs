//! CLI integration tests for MPQ extraction functionality
//!
//! These tests validate the command-line interface for MPQ extraction,
//! testing real invocations of the CLI with various options.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Get the path to the warcraft-rs binary
fn get_binary_path() -> Result<std::path::PathBuf> {
    // Check if we're running under cargo-llvm-cov (code coverage)
    if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
        // Try llvm-cov-target directory first (used by coverage)
        let coverage_binary = std::path::Path::new(&target_dir).join("debug/warcraft-rs");
        if coverage_binary.exists() {
            return Ok(coverage_binary);
        }
    }
    
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let target_dir = std::path::Path::new(manifest_dir).join("../target");

    // Try debug build first, then release
    let debug_binary = target_dir.join("debug/warcraft-rs");
    let release_binary = target_dir.join("release/warcraft-rs");

    if debug_binary.exists() {
        Ok(debug_binary)
    } else if release_binary.exists() {
        Ok(release_binary)
    } else {
        // Build the binary if it doesn't exist
        let output = std::process::Command::new("cargo")
            .args(&["build", "--bin", "warcraft-rs"])
            .current_dir(manifest_dir)
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to build warcraft-rs binary: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        // Try again after building
        if debug_binary.exists() {
            Ok(debug_binary)
        } else {
            Err(anyhow::anyhow!(
                "warcraft-rs binary not found even after building"
            ))
        }
    }
}

/// Find a suitable test MPQ archive
fn find_test_archive() -> Option<std::path::PathBuf> {
    let test_paths = [
        "/home/danielsreichenbach/Downloads/wow/1.12.1/Data/dbc.MPQ",
        "/home/danielsreichenbach/Downloads/wow/1.12.1/Data/interface.MPQ",
        "/home/danielsreichenbach/Downloads/wow/3.3.5a/Data/patch.MPQ",
        "/home/danielsreichenbach/Downloads/wow/4.3.4/4.3.4/Data/world.MPQ",
    ];

    test_paths
        .iter()
        .map(std::path::PathBuf::from)
        .find(|p| p.exists())
}

/// Test basic CLI extraction with individual files
#[test]
fn test_cli_extract_individual_files() -> Result<()> {
    let binary_path = get_binary_path()?;
    let archive_path =
        find_test_archive().ok_or_else(|| anyhow::anyhow!("No test archive found"))?;
    let temp_dir = TempDir::new().context("Failed to create temp directory")?;

    println!("Testing CLI extraction with: {}", archive_path.display());

    // First, list files to find some to extract
    let list_output = Command::new(&binary_path)
        .args(["mpq", "list", archive_path.to_str().unwrap()])
        .output()
        .context("Failed to execute list command")?;

    if !list_output.status.success() {
        return Err(anyhow::anyhow!(
            "List command failed: {}",
            String::from_utf8_lossy(&list_output.stderr)
        ));
    }

    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    let available_files: Vec<&str> = list_stdout
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with("Files"))
        .take(3) // Get first 3 files
        .collect();

    if available_files.is_empty() {
        println!("No files found in archive, skipping test");
        return Ok(());
    }

    println!(
        "Found {} files to extract: {:?}",
        available_files.len(),
        available_files
    );

    // Test extraction with specific files
    let mut cmd = Command::new(&binary_path);
    cmd.args([
        "mpq",
        "extract",
        archive_path.to_str().unwrap(),
        "--output",
        temp_dir.path().to_str().unwrap(),
    ]);

    // Add specific files to extract
    for file in &available_files {
        cmd.arg(file);
    }

    let extract_output = cmd.output().context("Failed to execute extract command")?;

    println!("Extract command output:");
    println!(
        "stdout: {}",
        String::from_utf8_lossy(&extract_output.stdout)
    );
    println!(
        "stderr: {}",
        String::from_utf8_lossy(&extract_output.stderr)
    );

    if !extract_output.status.success() {
        return Err(anyhow::anyhow!(
            "Extract command failed with exit code {}: {}",
            extract_output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&extract_output.stderr)
        ));
    }

    // Verify files were extracted
    let mut extracted_count = 0;
    for entry in fs::read_dir(temp_dir.path())? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            extracted_count += 1;
            println!("Extracted file: {}", entry.file_name().to_string_lossy());
        }
    }

    assert!(extracted_count > 0, "No files were extracted");
    println!("Successfully extracted {} files via CLI", extracted_count);

    Ok(())
}

/// Test CLI extraction with preserve-paths option
#[test]
fn test_cli_extract_preserve_paths() -> Result<()> {
    let binary_path = get_binary_path()?;
    let archive_path =
        find_test_archive().ok_or_else(|| anyhow::anyhow!("No test archive found"))?;
    let temp_dir = TempDir::new().context("Failed to create temp directory")?;

    println!(
        "Testing CLI extraction with preserve-paths: {}",
        archive_path.display()
    );

    // Extract with preserve-paths flag
    let extract_output = Command::new(&binary_path)
        .args([
            "mpq",
            "extract",
            archive_path.to_str().unwrap(),
            "--output",
            temp_dir.path().to_str().unwrap(),
            "--preserve-paths",
        ])
        .output()
        .context("Failed to execute extract command")?;

    println!("Extract command output:");
    println!(
        "stdout: {}",
        String::from_utf8_lossy(&extract_output.stdout)
    );
    if !extract_output.stderr.is_empty() {
        println!(
            "stderr: {}",
            String::from_utf8_lossy(&extract_output.stderr)
        );
    }

    if !extract_output.status.success() {
        return Err(anyhow::anyhow!(
            "Extract command failed with exit code {}: {}",
            extract_output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&extract_output.stderr)
        ));
    }

    // Check if directory structure was preserved
    fn count_files_recursive(dir: &Path) -> Result<(usize, bool)> {
        let mut file_count = 0;
        let mut has_subdirs = false;

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                file_count += 1;
            } else if entry.file_type()?.is_dir() {
                has_subdirs = true;
                let (sub_files, _) = count_files_recursive(&entry.path())?;
                file_count += sub_files;
            }
        }

        Ok((file_count, has_subdirs))
    }

    let (total_files, found_subdirs) = count_files_recursive(temp_dir.path())?;

    println!("Extract results with preserve-paths:");
    println!("  Total files extracted: {}", total_files);
    println!("  Subdirectories created: {}", found_subdirs);

    assert!(total_files > 0, "No files were extracted");

    Ok(())
}

/// Test CLI extraction with threading option
#[test]
fn test_cli_extract_with_threads() -> Result<()> {
    let binary_path = get_binary_path()?;
    let archive_path =
        find_test_archive().ok_or_else(|| anyhow::anyhow!("No test archive found"))?;
    let temp_dir = TempDir::new().context("Failed to create temp directory")?;

    println!(
        "Testing CLI extraction with threading: {}",
        archive_path.display()
    );

    // Test with different thread counts
    for thread_count in [1, 2, 4] {
        println!("Testing with {} threads", thread_count);

        // Clean output directory
        for entry in fs::read_dir(temp_dir.path())? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                fs::remove_file(entry.path())?;
            } else if entry.file_type()?.is_dir() {
                fs::remove_dir_all(entry.path())?;
            }
        }

        let start = std::time::Instant::now();

        let extract_output = Command::new(&binary_path)
            .args([
                "mpq",
                "extract",
                archive_path.to_str().unwrap(),
                "--output",
                temp_dir.path().to_str().unwrap(),
                "--threads",
                &thread_count.to_string(),
            ])
            .output()
            .context("Failed to execute extract command")?;

        let duration = start.elapsed();

        if !extract_output.status.success() {
            println!(
                "Extract failed with {} threads: {}",
                thread_count,
                String::from_utf8_lossy(&extract_output.stderr)
            );
            continue;
        }

        // Count extracted files
        let mut file_count = 0;
        for entry in fs::read_dir(temp_dir.path())? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                file_count += 1;
            }
        }

        println!(
            "  {} threads: {} files in {}ms",
            thread_count,
            file_count,
            duration.as_millis()
        );

        assert!(
            file_count > 0,
            "No files extracted with {} threads",
            thread_count
        );
    }

    Ok(())
}

/// Test CLI extraction with skip-errors option
#[test]
fn test_cli_extract_skip_errors() -> Result<()> {
    let binary_path = get_binary_path()?;
    let archive_path =
        find_test_archive().ok_or_else(|| anyhow::anyhow!("No test archive found"))?;
    let temp_dir = TempDir::new().context("Failed to create temp directory")?;

    println!(
        "Testing CLI extraction with skip-errors: {}",
        archive_path.display()
    );

    // Extract with skip-errors flag and include some non-existent files
    let extract_output = Command::new(&binary_path)
        .args([
            "mpq",
            "extract",
            archive_path.to_str().unwrap(),
            "--output",
            temp_dir.path().to_str().unwrap(),
            "--skip-errors",
            "nonexistent_file1.txt",
            "fake/path/file.dat",
            "another_missing.bin",
        ])
        .output()
        .context("Failed to execute extract command")?;

    println!("Extract command output:");
    println!(
        "stdout: {}",
        String::from_utf8_lossy(&extract_output.stdout)
    );
    println!(
        "stderr: {}",
        String::from_utf8_lossy(&extract_output.stderr)
    );

    // Command should succeed even though files don't exist (due to skip-errors)
    if !extract_output.status.success() {
        println!("Extract command failed, which may be expected for missing files");
    }

    // Should not crash and should handle errors gracefully
    let output_str = String::from_utf8_lossy(&extract_output.stdout);
    assert!(
        !output_str.contains("panic"),
        "CLI should not panic with skip-errors"
    );

    Ok(())
}

/// Test CLI info command
#[test]
fn test_cli_info_command() -> Result<()> {
    let binary_path = get_binary_path()?;
    let archive_path =
        find_test_archive().ok_or_else(|| anyhow::anyhow!("No test archive found"))?;

    println!("Testing CLI info command: {}", archive_path.display());

    let info_output = Command::new(&binary_path)
        .args(["mpq", "info", archive_path.to_str().unwrap()])
        .output()
        .context("Failed to execute info command")?;

    println!("Info command output:");
    println!("stdout: {}", String::from_utf8_lossy(&info_output.stdout));
    if !info_output.stderr.is_empty() {
        println!("stderr: {}", String::from_utf8_lossy(&info_output.stderr));
    }

    if !info_output.status.success() {
        return Err(anyhow::anyhow!(
            "Info command failed: {}",
            String::from_utf8_lossy(&info_output.stderr)
        ));
    }

    let output_str = String::from_utf8_lossy(&info_output.stdout);

    // Verify expected info is present
    assert!(
        output_str.contains("MPQ Archive Information"),
        "Missing archive information header"
    );
    assert!(
        output_str.contains("Format version"),
        "Missing format version info"
    );
    assert!(
        output_str.contains("Number of files"),
        "Missing file count info"
    );

    println!("Info command test passed");

    Ok(())
}

/// Test CLI list command with filters
#[test]
fn test_cli_list_with_filter() -> Result<()> {
    let binary_path = get_binary_path()?;
    let archive_path =
        find_test_archive().ok_or_else(|| anyhow::anyhow!("No test archive found"))?;

    println!(
        "Testing CLI list command with filter: {}",
        archive_path.display()
    );

    // Test list with wildcard filter
    let list_output = Command::new(&binary_path)
        .args([
            "mpq",
            "list",
            archive_path.to_str().unwrap(),
            "--filter",
            "*.dbc",
        ])
        .output()
        .context("Failed to execute list command")?;

    if !list_output.status.success() {
        println!(
            "List command failed: {}",
            String::from_utf8_lossy(&list_output.stderr)
        );
        return Ok(()); // Not all archives have .dbc files
    }

    let output_str = String::from_utf8_lossy(&list_output.stdout);
    println!("List with *.dbc filter:");
    println!("{}", output_str);

    // Test list with long format
    let list_long_output = Command::new(&binary_path)
        .args(["mpq", "list", archive_path.to_str().unwrap(), "--long"])
        .output()
        .context("Failed to execute list --long command")?;

    if list_long_output.status.success() {
        let long_output_str = String::from_utf8_lossy(&list_long_output.stdout);
        println!("List with --long format:");
        println!("{}", long_output_str);

        // Should contain size information in long format
        if !long_output_str.trim().is_empty() {
            assert!(
                long_output_str.contains("Size") || long_output_str.contains("bytes"),
                "Long format should show size information"
            );
        }
    }

    Ok(())
}

/// Test CLI help and version commands
#[test]
fn test_cli_help_and_version() -> Result<()> {
    let binary_path = get_binary_path()?;

    // Test help command
    let help_output = Command::new(&binary_path)
        .args(["--help"])
        .output()
        .context("Failed to execute help command")?;

    assert!(help_output.status.success(), "Help command should succeed");

    let help_str = String::from_utf8_lossy(&help_output.stdout);
    assert!(
        help_str.contains("warcraft-rs"),
        "Help should mention warcraft-rs"
    );
    assert!(
        help_str.contains("mpq"),
        "Help should mention mpq subcommand"
    );

    // Test version command
    let version_output = Command::new(&binary_path)
        .args(["--version"])
        .output()
        .context("Failed to execute version command")?;

    assert!(
        version_output.status.success(),
        "Version command should succeed"
    );

    let version_str = String::from_utf8_lossy(&version_output.stdout);
    assert!(
        !version_str.trim().is_empty(),
        "Version output should not be empty"
    );

    println!("CLI help and version tests passed");

    Ok(())
}

/// Test CLI error handling for invalid arguments
#[test]
fn test_cli_error_handling() -> Result<()> {
    let binary_path = get_binary_path()?;

    // Test with nonexistent archive
    let extract_output = Command::new(&binary_path)
        .args(["mpq", "extract", "/nonexistent/path/fake.mpq"])
        .output()
        .context("Failed to execute extract command")?;

    assert!(
        !extract_output.status.success(),
        "Should fail with nonexistent archive"
    );

    let error_str = String::from_utf8_lossy(&extract_output.stderr);
    assert!(!error_str.is_empty(), "Should provide error message");

    // Test with invalid command
    let invalid_output = Command::new(&binary_path)
        .args(["mpq", "invalid-command"])
        .output()
        .context("Failed to execute invalid command")?;

    assert!(
        !invalid_output.status.success(),
        "Should fail with invalid command"
    );

    println!("CLI error handling tests passed");

    Ok(())
}
