//! Comprehensive test with WoW 2.4.3 files: create V2 archive, verify integrity, add more files, verify again

use rand::prelude::*;
use std::collections::HashMap;
use wow_mpq::test_utils::{WowVersion, find_wow_data, print_setup_instructions};
use wow_mpq::{
    AddFileOptions, Archive, ArchiveBuilder, AttributesOption, FormatVersion, ListfileOption,
    MutableArchive,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Comprehensive WoW 2.4.3 File Integrity Test");
    println!("==============================================");

    // Find WoW 2.4.3 data
    let tbc_data_path = match find_wow_data(WowVersion::Tbc) {
        Some(path) => {
            println!("✅ Found TBC data at: {}", path.display());
            path
        }
        None => {
            println!("❌ No WoW 2.4.3 data found!");
            println!();
            print_setup_instructions();
            return Ok(());
        }
    };

    // Find MPQ archives in TBC directory
    let mpq_candidates = [
        "patch.MPQ",
        "patch.mpq",
        "common.MPQ",
        "common.mpq",
        "expansion.MPQ",
        "expansion.mpq",
        "patch-2.MPQ",
        "patch-2.mpq",
    ];
    let mut tbc_archives = Vec::new();

    for &mpq_name in &mpq_candidates {
        let mpq_path = tbc_data_path.join(mpq_name);
        if mpq_path.exists() {
            tbc_archives.push(mpq_path);
        }
    }

    if tbc_archives.is_empty() {
        println!("❌ No MPQ archives found in TBC directory");
        return Ok(());
    }

    println!("📦 Found {} TBC archives:", tbc_archives.len());
    for archive_path in &tbc_archives {
        println!(
            "  - {}",
            archive_path.file_name().unwrap().to_string_lossy()
        );
    }

    let mut rng = rand::thread_rng();

    // Step 1: Extract random files from first batch
    println!("\n🎲 Step 1: Extracting random files from TBC archives...");
    let mut first_batch_files = HashMap::new();
    let target_first_batch = 8;

    extract_random_files(
        &tbc_archives,
        &mut first_batch_files,
        target_first_batch,
        &mut rng,
    )?;

    if first_batch_files.is_empty() {
        println!("❌ No files could be extracted for testing");
        return Ok(());
    }

    println!(
        "✅ Extracted {} files for first batch",
        first_batch_files.len()
    );

    // Step 2: Create V2 archive with first batch
    println!("\n🔨 Step 2: Creating V2 archive with listfile and attributes...");
    let test_archive_path = "test_tbc_comprehensive.mpq";

    create_v2_archive(&first_batch_files, test_archive_path)?;
    println!("✅ Created V2 archive: {}", test_archive_path);

    // Step 3: Verify first batch files are identical
    println!("\n🔍 Step 3: Verifying first batch file integrity...");
    let first_batch_ok = verify_files_in_archive(&first_batch_files, test_archive_path)?;

    if !first_batch_ok {
        println!("❌ First batch verification failed!");
        cleanup_files(&[test_archive_path]);
        return Ok(());
    }

    println!("✅ First batch verification successful!");

    // Step 4: Extract second batch of random files
    println!("\n🎲 Step 4: Extracting second batch of random files...");
    let mut second_batch_files = HashMap::new();
    let target_second_batch = 6;

    // Make sure we don't get duplicates from first batch
    extract_random_files_excluding(
        &tbc_archives,
        &mut second_batch_files,
        target_second_batch,
        &first_batch_files,
        &mut rng,
    )?;

    if second_batch_files.is_empty() {
        println!("⚠️ No additional unique files could be extracted for second batch");
        println!("✅ Test completed successfully with first batch only");
        cleanup_files(&[test_archive_path]);
        return Ok(());
    }

    println!(
        "✅ Extracted {} additional files for second batch",
        second_batch_files.len()
    );

    // Step 5: Add second batch to archive
    println!("\n➕ Step 5: Adding second batch files to existing archive...");
    add_files_to_archive(&second_batch_files, test_archive_path)?;
    println!("✅ Added second batch files to archive");

    // Step 6: Verify ALL files (first + second batch) are identical
    println!("\n🔍 Step 6: Verifying ALL files in final archive...");

    // Combine both batches for verification
    let mut all_files = first_batch_files.clone();
    all_files.extend(second_batch_files);

    let final_verification_ok = verify_files_in_archive(&all_files, test_archive_path)?;

    println!("\n📊 Final Results:");
    println!("================");
    println!("First batch files:  {}", first_batch_files.len());
    println!(
        "Second batch files: {}",
        all_files.len() - first_batch_files.len()
    );
    println!("Total files tested: {}", all_files.len());

    if final_verification_ok {
        println!("✅ ALL FILES VERIFIED SUCCESSFULLY!");
        println!("✅ wow-mpq can accurately read and write WoW 2.4.3 game files");
        println!("✅ V2 archive format with listfile and attributes working correctly");
        println!("✅ Archive modification (add files) preserves data integrity");
    } else {
        println!("❌ FINAL VERIFICATION FAILED!");
    }

    // Cleanup
    cleanup_files(&[test_archive_path]);

    Ok(())
}

