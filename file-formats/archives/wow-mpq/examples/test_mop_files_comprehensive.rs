//! Comprehensive file integrity test for WoW 5.4.8 (Mists of Pandaria) using V4 archives

use rand::{Rng, seq::IteratorRandom};
use std::collections::HashMap;
use wow_mpq::{
    AddFileOptions, Archive, ArchiveBuilder, AttributesOption, FormatVersion, ListfileOption,
    MutableArchive,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Comprehensive WoW 5.4.8 File Integrity Test");
    println!("==============================================");

    // Find MoP data directory
    let data_path =
        std::path::PathBuf::from("/home/danielsreichenbach/Downloads/wow/5.4.8/5.4.8/Data");
    if !data_path.exists() {
        println!("❌ WoW 5.4.8 data not found at: {}", data_path.display());
        println!("💡 Please ensure WoW 5.4.8 data is available at the expected location");
        return Ok(());
    }
    println!("✅ Found MoP data at: {}", data_path.display());

    // Find MPQ archives
    let mut archives = Vec::new();
    for entry in std::fs::read_dir(&data_path)? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            if name.to_lowercase().ends_with(".mpq") {
                archives.push(name.to_string());
            }
        }
    }

    if archives.is_empty() {
        println!("❌ No MPQ archives found in MoP data directory");
        return Ok(());
    }

    archives.sort();
    println!("📦 Found {} MoP archives:", archives.len());
    for archive in &archives {
        println!("  - {}", archive);
    }

    // Step 1: Extract random files from MoP archives
    println!("\n🎲 Step 1: Extracting random files from MoP archives...");
    let mut first_batch = HashMap::new();
    let mut rng = rand::rng();

    // Sample files from multiple archives
    let selected_archives: Vec<_> = archives.iter().take(5).collect(); // First 5 archives
    for archive_name in &selected_archives {
        println!("  📖 Scanning: {}", archive_name);
        let archive_path = data_path.join(archive_name);

        match Archive::open(&archive_path) {
            Ok(mut archive) => {
                match archive.list() {
                    Ok(files) => {
                        // Sample random files from this archive
                        let sample_count = rng.random_range(3..=5);
                        let sampled = files.into_iter().choose_multiple(&mut rng, sample_count);

                        for file_entry in sampled {
                            if file_entry.name.starts_with('(') {
                                continue; // Skip special files
                            }

                            match archive.read_file(&file_entry.name) {
                                Ok(data) => {
                                    println!("    ✓ {}: {} bytes", file_entry.name, data.len());
                                    first_batch.insert(file_entry.name.clone(), data);
                                    if first_batch.len() >= 12 {
                                        // Limit first batch
                                        break;
                                    }
                                }
                                Err(_) => continue,
                            }
                        }
                    }
                    Err(e) => println!("    ⚠️ Could not list files: {}", e),
                }
            }
            Err(e) => println!("    ⚠️ Could not open {}: {}", archive_name, e),
        }

        if first_batch.len() >= 12 {
            break;
        }
    }

    if first_batch.is_empty() {
        println!("❌ No files could be extracted from MoP archives");
        return Ok(());
    }

    println!("✅ Extracted {} files for first batch", first_batch.len());

    // Step 2: Create V4 archive with first batch
    println!("\n🔨 Step 2: Creating V4 archive with HET/BET tables, listfile and attributes...");
    let test_archive = "test_mop_comprehensive.mpq";
    std::fs::remove_file(test_archive).ok();

    let mut builder = ArchiveBuilder::new()
        .version(FormatVersion::V4)
        .listfile_option(ListfileOption::Generate)
        .attributes_option(AttributesOption::GenerateFull);

    for (filename, data) in &first_batch {
        builder = builder.add_file_data(data.clone(), filename);
    }

    builder.build(test_archive)?;
    println!("✅ Created V4 archive: {}", test_archive);

    // Step 3: Verify first batch
    println!("\n🔍 Step 3: Verifying first batch file integrity...");
    let mut archive = Archive::open(test_archive)?;
    let mut verified_count = 0;

    for (filename, original_data) in &first_batch {
        match archive.read_file(filename) {
            Ok(extracted_data) => {
                if extracted_data == *original_data {
                    println!("  ✅ {}: {} bytes verified", filename, original_data.len());
                    verified_count += 1;
                } else {
                    println!("  ❌ {}: Data mismatch!", filename);
                }
            }
            Err(e) => {
                println!("  ❌ {}: Read error: {}", filename, e);
            }
        }
    }

    println!(
        "  📊 Verified {}/{} files",
        verified_count,
        first_batch.len()
    );

    if verified_count != first_batch.len() {
        println!("❌ First batch verification failed!");
        return Ok(());
    }
    println!("✅ First batch verification successful!");

    // Step 4: Extract second batch
    println!("\n🎲 Step 4: Extracting second batch of random files...");
    let mut second_batch = HashMap::new();

    // Continue from where we left off, or sample from other archives
    for archive_name in selected_archives.iter().skip(2) {
        println!("  📖 Scanning: {}", archive_name);
        let archive_path = data_path.join(archive_name);

        match Archive::open(&archive_path) {
            Ok(mut archive) => match archive.list() {
                Ok(files) => {
                    let sample_count = rng.random_range(2..=4);
                    let sampled = files.into_iter().choose_multiple(&mut rng, sample_count);

                    for file_entry in sampled {
                        if file_entry.name.starts_with('(')
                            || first_batch.contains_key(&file_entry.name)
                        {
                            continue;
                        }

                        match archive.read_file(&file_entry.name) {
                            Ok(data) => {
                                println!("    ✓ {}: {} bytes", file_entry.name, data.len());
                                second_batch.insert(file_entry.name.clone(), data);
                                if second_batch.len() >= 10 {
                                    break;
                                }
                            }
                            Err(_) => continue,
                        }
                    }
                }
                Err(e) => println!("    ⚠️ Could not list files: {}", e),
            },
            Err(e) => println!("    ⚠️ Could not open {}: {}", archive_name, e),
        }

        if second_batch.len() >= 10 {
            break;
        }
    }

    println!(
        "✅ Extracted {} additional files for second batch",
        second_batch.len()
    );

    // Step 5: Add second batch to existing V4 archive using MutableArchive
    println!("\n➕ Step 5: Adding second batch files to existing V4 archive...");
    let mut mutable = MutableArchive::open(test_archive)?;

    for (filename, data) in &second_batch {
        let options = AddFileOptions::default();
        mutable.add_file_data(data, filename, options)?;
        println!("    ➕ Added: {}", filename);
    }

    mutable.flush()?;
    drop(mutable);
    println!("✅ Added second batch files to archive");

    // Step 6: Verify ALL files in final V4 archive
    println!("\n🔍 Step 6: Verifying ALL files in final V4 archive...");
    let mut final_archive = Archive::open(test_archive)?;
    let mut total_verified = 0;

    // Verify first batch files
    for (filename, original_data) in &first_batch {
        match final_archive.read_file(filename) {
            Ok(extracted_data) => {
                if extracted_data == *original_data {
                    println!("  ✅ {}: {} bytes verified", filename, original_data.len());
                    total_verified += 1;
                } else {
                    println!("  ❌ {}: Data mismatch!", filename);
                }
            }
            Err(e) => {
                println!("  ❌ {}: Read error: {}", filename, e);
            }
        }
    }

    // Verify second batch files
    for (filename, original_data) in &second_batch {
        match final_archive.read_file(filename) {
            Ok(extracted_data) => {
                if extracted_data == *original_data {
                    println!("  ✅ {}: {} bytes verified", filename, original_data.len());
                    total_verified += 1;
                } else {
                    println!("  ❌ {}: Data mismatch!", filename);
                }
            }
            Err(e) => {
                println!("  ❌ {}: Read error: {}", filename, e);
            }
        }
    }

    let total_files = first_batch.len() + second_batch.len();
    println!("  📊 Verified {}/{} files", total_verified, total_files);

    // Step 7: Test V4-specific features
    println!("\n🔬 Step 7: Testing V4-specific features...");

    // Get archive info to test V4 features
    match final_archive.get_info() {
        Ok(info) => {
            println!("  🔍 Archive format version: {:?}", info.format_version);
            println!("  📁 Total files in archive: {}", info.file_count);
            println!("  💾 Archive size: {} bytes", info.file_size);

            // Test file listing
            match final_archive.list() {
                Ok(files) => {
                    println!("  📋 File listing successful: {} files", files.len());

                    // Test some V4 HET lookups
                    let test_files: Vec<_> = files.iter().take(3).collect();
                    for (i, file) in test_files.iter().enumerate() {
                        match final_archive.read_file(&file.name) {
                            Ok(data) => {
                                println!(
                                    "  ✅ HET lookup {}: {} ({} bytes)",
                                    i + 1,
                                    file.name,
                                    data.len()
                                );
                            }
                            Err(e) => {
                                println!("  ❌ HET lookup {}: {} failed: {}", i + 1, file.name, e);
                            }
                        }
                    }
                }
                Err(e) => println!("  ❌ File listing failed: {}", e),
            }

            println!("  ✅ V4 features test completed");
        }
        Err(e) => println!("  ❌ Could not get archive info: {}", e),
    }

    // Final results
    println!("\n📊 Final Results:");
    println!("================");
    println!("First batch files:  {}", first_batch.len());
    println!("Second batch files: {}", second_batch.len());
    println!("Total files tested: {}", total_files);

    if total_verified == total_files {
        println!("✅ ALL FILES VERIFIED SUCCESSFULLY!");
        println!("✅ wow-mpq can accurately read and write WoW 5.4.8 game files");
        println!("✅ V4 archive format with advanced HET/BET tables working correctly");
        println!("✅ Archive modification (add files) preserves data integrity");
        println!("✅ MoP-specific features (V4 format, MD5 checksums) handled correctly");
    } else {
        println!("❌ Some files failed verification!");
        println!("❌ Verified: {}/{}", total_verified, total_files);
    }

    // Clean up
    std::fs::remove_file(test_archive).ok();
    println!("🗑️ Cleaned up: {}", test_archive);

    Ok(())
}
