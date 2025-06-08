//! Analyzes the patch structure and locale system in WoW 2.4.3 (TBC)

use std::collections::HashMap;
use std::path::Path;
use wow_mpq::{Archive, PatchChain, Result};

fn main() -> Result<()> {
    println!("WoW 2.4.3 (TBC) Patch Structure Analysis");
    println!("========================================\n");

    let data_path = Path::new("/home/danielsreichenbach/Downloads/wow/2.4.3/Data");
    if !data_path.exists() {
        println!("WoW 2.4.3 data not found at: {}", data_path.display());
        return Ok(());
    }

    let locale = "enUS";
    let locale_path = data_path.join(locale);

    // Step 1: Analyze archive structure
    println!("Archive Structure:");
    println!("------------------");

    let archives = vec![
        ("common.MPQ", "Base game content"),
        ("expansion.MPQ", "Burning Crusade expansion"),
        ("patch.MPQ", "General patch 1"),
        ("patch-2.MPQ", "General patch 2"),
    ];

    let locale_archives = vec![
        (format!("locale-{}.MPQ", locale), "Base locale content"),
        (
            format!("expansion-locale-{}.MPQ", locale),
            "TBC locale content",
        ),
        (format!("patch-{}.MPQ", locale), "Locale patch 1"),
        (format!("patch-{}-2.MPQ", locale), "Locale patch 2"),
    ];

    // Analyze general archives
    println!("\nGeneral Archives:");
    for (archive_name, description) in &archives {
        let path = data_path.join(archive_name);
        if path.exists() {
            let mut archive = Archive::open(&path)?;
            let info = archive.get_info()?;
            println!("  {} - {}", archive_name, description);
            println!(
                "    Files: {:>6} | Size: {:>8.2} MB | Format: {:?}",
                info.file_count,
                info.file_size as f64 / 1024.0 / 1024.0,
                info.format_version
            );
        }
    }

    // Analyze locale archives
    println!("\nLocale Archives ({}):", locale);
    for (archive_name, description) in &locale_archives {
        let path = locale_path.join(archive_name);
        if path.exists() {
            let mut archive = Archive::open(&path)?;
            let info = archive.get_info()?;
            println!("  {} - {}", archive_name, description);
            println!(
                "    Files: {:>6} | Size: {:>8.2} MB | Format: {:?}",
                info.file_count,
                info.file_size as f64 / 1024.0 / 1024.0,
                info.format_version
            );
        }
    }

    // Step 2: DBC distribution analysis
    println!("\n\nDBC Distribution Analysis:");
    println!("--------------------------");

    let mut dbc_locations: HashMap<String, Vec<String>> = HashMap::new();

    // Check each archive for DBCs
    for (archive_name, _) in &archives {
        let path = data_path.join(archive_name);
        if path.exists() {
            if let Ok(mut archive) = Archive::open(&path) {
                if let Ok(files) = archive.list() {
                    for entry in files {
                        if entry.name.starts_with("DBFilesClient\\") && entry.name.ends_with(".dbc")
                        {
                            dbc_locations
                                .entry(entry.name.clone())
                                .or_default()
                                .push(archive_name.to_string());
                        }
                    }
                }
            }
        }
    }

    // Check locale archives
    for (archive_name, _) in &locale_archives {
        let path = locale_path.join(archive_name);
        if path.exists() {
            if let Ok(mut archive) = Archive::open(&path) {
                if let Ok(files) = archive.list() {
                    for entry in files {
                        if entry.name.starts_with("DBFilesClient\\") && entry.name.ends_with(".dbc")
                        {
                            dbc_locations
                                .entry(entry.name.clone())
                                .or_default()
                                .push(format!("{}/{}", locale, archive_name));
                        }
                    }
                }
            }
        }
    }

    // Analyze results
    let mut single_location = 0;
    let mut multiple_locations = 0;
    let mut locale_overridden = 0;

    for locations in dbc_locations.values() {
        if locations.len() == 1 {
            single_location += 1;
        } else {
            multiple_locations += 1;

            // Check if locale archives override
            if locations.iter().any(|l| l.contains("locale")) {
                locale_overridden += 1;
            }
        }
    }

    println!("Total unique DBCs: {}", dbc_locations.len());
    println!("DBCs in single archive: {}", single_location);
    println!("DBCs in multiple archives: {}", multiple_locations);
    println!("DBCs with locale overrides: {}", locale_overridden);

    // Step 3: Show specific examples
    println!("\n\nExample DBC Override Chains:");
    println!("-----------------------------");

    let example_dbcs = vec![
        "DBFilesClient\\Spell.dbc",
        "DBFilesClient\\Item.dbc",
        "DBFilesClient\\Map.dbc",
        "DBFilesClient\\Achievement.dbc", // Might not exist in TBC
        "DBFilesClient\\ItemExtendedCost.dbc", // TBC feature
    ];

    for dbc in example_dbcs {
        if let Some(locations) = dbc_locations.get(dbc) {
            println!("\n{}:", dbc);
            for (i, location) in locations.iter().enumerate() {
                println!(
                    "  {} {}",
                    if i == locations.len() - 1 {
                        "└─"
                    } else {
                        "├─"
                    },
                    location
                );
            }
            println!("  Override chain: {} -> Final", locations.join(" -> "));
        } else {
            println!("\n{}: Not found", dbc);
        }
    }

    // Step 4: Test actual override with patch chain
    println!("\n\nTesting Override Resolution:");
    println!("----------------------------");

    // Build proper patch chain
    let mut chain = PatchChain::new();

    // Add in TBC loading order
    chain.add_archive(data_path.join("common.MPQ"), 0)?;
    chain.add_archive(data_path.join("expansion.MPQ"), 1)?;
    chain.add_archive(locale_path.join(format!("locale-{}.MPQ", locale)), 100)?;
    chain.add_archive(
        locale_path.join(format!("expansion-locale-{}.MPQ", locale)),
        101,
    )?;
    chain.add_archive(data_path.join("patch.MPQ"), 1000)?;
    chain.add_archive(data_path.join("patch-2.MPQ"), 1001)?;
    chain.add_archive(locale_path.join(format!("patch-{}.MPQ", locale)), 2000)?;
    chain.add_archive(locale_path.join(format!("patch-{}-2.MPQ", locale)), 2001)?;

    // Test some DBCs
    let test_dbcs = vec![
        "DBFilesClient\\Spell.dbc",
        "DBFilesClient\\SpellItemEnchantment.dbc",
        "DBFilesClient\\Talent.dbc",
    ];

    println!("\nFinal resolution (highest priority wins):");
    for dbc in test_dbcs {
        if let Some(source) = chain.find_file_archive(dbc) {
            println!(
                "  {} -> {}",
                dbc,
                source.file_name().unwrap_or_default().to_string_lossy()
            );

            // Get size from final source
            if let Ok(data) = chain.read_file(dbc) {
                println!("    Final size: {} bytes", data.len());

                // For Spell.dbc, show the progression
                if dbc == "DBFilesClient\\Spell.dbc" {
                    println!("\n    Version comparison:");

                    // Try to get from each archive that has it
                    if let Some(locations) = dbc_locations.get(dbc) {
                        for location in locations {
                            let archive_path = if location.contains('/') {
                                locale_path.join(location.split('/').nth(1).unwrap_or(""))
                            } else {
                                data_path.join(location)
                            };

                            if let Ok(mut archive) = Archive::open(&archive_path) {
                                if let Ok(data) = archive.read_file(dbc) {
                                    println!("      {}: {} bytes", location, data.len());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Step 5: TBC-specific content
    println!("\n\nTBC-Specific Content:");
    println!("---------------------");

    // Look for Outland zones in AreaTable.dbc
    if let Ok(area_data) = chain.read_file("DBFilesClient\\AreaTable.dbc") {
        println!("AreaTable.dbc size: {} bytes", area_data.len());

        if area_data.len() >= 20 && &area_data[..4] == b"WDBC" {
            let record_count =
                u32::from_le_bytes([area_data[4], area_data[5], area_data[6], area_data[7]]);
            println!("  Total areas: {} (includes Outland zones)", record_count);
        }
    }

    // Check Map.dbc for Outland
    if let Ok(map_data) = chain.read_file("DBFilesClient\\Map.dbc") {
        println!("\nMap.dbc size: {} bytes", map_data.len());

        if map_data.len() >= 20 && &map_data[..4] == b"WDBC" {
            let record_count =
                u32::from_le_bytes([map_data[4], map_data[5], map_data[6], map_data[7]]);
            println!(
                "  Total maps: {} (should include Outland map ID 530)",
                record_count
            );
        }
    }

    Ok(())
}
