//! Verify that wow-mpq can accurately read and write WoW game files

use std::collections::HashMap;
use wow_mpq::test_utils::{find_any_wow_data, print_setup_instructions};
use wow_mpq::{Archive, ArchiveBuilder, AttributesOption, FormatVersion, ListfileOption};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Try to find a WoW MPQ archive to test with
    let source_mpq = if let Some(arg) = std::env::args().nth(1) {
        // Use command line argument if provided
        arg
    } else {
        // Try to find any available WoW data
        match find_any_wow_data() {
            Some((version, data_path)) => {
                println!(
                    "Found {} data at: {}",
                    version.display_name(),
                    data_path.display()
                );

                // Try common MPQ files in order of preference
                let mpq_candidates = ["patch.MPQ", "patch.mpq", "misc.MPQ", "dbc.MPQ"];
                let mut found_mpq = None;

                for &mpq_name in &mpq_candidates {
                    let mpq_path = data_path.join(mpq_name);
                    if mpq_path.exists() {
                        found_mpq = Some(mpq_path.to_string_lossy().to_string());
                        break;
                    }
                }

                match found_mpq {
                    Some(path) => path,
                    None => {
                        println!("Found WoW data directory but no MPQ files in it.");
                        print_setup_instructions();
                        return Ok(());
                    }
                }
            }
            None => {
                println!("No WoW game data found!");
                println!();
                print_setup_instructions();
                return Ok(());
            }
        }
    };

    println!("Testing file integrity with: {}", source_mpq);

    // Open the source archive
    let mut source = Archive::open(&source_mpq)?;
    let files = source.list()?;

    println!("Found {} files in source archive", files.len());

    // Extract some files
    let mut test_files = HashMap::new();
    let test_count = 10.min(files.len());

    for (i, file_info) in files.iter().enumerate() {
        if i >= test_count {
            break;
        }

        // Skip special files
        if file_info.name.starts_with('(') && file_info.name.ends_with(')') {
            continue;
        }

        match source.read_file(&file_info.name) {
            Ok(data) => {
                println!("  - {} ({} bytes)", file_info.name, data.len());
                test_files.insert(file_info.name.clone(), data);
            }
            Err(e) => {
                eprintln!("    Failed to read {}: {}", file_info.name, e);
            }
        }
    }

    if test_files.is_empty() {
        println!("No files could be extracted for testing");
        return Ok(());
    }

    // Create a new archive with these files
    println!("\nCreating test archive with {} files...", test_files.len());
    let test_archive = "verify_test.mpq";

    let mut builder = ArchiveBuilder::new()
        .version(FormatVersion::V1)
        .listfile_option(ListfileOption::Generate)
        .attributes_option(AttributesOption::GenerateFull);

    for (filename, data) in &test_files {
        builder = builder.add_file_data(data.clone(), filename);
    }

    builder.build(test_archive)?;

    // Verify the files
    println!("\nVerifying files...");
    let mut archive = Archive::open(test_archive)?;
    let mut all_match = true;

    for (filename, original_data) in &test_files {
        match archive.read_file(filename) {
            Ok(read_data) => {
                if read_data == *original_data {
                    println!("  ✓ {} - Verified", filename);
                } else {
                    println!("  ✗ {} - Mismatch!", filename);
                    all_match = false;
                }
            }
            Err(e) => {
                println!("  ✗ {} - Read error: {}", filename, e);
                all_match = false;
            }
        }
    }

    if all_match {
        println!("\n✅ All files verified successfully!");
        println!("wow-mpq can accurately read and write WoW game files.");
    } else {
        println!("\n❌ Some files failed verification!");
    }

    // Cleanup
    std::fs::remove_file(test_archive).ok();

    Ok(())
}
