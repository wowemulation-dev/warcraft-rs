//! Example: ADT File Parsing and Validation
//!
//! Demonstrates how to parse ADT files with automatic version detection
//! and provides detailed information about file structure and contents.

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use wow_adt::{AdtVersion, ParsedAdt, parse_adt_with_metadata};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_adt_file>", args[0]);
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} Azeroth_32_32.adt", args[0]);
        eprintln!("  {} Kalimdor_48_30.adt", args[0]);
        std::process::exit(1);
    }

    let adt_path = &args[1];

    // Check if file exists
    if !Path::new(adt_path).exists() {
        eprintln!("Error: File '{}' does not exist", adt_path);
        std::process::exit(1);
    }

    println!("Parsing ADT File: {}", adt_path);
    println!("{}", "=".repeat(60));

    // Open and parse the ADT file
    println!("Reading ADT file...");
    let file = File::open(adt_path)?;
    let mut reader = BufReader::new(file);

    let (adt, metadata) = match parse_adt_with_metadata(&mut reader) {
        Ok(result) => {
            println!("Successfully parsed ADT file");
            result
        }
        Err(e) => {
            println!("Failed to parse ADT file: {}", e);
            std::process::exit(1);
        }
    };

    // Display parse statistics
    println!();
    println!("Parse Statistics:");
    println!("  Discovery time: {:?}", metadata.discovery_duration);
    println!("  Parse time: {:?}", metadata.parse_duration);
    println!("  Total chunks: {}", metadata.chunk_count);

    // Display basic file information
    println!();
    println!("File Information:");
    println!("  Version: {}", get_version_description(metadata.version));
    println!("  File Type: {:?}", metadata.file_type);
    println!("  Format: {}", metadata.version);

    // Display warnings if any
    if !metadata.warnings.is_empty() {
        println!();
        println!("Warnings:");
        for warning in &metadata.warnings {
            println!("  - {}", warning);
        }
    }

    // Display file-specific information
    println!();
    match adt {
        ParsedAdt::Root(root) => display_root_info(&root),
        ParsedAdt::Tex0(tex) | ParsedAdt::Tex1(tex) => display_tex_info(&tex),
        ParsedAdt::Obj0(obj) | ParsedAdt::Obj1(obj) => display_obj_info(&obj),
        ParsedAdt::Lod(_) => println!("LOD file (level-of-detail data)"),
    }

    println!();
    println!("Parsing completed successfully");

    Ok(())
}

fn get_version_description(version: AdtVersion) -> &'static str {
    match version {
        AdtVersion::VanillaEarly => "Vanilla 1.x (Early) - Basic terrain",
        AdtVersion::VanillaLate => "Vanilla 1.9+ - Added vertex colors",
        AdtVersion::TBC => "The Burning Crusade 2.x - Added flight boundaries",
        AdtVersion::WotLK => "Wrath of the Lich King 3.x - Added advanced water",
        AdtVersion::Cataclysm => "Cataclysm 4.x - Split file architecture",
        AdtVersion::MoP => "Mists of Pandaria 5.x - Texture parameters",
    }
}

