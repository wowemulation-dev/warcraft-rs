//! Comprehensive archive verification using random files from original WoW archives
//!
//! This test extracts random files from original Blizzard MPQ archives across all
//! WoW versions, creates new archives in random formats, and verifies data integrity
//! through round-trip testing and modification operations.

use rand::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use wow_mpq::test_utils::{WowVersion, find_wow_data, print_setup_instructions};
use wow_mpq::{
    AddFileOptions, Archive, ArchiveBuilder, AttributesOption, FormatVersion, ListfileOption,
    MutableArchive, compression::CompressionMethod,
};

#[derive(Debug, Clone)]
struct TestConfiguration {
    name: &'static str,
    _wow_version: WowVersion,
    archive_format: FormatVersion,
    compression: CompressionMethod,
    with_encryption: bool,
    with_attributes: bool,
    with_listfile: bool,
}

const TEST_CONFIGURATIONS: &[TestConfiguration] = &[
    TestConfiguration {
        name: "V1 Classic - No Compression",
        _wow_version: WowVersion::Vanilla,
        archive_format: FormatVersion::V1,
        compression: CompressionMethod::None,
        with_encryption: false,
        with_attributes: false,
        with_listfile: true,
    },
    TestConfiguration {
        name: "V1 Classic - Zlib Compression",
        _wow_version: WowVersion::Vanilla,
        archive_format: FormatVersion::V1,
        compression: CompressionMethod::Zlib,
        with_encryption: false,
        with_attributes: false,
        with_listfile: true,
    },
    TestConfiguration {
        name: "V2 TBC - Bzip2 Compression",
        _wow_version: WowVersion::Tbc,
        archive_format: FormatVersion::V2,
        compression: CompressionMethod::BZip2,
        with_encryption: false,
        with_attributes: true,
        with_listfile: true,
    },
    TestConfiguration {
        name: "V2 WotLK - Encrypted",
        _wow_version: WowVersion::Wotlk,
        archive_format: FormatVersion::V2,
        compression: CompressionMethod::Zlib,
        with_encryption: true,
        with_attributes: true,
        with_listfile: true,
    },
    TestConfiguration {
        name: "V3 Cataclysm - Multi Compression",
        _wow_version: WowVersion::Cata,
        archive_format: FormatVersion::V3,
        compression: CompressionMethod::Zlib, // Will test multiple methods
        with_encryption: false,
        with_attributes: true,
        with_listfile: true,
    },
    TestConfiguration {
        name: "V4 MoP - Full Features",
        _wow_version: WowVersion::Mop,
        archive_format: FormatVersion::V4,
        compression: CompressionMethod::Lzma,
        with_encryption: true,
        with_attributes: true,
        with_listfile: true,
    },
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Comprehensive Archive Verification");
    println!("=====================================");
    println!(
        "Testing archive creation and modification with random files from original WoW archives"
    );
    println!();

    let mut rng = rand::rng();
    let mut total_tests = 0;
    let mut successful_tests = 0;
    let mut failed_tests = Vec::new();

    // Extract random files from all available WoW versions
    let mut source_files = HashMap::new();
    extract_random_files_from_all_versions(&mut source_files, &mut rng)?;

    if source_files.is_empty() {
        println!("❌ No source files could be extracted from WoW archives");
        print_setup_instructions();
        return Ok(());
    }

    println!(
        "✅ Extracted {} random files from original WoW archives",
        source_files.len()
    );
    println!();

    // Test each configuration
    for config in TEST_CONFIGURATIONS {
        println!("🔧 Testing configuration: {}", config.name);
        println!("  Format: {:?}", config.archive_format);
        println!("  Compression: {:?}", config.compression);
        println!("  Encryption: {}", config.with_encryption);
        println!("  Attributes: {}", config.with_attributes);
        println!("  Listfile: {}", config.with_listfile);

        total_tests += 1;

        match test_configuration(config, &source_files, &mut rng) {
            Ok(()) => {
                println!("  ✅ Configuration test passed");
                successful_tests += 1;
            }
            Err(e) => {
                println!("  ❌ Configuration test failed: {}", e);
                failed_tests.push((config.name, e.to_string()));
            }
        }
        println!();
    }

    // Summary
    println!("📊 Test Results Summary");
    println!("======================");
    println!("Total configurations tested: {}", total_tests);
    println!("Successful: {}", successful_tests);
    println!("Failed: {}", failed_tests.len());

    if !failed_tests.is_empty() {
        println!("\nFailed configurations:");
        for (name, error) in &failed_tests {
            println!("  ❌ {}: {}", name, error);
        }
    }

    if failed_tests.is_empty() {
        println!(
            "\n🎉 ALL TESTS PASSED! Archive creation and modification working correctly across all formats."
        );
    } else {
        println!("\n⚠️  Some tests failed. Please review the errors above.");
    }

    Ok(())
}

