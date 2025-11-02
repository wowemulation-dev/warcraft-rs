//! Example: Basic ADT File Parsing
//!
//! This example demonstrates how to parse an ADT (terrain) file and extract
//! basic information about the terrain chunks, textures, and models.

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use wow_adt::{AdtVersion, ParsedAdt, parse_adt};

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

    println!("Parsing ADT File: {adt_path}");
    println!("{}", "=".repeat(50));

    // Open and parse the ADT file
    let file = File::open(adt_path)?;
    let mut reader = BufReader::new(file);
    let adt = parse_adt(&mut reader)?;

    // Process based on file type
    match adt {
        ParsedAdt::Root(root) => {
            display_root_adt(&root);
        }
        ParsedAdt::Tex0(tex) => {
            println!("Texture file (Cataclysm+)");
            println!("  Version: {:?}", tex.version);
            println!("  Total textures: {}", tex.textures.len());
            println!("  Chunks with texture data: {}", tex.mcnk_textures.len());
        }
        ParsedAdt::Obj0(obj) => {
            println!("Object file (Cataclysm+)");
            println!("  Version: {:?}", obj.version);
            println!("  M2 models: {}", obj.models.len());
            println!("  WMO objects: {}", obj.wmos.len());
            println!("  Chunks with objects: {}", obj.mcnk_objects.len());
        }
        ParsedAdt::Lod(lod) => {
            println!("LOD file (Cataclysm+)");
            println!("  Version: {:?}", lod.version);
        }
        _ => {
            println!("Split file (Cataclysm+)");
        }
    }

    println!();
    println!("Parsing completed successfully!");

    Ok(())
}

fn display_root_adt(adt: &wow_adt::RootAdt) {
    // Display basic information
    println!("Basic Information:");
    println!("  Version: {:?}", adt.version);
    println!("  Format: {}", get_format_description(adt.version));

    // Count active terrain chunks
    let active_chunks = adt.terrain_chunk_count();
    println!("  Active terrain chunks: {active_chunks}/256");

    // Display texture information
    println!();
    println!("Textures:");
    println!("  Total textures: {}", adt.texture_count());

    if !adt.textures.is_empty() {
        println!("  Sample textures:");
        for (i, texture) in adt.textures.iter().take(5).enumerate() {
            println!("    {}. {}", i + 1, texture);
        }
        if adt.textures.len() > 5 {
            println!("    ... and {} more", adt.textures.len() - 5);
        }
    }

    // Display model information
    println!();
    println!("Doodad Models:");
    println!("  Total models: {}", adt.model_count());

    if !adt.models.is_empty() {
        println!("  Sample models:");
        for (i, model) in adt.models.iter().take(3).enumerate() {
            println!("    {}. {}", i + 1, model);
        }
        if adt.models.len() > 3 {
            println!("    ... and {} more", adt.models.len() - 3);
        }
    }

    // Display WMO information
    println!();
    println!("World Map Objects (WMOs):");
    println!("  Total WMOs: {}", adt.wmo_count());

    if !adt.wmos.is_empty() {
        println!("  Sample WMOs:");
        for (i, wmo) in adt.wmos.iter().take(3).enumerate() {
            println!("    {}. {}", i + 1, wmo);
        }
        if adt.wmos.len() > 3 {
            println!("    ... and {} more", adt.wmos.len() - 3);
        }
    }

    // Display water information (WotLK+)
    if let Some(mh2o) = &adt.water_data {
        let water_chunks = mh2o
            .entries
            .iter()
            .filter(|entry| entry.header.has_liquid())
            .count();

        if water_chunks > 0 {
            println!();
            println!("Water Information:");
            println!("  Chunks with water: {water_chunks}/256");

            // Find first water chunk with details
            for (i, entry) in mh2o.entries.iter().enumerate() {
                if let Some(instance) = entry.instances.first() {
                    println!("  Sample water chunk {i} details:");
                    println!("    - Liquid type: {}", instance.liquid_type);
                    println!(
                        "    - Dimensions: {}x{} tiles",
                        instance.width, instance.height
                    );
                    println!("    - Vertex count: {}", instance.vertex_count());
                    println!(
                        "    - Height range: {:.2} - {:.2}",
                        instance.min_height_level, instance.max_height_level
                    );
                    break;
                }
            }
        }
    }

    // Display flight boundary information (TBC+)
    if let Some(mfbo) = &adt.flight_bounds {
        println!();
        println!("Flight Boundaries (36 bytes, 2 planes × 9 values):");
        println!(
            "  Min plane first 2 values: ({}, {})",
            mfbo.min_plane[0], mfbo.min_plane[1]
        );
        println!(
            "  Max plane first 2 values: ({}, {})",
            mfbo.max_plane[0], mfbo.max_plane[1]
        );
        println!("  Total values per plane: 9 int16 coordinates");
    }
}

fn get_format_description(version: AdtVersion) -> &'static str {
    match version {
        AdtVersion::VanillaEarly => "Classic World of Warcraft (1.x - Early)",
        AdtVersion::VanillaLate => "Classic World of Warcraft (1.x - Late)",
        AdtVersion::TBC => "The Burning Crusade (2.x)",
        AdtVersion::WotLK => "Wrath of the Lich King (3.x)",
        AdtVersion::Cataclysm => "Cataclysm (4.x)",
        AdtVersion::MoP => "Mists of Pandaria (5.x)",
    }
}
