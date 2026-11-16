//! Demonstrates the correct MPQ archive loading order for different WoW versions
//! Based on the official client loading order used by TrinityCore
//!
//! This example consolidates all patch chain functionality and demonstrates:
//! - Correct loading priorities for each WoW version
//! - File resolution through patch chains
//! - Version-specific archive structures
//!
//! Usage:
//!     cargo run --example wow_patch_chains

use std::path::Path;
use wow_mpq::test_utils::{WowVersion, find_wow_data, print_setup_instructions};
use wow_mpq::{PatchChain, Result};

/// Priority constants that match WoW's internal loading order
#[allow(dead_code)]
mod priorities {
    // Base archives (loaded first)
    pub const BASE: i32 = 0;
    pub const BASE_2: i32 = 1;

    // Expansion archives
    pub const EXPANSION: i32 = 100;
    pub const LICHKING: i32 = 200;
    pub const CATACLYSM: i32 = 300;
    pub const PANDARIA: i32 = 400;

    // Locale archives (override base/expansion)
    pub const LOCALE_BASE: i32 = 1000;
    pub const SPEECH_BASE: i32 = 1100;
    pub const EXPANSION_LOCALE: i32 = 1200;
    pub const LICHKING_LOCALE: i32 = 1300;
    pub const EXPANSION_SPEECH: i32 = 1400;
    pub const LICHKING_SPEECH: i32 = 1500;
    pub const CATACLYSM_LOCALE: i32 = 1600;
    pub const PANDARIA_LOCALE: i32 = 1700;

    // Patches (loaded after all base content)
    pub const PATCH_1: i32 = 10000;
    pub const PATCH_2: i32 = 10001;
    pub const PATCH_3: i32 = 10002;
    pub const PATCH_4: i32 = 10003;
    pub const PATCH_5: i32 = 10004;

    // Locale patches (highest priority)
    pub const PATCH_LOCALE_1: i32 = 20000;
    pub const PATCH_LOCALE_2: i32 = 20001;
    pub const PATCH_LOCALE_3: i32 = 20002;
    pub const PATCH_LOCALE_4: i32 = 20003;
    pub const PATCH_LOCALE_5: i32 = 20004;
}

/// Setup patch chain for WoW 1.12.1 (Vanilla)
/// Loading order based on original client behavior
pub fn setup_vanilla_1_12_1(data_path: &Path, _locale: &str) -> Result<PatchChain> {
    let mut chain = PatchChain::new();

    println!("Setting up WoW 1.12.1 patch chain...");

    // Base archives - these contain categorized content
    let base_archives = [
        ("dbc.MPQ", priorities::BASE),
        ("fonts.MPQ", priorities::BASE),
        ("interface.MPQ", priorities::BASE),
        ("misc.MPQ", priorities::BASE),
        ("model.MPQ", priorities::BASE),
        ("sound.MPQ", priorities::BASE),
        ("speech.MPQ", priorities::BASE),
        ("terrain.MPQ", priorities::BASE),
        ("texture.MPQ", priorities::BASE),
        ("wmo.MPQ", priorities::BASE),
    ];

    for (archive, priority) in &base_archives {
        let path = data_path.join(archive);
        if path.exists() {
            chain.add_archive(&path, *priority)?;
            println!("  Added: {archive} (priority: {priority})");
        }
    }

    // Patches - these override base content
    let patches = [
        ("patch.MPQ", priorities::PATCH_1),
        ("patch-2.MPQ", priorities::PATCH_2),
    ];

    for (patch, priority) in &patches {
        let path = data_path.join(patch);
        if path.exists() {
            chain.add_archive(&path, *priority)?;
            println!("  Added: {patch} (priority: {priority})");
        }
    }

    // Note: Vanilla didn't have locale-specific patches in the same way as later versions

    Ok(chain)
}

