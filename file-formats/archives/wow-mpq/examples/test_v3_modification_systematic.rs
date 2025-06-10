//! Systematic wow-mpq test for V3 archive modification
//! Replicates the exact StormLib scenario to identify differences

use std::fs;
use wow_mpq::{
    AddFileOptions, Archive, ArchiveBuilder, AttributesOption, FormatVersion, ListfileOption,
    MutableArchive,
};

const PROBLEMATIC_FILE: &str = "World\\Maps\\Azeroth\\Azeroth_28_51_tex1.adt";
const CATA_ARCHIVE: &str = "/home/danielsreichenbach/Downloads/wow/4.3.4/4.3.4/Data/world.MPQ";
const TEST_ARCHIVE: &str = "wowmpq_v3_modification_test.mpq";

// Test data for additional files (same as StormLib test)
const TEST_FILE_1: &str = "test_file_1.txt";
const TEST_FILE_2: &str = "test_file_2.txt";
const TEST_DATA_1: &[u8] = b"This is test data for file 1";
const TEST_DATA_2: &[u8] = b"This is test data for file 2 - a bit longer to test different sizes";

fn extract_from_original() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    println!(
        "📥 Extracting {} from original archive...",
        PROBLEMATIC_FILE
    );

    let mut archive = Archive::open(CATA_ARCHIVE)?;
    let data = archive.read_file(PROBLEMATIC_FILE)?;

    println!("  ✅ Extracted {} bytes", data.len());
    Ok(data)
}

fn create_v3_archive_with_file(file_data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔨 Creating V3 archive with initial file...");

    ArchiveBuilder::new()
        .version(FormatVersion::V3)
        .listfile_option(ListfileOption::Generate)
        .attributes_option(AttributesOption::GenerateFull)
        .add_file_data_with_options(
            file_data.to_vec(),
            PROBLEMATIC_FILE,
            0x02,  // Zlib compression (same as StormLib)
            false, // Not encrypted
            0,     // Default locale
        )
        .build(TEST_ARCHIVE)?;

    println!("  ✅ Created V3 archive successfully");
    Ok(())
}

fn add_files_to_archive() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n➕ Adding additional files to existing V3 archive...");

    let mut mutable = MutableArchive::open(TEST_ARCHIVE)?;

    // Add first test file
    let options1 = AddFileOptions {
        compression: wow_mpq::compression::CompressionMethod::Zlib,
        ..Default::default()
    };
    mutable.add_file_data(TEST_DATA_1, TEST_FILE_1, options1)?;
    println!("  ✅ Added {}", TEST_FILE_1);

    // Add second test file
    let options2 = AddFileOptions {
        compression: wow_mpq::compression::CompressionMethod::Zlib,
        ..Default::default()
    };
    mutable.add_file_data(TEST_DATA_2, TEST_FILE_2, options2)?;
    println!("  ✅ Added {}", TEST_FILE_2);

    // Flush archive
    mutable.flush()?;
    println!("  ✅ Successfully added files to archive");

    Ok(())
}