fn display_root_info(root: &wow_adt::RootAdt) {
    println!("Root ADT Contents:");
    println!("  Terrain chunks: {}", root.mcnk_chunks.len());
    println!("  Textures: {}", root.textures.len());
    println!("  M2 models: {}", root.models.len());
    println!("  WMO objects: {}", root.wmos.len());
    println!("  Doodad placements: {}", root.doodad_placements.len());
    println!("  WMO placements: {}", root.wmo_placements.len());

    // Version-specific features
    if let Some(water) = &root.water_data {
        let chunks_with_water = water
            .entries
            .iter()
            .filter(|entry| entry.header.has_liquid())
            .count();
        println!("  Water chunks: {}", chunks_with_water);
    }

    if root.flight_bounds.is_some() {
        println!("  Flight boundaries: Present");
    }

    if root.texture_flags.is_some() {
        println!("  Texture flags: Present");
    }

    if root.texture_amplifier.is_some() {
        println!("  Texture amplifier: Present");
    }

    if root.texture_params.is_some() {
        println!("  Texture parameters: Present");
    }

    // Display sample textures
    if !root.textures.is_empty() {
        println!();
        println!("Sample Textures:");
        for (i, texture) in root.textures.iter().take(5).enumerate() {
            println!("    [{}] {}", i, texture);
        }
        if root.textures.len() > 5 {
            println!("    ... and {} more", root.textures.len() - 5);
        }
    }

    // Display terrain coverage
    let active_chunks = root.mcnk_chunks.len();
    println!();
    println!("Terrain Coverage:");
    if active_chunks == 0 {
        println!("  No terrain chunks (split-format file)");
    } else if active_chunks < 256 {
        println!(
            "  Partial: {}/256 chunks ({:.1}%)",
            active_chunks,
            (active_chunks as f32 / 256.0) * 100.0
        );
    } else {
        println!("  Complete: 256/256 chunks (100%)");
    }
}

fn display_tex_info(tex: &wow_adt::Tex0Adt) {
    println!("Texture File Contents:");
    println!("  Textures: {}", tex.textures.len());
    println!("  MCNK texture chunks: {}", tex.mcnk_textures.len());

    if tex.texture_params.is_some() {
        println!("  Texture parameters: Present");
    }

    // Sample textures
    if !tex.textures.is_empty() {
        println!();
        println!("Sample Textures:");
        for (i, texture) in tex.textures.iter().take(10).enumerate() {
            println!("    [{}] {}", i, texture);
        }
        if tex.textures.len() > 10 {
            println!("    ... and {} more", tex.textures.len() - 10);
        }
    }

    // Analyze texture chunks
    let chunks_with_layers = tex
        .mcnk_textures
        .iter()
        .filter(|chunk| chunk.layers.is_some())
        .count();
    let chunks_with_alpha = tex
        .mcnk_textures
        .iter()
        .filter(|chunk| chunk.alpha_maps.is_some())
        .count();

    println!();
    println!("Texture Coverage:");
    println!("  Chunks with layers: {}", chunks_with_layers);
    println!("  Chunks with alpha maps: {}", chunks_with_alpha);
}

fn display_obj_info(obj: &wow_adt::Obj0Adt) {
    println!("Object File Contents:");
    println!("  M2 models: {}", obj.models.len());
    println!("  WMO objects: {}", obj.wmos.len());
    println!("  Doodad placements: {}", obj.doodad_placements.len());
    println!("  WMO placements: {}", obj.wmo_placements.len());
    println!("  MCNK object chunks: {}", obj.mcnk_objects.len());

    // Sample models
    if !obj.models.is_empty() {
        println!();
        println!("Sample M2 Models:");
        for (i, model) in obj.models.iter().take(5).enumerate() {
            println!("    [{}] {}", i, model);
        }
        if obj.models.len() > 5 {
            println!("    ... and {} more", obj.models.len() - 5);
        }
    }

    // Sample WMOs
    if !obj.wmos.is_empty() {
        println!();
        println!("Sample WMO Objects:");
        for (i, wmo) in obj.wmos.iter().take(5).enumerate() {
            println!("    [{}] {}", i, wmo);
        }
        if obj.wmos.len() > 5 {
            println!("    ... and {} more", obj.wmos.len() - 5);
        }
    }

    // Analyze object distribution
    let total_doodad_refs: usize = obj
        .mcnk_objects
        .iter()
        .map(|chunk| chunk.doodad_refs.len())
        .sum();
    let total_wmo_refs: usize = obj
        .mcnk_objects
        .iter()
        .map(|chunk| chunk.wmo_refs.len())
        .sum();

    println!();
    println!("Object Distribution:");
    println!("  Total doodad references: {}", total_doodad_refs);
    println!("  Total WMO references: {}", total_wmo_refs);
}