/// Setup patch chain for WoW 2.4.3 (The Burning Crusade)
/// This version introduced the common.MPQ structure
pub fn setup_tbc_2_4_3(data_path: &Path, locale: &str) -> Result<PatchChain> {
    let mut chain = PatchChain::new();

    println!("Setting up WoW 2.4.3 patch chain for locale: {locale}...");

    // Base archives
    let archives = [
        ("common.MPQ", priorities::BASE),
        ("common-2.MPQ", priorities::BASE_2),
        ("expansion.MPQ", priorities::EXPANSION),
    ];

    for (archive, priority) in &archives {
        let path = data_path.join(archive);
        if path.exists() {
            chain.add_archive(&path, *priority)?;
            println!("  Added: {archive} (priority: {priority})");
        }
    }

    // Locale archives
    let locale_path = data_path.join(locale);
    let locale_archives = [
        (format!("locale-{locale}.MPQ"), priorities::LOCALE_BASE),
        (format!("speech-{locale}.MPQ"), priorities::SPEECH_BASE),
        (
            format!("expansion-locale-{locale}.MPQ"),
            priorities::EXPANSION_LOCALE,
        ),
        (
            format!("expansion-speech-{locale}.MPQ"),
            priorities::EXPANSION_SPEECH,
        ),
    ];

    for (archive, priority) in &locale_archives {
        let path = locale_path.join(archive);
        if path.exists() {
            chain.add_archive(&path, *priority)?;
            println!("  Added: {locale}/{archive} (priority: {priority})");
        }
    }

    // General patches
    let patches = [
        ("patch.MPQ", priorities::PATCH_1),
        ("patch-2.MPQ", priorities::PATCH_2),
    ];

    for (patch, priority) in &patches {
        let path = data_path.join(patch);
        if path.exists() {
            chain.add_archive(&path, *priority)?;
            println!("  Added: {patch} (priority: {priority})");
        }
    }

    // Locale patches
    let locale_patches = [
        (format!("patch-{locale}.MPQ"), priorities::PATCH_LOCALE_1),
        (format!("patch-{locale}-2.MPQ"), priorities::PATCH_LOCALE_2),
    ];

    for (patch, priority) in &locale_patches {
        let path = locale_path.join(patch);
        if path.exists() {
            chain.add_archive(&path, *priority)?;
            println!("  Added: {locale}/{patch} (priority: {priority})");
        }
    }

    Ok(chain)
}

/// Setup patch chain for WoW 3.3.5a (Wrath of the Lich King)
/// This is the definitive loading order from TrinityCore
pub fn setup_wotlk_3_3_5a(data_path: &Path, locale: &str) -> Result<PatchChain> {
    let mut chain = PatchChain::new();

    println!("Setting up WoW 3.3.5a patch chain for locale: {locale}...");
    println!("Using TrinityCore's definitive loading order");

    // Step 1-4: Base and expansion archives
    let base_archives = [
        ("common.MPQ", priorities::BASE),
        ("common-2.MPQ", priorities::BASE_2),
        ("expansion.MPQ", priorities::EXPANSION),
        ("lichking.MPQ", priorities::LICHKING),
    ];

    for (archive, priority) in &base_archives {
        let path = data_path.join(archive);
        if path.exists() {
            chain.add_archive(&path, *priority)?;
            println!("  Added: {archive} (priority: {priority})");
        }
    }

    // Step 5-10: Locale and speech archives
    let locale_archives = [
        (format!("locale-{locale}.MPQ"), priorities::LOCALE_BASE),
        (format!("speech-{locale}.MPQ"), priorities::SPEECH_BASE),
        (
            format!("expansion-locale-{locale}.MPQ"),
            priorities::EXPANSION_LOCALE,
        ),
        (
            format!("lichking-locale-{locale}.MPQ"),
            priorities::LICHKING_LOCALE,
        ),
        (
            format!("expansion-speech-{locale}.MPQ"),
            priorities::EXPANSION_SPEECH,
        ),
        (
            format!("lichking-speech-{locale}.MPQ"),
            priorities::LICHKING_SPEECH,
        ),
    ];

    for (archive, priority) in &locale_archives {
        let path = data_path.join(archive);
        if path.exists() {
            chain.add_archive(&path, *priority)?;
            println!("  Added: {archive} (priority: {priority})");
        }
    }

    // Step 11-13: General patches
    let patches = [
        ("patch.MPQ", priorities::PATCH_1),
        ("patch-2.MPQ", priorities::PATCH_2),
        ("patch-3.MPQ", priorities::PATCH_3),
    ];

    for (patch, priority) in &patches {
        let path = data_path.join(patch);
        if path.exists() {
            chain.add_archive(&path, *priority)?;
            println!("  Added: {patch} (priority: {priority})");
        }
    }

    // Step 14-16: Locale patches (in locale subdirectory)
    let locale_path = data_path.join(locale);
    let locale_patches = [
        (format!("patch-{locale}.MPQ"), priorities::PATCH_LOCALE_1),
        (format!("patch-{locale}-2.MPQ"), priorities::PATCH_LOCALE_2),
        (format!("patch-{locale}-3.MPQ"), priorities::PATCH_LOCALE_3),
    ];

    for (patch, priority) in &locale_patches {
        let path = locale_path.join(patch);
        if path.exists() {
            chain.add_archive(&path, *priority)?;
            println!("  Added: {locale}/{patch} (priority: {priority})");
        }
    }

    Ok(chain)
}

