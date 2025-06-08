//! Demonstrates patch chain functionality with DBC files from WoW 1.12.1
//! Shows how patches override base archive content

use std::path::Path;
use wow_mpq::{Archive, PatchChain, Result};

fn main() -> Result<()> {
    println!("WoW 1.12.1 DBC Patch Chain Demo");
    println!("================================\n");

    // Path to WoW 1.12.1 data
    let data_path = Path::new("/home/danielsreichenbach/Downloads/wow/1.12.1/Data");
    if !data_path.exists() {
        println!("WoW 1.12.1 data not found at: {}", data_path.display());
        return Ok(());
    }

    // Choose a DBC file that's commonly patched
    let dbc_file = "DBFilesClient\\Spell.dbc";
    println!("Target DBC file: {}\n", dbc_file);

    // Step 1: Extract from base dbc.MPQ archive
    println!("Step 1: Extracting from base dbc.MPQ archive");
    println!("---------------------------------------------");

    let dbc_mpq = data_path.join("dbc.MPQ");
    let mut base_archive = Archive::open(&dbc_mpq)?;

    // Get archive info
    let info = base_archive.get_info()?;
    println!("Archive: dbc.MPQ");
    println!("  Format: {:?}", info.format_version);
    println!("  Files: {}", info.file_count);
    println!("  Size: {:.2} MB", info.file_size as f64 / 1024.0 / 1024.0);

    // Try to extract the DBC file from base archive
    match base_archive.read_file(dbc_file) {
        Ok(base_data) => {
            println!("\nBase Spell.dbc:");
            println!("  Size: {} bytes", base_data.len());
            // Calculate a simple checksum for comparison
            let checksum: u32 = base_data.iter().map(|&b| b as u32).sum();
            println!("  Checksum: 0x{:08x}", checksum);

            // Show first few bytes (DBC header)
            if base_data.len() >= 20 {
                println!("  Header: {:?}", &base_data[..20]);

                // Parse DBC header (4 bytes magic, 4 bytes record count, 4 bytes field count, 4 bytes record size, 4 bytes string block size)
                if base_data.len() >= 20 && &base_data[..4] == b"WDBC" {
                    let record_count = u32::from_le_bytes([
                        base_data[4],
                        base_data[5],
                        base_data[6],
                        base_data[7],
                    ]);
                    let field_count = u32::from_le_bytes([
                        base_data[8],
                        base_data[9],
                        base_data[10],
                        base_data[11],
                    ]);
                    let record_size = u32::from_le_bytes([
                        base_data[12],
                        base_data[13],
                        base_data[14],
                        base_data[15],
                    ]);
                    let string_block_size = u32::from_le_bytes([
                        base_data[16],
                        base_data[17],
                        base_data[18],
                        base_data[19],
                    ]);

                    println!("  DBC Info:");
                    println!("    Records: {}", record_count);
                    println!("    Fields: {}", field_count);
                    println!("    Record size: {} bytes", record_size);
                    println!("    String block: {} bytes", string_block_size);
                }

                // Save base version
                std::fs::write("spell_base.dbc", &base_data)?;
                println!("\n  Saved to: spell_base.dbc");
            }
        }
        Err(e) => {
            println!("\n  {} not found in dbc.MPQ: {}", dbc_file, e);
            println!("  This file might only exist in patches.");
        }
    }

    // Step 2: Create patch chain and extract from patches
    println!("\n\nStep 2: Creating patch chain with all archives");
    println!("----------------------------------------------");

    let mut chain = PatchChain::new();

    // Add base archive
    chain.add_archive(&dbc_mpq, 0)?;
    println!("Added: dbc.MPQ (priority: 0)");

    // Add patches in order
    let patch_mpq = data_path.join("patch.MPQ");
    if patch_mpq.exists() {
        chain.add_archive(&patch_mpq, 100)?;
        println!("Added: patch.MPQ (priority: 100)");

        // Check what patch.MPQ contains
        let mut patch_archive = Archive::open(&patch_mpq)?;
        let patch_info = patch_archive.get_info()?;
        println!("\nPatch.MPQ info:");
        println!("  Files: {}", patch_info.file_count);
        println!(
            "  Size: {:.2} MB",
            patch_info.file_size as f64 / 1024.0 / 1024.0
        );
    }

    let patch2_mpq = data_path.join("patch-2.MPQ");
    if patch2_mpq.exists() {
        chain.add_archive(&patch2_mpq, 200)?;
        println!("Added: patch-2.MPQ (priority: 200)");

        // Check what patch-2.MPQ contains
        let mut patch2_archive = Archive::open(&patch2_mpq)?;
        let patch2_info = patch2_archive.get_info()?;
        println!("\nPatch-2.MPQ info:");
        println!("  Files: {}", patch2_info.file_count);
        println!(
            "  Size: {:.2} MB",
            patch2_info.file_size as f64 / 1024.0 / 1024.0
        );
    }

    // Step 3: Extract the same DBC through patch chain
    println!("\n\nStep 3: Extracting through patch chain");
    println!("--------------------------------------");

    match chain.read_file(dbc_file) {
        Ok(patched_data) => {
            println!("Patched Spell.dbc:");
            println!("  Size: {} bytes", patched_data.len());
            // Calculate checksum
            let checksum: u32 = patched_data.iter().map(|&b| b as u32).sum();
            println!("  Checksum: 0x{:08x}", checksum);

            // Check which archive it came from
            if let Some(source_archive) = chain.find_file_archive(dbc_file) {
                println!(
                    "  Source: {}",
                    source_archive
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                );
            }

            // Show header and parse if it's a DBC
            if patched_data.len() >= 20 {
                println!("  Header: {:?}", &patched_data[..20]);

                if &patched_data[..4] == b"WDBC" {
                    let record_count = u32::from_le_bytes([
                        patched_data[4],
                        patched_data[5],
                        patched_data[6],
                        patched_data[7],
                    ]);
                    let field_count = u32::from_le_bytes([
                        patched_data[8],
                        patched_data[9],
                        patched_data[10],
                        patched_data[11],
                    ]);
                    let record_size = u32::from_le_bytes([
                        patched_data[12],
                        patched_data[13],
                        patched_data[14],
                        patched_data[15],
                    ]);
                    let string_block_size = u32::from_le_bytes([
                        patched_data[16],
                        patched_data[17],
                        patched_data[18],
                        patched_data[19],
                    ]);

                    println!("  DBC Info:");
                    println!("    Records: {}", record_count);
                    println!("    Fields: {}", field_count);
                    println!("    Record size: {} bytes", record_size);
                    println!("    String block: {} bytes", string_block_size);
                }

                // Save patched version
                std::fs::write("spell_patched.dbc", &patched_data)?;
                println!("\n  Saved to: spell_patched.dbc");
            }

            // Compare with base version if we have it
            if let Ok(base_data) = std::fs::read("spell_base.dbc") {
                println!("\n\nComparison:");
                println!("-----------");
                println!("Base size:    {} bytes", base_data.len());
                println!("Patched size: {} bytes", patched_data.len());
                println!(
                    "Difference:   {} bytes",
                    patched_data.len() as i64 - base_data.len() as i64
                );

                if base_data != patched_data {
                    println!("\n✓ The patched DBC is different from the base version!");
                    println!("  This confirms the patch chain is working correctly.");
                } else {
                    println!("\n✗ The files are identical (no patches modified this DBC)");
                }
            }
        }
        Err(e) => {
            println!("Failed to extract {} through patch chain: {}", dbc_file, e);
        }
    }

    // Step 4: Look for other commonly patched DBCs
    println!("\n\nStep 4: Checking other commonly patched DBCs");
    println!("--------------------------------------------");

    let common_dbcs = vec![
        "DBFilesClient\\Item.dbc",
        "DBFilesClient\\ItemSet.dbc",
        "DBFilesClient\\Talent.dbc",
        "DBFilesClient\\SkillLineAbility.dbc",
        "DBFilesClient\\SpellItemEnchantment.dbc",
    ];

    for dbc in common_dbcs {
        // Check where this DBC comes from in the chain
        if let Some(source) = chain.find_file_archive(dbc) {
            let source_name = source.file_name().unwrap_or_default().to_string_lossy();
            println!("  {} -> {}", dbc, source_name);

            // If it's from a patch, show that it was patched
            if source_name.contains("patch") {
                println!("    ^ This DBC was modified by patches!");
            }
        }
    }

    Ok(())
}
