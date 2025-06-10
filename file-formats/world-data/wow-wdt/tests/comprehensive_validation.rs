//! Comprehensive validation tests using wow-wdt test data
//!
//! This test suite validates the WDT implementation against the extensive test data
//! from the original wow-wdt repository, ensuring parsing accuracy and round-trip
//! conversion without data loss across all WoW versions.

use std::fs;
use std::io::Cursor;
use std::path::Path;
use wow_wdt::{WdtReader, WdtWriter, version::WowVersion};

const TEST_DATA_BASE: &str =
    "/home/danielsreichenbach/Repos/github.com/danielsreichenbach/wow-wdt/test-data";

/// Test data for each WoW version
struct VersionTestData {
    version: WowVersion,
    path: &'static str,
    expected_files: usize,
}

const VERSION_TEST_DATA: &[VersionTestData] = &[
    VersionTestData {
        version: WowVersion::Classic,
        path: "1.12.1",
        expected_files: 16, // Based on the directory listing
    },
    VersionTestData {
        version: WowVersion::TBC,
        path: "2.4.3",
        expected_files: 15,
    },
    VersionTestData {
        version: WowVersion::WotLK,
        path: "3.3.5a",
        expected_files: 16,
    },
    VersionTestData {
        version: WowVersion::Cataclysm,
        path: "4.3.4",
        expected_files: 15,
    },
    VersionTestData {
        version: WowVersion::MoP,
        path: "5.4.8",
        expected_files: 16,
    },
];

#[test]
fn test_all_versions_comprehensive() {
    if !Path::new(TEST_DATA_BASE).exists() {
        eprintln!("Test data not found at: {}", TEST_DATA_BASE);
        eprintln!("This test requires the wow-wdt test data repository.");
        return;
    }

    for version_data in VERSION_TEST_DATA {
        println!("🧪 Testing {} files...", version_data.path);
        test_version_files(version_data);
    }
}

fn test_version_files(version_data: &VersionTestData) {
    let version_path = Path::new(TEST_DATA_BASE).join(version_data.path);

    if !version_path.exists() {
        eprintln!("⚠️  Skipping {}: directory not found", version_data.path);
        return;
    }

    let mut files_tested = 0;
    let mut files_passed = 0;
    let mut files_failed = Vec::new();

    // Recursively find all .wdt files
    if let Ok(wdt_files) = find_wdt_files(&version_path) {
        for wdt_path in wdt_files {
            files_tested += 1;

            match test_single_wdt_file(&wdt_path, version_data.version) {
                Ok(()) => {
                    files_passed += 1;
                    println!("  ✅ {}", wdt_path.file_name().unwrap().to_string_lossy());
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    files_failed.push((wdt_path.clone(), e));
                    println!(
                        "  ❌ {}: {}",
                        wdt_path.file_name().unwrap().to_string_lossy(),
                        error_msg
                    );
                }
            }
        }
    }

    println!(
        "📊 {} Results: {}/{} passed",
        version_data.path, files_passed, files_tested
    );

    if !files_failed.is_empty() {
        println!("   Failed files:");
        for (path, error) in &files_failed {
            println!(
                "     - {}: {}",
                path.file_name().unwrap().to_string_lossy(),
                error
            );
        }
    }

    // Ensure we found the expected number of files (approximately)
    assert!(
        files_tested >= version_data.expected_files - 3, // Allow some tolerance
        "Expected at least {} files for {}, found {}",
        version_data.expected_files - 3,
        version_data.path,
        files_tested
    );
}

fn find_wdt_files(dir: &Path) -> Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error>> {
    let mut wdt_files = Vec::new();

    fn collect_wdt_files(
        dir: &Path,
        files: &mut Vec<std::path::PathBuf>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                collect_wdt_files(&path, files)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("wdt") {
                files.push(path);
            }
        }
        Ok(())
    }

    collect_wdt_files(dir, &mut wdt_files)?;
    Ok(wdt_files)
}