/// Setup patch chain for WoW 4.3.4 (Cataclysm)
pub fn setup_cata_4_3_4(data_path: &Path, locale: &str) -> Result<PatchChain> {
    let mut chain = PatchChain::new();

    println!("Setting up WoW 4.3.4 patch chain for locale: {locale}...");

    // Base archives (Cataclysm restructured some files)
    let base_archives = [
        ("art.MPQ", priorities::BASE),
        ("expansion1.MPQ", priorities::EXPANSION),
        ("expansion2.MPQ", priorities::LICHKING),
        ("expansion3.MPQ", priorities::CATACLYSM),
        ("sound.MPQ", priorities::BASE),
        ("world.MPQ", priorities::BASE),
    ];

    for (archive, priority) in &base_archives {
        let path = data_path.join(archive);
        if path.exists() {
            chain.add_archive(&path, *priority)?;
            println!("  Added: {archive} (priority: {priority})");
        }
    }

    // Locale archives
    let locale_archives = [
        (format!("locale-{locale}.MPQ"), priorities::LOCALE_BASE),
        (
            format!("expansion1-locale-{locale}.MPQ"),
            priorities::EXPANSION_LOCALE,
        ),
        (
            format!("expansion2-locale-{locale}.MPQ"),
            priorities::LICHKING_LOCALE,
        ),
        (
            format!("expansion3-locale-{locale}.MPQ"),
            priorities::CATACLYSM_LOCALE,
        ),
    ];

    for (archive, priority) in &locale_archives {
        let path = data_path.join(archive);
        if path.exists() {
            chain.add_archive(&path, *priority)?;
            println!("  Added: {archive} (priority: {priority})");
        }
    }

    // Patches (Cataclysm had many patches)
    for i in 1..=5 {
        let patch_name = if i == 1 {
            "wow-update-base-15211.MPQ".to_string()
        } else {
            format!("wow-update-base-{}.MPQ", 15210 + i)
        };

        let path = data_path.join(&patch_name);
        if path.exists() {
            chain.add_archive(&path, priorities::PATCH_1 + i - 1)?;
            println!(
                "  Added: {} (priority: {})",
                patch_name,
                priorities::PATCH_1 + i - 1
            );
        }
    }

    // Locale patches
    let locale_path = data_path.join(locale);
    for i in 1..=5 {
        let patch_name = format!("wow-update-{}-{}.MPQ", locale, 15210 + i);
        let path = locale_path.join(&patch_name);
        if path.exists() {
            chain.add_archive(&path, priorities::PATCH_LOCALE_1 + i - 1)?;
            println!(
                "  Added: {}/{} (priority: {})",
                locale,
                patch_name,
                priorities::PATCH_LOCALE_1 + i - 1
            );
        }
    }

    Ok(chain)
}

/// Setup patch chain for WoW 5.4.8 (Mists of Pandaria)
pub fn setup_mop_5_4_8(data_path: &Path, locale: &str) -> Result<PatchChain> {
    let mut chain = PatchChain::new();

    println!("Setting up WoW 5.4.8 patch chain for locale: {locale}...");

    // Base archives
    let base_archives = [
        ("art.MPQ", priorities::BASE),
        ("expansion1.MPQ", priorities::EXPANSION),
        ("expansion2.MPQ", priorities::LICHKING),
        ("expansion3.MPQ", priorities::CATACLYSM),
        ("expansion4.MPQ", priorities::PANDARIA),
        ("misc.MPQ", priorities::BASE),
        ("sound.MPQ", priorities::BASE),
        ("world.MPQ", priorities::BASE),
        ("world2.MPQ", priorities::BASE),
    ];

    for (archive, priority) in &base_archives {
        let path = data_path.join(archive);
        if path.exists() {
            chain.add_archive(&path, *priority)?;
            println!("  Added: {archive} (priority: {priority})");
        }
    }

    // Locale archives
    let locale_archives = [
        (format!("locale-{locale}.MPQ"), priorities::LOCALE_BASE),
        (
            format!("expansion1-locale-{locale}.MPQ"),
            priorities::EXPANSION_LOCALE,
        ),
        (
            format!("expansion2-locale-{locale}.MPQ"),
            priorities::LICHKING_LOCALE,
        ),
        (
            format!("expansion3-locale-{locale}.MPQ"),
            priorities::CATACLYSM_LOCALE,
        ),
        (
            format!("expansion4-locale-{locale}.MPQ"),
            priorities::PANDARIA_LOCALE,
        ),
    ];

    for (archive, priority) in &locale_archives {
        let path = data_path.join(archive);
        if path.exists() {
            chain.add_archive(&path, *priority)?;
            println!("  Added: {archive} (priority: {priority})");
        }
    }

    // Patches (MoP had many incremental patches)
    for i in 1..=8 {
        let patch_name = format!("wow-update-base-{}.MPQ", 17000 + i);
        let path = data_path.join(&patch_name);
        if path.exists() {
            chain.add_archive(&path, priorities::PATCH_1 + i - 1)?;
            println!(
                "  Added: {} (priority: {})",
                patch_name,
                priorities::PATCH_1 + i - 1
            );
        }
    }

    // Locale patches
    let locale_path = data_path.join(locale);
    for i in 1..=8 {
        let patch_name = format!("wow-update-{}-{}.MPQ", locale, 17000 + i);
        let path = locale_path.join(&patch_name);
        if path.exists() {
            chain.add_archive(&path, priorities::PATCH_LOCALE_1 + i - 1)?;
            println!(
                "  Added: {}/{} (priority: {})",
                locale,
                patch_name,
                priorities::PATCH_LOCALE_1 + i - 1
            );
        }
    }

    Ok(chain)
}

