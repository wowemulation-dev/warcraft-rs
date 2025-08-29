//! Example: Detecting and working with different M2 dependency formats
//!
//! This example demonstrates the automatic format detection for ANIM and SKIN files
//! across different World of Warcraft versions.
//!
//! Usage: cargo run --example detect_formats -- <m2_file> [skin_file] [anim_file]

use wow_m2::{AnimFile, M2Model, SkinFile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <m2_file> [skin_file] [anim_file]", args[0]);
        eprintln!("Example: {} Character/Human/Male/HumanMale.m2", args[0]);
        std::process::exit(1);
    }

    // Load the M2 model
    let m2_path = &args[1];
    println!("Loading M2 model from: {m2_path}");
    let m2_format = M2Model::load(m2_path)?;
    let model = m2_format.model();

    println!("\n=== M2 Model Info ===");
    println!(
        "Format: {}",
        if m2_format.is_chunked() {
            "Chunked (MD21)"
        } else {
            "Legacy (MD20)"
        }
    );
    println!("Name: {:?}", model.name);
    println!("Version: {:?}", model.header.version());
    println!("Global Sequences: {}", model.global_sequences.len());
    println!("Animations: {}", model.animations.len());
    println!("Bones: {}", model.bones.len());
    println!("Vertices: {}", model.vertices.len());

    // Check for embedded vs external data
    println!("\n=== Dependency Analysis ===");
    if let Some(version) = model.header.version() {
        if version < wow_m2::version::M2Version::WotLK {
            println!("✅ Pre-WotLK model: SKIN and ANIM data embedded");
        } else {
            println!("⚠️  Post-WotLK model: SKIN and ANIM data external");
        }
    } else {
        println!("⚠️  Unknown version - cannot determine dependency format");
    }

    // Load and detect SKIN format if provided
    if args.len() > 2 {
        let skin_path = &args[2];
        println!("\n=== SKIN File Analysis ===");
        println!("Loading SKIN from: {skin_path}");

        let skin_file = SkinFile::load(skin_path)?;

        match &skin_file {
            SkinFile::New(skin) => {
                println!("Format: New (with version field)");
                println!("Version: {}", skin.header.version);
                println!("Typically used for: Camera files");
                println!("Vertices: {}", skin.header.vertex_count);
                println!("Indices: {}", skin.indices.len());
                println!("Submeshes: {}", skin.submeshes.len());
            }
            SkinFile::Old(skin) => {
                println!("Format: Old (no version field)");
                println!("Typically used for: Character/creature models");
                println!("Indices: {}", skin.indices.len());
                println!("Bone indices: {}", skin.bone_indices.len());
                println!("Submeshes: {}", skin.submeshes.len());
            }
        }

        // Display unified access
        println!("\n=== Unified Access ===");
        println!("Indices (format-agnostic): {}", skin_file.indices().len());
        println!(
            "Submeshes (format-agnostic): {}",
            skin_file.submeshes().len()
        );
    }

    // Load and detect ANIM format if provided
    if args.len() > 3 {
        let anim_path = &args[3];
        println!("\n=== ANIM File Analysis ===");
        println!("Loading ANIM from: {anim_path}");

        let anim_file = AnimFile::load(anim_path)?;

        match &anim_file.format {
            wow_m2::AnimFormat::Legacy => {
                println!("Format: Legacy (raw data, no magic)");
                println!("Typically used in: Cataclysm, Mists of Pandaria");
                println!("Sections: {}", anim_file.sections.len());
            }
            wow_m2::AnimFormat::Modern => {
                println!("Format: Modern (MAOF chunked)");
                println!("Typically used in: Legion and later");
                println!("Sections: {}", anim_file.sections.len());

                for (i, section) in anim_file.sections.iter().enumerate() {
                    println!("\n  Section {i}:");
                    println!("    Animation ID: {}", section.header.id);
                    println!("    Start frame: {}", section.header.start);
                    println!("    End frame: {}", section.header.end);
                    println!("    Bone animations: {}", section.bone_animations.len());
                }
            }
        }

        // Display memory usage
        let memory = anim_file.memory_usage();
        println!("\n=== Memory Analysis ===");
        println!("Total memory: {} bytes", memory.approximate_bytes);
    }

    // Provide recommendations
    println!("\n=== Recommendations ===");

    if let Some(version) = model.header.version() {
        if version < wow_m2::version::M2Version::WotLK {
            println!("• This model has embedded dependencies - no external files needed");
        } else {
            println!("• This model requires external files:");
            println!("  - SKIN files: model00.skin, model01.skin, etc.");
            println!("  - ANIM files: model[AnimID]-[SubAnimID].anim");
        }
    }

    if args.len() == 2 {
        println!("• Try loading with SKIN/ANIM files to see format detection in action");
        println!(
            "  Example: {} {} model00.skin model0-0.anim",
            args[0], m2_path
        );
    }

    Ok(())
}
