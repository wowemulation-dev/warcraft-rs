//! Demonstrates patch chain functionality with DBC files from WoW 4.3.4 (Cataclysm)
//! Shows the evolution to DB2 format and complex patch structure

use std::path::Path;
use wow_mpq::{Archive, PatchChain, Result};

fn main() -> Result<()> {
    println!("WoW 4.3.4 (Cataclysm) DBC/DB2 Patch Chain Demo");
    println!("==============================================\n");

    // Path to WoW 4.3.4 data
    let data_path = Path::new("/home/danielsreichenbach/Downloads/wow/4.3.4/4.3.4/Data");
    if !data_path.exists() {
        println!("WoW 4.3.4 data not found at: {}", data_path.display());
        return Ok(());
    }

    let locale = "enUS";
    let locale_path = data_path.join(locale);

    // Choose files to examine - Cata introduces DB2 format
    let dbc_file = "DBFilesClient\\Spell.dbc";
    let db2_file = "DBFilesClient\\Item.db2"; // New DB2 format

    println!("Target database files:");
    println!("  DBC format: {}", dbc_file);
    println!("  DB2 format: {} (new in Cata)", db2_file);
    println!("  Locale: {}\n", locale);

    // Step 1: Analyze base archives
    println!("Step 1: Analyzing base archives");
    println!("-------------------------------");

    // Cataclysm restructured base archives
    let art_mpq = data_path.join("art.MPQ");
    if art_mpq.exists() {
        let mut art_archive = Archive::open(&art_mpq)?;
        let info = art_archive.get_info()?;
        println!("Archive: art.MPQ");
        println!("  Format: {:?}", info.format_version);
        println!("  Files: {}", info.file_count);
        println!("  Size: {:.2} MB", info.file_size as f64 / 1024.0 / 1024.0);
    }

    // Step 2: Create Cataclysm patch chain
    println!("\n\nStep 2: Creating Cataclysm patch chain");
    println!("--------------------------------------");

    let mut chain = PatchChain::new();

    // Priority structure for Cataclysm
    const BASE: i32 = 0;
    const LOCALE_BASE: i32 = 100;
    const PATCH_BASE: i32 = 1000;
    const LOCALE_PATCH_BASE: i32 = 2000;
    const WOW_UPDATE_BASE: i32 = 3000; // New in Cata

    // Base archives (new structure in Cata)
    let base_archives = vec![
        "art.MPQ",
        "expansion1.MPQ",
        "expansion2.MPQ",
        "expansion3.MPQ",
        "sound.MPQ",
        "world.MPQ",
    ];

    let mut priority = BASE;
    for archive_name in base_archives {
        let archive_path = data_path.join(archive_name);
        if archive_path.exists() {
            chain.add_archive(&archive_path, priority)?;
            println!("Added: {} (priority: {})", archive_name, priority);
            priority += 1;
        }
    }

    // Locale base archives
    let locale_mpq = locale_path.join(format!("locale-{}.MPQ", locale));
    if locale_mpq.exists() {
        chain.add_archive(&locale_mpq, LOCALE_BASE)?;
        println!(
            "Added: {}/locale-{}.MPQ (priority: {})",
            locale, locale, LOCALE_BASE
        );
    }

    let expansion_locale = locale_path.join(format!("expansion1-locale-{}.MPQ", locale));
    if expansion_locale.exists() {
        chain.add_archive(&expansion_locale, LOCALE_BASE + 1)?;
        println!(
            "Added: {}/expansion1-locale-{}.MPQ (priority: {})",
            locale,
            locale,
            LOCALE_BASE + 1
        );
    }

    let expansion2_locale = locale_path.join(format!("expansion2-locale-{}.MPQ", locale));
    if expansion2_locale.exists() {
        chain.add_archive(&expansion2_locale, LOCALE_BASE + 2)?;
        println!(
            "Added: {}/expansion2-locale-{}.MPQ (priority: {})",
            locale,
            locale,
            LOCALE_BASE + 2
        );
    }

    let expansion3_locale = locale_path.join(format!("expansion3-locale-{}.MPQ", locale));
    if expansion3_locale.exists() {
        chain.add_archive(&expansion3_locale, LOCALE_BASE + 3)?;
        println!(
            "Added: {}/expansion3-locale-{}.MPQ (priority: {})",
            locale,
            locale,
            LOCALE_BASE + 3
        );
    }

    // Base patches (numbered differently in Cata)
    let mut patch_priority = PATCH_BASE;
    for i in 1..=20 {
        // Cata had many more patches
        let patch_name = format!("base-{}.MPQ", i);
        let patch_path = data_path.join(&patch_name);
        if patch_path.exists() {
            chain.add_archive(&patch_path, patch_priority)?;
            println!("Added: {} (priority: {})", patch_name, patch_priority);
            patch_priority += 1;
        }
    }

    // Locale patches
    let mut locale_patch_priority = LOCALE_PATCH_BASE;
    for i in 1..=20 {
        let patch_name = format!("base-{}.MPQ", i);
        let locale_patch_path = locale_path.join(&patch_name);
        if locale_patch_path.exists() {
            chain.add_archive(&locale_patch_path, locale_patch_priority)?;
            println!(
                "Added: {}/{} (priority: {})",
                locale, patch_name, locale_patch_priority
            );
            locale_patch_priority += 1;
        }
    }

    // wow-update archives (new in Cata)
    let mut update_priority = WOW_UPDATE_BASE;
    for i in 1..=20 {
        let update_name = format!("wow-update-{}.MPQ", i);
        let update_path = data_path.join(&update_name);
        if update_path.exists() {
            chain.add_archive(&update_path, update_priority)?;
            println!("Added: {} (priority: {})", update_name, update_priority);
            update_priority += 1;
        }

        // Locale wow-update
        let locale_update_path = locale_path.join(&update_name);
        if locale_update_path.exists() {
            chain.add_archive(&locale_update_path, update_priority + 1000)?;
            println!(
                "Added: {}/{} (priority: {})",
                locale,
                update_name,
                update_priority + 1000
            );
        }
    }

    // Step 3: Extract through patch chain
    println!("\n\nStep 3: Extracting through patch chain");
    println!("--------------------------------------");

    // Extract Spell.dbc
    match chain.read_file(dbc_file) {
        Ok(patched_data) => {
            println!("\n{}:", dbc_file);
            println!("  Size: {} bytes", patched_data.len());

            // Check source
            if let Some(source) = chain.find_file_archive(dbc_file) {
                println!(
                    "  Source: {}",
                    source.file_name().unwrap_or_default().to_string_lossy()
                );
            }

            analyze_dbc_header(&patched_data);
        }
        Err(e) => {
            println!("Failed to extract {}: {}", dbc_file, e);
        }
    }

    // Try DB2 format
    println!("\n{}:", db2_file);
    match chain.read_file(db2_file) {
        Ok(data) => {
            println!("  Size: {} bytes", data.len());
            if let Some(source) = chain.find_file_archive(db2_file) {
                println!(
                    "  Source: {}",
                    source.file_name().unwrap_or_default().to_string_lossy()
                );
            }

            // Check for DB2 header
            if data.len() >= 4 && &data[..4] == b"WDB2" {
                println!("  ✓ This is a DB2 format file (new in Cataclysm)!");
            }
        }
        Err(_) => {
            println!("  Not found - might be DBC format instead");
        }
    }

    // Step 4: Analyze Cataclysm-specific content
    println!("\n\nStep 4: Checking Cataclysm-specific content");
    println!("-------------------------------------------");

    let cata_features = vec![
        ("DBFilesClient\\Guild.dbc", "Guild system revamp"),
        ("DBFilesClient\\GuildPerkSpells.dbc", "Guild perks"),
        ("DBFilesClient\\MountCapability.dbc", "Flying in Azeroth"),
        ("DBFilesClient\\DestructibleModelData.dbc", "World changes"),
        ("DBFilesClient\\PhaseXPhaseGroup.dbc", "Phasing system"),
        ("DBFilesClient\\Item-sparse.db2", "New item data structure"),
    ];

    for (file, description) in cata_features {
        if let Some(source) = chain.find_file_archive(file) {
            let source_name = source.file_name().unwrap_or_default().to_string_lossy();
            println!("  {} - {}", file, description);
            println!("    Source: {}", source_name);
        }
    }

    // Step 5: Summary
    println!("\n\nSummary");
    println!("-------");
    println!("Total archives in chain: {}", chain.archive_count());

    let chain_info = chain.get_chain_info();
    println!("\nHighest priority archives:");
    for info in chain_info.iter().rev().take(5) {
        println!(
            "  {} ({}): {} files",
            info.path.file_name().unwrap_or_default().to_string_lossy(),
            info.priority,
            info.file_count
        );
    }

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
