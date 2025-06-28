//! Example: ADT File Validation
//!
//! This example demonstrates how to validate ADT files at different validation levels
//! and provides detailed information about potential issues.

use std::env;
use std::fs::File;
use std::path::Path;
use wow_adt::{Adt, ValidationLevel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: {} <path_to_adt_file> [validation_level]", args[0]);
        eprintln!();
        eprintln!("Validation levels:");
        eprintln!("  basic     - Basic structure validation (default)");
        eprintln!("  standard  - Standard validation with warnings");
        eprintln!("  strict    - Strict validation (fails on warnings)");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} Azeroth_32_32.adt", args[0]);
        eprintln!("  {} Azeroth_32_32.adt strict", args[0]);
        std::process::exit(1);
    }

    let adt_path = &args[1];
    let validation_level = if args.len() > 2 {
        match args[2].as_str() {
            "basic" => ValidationLevel::Basic,
            "standard" => ValidationLevel::Standard,
            "strict" => ValidationLevel::Strict,
            _ => {
                eprintln!("Error: Unknown validation level '{}'", args[2]);
                eprintln!("Valid levels: basic, standard, strict");
                std::process::exit(1);
            }
        }
    } else {
        ValidationLevel::Basic
    };

    // Check if file exists
    if !Path::new(adt_path).exists() {
        eprintln!("Error: File '{adt_path}' does not exist");
        std::process::exit(1);
    }

    println!("🔍 Validating ADT File: {adt_path}");
    println!("📊 Validation Level: {validation_level:?}");
    println!("{}", "=".repeat(60));

    // Open and parse the ADT file
    println!("📖 Reading ADT file...");
    let file = File::open(adt_path)?;
    let adt = match Adt::from_reader(file) {
        Ok(adt) => {
            println!("✅ Successfully parsed ADT file");
            adt
        }
        Err(e) => {
            println!("❌ Failed to parse ADT file: {e}");
            std::process::exit(1);
        }
    };

    // Display basic file information
    println!();
    println!("📋 File Information:");
    println!("  • Version: {:?}", adt.version);
    println!("  • Format: {}", get_format_description(&adt));

    // Count chunks for overview
    let chunk_counts = get_chunk_counts(&adt);
    println!("  • Structure overview:");
    for (chunk_type, count) in chunk_counts {
        println!("    - {chunk_type}: {count}");
    }

    // Perform validation
    println!();
    println!("🔍 Running {validation_level:?} validation...");
    println!("{}", "-".repeat(40));

    let validation_start = std::time::Instant::now();
    let validation_result = match validation_level {
        ValidationLevel::Basic => adt.validate().map(|_| ()),
        _ => adt.validate_with_report(validation_level).map(|_| ()),
    };
    let validation_duration = validation_start.elapsed();

    match validation_result {
        Ok(_) => {
            println!("✅ Validation PASSED");
            println!("⏱️  Validation completed in {validation_duration:?}");

            // Provide additional insights for successful validation
            provide_validation_insights(&adt, validation_level);
        }
        Err(e) => {
            println!("❌ Validation FAILED");
            println!("⏱️  Validation completed in {validation_duration:?}");
            println!();
            println!("🚨 Validation Error:");
            println!("  {e}");

            // Provide suggestions for common issues
            provide_validation_suggestions(&e.to_string());

            std::process::exit(1);
        }
    }

    println!();
    println!("✨ Validation completed successfully!");

    Ok(())
}

fn get_format_description(adt: &Adt) -> &'static str {
    use wow_adt::AdtVersion;

    match adt.version {
        AdtVersion::Vanilla => "Classic World of Warcraft (1.x)",
        AdtVersion::TBC => "The Burning Crusade (2.x)",
        AdtVersion::WotLK => "Wrath of the Lich King (3.x)",
        AdtVersion::Cataclysm => "Cataclysm+ (4.x+)",
        AdtVersion::MoP => "Mists of Pandaria (5.x)",
        AdtVersion::WoD => "Warlords of Draenor (6.x)",
        AdtVersion::Legion => "Legion (7.x)",
        AdtVersion::BfA => "Battle for Azeroth (8.x)",
        AdtVersion::Shadowlands => "Shadowlands (9.x)",
        AdtVersion::Dragonflight => "Dragonflight (10.x)",
    }
}

