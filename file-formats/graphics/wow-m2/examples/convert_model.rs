//! Example: Converting M2 models between versions
//!
//! This example shows how to convert an M2 model from one version to another.
//!
//! Usage: cargo run --example convert_model -- <input_file> <output_file> <target_version>
//! Target versions: classic, tbc, wrath, cata, mop, wod, legion, bfa, sl, df

use wow_m2::{M2Converter, M2Model, M2Version};

fn parse_version(version_str: &str) -> Result<M2Version, String> {
    match version_str.to_lowercase().as_str() {
        "classic" | "vanilla" | "tbc" | "wrath" | "wotlk" => Ok(M2Version::Vanilla),
        "cata" | "cataclysm" => Ok(M2Version::Cataclysm),
        "mop" => Ok(M2Version::MoP),
        "wod" => Ok(M2Version::WoD),
        "legion" => Ok(M2Version::Legion),
        "bfa" => Ok(M2Version::BfA),
        "sl" | "shadowlands" => Ok(M2Version::Shadowlands),
        "df" | "dragonflight" => Ok(M2Version::Dragonflight),
        "tww" | "thewarwithin" | "modern" => Ok(M2Version::TheWarWithin),
        _ => Err(format!("Unknown version: {version_str}")),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!(
            "Usage: {} <input_file> <output_file> <target_version>",
            args[0]
        );
        eprintln!("Target versions:");
        eprintln!("  classic/vanilla/tbc/wrath/wotlk - Classic WoW (1.x-3.x)");
        eprintln!("  cata/cataclysm - Cataclysm (4.x)");
        eprintln!("  mop - Mists of Pandaria (5.x)");
        eprintln!("  wod - Warlords of Draenor (6.x)");
        eprintln!("  legion - Legion (7.x)");
        eprintln!("  bfa - Battle for Azeroth (8.x)");
        eprintln!("  sl/shadowlands - Shadowlands (9.x)");
        eprintln!("  df/dragonflight - Dragonflight (10.x)");
        eprintln!("  tww/thewarwithin/modern - The War Within+ (11.x+)");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];
    let target_version = parse_version(&args[3])?;

    // Load the model
    println!("Loading model from: {input_path}");
    let m2_format = M2Model::load(input_path)?;
    let model = m2_format.model();
    let source_version = model.header.version();

    println!("Source version: {source_version:?}");
    println!("Target version: {target_version:?}");

    // Check if conversion is needed
    if source_version == Some(target_version) {
        println!("Model is already in the target version!");
        return Ok(());
    }

    // Create converter and convert the model
    let converter = M2Converter::new();

    println!("\nConverting model...");
    let converted_model = converter.convert(model, target_version)?;

    // Save the converted model
    println!("Saving converted model to: {output_path}");
    converted_model.save(output_path)?;

    // Print conversion summary
    println!("\n=== Conversion Summary ===");
    println!("Original version: {source_version:?}");
    println!("Converted to: {target_version:?}");
    println!(
        "Vertices: {} -> {}",
        model.vertices.len(),
        converted_model.vertices.len()
    );
    println!(
        "Bones: {} -> {}",
        model.bones.len(),
        converted_model.bones.len()
    );
    println!(
        "Textures: {} -> {}",
        model.textures.len(),
        converted_model.textures.len()
    );
    println!(
        "Animations: {} -> {}",
        model.animations.len(),
        converted_model.animations.len()
    );

    // Warn about potential data loss when downgrading
    if let Some(src_ver) = source_version {
        if target_version < src_ver {
            println!("\n⚠️  Warning: Downgrading may result in data loss!");
            println!("Some features from newer versions may not be supported in older formats.");
        }
    }

    println!("\nConversion completed successfully!");

    Ok(())
}
