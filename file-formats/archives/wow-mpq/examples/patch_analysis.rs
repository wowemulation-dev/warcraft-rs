//! Analyzes which files were modified by patches in WoW 1.12.1

use std::collections::HashSet;
use std::path::Path;
use wow_mpq::{Archive, Result};

fn main() -> Result<()> {
    println!("WoW 1.12.1 Patch Analysis");
    println!("=========================\n");

    let data_path = Path::new("/home/danielsreichenbach/Downloads/wow/1.12.1/Data");
    if !data_path.exists() {
        println!("WoW 1.12.1 data not found at: {}", data_path.display());
        return Ok(());
    }

    // Open dbc.MPQ to get base DBC files
    println!("Analyzing dbc.MPQ (base DBC files)...");
    let mut dbc_archive = Archive::open(data_path.join("dbc.MPQ"))?;
    let dbc_files = dbc_archive.list()?;

    let mut dbc_filenames: HashSet<String> = HashSet::new();
    for entry in &dbc_files {
        if entry.name.starts_with("DBFilesClient\\") && entry.name.ends_with(".dbc") {
            dbc_filenames.insert(entry.name.clone());
        }
    }
    println!("Found {} DBC files in base archive", dbc_filenames.len());

    // Check patch.MPQ
    println!("\nAnalyzing patch.MPQ...");
    let mut patch_archive = Archive::open(data_path.join("patch.MPQ"))?;
    let patch_files = patch_archive.list()?;

    let mut patch_dbcs: Vec<String> = Vec::new();
    let mut patch_new_dbcs: Vec<String> = Vec::new();

    for entry in &patch_files {
        if entry.name.starts_with("DBFilesClient\\") && entry.name.ends_with(".dbc") {
            if dbc_filenames.contains(&entry.name) {
                patch_dbcs.push(entry.name.clone());
            } else {
                patch_new_dbcs.push(entry.name.clone());
            }
        }
    }

    println!("  Modified DBCs: {}", patch_dbcs.len());
    println!("  New DBCs: {}", patch_new_dbcs.len());

    // Check patch-2.MPQ
    println!("\nAnalyzing patch-2.MPQ...");
    let mut patch2_archive = Archive::open(data_path.join("patch-2.MPQ"))?;
    let patch2_files = patch2_archive.list()?;

    let mut patch2_dbcs: Vec<String> = Vec::new();
    let mut patch2_new_dbcs: Vec<String> = Vec::new();

    for entry in &patch2_files {
        if entry.name.starts_with("DBFilesClient\\") && entry.name.ends_with(".dbc") {
            if dbc_filenames.contains(&entry.name) || patch_dbcs.contains(&entry.name) {
                patch2_dbcs.push(entry.name.clone());
            } else {
                patch2_new_dbcs.push(entry.name.clone());
            }
        }
    }

    println!("  Modified DBCs: {}", patch2_dbcs.len());
    println!("  New DBCs: {}", patch2_new_dbcs.len());

    // Show details about commonly patched DBCs
    println!("\n\nDetailed Analysis of Patched DBCs:");
    println!("==================================");

    let important_dbcs = vec![
        "DBFilesClient\\Spell.dbc",
        "DBFilesClient\\Item.dbc",
        "DBFilesClient\\Talent.dbc",
        "DBFilesClient\\SkillLineAbility.dbc",
        "DBFilesClient\\CreatureDisplayInfo.dbc",
    ];

    for dbc_name in important_dbcs {
        println!("\n{dbc_name}");
        println!("{}", "-".repeat(dbc_name.len()));

        // Get from each archive
        let mut base_size = 0;
        let mut patch_size = 0;

        if let Ok(data) = dbc_archive.read_file(dbc_name) {
            base_size = data.len();
            println!("  Base (dbc.MPQ): {base_size} bytes");
        }

        if let Ok(data) = patch_archive.read_file(dbc_name) {
            patch_size = data.len();
            println!(
                "  Patch 1: {} bytes (+{} bytes)",
                patch_size,
                patch_size as i64 - base_size as i64
            );
        }

        if let Ok(data) = patch2_archive.read_file(dbc_name) {
            let patch2_size = data.len();
            let total_change = patch2_size as i64 - base_size as i64;
            let from_patch1 = patch2_size as i64 - patch_size.max(base_size) as i64;
            println!(
                "  Patch 2: {patch2_size} bytes (+{total_change} total, +{from_patch1} from patch 1)"
            );
        }
    }

    // Show some of the new DBCs added by patches
    println!("\n\nNew DBCs added by patches:");
    println!("=========================");

    if !patch_new_dbcs.is_empty() {
        println!("\nAdded in patch.MPQ:");
        for dbc in patch_new_dbcs.iter().take(5) {
            println!("  + {dbc}");
        }
        if patch_new_dbcs.len() > 5 {
            println!("  ... and {} more", patch_new_dbcs.len() - 5);
        }
    }

    if !patch2_new_dbcs.is_empty() {
        println!("\nAdded in patch-2.MPQ:");
        for dbc in patch2_new_dbcs.iter().take(5) {
            println!("  + {dbc}");
        }
        if patch2_new_dbcs.len() > 5 {
            println!("  ... and {} more", patch2_new_dbcs.len() - 5);
        }
    }

    // Clean up extracted files
    let _ = std::fs::remove_file("spell_base.dbc");
    let _ = std::fs::remove_file("spell_patched.dbc");

    Ok(())
}
