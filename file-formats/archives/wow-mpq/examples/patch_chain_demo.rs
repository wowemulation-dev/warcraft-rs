//! Demonstrates patch chain functionality for handling multiple MPQ archives

use std::path::Path;
use wow_mpq::{PatchChain, Result};

fn main() -> Result<()> {
    // This example demonstrates how to use PatchChain to handle
    // multiple MPQ archives with priority-based file resolution

    println!("MPQ Patch Chain Demo");
    println!("====================");

    // Check if we have WoW data available
    let wow_path = Path::new("/home/danielsreichenbach/Downloads/wow/1.12.1/Data");
    if !wow_path.exists() {
        println!("WoW 1.12.1 data not found at: {}", wow_path.display());
        println!("This demo requires actual WoW MPQ files to demonstrate patch chains.");
        return Ok(());
    }

    // Create a patch chain
    let mut chain = PatchChain::new();

    // Add base archives with priority 0
    println!("\nAdding base archives...");
    for archive in &["dbc.MPQ", "interface.MPQ", "misc.MPQ", "model.MPQ"] {
        let path = wow_path.join(archive);
        if path.exists() {
            chain.add_archive(&path, 0)?;
            println!("  Added: {} (priority: 0)", archive);
        }
    }

    // Add patch archives with higher priority
    println!("\nAdding patch archives...");
    if wow_path.join("patch.MPQ").exists() {
        chain.add_archive(wow_path.join("patch.MPQ"), 100)?;
        println!("  Added: patch.MPQ (priority: 100)");
    }
    if wow_path.join("patch-2.MPQ").exists() {
        chain.add_archive(wow_path.join("patch-2.MPQ"), 200)?;
        println!("  Added: patch-2.MPQ (priority: 200)");
    }

    // Get chain information
    println!("\nPatch chain contains {} archives:", chain.archive_count());
    let chain_info = chain.get_chain_info();
    for info in chain_info {
        println!(
            "  {} (priority: {}, files: {})",
            info.path.file_name().unwrap_or_default().to_string_lossy(),
            info.priority,
            info.file_count
        );
    }

    // Test file lookup
    println!("\nTesting file resolution...");
    let test_files = vec![
        "Interface\\FrameXML\\UIParent.lua",
        "Interface\\FrameXML\\GameTooltip.xml",
        "DBFilesClient\\Spell.dbc",
    ];

    for filename in test_files {
        if let Some(archive_path) = chain.find_file_archive(filename) {
            println!(
                "  {} -> {}",
                filename,
                archive_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            );
        } else {
            println!("  {} -> Not found", filename);
        }
    }

    // Extract a file that might be patched
    println!("\nExtracting a potentially patched file...");
    let target_file = "Interface\\FrameXML\\UIParent.lua";
    match chain.read_file(target_file) {
        Ok(data) => {
            println!(
                "  Successfully extracted {} ({} bytes)",
                target_file,
                data.len()
            );
            // Show first few lines if it's a text file
            if let Ok(text) = std::str::from_utf8(&data[..data.len().min(200)]) {
                println!("  First few lines:");
                for line in text.lines().take(3) {
                    println!("    {}", line);
                }
            }
        }
        Err(e) => {
            println!("  Failed to extract {}: {}", target_file, e);
        }
    }

    Ok(())
}
