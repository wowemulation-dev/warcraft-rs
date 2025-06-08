//! Demonstrates patch chain functionality with DBC files from WoW 5.4.8 (MoP)
//! Shows the most complex patch structure with CASC preparation

use std::path::Path;
use wow_mpq::{Archive, PatchChain, Result};

fn main() -> Result<()> {
    println!("WoW 5.4.8 (Mists of Pandaria) DBC/DB2 Patch Chain Demo");
    println!("======================================================\n");

    // Path to WoW 5.4.8 data
    let data_path = Path::new("/home/danielsreichenbach/Downloads/wow/5.4.8/5.4.8/Data");
    if !data_path.exists() {
        println!("WoW 5.4.8 data not found at: {}", data_path.display());
        return Ok(());
    }

    let locale = "enUS";
    let locale_path = data_path.join(locale);

    // Choose files to examine
    let dbc_file = "DBFilesClient\\Spell.dbc";
    let monk_file = "DBFilesClient\\ChrClasses.dbc"; // Should include Monk
    let pet_battle_file = "DBFilesClient\\BattlePetSpecies.db2"; // MoP feature

    println!("Target database files:");
    println!("  Core DBC: {}", dbc_file);
    println!("  Class data: {} (includes Monk)", monk_file);
    println!("  MoP feature: {} (Pet Battles)", pet_battle_file);
    println!("  Locale: {}\n", locale);

    // Step 1: Analyze base archives
    println!("Step 1: Analyzing base archives");
    println!("-------------------------------");

    let misc_mpq = data_path.join("misc.MPQ");
    if misc_mpq.exists() {
        let mut misc_archive = Archive::open(&misc_mpq)?;
        let info = misc_archive.get_info()?;
        println!("Archive: misc.MPQ");
        println!("  Format: {:?}", info.format_version);
        println!("  Files: {}", info.file_count);
        println!("  Size: {:.2} MB", info.file_size as f64 / 1024.0 / 1024.0);
    }

    // Step 2: Create MoP patch chain
    println!("\n\nStep 2: Creating MoP patch chain");
    println!("--------------------------------");

    let mut chain = PatchChain::new();

    // Priority structure for MoP (most complex yet)
    const BASE: i32 = 0;
    const LOCALE_BASE: i32 = 100;
    const PATCH_BASE: i32 = 1000;
    const LOCALE_PATCH_BASE: i32 = 2000;
    const WOW_UPDATE_BASE: i32 = 3000;
    const LOCALE_UPDATE_BASE: i32 = 4000;

    // Base archives (MoP structure)
    let base_archives = vec![
        "art.MPQ",
        "expansion1.MPQ",
        "expansion2.MPQ",
        "expansion3.MPQ",
        "expansion4.MPQ", // MoP
        "locale.MPQ",
        "misc.MPQ",
        "model.MPQ",
        "sound.MPQ",
        "texture.MPQ",
        "world.MPQ",
        "world2.MPQ",
    ];

    let mut priority = BASE;
    for archive_name in base_archives {
        let archive_path = data_path.join(archive_name);
        if archive_path.exists() {
            chain.add_archive(&archive_path, priority)?;
            if priority < 10 || archive_name.contains("expansion4") {
                println!("Added: {} (priority: {})", archive_name, priority);
            }
            priority += 1;
        }
    }

    if priority > 10 {
        println!("... {} more base archives ...", priority - 10);
    }

    // Locale base archives
    let locale_archives = vec![
        format!("locale-{}.MPQ", locale),
        format!("expansion1-locale-{}.MPQ", locale),
        format!("expansion2-locale-{}.MPQ", locale),
        format!("expansion3-locale-{}.MPQ", locale),
        format!("expansion4-locale-{}.MPQ", locale), // MoP locale
    ];

    let mut locale_priority = LOCALE_BASE;
    for archive_name in locale_archives {
        let archive_path = locale_path.join(&archive_name);
        if archive_path.exists() {
            chain.add_archive(&archive_path, locale_priority)?;
            println!(
                "Added: {}/{} (priority: {})",
                locale, archive_name, locale_priority
            );
            locale_priority += 1;
        }
    }

    // Base patches (MoP had extensive patching)
    let mut patch_priority = PATCH_BASE;
    let mut patch_count = 0;
    for i in 1..=50 {
        // MoP had many patches
        let patch_name = format!("base-{}.MPQ", i);
        let patch_path = data_path.join(&patch_name);
        if patch_path.exists() {
            chain.add_archive(&patch_path, patch_priority)?;
            if patch_count < 3 {
                println!("Added: {} (priority: {})", patch_name, patch_priority);
            }
            patch_priority += 1;
            patch_count += 1;
        }
    }

    if patch_count > 3 {
        println!("... {} more base patches ...", patch_count - 3);
    }

    // Locale patches
    let mut locale_patch_priority = LOCALE_PATCH_BASE;
    let mut locale_patch_count = 0;
    for i in 1..=50 {
        let patch_name = format!("base-{}.MPQ", i);
        let locale_patch_path = locale_path.join(&patch_name);
        if locale_patch_path.exists() {
            chain.add_archive(&locale_patch_path, locale_patch_priority)?;
            if locale_patch_count < 3 {
                println!(
                    "Added: {}/{} (priority: {})",
                    locale, patch_name, locale_patch_priority
                );
            }
            locale_patch_priority += 1;
            locale_patch_count += 1;
        }
    }

    if locale_patch_count > 3 {
        println!("... {} more locale patches ...", locale_patch_count - 3);
    }

    // wow-update archives (extensive in MoP)
    let mut update_priority = WOW_UPDATE_BASE;
    let mut update_count = 0;
    for i in 13156..=18500 {
        // MoP update range
        let update_name = format!("wow-update-{}.MPQ", i);
        let update_path = data_path.join(&update_name);
        if update_path.exists() {
            chain.add_archive(&update_path, update_priority)?;
            if update_count < 3 || i > 18490 {
                println!("Added: {} (priority: {})", update_name, update_priority);
            }
            update_priority += 1;
            update_count += 1;
        }

        // Locale wow-update
        let locale_update_path = locale_path.join(&update_name);
        if locale_update_path.exists() {
            chain.add_archive(&locale_update_path, LOCALE_UPDATE_BASE + (i - 13156))?;
            if update_count < 3 {
                println!(
                    "Added: {}/{} (priority: {})",
                    locale,
                    update_name,
                    LOCALE_UPDATE_BASE + (i - 13156)
                );
            }
        }
    }

    if update_count > 6 {
        println!("... {} more wow-update archives ...", update_count - 6);
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

    // Check ChrClasses.dbc for Monk
    println!("\n{}:", monk_file);
    match chain.read_file(monk_file) {
        Ok(data) => {
            println!("  Size: {} bytes", data.len());
            if let Some(source) = chain.find_file_archive(monk_file) {
                println!(
                    "  Source: {}",
                    source.file_name().unwrap_or_default().to_string_lossy()
                );
            }

            if data.len() >= 20 && &data[..4] == b"WDBC" {
                let record_count = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                println!("  Classes: {} (should include Monk as #11)", record_count);
            }
        }
        Err(_) => {
            println!("  Not found in patch chain");
        }
    }

    // Try Pet Battle DB2
    println!("\n{}:", pet_battle_file);
    match chain.read_file(pet_battle_file) {
        Ok(data) => {
            println!("  Size: {} bytes", data.len());
            if let Some(source) = chain.find_file_archive(pet_battle_file) {
                println!(
                    "  Source: {}",
                    source.file_name().unwrap_or_default().to_string_lossy()
                );
            }

            if data.len() >= 4 && &data[..4] == b"WDB2" {
                println!("  ✓ Pet Battle system data found (MoP feature)!");
            }
        }
        Err(_) => {
            println!("  Not found - Pet Battles are a MoP feature");
        }
    }

    // Step 4: Analyze MoP-specific content
    println!("\n\nStep 4: Checking MoP-specific content");
    println!("-------------------------------------");

    let mop_features = vec![
        ("DBFilesClient\\Scenario.dbc", "Scenario system"),
        ("DBFilesClient\\ScenarioStep.dbc", "Scenario steps"),
        (
            "DBFilesClient\\BattlePetAbility.db2",
            "Pet battle abilities",
        ),
        ("DBFilesClient\\BattlePetSpecies.db2", "Pet battle species"),
        ("DBFilesClient\\ItemUpgrade.dbc", "Item upgrade system"),
        ("DBFilesClient\\TransmogSet.dbc", "Transmog collections"),
        ("DBFilesClient\\Mount.dbc", "Account-wide mounts"),
        (
            "DBFilesClient\\SpellAuraOptions.dbc",
            "Expanded spell system",
        ),
    ];

    for (file, description) in mop_features {
        if let Some(source) = chain.find_file_archive(file) {
            let source_name = source.file_name().unwrap_or_default().to_string_lossy();
            println!("  {} - {}", file, description);
            println!("    Source: {}", source_name);

            if source_name.contains("expansion4") {
                println!("    ^ MoP core content");
            } else if source_name.contains("wow-update") {
                let update_num = source_name
                    .split('-')
                    .next_back()
                    .and_then(|s| s.strip_suffix(".MPQ"))
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(0);

                if update_num >= 16016 {
                    // 5.0.4 release
                    println!("    ^ Added during MoP lifecycle");
                }
            }
        }
    }

    // Step 5: Summary and CASC preparation notes
    println!("\n\nSummary");
    println!("-------");
    println!("Total archives in chain: {}", chain.archive_count());

    let chain_info = chain.get_chain_info();
    let highest_priority = chain_info.last().map(|i| i.priority).unwrap_or(0);
    println!("Highest priority: {}", highest_priority);
    println!("Archive complexity: Very High (preparing for CASC transition)");

    println!("\nPatch distribution:");
    let base_count = chain_info
        .iter()
        .filter(|i| i.priority < LOCALE_BASE)
        .count();
    let locale_count = chain_info
        .iter()
        .filter(|i| i.priority >= LOCALE_BASE && i.priority < PATCH_BASE)
        .count();
    let patch_count = chain_info
        .iter()
        .filter(|i| i.priority >= PATCH_BASE && i.priority < WOW_UPDATE_BASE)
        .count();
    let update_count = chain_info
        .iter()
        .filter(|i| i.priority >= WOW_UPDATE_BASE)
        .count();

    println!("  Base archives: {}", base_count);
    println!("  Locale archives: {}", locale_count);
    println!("  Patch archives: {}", patch_count);
    println!("  Update archives: {}", update_count);

    println!("\nNote: WoW 6.0 (Warlords) switched to CASC storage system");

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
