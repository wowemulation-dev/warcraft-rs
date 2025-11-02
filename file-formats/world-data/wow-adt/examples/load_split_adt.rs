//! Load and merge Cataclysm+ split ADT files.
//!
//! This example demonstrates loading complete ADT split file sets (root, texture, object, LOD)
//! using the high-level AdtSet API, and merging them into a unified structure.
//!
//! # Usage
//!
//! ```bash
//! # Load split files from a Cataclysm+ ADT
//! cargo run --example load_split_adt -- /path/to/World/Maps/Azeroth/Azeroth_30_30.adt
//!
//! # Show detailed information
//! cargo run --example load_split_adt -- /path/to/adt_file.adt --verbose
//! ```

use std::env;
use std::path::PathBuf;
use wow_adt::AdtSet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <adt_file> [--verbose]", args[0]);
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} World/Maps/Azeroth/Azeroth_30_30.adt", args[0]);
        eprintln!("  {} Azeroth_32_48.adt --verbose", args[0]);
        eprintln!();
        eprintln!("Note: For Cataclysm+ ADTs, provide the root file path.");
        eprintln!("      The example will automatically discover _tex0, _obj0, and _lod files.");
        return Ok(());
    }

    let adt_path = PathBuf::from(&args[1]);
    let verbose = args.contains(&"--verbose".to_string());

    println!("Loading ADT file: {}", adt_path.display());
    println!();

    // Load complete split file set
    let adt_set = AdtSet::load_from_path(&adt_path)?;

    // Display version and completeness
    println!("ADT Version: {:?}", adt_set.version());
    println!("Complete split file set: {}", adt_set.is_complete());
    println!();

    // Display which files were loaded
    println!("Loaded files:");
    println!("  ✓ Root ADT (terrain geometry)");

    if adt_set.texture.is_some() {
        println!("  ✓ Texture file (_tex0.adt)");
    } else {
        println!("  ✗ Texture file not found");
    }

    if adt_set.object.is_some() {
        println!("  ✓ Object file (_obj0.adt)");
    } else {
        println!("  ✗ Object file not found");
    }

    if adt_set.lod.is_some() {
        println!("  ✓ LOD file (_lod.adt)");
    } else {
        println!("  ✗ LOD file not found (optional)");
    }

    println!();

    // Display root file information
    println!("Root ADT Information:");
    println!("  MCNK chunks: {}", adt_set.root.mcnk_chunks.len());

    if let Some(water) = &adt_set.root.water_data {
        let chunks_with_water = water
            .entries
            .iter()
            .filter(|e| e.header.has_liquid())
            .count();
        println!("  Water chunks: {}", chunks_with_water);
    }

    if let Some(_flight_bounds) = &adt_set.root.flight_bounds {
        println!("  Flight bounds: present");
    }

    // Display texture file information
    if let Some(tex) = &adt_set.texture {
        println!();
        println!("Texture File Information:");
        println!("  Textures: {}", tex.textures.len());
        println!("  MCNK texture chunks: {}", tex.mcnk_textures.len());

        if verbose && !tex.textures.is_empty() {
            println!();
            println!("  Texture list:");
            for (idx, texture) in tex.textures.iter().take(10).enumerate() {
                println!("    {}: {}", idx, texture);
            }
            if tex.textures.len() > 10 {
                println!("    ... and {} more", tex.textures.len() - 10);
            }
        }
    }

    // Display object file information
    if let Some(obj) = &adt_set.object {
        println!();
        println!("Object File Information:");
        println!("  M2 models: {}", obj.models.len());
        println!("  WMO objects: {}", obj.wmos.len());
        println!("  Doodad placements: {}", obj.doodad_placements.len());
        println!("  WMO placements: {}", obj.wmo_placements.len());
        println!("  MCNK object chunks: {}", obj.mcnk_objects.len());

        if verbose && !obj.models.is_empty() {
            println!();
            println!("  M2 model list:");
            for (idx, model) in obj.models.iter().take(10).enumerate() {
                println!("    {}: {}", idx, model);
            }
            if obj.models.len() > 10 {
                println!("    ... and {} more", obj.models.len() - 10);
            }
        }
    }

    // Display LOD file information
    if let Some(_lod) = &adt_set.lod {
        println!();
        println!("LOD File Information:");
        println!("  LOD data present (Legion+)");
    }

    // Merge split files into unified structure
    println!();
    println!("Merging split files...");
    let merged = adt_set.merge()?;

    println!();
    println!("Merged ADT Information:");
    println!("  MCNK chunks: {}", merged.mcnk_chunks.len());
    println!("  Textures: {}", merged.textures.len());
    println!("  M2 models: {}", merged.models.len());
    println!("  WMO objects: {}", merged.wmos.len());
    println!("  Doodad placements: {}", merged.doodad_placements.len());
    println!("  WMO placements: {}", merged.wmo_placements.len());

    if verbose {
        println!();
        println!("MCNK Chunk Details:");
        let chunks_with_layers = merged
            .mcnk_chunks
            .iter()
            .filter(|c| c.layers.is_some())
            .count();
        let chunks_with_alpha = merged
            .mcnk_chunks
            .iter()
            .filter(|c| c.alpha.is_some())
            .count();
        let chunks_with_heights = merged
            .mcnk_chunks
            .iter()
            .filter(|c| c.heights.is_some())
            .count();
        let chunks_with_normals = merged
            .mcnk_chunks
            .iter()
            .filter(|c| c.normals.is_some())
            .count();

        println!("  Chunks with texture layers: {}", chunks_with_layers);
        println!("  Chunks with alpha maps: {}", chunks_with_alpha);
        println!("  Chunks with height data: {}", chunks_with_heights);
        println!("  Chunks with normals: {}", chunks_with_normals);
    }

    println!();
    println!("✓ Successfully loaded and merged split ADT files");

    Ok(())
}