fn get_chunk_counts(adt: &Adt) -> Vec<(&'static str, String)> {
    let mut counts = Vec::new();

    counts.push(("MVER", "✓".to_string()));
    if adt.mhdr.is_some() {
        counts.push(("MHDR", "✓".to_string()));
    }
    if adt.mcin.is_some() {
        counts.push(("MCIN", "✓".to_string()));
    }

    if let Some(mtex) = &adt.mtex {
        counts.push(("MTEX", format!("{} textures", mtex.filenames.len())));
    }

    if let Some(mmdx) = &adt.mmdx {
        counts.push(("MMDX", format!("{} models", mmdx.filenames.len())));
    }

    if let Some(mwmo) = &adt.mwmo {
        counts.push(("MWMO", format!("{} WMOs", mwmo.filenames.len())));
    }

    if let Some(mddf) = &adt.mddf {
        counts.push(("MDDF", format!("{} doodads", mddf.doodads.len())));
    }

    if let Some(modf) = &adt.modf {
        counts.push(("MODF", format!("{} WMO placements", modf.models.len())));
    }

    if adt.mfbo.is_some() {
        counts.push(("MFBO", "✓".to_string()));
    }
    if adt.mh2o.is_some() {
        counts.push(("MH2O", "✓".to_string()));
    }

    // Count MCNK chunks
    let mcnk_count = adt.mcnk_chunks.len();
    if mcnk_count > 0 {
        counts.push(("MCNK", format!("{mcnk_count} terrain chunks")));
    }

    counts
}

fn provide_validation_insights(adt: &Adt, level: ValidationLevel) {
    println!();
    println!("💡 Validation Insights:");

    // Terrain chunk analysis
    let active_chunks = adt.mcnk_chunks.len();
    if active_chunks == 0 {
        println!("  • No terrain chunks found (this might be a split-format file)");
    } else if active_chunks < 256 {
        println!("  • Partial terrain coverage: {active_chunks}/256 chunks active");
    } else {
        println!("  • Full terrain coverage: all 256 chunks present");
    }

    // Version-specific insights
    match adt.version {
        wow_adt::AdtVersion::Vanilla => {
            println!("  • Classic format detected - no modern features expected");
        }
        wow_adt::AdtVersion::TBC => {
            println!("  • TBC format - may include MFBO flight boundaries");
        }
        wow_adt::AdtVersion::WotLK => {
            println!("  • WotLK format - may include MH2O water data");
        }
        _ => {
            println!("  • Modern format - may include split ADT files and extended features");
        }
    }

    // Water analysis
    if let Some(mh2o) = &adt.mh2o {
        let water_chunks = mh2o
            .chunks
            .iter()
            .filter(|chunk| !chunk.instances.is_empty())
            .count();
        if water_chunks > 0 {
            println!("  • Water features: {water_chunks} chunks contain water data");
        }
    }

    match level {
        ValidationLevel::Strict => {
            println!("  • Strict validation passed - file meets all quality standards");
        }
        ValidationLevel::Standard => {
            println!(
                "  • Standard validation passed - file is well-formed with minor issues tolerated"
            );
        }
        ValidationLevel::Basic => {
            println!("  • Basic validation passed - file structure is valid");
        }
    }
}

fn provide_validation_suggestions(error_msg: &str) {
    println!();
    println!("💡 Suggestions:");

    if error_msg.contains("MCNK") {
        println!("  • MCNK chunk issues often occur in split-format ADT files");
        println!("  • For Cataclysm+ files, check for corresponding _tex0.adt and _obj0.adt files");
    }

    if error_msg.contains("MH2O") {
        println!("  • MH2O water data parsing issues are common in WotLK+ files");
        println!("  • Some original Blizzard files have incomplete water data");
    }

    if error_msg.contains("MFBO") {
        println!("  • MFBO chunk size variations exist between expansions");
        println!("  • TBC uses 8-byte format, Cataclysm+ uses extended 36-byte format");
    }

    if error_msg.contains("model reference")
        || error_msg.contains("MMID")
        || error_msg.contains("MWID")
    {
        println!("  • Model reference validation failures indicate corrupted doodad/WMO indices");
        println!("  • This is often found in original Blizzard files and may be tolerable");
    }

    println!("  • Try using a lower validation level (basic/standard) if strict validation fails");
    println!("  • Consider if this is a split-format file that requires additional components");
}