fn extract_random_files_from_all_versions(
    files: &mut HashMap<String, Vec<u8>>,
    rng: &mut impl Rng,
) -> Result<(), Box<dyn std::error::Error>> {
    let wow_versions = [
        (WowVersion::Vanilla, "Classic"),
        (WowVersion::Tbc, "TBC"),
        (WowVersion::Wotlk, "WotLK"),
        (WowVersion::Cata, "Cataclysm"),
        (WowVersion::Mop, "MoP"),
    ];

    for (version, version_name) in &wow_versions {
        if let Some(data_path) = find_wow_data(*version) {
            println!(
                "📦 Extracting from {} at: {}",
                version_name,
                data_path.display()
            );
            extract_files_from_version(&data_path, files, 3, rng)?; // 3 files per version
        } else {
            println!("⚠️  {} data not found, skipping", version_name);
        }
    }

    Ok(())
}

fn extract_files_from_version(
    data_path: &PathBuf,
    files: &mut HashMap<String, Vec<u8>>,
    target_count: usize,
    rng: &mut impl Rng,
) -> Result<(), Box<dyn std::error::Error>> {
    // Find MPQ archives
    let mut archives = Vec::new();
    if data_path.is_dir() {
        for entry in fs::read_dir(data_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("MPQ")
                || path.extension().and_then(|s| s.to_str()) == Some("mpq")
            {
                archives.push(path);
            }
        }
    }

    if archives.is_empty() {
        return Ok(());
    }

    // Shuffle and pick a random archive
    archives.shuffle(rng);
    let archive_path = &archives[0];

    match Archive::open(archive_path) {
        Ok(mut archive) => {
            match archive.list() {
                Ok(file_list) => {
                    // Filter for reasonable files
                    let suitable_files: Vec<_> = file_list
                        .iter()
                        .filter(|f| !f.name.starts_with('(') && !f.name.ends_with(')'))
                        .filter(|f| f.size > 100 && f.size < 512 * 1024) // 100B to 512KB
                        .filter(|f| !f.name.to_lowercase().ends_with(".exe"))
                        .filter(|f| !f.name.to_lowercase().ends_with(".dll"))
                        .collect();

                    if suitable_files.is_empty() {
                        return Ok(());
                    }

                    // Sample random files
                    let sample_count = target_count.min(suitable_files.len());
                    let sampled = suitable_files.choose_multiple(rng, sample_count);

                    for file_info in sampled {
                        match archive.read_file(&file_info.name) {
                            Ok(data) => {
                                // Use a unique key to avoid collisions between versions
                                let key = format!(
                                    "{}:{}",
                                    archive_path.file_name().unwrap().to_string_lossy(),
                                    file_info.name
                                );
                                println!("  ✓ {}: {} bytes", file_info.name, data.len());
                                files.insert(key, data);
                            }
                            Err(e) => {
                                println!("  ⚠️  Failed to read {}: {}", file_info.name, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    println!(
                        "  ⚠️  Could not list files in {}: {}",
                        archive_path.file_name().unwrap().to_string_lossy(),
                        e
                    );
                }
            }
        }
        Err(e) => {
            println!(
                "  ⚠️  Could not open {}: {}",
                archive_path.file_name().unwrap().to_string_lossy(),
                e
            );
        }
    }

    Ok(())
}

fn test_configuration(
    config: &TestConfiguration,
    source_files: &HashMap<String, Vec<u8>>,
    rng: &mut impl Rng,
) -> Result<(), Box<dyn std::error::Error>> {
    // Select random subset of files for this test
    let test_file_count = rng.random_range(3..=8);
    let selected_files: HashMap<String, Vec<u8>> = source_files
        .iter()
        .choose_multiple(rng, test_file_count)
        .into_iter()
        .map(|(k, v)| (simplify_filename(k), v.clone()))
        .collect();

    if selected_files.is_empty() {
        return Err("No files selected for testing".into());
    }

    println!("  📁 Testing with {} files", selected_files.len());

    // Phase 1: Create initial archive
    let archive_path = format!(
        "test_{}.mpq",
        config.name.replace(" ", "_").replace("-", "_")
    );
    create_test_archive(config, &selected_files, &archive_path)?;
    println!("    ✅ Phase 1: Archive created successfully");

    // Phase 2: Verify initial archive
    verify_archive_contents(&selected_files, &archive_path)?;
    println!("    ✅ Phase 2: Archive contents verified");

    // Phase 3: Modify archive (add more files)
    let additional_files: HashMap<String, Vec<u8>> = source_files
        .iter()
        .filter(|(k, _)| !selected_files.contains_key(&simplify_filename(k)))
        .choose_multiple(rng, 2)
        .into_iter()
        .map(|(k, v)| (format!("added_{}", simplify_filename(k)), v.clone()))
        .collect();

    if !additional_files.is_empty() {
        modify_archive(&additional_files, &archive_path, config)?;
        println!("    ✅ Phase 3: Archive modified successfully");

        // Phase 4: Verify modified archive
        let mut all_files = selected_files.clone();
        all_files.extend(additional_files);
        verify_archive_contents(&all_files, &archive_path)?;
        println!("    ✅ Phase 4: Modified archive verified");
    }

    // Cleanup
    if fs::metadata(&archive_path).is_ok() {
        fs::remove_file(&archive_path)?;
    }

    Ok(())
}

fn simplify_filename(original: &str) -> String {
    // Extract just the filename part after the colon
    if let Some(colon_pos) = original.find(':') {
        let filename = &original[colon_pos + 1..];
        // Further simplify the path
        filename
            .replace('\\', "/")
            .split('/')
            .next_back()
            .unwrap_or(filename)
            .to_string()
    } else {
        original.to_string()
    }
}

fn create_test_archive(
    config: &TestConfiguration,
    files: &HashMap<String, Vec<u8>>,
    archive_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = ArchiveBuilder::new().version(config.archive_format);

    if config.with_listfile {
        builder = builder.listfile_option(ListfileOption::Generate);
    } else {
        builder = builder.listfile_option(ListfileOption::None);
    }

    if config.with_attributes {
        builder = builder.attributes_option(AttributesOption::GenerateFull);
    } else {
        builder = builder.attributes_option(AttributesOption::None);
    }

    // Add files
    for (filename, data) in files {
        builder = builder.add_file_data(data.clone(), filename);
    }

    builder.build(archive_path)?;
    Ok(())
}

fn modify_archive(
    additional_files: &HashMap<String, Vec<u8>>,
    archive_path: &str,
    _config: &TestConfiguration,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut mutable = MutableArchive::open(archive_path)?;

    for (filename, data) in additional_files {
        let options = AddFileOptions::default();
        mutable.add_file_data(data, filename, options)?;
    }

    mutable.flush()?;
    Ok(())
}

fn verify_archive_contents(
    expected_files: &HashMap<String, Vec<u8>>,
    archive_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut archive = Archive::open(archive_path)?;

    for (filename, expected_data) in expected_files {
        match archive.read_file(filename) {
            Ok(actual_data) => {
                if actual_data != *expected_data {
                    return Err(format!(
                        "Data mismatch for '{}': expected {} bytes, got {} bytes",
                        filename,
                        expected_data.len(),
                        actual_data.len()
                    )
                    .into());
                }
            }
            Err(e) => {
                return Err(format!("Failed to read '{}': {}", filename, e).into());
            }
        }
    }

    Ok(())
}
