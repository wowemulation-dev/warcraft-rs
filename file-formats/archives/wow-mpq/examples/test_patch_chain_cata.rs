// Test PatchChain with real WoW 4.3.4 (Cataclysm) patch files
//
// This demonstrates the complete patch chain workflow:
// 1. Load base archives (art.MPQ)
// 2. Add incremental patches (wow-update-base-*.MPQ)
// 3. Read patched files automatically
// 4. Verify results match StormLib behavior

use wow_mpq::PatchChain;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let data_dir = "/home/danielsreichenbach/Downloads/wow/4.3.4/4.3.4/Data";

    println!("=======================================================");
    println!("  WoW 4.3.4 Cataclysm Patch Chain Test");
    println!("=======================================================\n");

    // Test 1: Single patch file (no base)
    println!("Test 1: Reading patch file alone (no base file)");
    println!("-------------------------------------------------------");

    let mut chain = PatchChain::new();

    // Add only the patch MPQ
    let patch_path = format!("{}/wow-update-base-15211.MPQ", data_dir);
    println!("Adding patch: {}", patch_path);
    chain.add_archive(&patch_path, 100)?;

    let filename = "Creature/DrakeMount/FelDrakeMount01.skin";
    println!("Reading file: {}", filename);

    match chain.read_file(filename) {
        Ok(data) => {
            println!("  ✓ Successfully read {} bytes", data.len());

            // Check signature
            if data.len() >= 4 {
                let sig = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                print!("  Signature: 0x{:08X}", sig);

                if sig == 0x4E494B53 {
                    println!(" (SKIN) ✓");
                } else if sig == 0x48435450 {
                    println!(" (PTCH) - ERROR: Should be patched!");
                } else {
                    println!(" (UNKNOWN)");
                }
            }

            // StormLib returns 26,464 bytes with SKIN signature
            println!("  Expected: 26,464 bytes (from StormLib comparison)");
            if data.len() == 26464 {
                println!("  ✓✓✓ SIZE MATCHES STORMLIB EXACTLY!");
            } else {
                println!("  ⚠ Size mismatch (got {} bytes)", data.len());
            }
        }
        Err(e) => {
            println!("  ✗ Failed to read: {}", e);

            // This is expected if no base file exists
            if e.to_string().contains("No base file found") {
                println!("\n  This patch requires a base file that doesn't exist.");
                println!("  The patch may be for a file added in this patch version.");
            }
        }
    }

    // Test 2: Multi-archive patch chain
    println!("\n\nTest 2: Complete patch chain (base + patches)");
    println!("-------------------------------------------------------");

    let mut full_chain = PatchChain::new();

    // Add base archives (priority 0)
    let base_archives = vec![format!("{}/art.MPQ", data_dir)];

    println!("Adding base archives:");
    for base_path in &base_archives {
        println!("  - {} (priority: 0)", base_path);
        if let Err(e) = full_chain.add_archive(base_path, 0) {
            println!("    ⚠ Failed to add: {}", e);
        }
    }

    // Add patch archives in order (increasing priority)
    let patch_archives = vec![
        (format!("{}/wow-update-base-15211.MPQ", data_dir), 100),
        (format!("{}/wow-update-base-15354.MPQ", data_dir), 200),
        (format!("{}/wow-update-base-15595.MPQ", data_dir), 300),
    ];

    println!("\nAdding patch archives:");
    for (patch_path, priority) in &patch_archives {
        println!("  - {} (priority: {})", patch_path, priority);
        if let Err(e) = full_chain.add_archive(patch_path, *priority) {
            println!("    ⚠ Failed to add: {}", e);
        }
    }

    // Show chain info
    println!("\nPatch chain information:");
    let chain_info = full_chain.get_chain_info();
    for info in &chain_info {
        println!(
            "  {} - Priority: {}, Files: {}, Size: {} MB",
            info.path.file_name().unwrap().to_str().unwrap(),
            info.priority,
            info.file_count,
            info.archive_size / 1024 / 1024
        );
    }

    // Test reading the patched file through the full chain
    println!("\nReading file through complete patch chain:");
    println!("  File: {}", filename);

    match full_chain.read_file(filename) {
        Ok(data) => {
            println!("  ✓ Successfully read {} bytes", data.len());

            // Check signature
            if data.len() >= 4 {
                let sig = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                print!("  Signature: 0x{:08X}", sig);

                if sig == 0x4E494B53 {
                    println!(" (SKIN) ✓");
                } else if sig == 0x48435450 {
                    println!(" (PTCH) - ERROR: Should be patched!");
                } else {
                    println!(" (UNKNOWN)");
                }
            }

            println!("\n  Result verification:");
            println!("    Expected size: 26,464 bytes (StormLib)");
            println!("    Actual size:   {} bytes", data.len());

            if data.len() == 26464 {
                println!("    ✓✓✓ PERFECT MATCH WITH STORMLIB!");
            }
        }
        Err(e) => {
            println!("  ✗ Failed to read: {}", e);
        }
    }

    // Test 3: Check if chain correctly identifies patch files
    println!("\n\nTest 3: Verify patch file detection");
    println!("-------------------------------------------------------");

    if full_chain.contains_file(filename) {
        let archive_path = full_chain.find_file_archive(filename);
        println!("  File found in chain");
        if let Some(path) = archive_path {
            println!("  Highest priority location: {}", path.display());

            // Check if it's a patch file by examining priority
            let priority = full_chain.get_priority(path);
            if let Some(p) = priority {
                println!("  Archive priority: {}", p);
                if p >= 100 {
                    println!("  ✓ This is from a patch archive (automatic application expected)");
                } else {
                    println!("  This is from a base archive");
                }
            }
        }
    } else {
        println!("  ✗ File not found in chain");
    }

    println!("\n=======================================================");
    println!("  Patch chain test complete!");
    println!("=======================================================");

    Ok(())
}