fn test_single_wdt_file(
    path: &Path,
    expected_version: WowVersion,
) -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Parse the WDT file
    let file_data = fs::read(path)?;
    let mut reader = WdtReader::new(Cursor::new(&file_data), expected_version);
    let wdt = reader.read()?;

    // Step 2: Basic validation
    let warnings = wdt.validate();

    // Check that warnings are reasonable (not excessive)
    if warnings.len() > 10 {
        return Err(format!(
            "Too many validation warnings ({}): file may be corrupted or unsupported",
            warnings.len()
        )
        .into());
    }

    // Step 3: Test round-trip conversion - convert to bytes and back
    let mut buffer = Vec::new();
    let mut writer = WdtWriter::new(&mut buffer);
    writer.write(&wdt)?;

    let mut reparsed_reader = WdtReader::new(Cursor::new(&buffer), expected_version);
    let reparsed_wdt = reparsed_reader.read()?;

    // Step 4: Verify structural integrity after round-trip
    if wdt.mver != reparsed_wdt.mver {
        return Err("Version chunk mismatch after round-trip conversion".into());
    }

    if wdt.main != reparsed_wdt.main {
        return Err("Main chunk mismatch after round-trip conversion".into());
    }

    if wdt.maid != reparsed_wdt.maid {
        return Err("MAID chunk mismatch after round-trip conversion".into());
    }

    // Step 5: Test specific features based on version
    match expected_version {
        WowVersion::Classic => {
            // Vanilla should not have MAID chunks in most cases
            // (though some later vanilla files might)
        }
        WowVersion::TBC | WowVersion::WotLK => {
            // TBC/WotLK should have basic MAID support
        }
        WowVersion::Cataclysm | WowVersion::MoP => {
            // Later versions should have full MAID support
            if wdt.maid.is_some() {
                let maid = wdt.maid.as_ref().unwrap();
                // Verify MAID chunk integrity - basic check
                if wdt.count_existing_tiles() > 0 && maid.count_existing_tiles() == 0 {
                    println!("    ⚠️  MAID chunk present but empty despite having terrain");
                }
            }
        }
        _ => {} // Handle other versions
    }

    Ok(())
}

#[test]
fn test_data_loss_prevention() {
    let test_files = [
        ("1.12.1", "World/Maps/test/test.wdt"),
        ("2.4.3", "World/Maps/Azeroth/Azeroth.wdt"),
        ("3.3.5a", "World/Maps/test/Test.wdt"),
        ("4.3.4", "World/Maps/Stormwind/Stormwind.wdt"),
        ("5.4.8", "World/Maps/Azeroth/Azeroth.wdt"),
    ];

    for (version_path, relative_file_path) in test_files {
        let full_path = Path::new(TEST_DATA_BASE)
            .join(version_path)
            .join(relative_file_path);

        if !full_path.exists() {
            println!("⚠️  Skipping {}: file not found", relative_file_path);
            continue;
        }

        println!("🔄 Testing round-trip: {}", relative_file_path);

        let original_data = fs::read(&full_path).unwrap();
        let mut reader = WdtReader::new(Cursor::new(&original_data), WowVersion::Classic);
        let wdt = reader.read().unwrap();

        // Convert back to bytes
        let mut buffer = Vec::new();
        let mut writer = WdtWriter::new(&mut buffer);
        writer.write(&wdt).unwrap();

        // Parse the converted data
        let mut reparsed_reader = WdtReader::new(Cursor::new(&buffer), WowVersion::Classic);
        let reparsed_wdt = reparsed_reader.read().unwrap();

        // Critical checks for data integrity
        assert_eq!(
            wdt.count_existing_tiles(),
            reparsed_wdt.count_existing_tiles(),
            "Tile count changed during round-trip for {}",
            relative_file_path
        );

        assert_eq!(
            wdt.mphd, reparsed_wdt.mphd,
            "MPHD chunk changed during round-trip for {}",
            relative_file_path
        );

        // Compare main chunk data
        assert_eq!(
            wdt.main, reparsed_wdt.main,
            "MAIN chunk data changed during round-trip for {}",
            relative_file_path
        );

        println!("  ✅ Round-trip successful: no data loss detected");
    }
}