fn extract_random_files(
    archives: &[std::path::PathBuf],
    files: &mut HashMap<String, Vec<u8>>,
    target_count: usize,
    rng: &mut impl rand::Rng,
) -> Result<(), Box<dyn std::error::Error>> {
    for archive_path in archives {
        if files.len() >= target_count {
            break;
        }

        println!(
            "  📖 Scanning: {}",
            archive_path.file_name().unwrap().to_string_lossy()
        );

        let mut archive = Archive::open(archive_path)?;
        let file_list = match archive.list() {
            Ok(list) => list,
            Err(e) => {
                println!("    ⚠️ Could not list files: {}", e);
                continue;
            }
        };

        // Filter out special files and get random selection
        let regular_files: Vec<_> = file_list
            .iter()
            .filter(|f| !f.name.starts_with('(') || !f.name.ends_with(')'))
            .filter(|f| f.size > 0 && f.size < 1024 * 1024) // Skip empty and very large files
            .collect();

        if regular_files.is_empty() {
            continue;
        }

        let sample_size = (target_count - files.len()).min(regular_files.len()).min(3);
        let sampled: Vec<_> = regular_files.choose_multiple(rng, sample_size).collect();

        for file_info in sampled {
            match archive.read_file(&file_info.name) {
                Ok(data) => {
                    println!("    ✓ {}: {} bytes", file_info.name, data.len());
                    files.insert(file_info.name.clone(), data);
                }
                Err(e) => {
                    println!("    ✗ Failed to read {}: {}", file_info.name, e);
                }
            }

            if files.len() >= target_count {
                break;
            }
        }
    }

    Ok(())
}

fn extract_random_files_excluding(
    archives: &[std::path::PathBuf],
    files: &mut HashMap<String, Vec<u8>>,
    target_count: usize,
    exclude: &HashMap<String, Vec<u8>>,
    rng: &mut impl rand::Rng,
) -> Result<(), Box<dyn std::error::Error>> {
    for archive_path in archives {
        if files.len() >= target_count {
            break;
        }

        println!(
            "  📖 Scanning: {}",
            archive_path.file_name().unwrap().to_string_lossy()
        );

        let mut archive = Archive::open(archive_path)?;
        let file_list = match archive.list() {
            Ok(list) => list,
            Err(e) => {
                println!("    ⚠️ Could not list files: {}", e);
                continue;
            }
        };

        // Filter out special files, already extracted files, and get random selection
        let regular_files: Vec<_> = file_list
            .iter()
            .filter(|f| !f.name.starts_with('(') || !f.name.ends_with(')'))
            .filter(|f| !exclude.contains_key(&f.name))
            .filter(|f| f.size > 0 && f.size < 1024 * 1024) // Skip empty and very large files
            .collect();

        if regular_files.is_empty() {
            continue;
        }

        let sample_size = (target_count - files.len()).min(regular_files.len()).min(3);
        let sampled: Vec<_> = regular_files.choose_multiple(rng, sample_size).collect();

        for file_info in sampled {
            match archive.read_file(&file_info.name) {
                Ok(data) => {
                    println!("    ✓ {}: {} bytes", file_info.name, data.len());
                    files.insert(file_info.name.clone(), data);
                }
                Err(e) => {
                    println!("    ✗ Failed to read {}: {}", file_info.name, e);
                }
            }

            if files.len() >= target_count {
                break;
            }
        }
    }

    Ok(())
}

fn create_v2_archive(
    files: &HashMap<String, Vec<u8>>,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = ArchiveBuilder::new()
        .version(FormatVersion::V2)
        .listfile_option(ListfileOption::Generate)
        .attributes_option(AttributesOption::GenerateFull);

    for (filename, data) in files {
        builder = builder.add_file_data(data.clone(), filename);
    }

    builder.build(output_path)?;
    Ok(())
}

fn add_files_to_archive(
    files: &HashMap<String, Vec<u8>>,
    archive_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut mutable = MutableArchive::open(archive_path)?;

    for (filename, data) in files {
        mutable.add_file_data(data.as_ref(), filename, AddFileOptions::default())?;
        println!("    ➕ Added: {}", filename);
    }

    mutable.flush()?;
    Ok(())
}

fn verify_files_in_archive(
    expected_files: &HashMap<String, Vec<u8>>,
    archive_path: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut archive = Archive::open(archive_path)?;
    let mut all_match = true;
    let mut verified_count = 0;

    for (filename, expected_data) in expected_files {
        match archive.read_file(filename) {
            Ok(actual_data) => {
                if actual_data == *expected_data {
                    println!("  ✅ {}: {} bytes verified", filename, actual_data.len());
                    verified_count += 1;
                } else {
                    println!(
                        "  ❌ {}: Data mismatch! Expected {} bytes, got {} bytes",
                        filename,
                        expected_data.len(),
                        actual_data.len()
                    );
                    all_match = false;
                }
            }
            Err(e) => {
                println!("  ❌ {}: Read error: {}", filename, e);
                all_match = false;
            }
        }
    }

    println!(
        "  📊 Verified {}/{} files",
        verified_count,
        expected_files.len()
    );
    Ok(all_match)
}

fn cleanup_files(files: &[&str]) {
    for &file in files {
        if std::path::Path::new(file).exists() {
            std::fs::remove_file(file).ok();
            println!("🗑️ Cleaned up: {}", file);
        }
    }
}