/// Demonstrate patch chain usage for a specific WoW version
fn demonstrate_patch_chain(chain: &mut PatchChain) -> Result<()> {
    println!("\nPatch chain summary:");
    println!("  Total archives: {}", chain.archive_count());

    // Get chain information
    let chain_info = chain.get_chain_info();
    println!("\nArchives in priority order (highest to lowest):");
    for info in &chain_info {
        println!(
            "  {} (priority: {}, files: {})",
            info.path.file_name().unwrap_or_default().to_string_lossy(),
            info.priority,
            info.file_count
        );
    }

    // Test file resolution
    println!("\nTesting file resolution:");
    let test_files = vec![
        "Interface/FrameXML/UIParent.lua",
        "Interface/GlueXML/CharacterCreate.lua",
        "DBFilesClient/Spell.dbc",
        "DBFilesClient/Item.dbc",
        "World/Maps/Azeroth/Azeroth_32_48.adt",
    ];

    for filename in &test_files {
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
            println!("  {filename} -> Not found");
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    println!("World of Warcraft MPQ Patch Chain Examples");
    println!("==========================================");
    println!();
    println!("This demonstrates the correct loading order for each WoW version");
    println!("based on TrinityCore's implementation and official client behavior.");
    println!();

    let locale = "enUS"; // Change to your locale: enUS, deDE, frFR, etc.

    // Try to find and demonstrate each WoW version
    let versions = [
        (
            WowVersion::Vanilla,
            "1.12.1",
            "/home/danielsreichenbach/Downloads/wow/1.12.1/Data",
        ),
        (
            WowVersion::Tbc,
            "2.4.3",
            "/home/danielsreichenbach/Downloads/wow/2.4.3/Data",
        ),
        (
            WowVersion::Wotlk,
            "3.3.5a",
            "/home/danielsreichenbach/Downloads/wow/3.3.5a/Data",
        ),
        (
            WowVersion::Cata,
            "4.3.4",
            "/home/danielsreichenbach/Downloads/wow/4.3.4/4.3.4/Data",
        ),
        (
            WowVersion::Mop,
            "5.4.8",
            "/home/danielsreichenbach/Downloads/wow/5.4.8/5.4.8/Data",
        ),
    ];

    let mut found_any = false;

    for (version, version_str, hardcoded_path) in &versions {
        // Try to find data using test utils first, then fallback to hardcoded path
        let data_path = find_wow_data(*version).or_else(|| {
            let path = Path::new(hardcoded_path);
            if path.exists() {
                Some(path.to_path_buf())
            } else {
                None
            }
        });

        if let Some(path) = data_path {
            found_any = true;
            println!("\n{}", "=".repeat(60));
            println!("WoW {version_str} Demo");
            println!("{}", "=".repeat(60));
            println!("Data path: {}", path.display());
            println!();

            let result = match version {
                WowVersion::Vanilla => setup_vanilla_1_12_1(&path, locale),
                WowVersion::Tbc => setup_tbc_2_4_3(&path, locale),
                WowVersion::Wotlk => setup_wotlk_3_3_5a(&path, locale),
                WowVersion::Cata => setup_cata_4_3_4(&path, locale),
                WowVersion::Mop => setup_mop_5_4_8(&path, locale),
            };

            match result {
                Ok(mut chain) => {
                    demonstrate_patch_chain(&mut chain)?;
                }
                Err(e) => {
                    println!("Error setting up patch chain: {e}");
                }
            }
        }
    }

    if !found_any {
        println!("No WoW data found!");
        println!();
        print_setup_instructions();
        println!();
        println!("Note: You can also set hardcoded paths in this example file.");
    }

    Ok(())
}
