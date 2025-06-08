//! Demonstrates patch chain functionality with DBC files from WoW 3.3.5a (WotLK)
//! Shows the definitive loading order from TrinityCore

use std::path::Path;
use wow_mpq::{Archive, PatchChain, Result};

fn main() -> Result<()> {
    println!("WoW 3.3.5a (WotLK) DBC Patch Chain Demo");
    println!("=======================================\n");

    // Path to WoW 3.3.5a data
    let data_path = Path::new("/home/danielsreichenbach/Downloads/wow/3.3.5a/Data");
    if !data_path.exists() {
        println!("WoW 3.3.5a data not found at: {}", data_path.display());
        return Ok(());
    }

    let locale = "enUS";
    let locale_path = data_path.join(locale);

    // Choose DBC files to examine
    let dbc_file = "DBFilesClient\\Spell.dbc";
    let achievement_dbc = "DBFilesClient\\Achievement.dbc"; // WotLK feature

    println!("Target DBC files:");
    println!("  General: {}", dbc_file);
    println!("  WotLK feature: {}", achievement_dbc);
    println!("  Locale: {}\n", locale);

    // Step 1: Analyze base archives
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
    }

    let lichking_mpq = data_path.join("lichking.MPQ");
    if lichking_mpq.exists() {
        let mut lk_archive = Archive::open(&lichking_mpq)?;
        let info = lk_archive.get_info()?;
        println!("\nArchive: lichking.MPQ");
        println!("  Format: {:?}", info.format_version);
        println!("  Files: {}", info.file_count);
        println!("  Size: {:.2} MB", info.file_size as f64 / 1024.0 / 1024.0);
    }

    // Step 2: Create WotLK patch chain following TrinityCore's exact order
    println!("\n\nStep 2: Creating WotLK patch chain (TrinityCore loading order)");
    println!("--------------------------------------------------------------");

    let mut chain = PatchChain::new();

    // Base priority constants from TrinityCore
    const BASE: i32 = 0;
    const BASE_2: i32 = 1;
    const EXPANSION: i32 = 2;
    const LICHKING: i32 = 3;
    const LOCALE_BASE: i32 = 100;
    const LOCALE_EXPANSION: i32 = 101;
    const LOCALE_LICHKING: i32 = 102;
    const PATCH_BASE: i32 = 1000;
    const LOCALE_PATCH_BASE: i32 = 2000;

    // The exact loading order from TrinityCore:
    // Base and expansion archives
    if common_mpq.exists() {
        chain.add_archive(&common_mpq, BASE)?;
        println!("Added: common.MPQ (priority: {})", BASE);
    }

    let common2_mpq = data_path.join("common-2.MPQ");
    if common2_mpq.exists() {
        chain.add_archive(&common2_mpq, BASE_2)?;
        println!("Added: common-2.MPQ (priority: {})", BASE_2);
    }

    let expansion_mpq = data_path.join("expansion.MPQ");
    if expansion_mpq.exists() {
        chain.add_archive(&expansion_mpq, EXPANSION)?;
        println!("Added: expansion.MPQ (priority: {})", EXPANSION);
    }

    if lichking_mpq.exists() {
        chain.add_archive(&lichking_mpq, LICHKING)?;
        println!("Added: lichking.MPQ (priority: {})", LICHKING);
    }

    // Locale archives
    let locale_mpq = locale_path.join(format!("locale-{}.MPQ", locale));
    if locale_mpq.exists() {
        chain.add_archive(&locale_mpq, LOCALE_BASE)?;
        println!(
            "Added: {}/locale-{}.MPQ (priority: {})",
            locale, locale, LOCALE_BASE
        );
    }

    let expansion_locale_mpq = locale_path.join(format!("expansion-locale-{}.MPQ", locale));
    if expansion_locale_mpq.exists() {
        chain.add_archive(&expansion_locale_mpq, LOCALE_EXPANSION)?;
        println!(
            "Added: {}/expansion-locale-{}.MPQ (priority: {})",
            locale, locale, LOCALE_EXPANSION
        );
    }

    let lichking_locale_mpq = locale_path.join(format!("lichking-locale-{}.MPQ", locale));
    if lichking_locale_mpq.exists() {
        chain.add_archive(&lichking_locale_mpq, LOCALE_LICHKING)?;
        println!(
            "Added: {}/lichking-locale-{}.MPQ (priority: {})",
            locale, locale, LOCALE_LICHKING
        );
    }

    // General patches
    let patch_mpq = data_path.join("patch.MPQ");
    if patch_mpq.exists() {
        chain.add_archive(&patch_mpq, PATCH_BASE)?;
        println!("Added: patch.MPQ (priority: {})", PATCH_BASE);
    }

    let patch2_mpq = data_path.join("patch-2.MPQ");
    if patch2_mpq.exists() {
        chain.add_archive(&patch2_mpq, PATCH_BASE + 1)?;
        println!("Added: patch-2.MPQ (priority: {})", PATCH_BASE + 1);
    }

    let patch3_mpq = data_path.join("patch-3.MPQ");
    if patch3_mpq.exists() {
        chain.add_archive(&patch3_mpq, PATCH_BASE + 2)?;
        println!("Added: patch-3.MPQ (priority: {})", PATCH_BASE + 2);
    }

    // Locale patches
    let locale_patch_mpq = locale_path.join(format!("patch-{}.MPQ", locale));
    if locale_patch_mpq.exists() {
        chain.add_archive(&locale_patch_mpq, LOCALE_PATCH_BASE)?;
        println!(
            "Added: {}/patch-{}.MPQ (priority: {})",
            locale, locale, LOCALE_PATCH_BASE
        );
    }

    let locale_patch2_mpq = locale_path.join(format!("patch-{}-2.MPQ", locale));
    if locale_patch2_mpq.exists() {
        chain.add_archive(&locale_patch2_mpq, LOCALE_PATCH_BASE + 1)?;
        println!(
            "Added: {}/patch-{}-2.MPQ (priority: {})",
            locale,
            locale,
            LOCALE_PATCH_BASE + 1
        );
    }

    let locale_patch3_mpq = locale_path.join(format!("patch-{}-3.MPQ", locale));
    if locale_patch3_mpq.exists() {
        chain.add_archive(&locale_patch3_mpq, LOCALE_PATCH_BASE + 2)?;
        println!(
            "Added: {}/patch-{}-3.MPQ (priority: {})",
            locale,
            locale,
            LOCALE_PATCH_BASE + 2
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
        }
        Err(e) => {
            println!("Failed to extract {}: {}", dbc_file, e);
        }
    }

    // Extract Achievement.dbc (WotLK feature)
    println!("\n{}:", achievement_dbc);
    match chain.read_file(achievement_dbc) {
        Ok(data) => {
            println!("  Size: {} bytes", data.len());
            if let Some(source) = chain.find_file_archive(achievement_dbc) {
                println!(
                    "  Source: {}",
                    source.file_name().unwrap_or_default().to_string_lossy()
                );
            }
            analyze_dbc_header(&data);
            println!("  ✓ Achievements were introduced in WotLK!");
        }
        Err(_) => {
            println!("  Not found in patch chain");
        }
    }

    // Step 4: Analyze WotLK-specific DBCs
    println!("\n\nStep 4: Checking WotLK-specific content");
    println!("---------------------------------------");

    let wotlk_dbcs = vec![
        ("DBFilesClient\\Achievement.dbc", "Achievement system"),
        (
            "DBFilesClient\\AchievementCategory.dbc",
            "Achievement categories",
        ),
        (
            "DBFilesClient\\AchievementCriteria.dbc",
            "Achievement criteria",
        ),
        ("DBFilesClient\\Vehicle.dbc", "Vehicle system"),
        ("DBFilesClient\\VehicleSeat.dbc", "Vehicle seats"),
        ("DBFilesClient\\GlyphProperties.dbc", "Glyph system"),
        ("DBFilesClient\\BattlemasterList.dbc", "Dungeon finder"),
        ("DBFilesClient\\MapDifficulty.dbc", "Heroic/normal modes"),
    ];

    for (dbc, description) in wotlk_dbcs {
        if let Some(source) = chain.find_file_archive(dbc) {
            let source_name = source.file_name().unwrap_or_default().to_string_lossy();
            println!("  {} - {}", dbc, description);
            println!("    Source: {}", source_name);

            if source_name.contains("lichking") {
                println!("    ^ WotLK core content");
            } else if source_name.contains("patch") {
                println!("    ^ Added/modified by patches");
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

    // Show Northrend content
    println!("\nNorthrend Content Check:");
    if let Ok(map_data) = chain.read_file("DBFilesClient\\Map.dbc") {
        if map_data.len() >= 20 && &map_data[..4] == b"WDBC" {
            let record_count =
                u32::from_le_bytes([map_data[4], map_data[5], map_data[6], map_data[7]]);
            println!(
                "  Map.dbc: {} maps (includes Northrend continent ID 571)",
                record_count
            );
        }
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
