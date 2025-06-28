//! Example: Basic ADT File Parsing
//!
//! This example demonstrates how to parse an ADT (terrain) file and extract
//! basic information about the terrain chunks, textures, and models.

use std::env;
use std::fs::File;
use std::path::Path;
use wow_adt::{Adt, ValidationLevel, WaterLevelData};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_adt_file>", args[0]);
        eprintln!();
        eprintln!("Example:");
        eprintln!("  {} Azeroth_32_32.adt", args[0]);
        std::process::exit(1);
    }

    let adt_path = &args[1];

    // Check if file exists
    if !Path::new(adt_path).exists() {
        eprintln!("Error: File '{adt_path}' does not exist");
        std::process::exit(1);
    }

    println!("🏔️  Parsing ADT File: {adt_path}");
    println!("{}", "=".repeat(50));

    // Open and parse the ADT file
    let file = File::open(adt_path)?;
    let adt = Adt::from_reader(file)?;

    // Display basic information
    println!("📊 Basic Information:");
    println!("  • Version: {:?}", adt.version);
    println!("  • Format: {}", get_format_description(&adt));

    // Count active terrain chunks
    let mut active_chunks = 0;
    if let Some(mcin) = &adt.mcin {
        for entry in &mcin.entries {
            if entry.offset > 0 && entry.size > 0 {
                active_chunks += 1;
            }
        }
    }
    println!("  • Active terrain chunks: {active_chunks}/256");

    // Display texture information
    if let Some(mtex) = &adt.mtex {
        println!();
        println!("🎨 Textures:");
        println!("  • Total textures: {}", mtex.filenames.len());

        if !mtex.filenames.is_empty() {
            println!("  • Sample textures:");
            for (i, texture) in mtex.filenames.iter().take(5).enumerate() {
                println!("    {}. {}", i + 1, texture);
            }
            if mtex.filenames.len() > 5 {
                println!("    ... and {} more", mtex.filenames.len() - 5);
            }
        }
    }

    // Display model information
    if let Some(mmdx) = &adt.mmdx {
        println!();
        println!("🌲 Doodad Models:");
        println!("  • Total models: {}", mmdx.filenames.len());

        if !mmdx.filenames.is_empty() {
            println!("  • Sample models:");
            for (i, model) in mmdx.filenames.iter().take(3).enumerate() {
                println!("    {}. {}", i + 1, model);
            }
            if mmdx.filenames.len() > 3 {
                println!("    ... and {} more", mmdx.filenames.len() - 3);
            }
        }
    }

    // Display WMO information
    if let Some(mwmo) = &adt.mwmo {
        println!();
        println!("🏰 World Map Objects (WMOs):");
        println!("  • Total WMOs: {}", mwmo.filenames.len());

        if !mwmo.filenames.is_empty() {
            println!("  • Sample WMOs:");
            for (i, wmo) in mwmo.filenames.iter().take(3).enumerate() {
                println!("    {}. {}", i + 1, wmo);
            }
            if mwmo.filenames.len() > 3 {
                println!("    ... and {} more", mwmo.filenames.len() - 3);
            }
        }
    }

    // Display water information (WotLK+)
    if let Some(mh2o) = &adt.mh2o {
        let water_chunks = mh2o
            .chunks
            .iter()
            .filter(|chunk| !chunk.instances.is_empty())
            .count();

        if water_chunks > 0 {
            println!();
            println!("💧 Water Information:");
            println!("  • Chunks with water: {water_chunks}/256");

            // Find first water chunk with details
            for (i, chunk) in mh2o.chunks.iter().enumerate() {
                if let Some(instance) = chunk.instances.first() {
                    println!("  • Sample water chunk {i} details:");
                    println!("    - Liquid type: {}", instance.liquid_type);
                    println!("    - Has vertex data: {}", instance.vertex_data.is_some());
                    match &instance.level_data {
                        WaterLevelData::Uniform {
                            min_height,
                            max_height,
                        } => {
                            println!("    - Water level: {min_height:.2} - {max_height:.2}");
                        }
                        WaterLevelData::Variable {
                            min_height,
                            max_height,
                            ..
                        } => {
                            println!(
                                "    - Water level: {min_height:.2} - {max_height:.2} (variable)"
                            );
                        }
                    }
                    break;
                }
            }
        }
    }

    // Display flight boundary information (TBC+)
    if let Some(mfbo) = &adt.mfbo {
        println!();
        println!("✈️  Flight Boundaries:");
        println!("  • Min boundary: ({}, {})", mfbo.min[0], mfbo.min[1]);
        println!("  • Max boundary: ({}, {})", mfbo.max[0], mfbo.max[1]);
        if !mfbo.additional_data.is_empty() {
            println!("  • Additional data: {} bytes", mfbo.additional_data.len());
        }
    }

    // Perform basic validation
    println!();
    println!("🔍 Validation:");
    match adt.validate() {
        Ok(_) => println!("  ✅ Basic validation passed"),
        Err(e) => println!("  ❌ Validation failed: {e}"),
    }

    match adt.validate_with_report(ValidationLevel::Standard) {
        Ok(_) => println!("  ✅ Standard validation passed"),
        Err(e) => println!("  ⚠️  Standard validation warnings: {e}"),
    }

    println!();
    println!("✨ Parsing completed successfully!");

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