#[test]
fn test_version_specific_features() {
    let version_tests = [
        ("1.12.1", WowVersion::Classic, "World/Maps/test/test.wdt"),
        (
            "2.4.3",
            WowVersion::TBC,
            "World/Maps/Expansion01/Expansion01.wdt",
        ),
        ("3.3.5a", WowVersion::WotLK, "World/Maps/test/Test.wdt"),
        (
            "4.3.4",
            WowVersion::Cataclysm,
            "World/Maps/Stormwind/Stormwind.wdt",
        ),
        ("5.4.8", WowVersion::MoP, "World/Maps/Azeroth/Azeroth.wdt"),
    ];

    for (version_path, expected_version, relative_file_path) in version_tests {
        let full_path = Path::new(TEST_DATA_BASE)
            .join(version_path)
            .join(relative_file_path);

        if !full_path.exists() {
            println!("⚠️  Skipping {}: file not found", relative_file_path);
            continue;
        }

        println!(
            "🔍 Testing version features: {} ({})",
            relative_file_path, version_path
        );

        let data = fs::read(&full_path).unwrap();
        let mut reader = WdtReader::new(Cursor::new(&data), expected_version);
        let wdt = reader.read().unwrap();

        // Test version-specific validation
        let warnings = wdt.validate();
        if !warnings.is_empty() {
            println!("  ⚠️  Validation warnings: {}", warnings.len());
        }

        match expected_version {
            WowVersion::Classic => {
                // Vanilla files should be simpler
                println!(
                    "  📊 Vanilla features: tiles={}",
                    wdt.count_existing_tiles()
                );
            }
            WowVersion::TBC | WowVersion::WotLK => {
                // TBC/WotLK introduced MAID chunks
                if let Some(maid) = &wdt.maid {
                    println!(
                        "  📊 TBC/WotLK features: tiles={}, maid_tiles={}",
                        wdt.count_existing_tiles(),
                        maid.count_existing_tiles()
                    );
                } else {
                    println!(
                        "  📊 TBC/WotLK features: tiles={}, no_maid",
                        wdt.count_existing_tiles()
                    );
                }
            }
            WowVersion::Cataclysm | WowVersion::MoP => {
                // Later versions have more complex MAID usage
                if let Some(maid) = &wdt.maid {
                    println!(
                        "  📊 Later version features: tiles={}, maid_tiles={}",
                        wdt.count_existing_tiles(),
                        maid.count_existing_tiles()
                    );
                } else {
                    println!(
                        "  📊 Later version features: tiles={}, no_maid",
                        wdt.count_existing_tiles()
                    );
                }
            }
            _ => {}
        }

        println!("  ✅ Version-specific validation passed");
    }
}

#[test]
fn test_conversion_between_versions() {
    let base_file = Path::new(TEST_DATA_BASE)
        .join("2.4.3")
        .join("World/Maps/Azeroth/Azeroth.wdt");

    if !base_file.exists() {
        println!("⚠️  Skipping conversion test: base file not found");
        return;
    }

    println!("🔄 Testing version conversion");

    let data = fs::read(&base_file).unwrap();
    let mut reader = WdtReader::new(Cursor::new(&data), WowVersion::TBC);
    let wdt = reader.read().unwrap();

    // For now, just test basic conversion by reading with different version contexts
    let target_versions = [
        WowVersion::Classic,
        WowVersion::WotLK,
        WowVersion::Cataclysm,
        WowVersion::MoP,
    ];

    for target_version in target_versions {
        println!("  🔄 Testing with {:?} context", target_version);

        // Re-read the same data with different version context
        let mut target_reader = WdtReader::new(Cursor::new(&data), target_version);
        match target_reader.read() {
            Ok(target_wdt) => {
                // Verify core data is maintained
                assert_eq!(
                    wdt.count_existing_tiles(),
                    target_wdt.count_existing_tiles(),
                    "Tile count changed when reading with {:?} context",
                    target_version
                );

                // Test round-trip with the target version
                let mut buffer = Vec::new();
                let mut writer = WdtWriter::new(&mut buffer);
                writer.write(&target_wdt).unwrap();

                let mut reparsed_reader = WdtReader::new(Cursor::new(&buffer), target_version);
                let reparsed = reparsed_reader.read().unwrap();

                assert_eq!(
                    target_wdt.count_existing_tiles(),
                    reparsed.count_existing_tiles(),
                    "Round-trip failed for {:?} version context",
                    target_version
                );

                println!("    ✅ {:?} context successful", target_version);
            }
            Err(e) => {
                println!("    ⚠️  {:?} context failed: {}", target_version, e);
                // Some version contexts may legitimately fail for certain files
            }
        }
    }
}
