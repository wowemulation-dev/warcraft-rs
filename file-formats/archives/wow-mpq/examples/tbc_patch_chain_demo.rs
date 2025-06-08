//! Demonstrates patch chain functionality with DBC files from WoW 2.4.3 (TBC)
//! Shows how locale-specific patches override base content

use std::path::Path;
use wow_mpq::{Archive, PatchChain, Result};

fn main() -> Result<()> {
    println!("WoW 2.4.3 (TBC) DBC Patch Chain Demo");
    println!("====================================\n");

    // Path to WoW 2.4.3 data
    let data_path = Path::new("/home/danielsreichenbach/Downloads/wow/2.4.3/Data");
    if !data_path.exists() {
        println!("WoW 2.4.3 data not found at: {}", data_path.display());
        return Ok(());
    }

    // For TBC, we'll look at both general and locale-specific files
    let locale = "enUS";
    let locale_path = data_path.join(locale);

    // Choose DBC files to examine
    let dbc_file = "DBFilesClient\\Spell.dbc";
    let locale_specific_file = "DBFilesClient\\SpellStrings.dbc"; // Often locale-specific

    println!("Target DBC files:");
    println!("  General: {}", dbc_file);
    println!("  Locale-specific: {}", locale_specific_file);
    println!("  Locale: {}\n", locale);

    // Step 1: Analyze base archives (common.MPQ)
    println!("Step 1: Analyzing base archives");
    println!("-------------------------------");

    let common_mpq = data_path.join("common.MPQ");
    if common_mpq.exists() {
        let mut common_archive = Archive::open(&common_mpq)?;
        let info = common_archive.get_info()?;
        println!("Archive: common.MPQ");
        println!("  Format: {:?}", info.format_version);
        println!("  Files: {}", info.file_count);
        println!("  Size: {:.2} MB", info.file_size as f64 / 1024.0 / 1024.0);

        // Try to find Spell.dbc in common.MPQ
        match common_archive.read_file(dbc_file) {
            Ok(data) => {
                println!("\n  {} found in common.MPQ:", dbc_file);
                println!("    Size: {} bytes", data.len());
                analyze_dbc_header(&data);

                // Save for comparison
                std::fs::write("spell_common.dbc", &data)?;
            }
            Err(_) => {
                println!("  {} not found in common.MPQ", dbc_file);
            }
        }
    }

    // Check expansion.MPQ
    let expansion_mpq = data_path.join("expansion.MPQ");
    if expansion_mpq.exists() {
        let mut exp_archive = Archive::open(&expansion_mpq)?;
        let info = exp_archive.get_info()?;
        println!("\nArchive: expansion.MPQ");
        println!("  Format: {:?}", info.format_version);
        println!("  Files: {}", info.file_count);
        println!("  Size: {:.2} MB", info.file_size as f64 / 1024.0 / 1024.0);
    }

    // Step 2: Create TBC patch chain following the official loading order
    println!("\n\nStep 2: Creating TBC patch chain (official loading order)");
    println!("--------------------------------------------------------");

    let mut chain = PatchChain::new();
    let mut priority = 0;

    // Base archives (loaded first)
    if common_mpq.exists() {
        chain.add_archive(&common_mpq, priority)?;
        println!("Added: common.MPQ (priority: {})", priority);
        priority += 1;
    }

    let common2_mpq = data_path.join("common-2.MPQ");
    if common2_mpq.exists() {
        chain.add_archive(&common2_mpq, priority)?;
        println!("Added: common-2.MPQ (priority: {})", priority);
        priority += 1;
    }

    if expansion_mpq.exists() {
        chain.add_archive(&expansion_mpq, priority)?;
        println!("Added: expansion.MPQ (priority: {})", priority);
    }

    // Locale archives (override base content)
    let locale_priority_base = 100;
    let locale_mpq = locale_path.join(format!("locale-{}.MPQ", locale));
    if locale_mpq.exists() {
        chain.add_archive(&locale_mpq, locale_priority_base)?;
        println!(
            "Added: {}/locale-{}.MPQ (priority: {})",
            locale, locale, locale_priority_base
        );

        // Check what's in the locale archive
        let mut locale_archive = Archive::open(&locale_mpq)?;
        let info = locale_archive.get_info()?;
        println!(
            "  Locale archive: {} files, {:.2} MB",
            info.file_count,
            info.file_size as f64 / 1024.0 / 1024.0
        );
    }

    let expansion_locale_mpq = locale_path.join(format!("expansion-locale-{}.MPQ", locale));
    if expansion_locale_mpq.exists() {
        chain.add_archive(&expansion_locale_mpq, locale_priority_base + 1)?;
        println!(
            "Added: {}/expansion-locale-{}.MPQ (priority: {})",
            locale,
            locale,
            locale_priority_base + 1
        );
    }

    // General patches (override everything before)
    let patch_priority_base = 1000;
    let patch_mpq = data_path.join("patch.MPQ");
    if patch_mpq.exists() {
        chain.add_archive(&patch_mpq, patch_priority_base)?;
        println!("Added: patch.MPQ (priority: {})", patch_priority_base);

        let mut patch_archive = Archive::open(&patch_mpq)?;
        let info = patch_archive.get_info()?;
        println!(
            "  Patch: {} files, {:.2} MB",
            info.file_count,
            info.file_size as f64 / 1024.0 / 1024.0
        );
    }

    let patch2_mpq = data_path.join("patch-2.MPQ");
    if patch2_mpq.exists() {
        chain.add_archive(&patch2_mpq, patch_priority_base + 1)?;
        println!("Added: patch-2.MPQ (priority: {})", patch_priority_base + 1);
    }

    // Locale patches (highest priority)
    let locale_patch_priority = 2000;
    let locale_patch_mpq = locale_path.join(format!("patch-{}.MPQ", locale));
    if locale_patch_mpq.exists() {
        chain.add_archive(&locale_patch_mpq, locale_patch_priority)?;
        println!(
            "Added: {}/patch-{}.MPQ (priority: {})",
            locale, locale, locale_patch_priority
        );
    }

    let locale_patch2_mpq = locale_path.join(format!("patch-{}-2.MPQ", locale));
    if locale_patch2_mpq.exists() {
        chain.add_archive(&locale_patch2_mpq, locale_patch_priority + 1)?;
        println!(
            "Added: {}/patch-{}-2.MPQ (priority: {})",
            locale,
            locale,
            locale_patch_priority + 1
        );
    }

    // Step 3: Extract through patch chain
    println!("\n\nStep 3: Extracting through patch chain");
    println!("--------------------------------------");

    // Extract Spell.dbc
    match chain.read_file(dbc_file) {
        Ok(patched_data) => {
            println!("\n{}:", dbc_file);
            println!("  Size: {} bytes", patched_data.len());

            // Calculate checksum
            let checksum: u32 = patched_data.iter().map(|&b| b as u32).sum();
            println!("  Checksum: 0x{:08x}", checksum);

            // Check source
            if let Some(source) = chain.find_file_archive(dbc_file) {
                println!(
                    "  Source: {}",
                    source.file_name().unwrap_or_default().to_string_lossy()
                );
            }

            analyze_dbc_header(&patched_data);

            // Save and compare
            std::fs::write("spell_patched.dbc", &patched_data)?;

            if let Ok(base_data) = std::fs::read("spell_common.dbc") {
                println!("\n  Comparison with common.MPQ version:");
                println!("    Base size:    {} bytes", base_data.len());
                println!("    Patched size: {} bytes", patched_data.len());
                println!(
                    "    Difference:   {} bytes",
                    patched_data.len() as i64 - base_data.len() as i64
                );

                if base_data != patched_data {
                    println!("    ✓ File was modified by patches!");
                }
            }
        }
        Err(e) => {
            println!("Failed to extract {}: {}", dbc_file, e);
        }
    }

    // Try locale-specific file
    println!("\n{}:", locale_specific_file);
    match chain.read_file(locale_specific_file) {
        Ok(data) => {
            println!("  Size: {} bytes", data.len());
            if let Some(source) = chain.find_file_archive(locale_specific_file) {
                println!(
                    "  Source: {}",
                    source.file_name().unwrap_or_default().to_string_lossy()
                );

                // Check if it came from a locale archive
                let source_name = source.file_name().unwrap_or_default().to_string_lossy();
                if source_name.contains("locale") {
                    println!("  ✓ This is a locale-specific file!");
                }
            }
        }
        Err(_) => {
            println!("  Not found in patch chain");
        }
    }

    // Step 4: Analyze TBC-specific DBCs
    println!("\n\nStep 4: Checking TBC-specific content");
    println!("-------------------------------------");

    let tbc_dbcs = vec![
        "DBFilesClient\\SpellShapeshiftForm.dbc", // TBC feature
        "DBFilesClient\\ItemExtendedCost.dbc",    // TBC currencies
        "DBFilesClient\\Map.dbc",                 // Should include Outland
        "DBFilesClient\\AreaTable.dbc",           // TBC zones
        "DBFilesClient\\CharTitles.dbc",          // TBC titles
    ];

    for dbc in tbc_dbcs {
        if let Some(source) = chain.find_file_archive(dbc) {
            let source_name = source.file_name().unwrap_or_default().to_string_lossy();
            println!("  {} -> {}", dbc, source_name);

            if source_name.contains("expansion") {
                println!("    ^ TBC expansion content");
            } else if source_name.contains("patch") {
                println!("    ^ Modified by patches");
            }
        }
    }

    // Step 5: Summary
    println!("\n\nSummary");
    println!("-------");
    println!("Total archives in chain: {}", chain.archive_count());

    let chain_info = chain.get_chain_info();
    println!("\nLoading order (priority):");
    for info in chain_info {
        println!(
            "  {} ({}): {} files",
            info.path.file_name().unwrap_or_default().to_string_lossy(),
            info.priority,
            info.file_count
        );
    }

    // Cleanup
    let _ = std::fs::remove_file("spell_common.dbc");
    let _ = std::fs::remove_file("spell_patched.dbc");

    Ok(())
}

fn analyze_dbc_header(data: &[u8]) {
    if data.len() >= 20 && &data[..4] == b"WDBC" {
        let record_count = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let field_count = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let record_size = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        let string_block_size = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);

        println!("    DBC structure:");
        println!("      Records: {}", record_count);
        println!("      Fields: {}", field_count);
        println!("      Record size: {} bytes", record_size);
        println!("      String block: {} bytes", string_block_size);
    }
}