fn verify_all_files(original_data: &[u8]) -> Result<bool, Box<dyn std::error::Error>> {
    println!("\n🔍 Verifying all files in modified V3 archive...");

    let mut archive = Archive::open(TEST_ARCHIVE)?;
    let mut all_good = true;

    // Verify original problematic file
    println!("  🔍 Verifying {}...", PROBLEMATIC_FILE);
    match archive.read_file(PROBLEMATIC_FILE) {
        Ok(data) => {
            if data.len() != original_data.len() {
                println!(
                    "    ❌ Size mismatch: {} vs {}",
                    data.len(),
                    original_data.len()
                );
                all_good = false;
            } else if data != original_data {
                println!("    ❌ Data mismatch!");
                all_good = false;
            } else {
                println!("    ✅ File verified successfully ({} bytes)", data.len());
            }
        }
        Err(e) => {
            println!("    ❌ Read error: {}", e);
            println!("    Error details: {:?}", e);
            all_good = false;
        }
    }

    // Verify test files
    let test_files = [(TEST_FILE_1, TEST_DATA_1), (TEST_FILE_2, TEST_DATA_2)];

    for (filename, expected_data) in &test_files {
        println!("  🔍 Verifying {}...", filename);

        match archive.read_file(filename) {
            Ok(data) => {
                if data.len() != expected_data.len() {
                    println!(
                        "    ❌ Size mismatch: {} vs {}",
                        data.len(),
                        expected_data.len()
                    );
                    all_good = false;
                } else if data != *expected_data {
                    println!("    ❌ Data mismatch!");
                    all_good = false;
                } else {
                    println!("    ✅ File verified successfully ({} bytes)", data.len());
                }
            }
            Err(e) => {
                println!("    ❌ Read error: {}", e);
                println!("    Error details: {:?}", e);
                all_good = false;
            }
        }
    }

    Ok(all_good)
}

fn compare_with_stormlib() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Comparing with StormLib archive...");

    let stormlib_archive = "stormlib_v3_modification_test.mpq";
    let wowmpq_archive = TEST_ARCHIVE;

    if !std::path::Path::new(stormlib_archive).exists() {
        println!("  ⚠️ StormLib archive not found (run StormLib test first)");
        return Ok(());
    }

    let stormlib_size = fs::metadata(stormlib_archive)?.len();
    let wowmpq_size = fs::metadata(wowmpq_archive)?.len();

    println!("  StormLib archive size: {} bytes", stormlib_size);
    println!("  wow-mpq archive size:  {} bytes", wowmpq_size);

    if stormlib_size == wowmpq_size {
        println!("  ✅ Archive sizes match exactly!");
    } else {
        println!(
            "  ⚠️ Archive sizes differ by {} bytes",
            (stormlib_size as i64 - wowmpq_size as i64).abs()
        );
    }

    // Try to read the problematic file from StormLib archive with wow-mpq
    println!("\n🔄 Cross-compatibility test: Reading StormLib archive with wow-mpq...");
    match Archive::open(stormlib_archive) {
        Ok(mut archive) => match archive.read_file(PROBLEMATIC_FILE) {
            Ok(data) => {
                println!(
                    "  ✅ Successfully read {} bytes from StormLib archive",
                    data.len()
                );
            }
            Err(e) => {
                println!("  ❌ Failed to read from StormLib archive: {}", e);
            }
        },
        Err(e) => {
            println!("  ❌ Failed to open StormLib archive: {}", e);
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Systematic wow-mpq V3 Archive Modification Test");
    println!("=================================================");

    // Clean up any existing test archive
    fs::remove_file(TEST_ARCHIVE).ok();

    // Step 1: Extract problematic file from original
    let original_data = extract_from_original()?;

    // Step 2: Create V3 archive with the file
    create_v3_archive_with_file(&original_data)?;

    // Step 3: Add more files to the archive
    add_files_to_archive()?;

    // Step 4: Verify all files
    let success = verify_all_files(&original_data)?;

    // Step 5: Compare with StormLib
    compare_with_stormlib()?;

    println!("\n📊 wow-mpq Test Results:");
    println!("========================");
    println!("Original file size: {} bytes", original_data.len());
    println!("Archive created: {}", TEST_ARCHIVE);
    println!(
        "Verification: {}",
        if success { "✅ SUCCESS" } else { "❌ FAILED" }
    );

    if success {
        println!("\n✅ wow-mpq successfully handles V3 archive modification!");
        println!("💾 Archive saved as {} for comparison", TEST_ARCHIVE);
    } else {
        println!("\n❌ wow-mpq V3 archive modification needs investigation");
    }

    // Keep archive for analysis
    println!("💾 Archive kept for analysis: {}", TEST_ARCHIVE);

    Ok(())
}
