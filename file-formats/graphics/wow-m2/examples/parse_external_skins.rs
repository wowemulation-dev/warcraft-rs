//! Example: Parsing external skin files from WotLK+ M2 models
//!
//! This example demonstrates how to load and parse external .skin files
//! that are used by WotLK+ (version 264+) M2 models, and how to properly
//! resolve the two-level indirection used in these files.
//!
//! Usage: cargo run --example parse_external_skins -- <path_to_wotlk_m2_file> <path_to_skin_file>

use std::fs;
use std::io::Cursor;
use std::path::Path;
use wow_m2::{SkinFile, parse_m2};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "Usage: {} <path_to_wotlk_m2_file> <path_to_skin_file>",
            args[0]
        );
        eprintln!("Example: {} HumanMale.m2 HumanMale00.skin", args[0]);
        eprintln!(
            "\nNote: This example requires a WotLK+ M2 file (version 264+) and its corresponding .skin file"
        );
        std::process::exit(1);
    }

    let m2_path = &args[1];
    let skin_path = &args[2];

    // Check if files exist
    if !Path::new(m2_path).exists() {
        eprintln!("Error: M2 file not found: {}", m2_path);
        std::process::exit(1);
    }
    if !Path::new(skin_path).exists() {
        eprintln!("Error: Skin file not found: {}", skin_path);
        std::process::exit(1);
    }

    // Load the M2 model
    println!("Loading M2 file: {}", m2_path);
    let m2_data = fs::read(m2_path)?;
    let mut cursor = Cursor::new(&m2_data);
    let m2_format = parse_m2(&mut cursor)?;
    let model = m2_format.model();

    // Display model information
    println!("\n=== M2 Model Information ===");
    println!("Model name: {:?}", model.name);
    println!("Version: {}", model.header.version);
    println!(
        "Format: {}",
        if m2_format.is_chunked() {
            "Chunked (MD21)"
        } else {
            "Legacy (MD20)"
        }
    );
    println!("Vertices: {}", model.vertices.len());
    println!("Bones: {}", model.bones.len());

    // Check if this model uses external skins
    if model.header.version < 264 {
        println!(
            "\n⚠️ This is a pre-WotLK model (version {}) that uses embedded skins.",
            model.header.version
        );
        println!("This example is for WotLK+ models with external .skin files.");
        std::process::exit(1);
    }

    println!("\n=== External Skin File ===");
    println!("This is a WotLK+ model that uses external .skin files");

    // Load the external skin file
    println!("\nLoading skin file: {}", skin_path);
    let skin_file = SkinFile::load(skin_path)?;

    // Get raw arrays
    let raw_indices = skin_file.indices();
    let raw_triangles = skin_file.triangles();
    let submeshes = skin_file.submeshes();

    println!("\n=== Raw Skin Data ===");
    println!(
        "Raw indices array size: {} (vertex lookup table)",
        raw_indices.len()
    );
    println!(
        "Raw triangles array size: {} (indices into lookup table)",
        raw_triangles.len()
    );
    println!("Submeshes: {}", submeshes.len());

    // Show first few values from raw arrays
    if raw_indices.len() >= 12 {
        println!("\nFirst 12 values in indices array (lookup table):");
        println!("  {:?}", &raw_indices[..12]);
    }

    if raw_triangles.len() >= 12 {
        println!("\nFirst 12 values in triangles array (lookup indices):");
        println!("  {:?}", &raw_triangles[..12]);
    }

    // Get resolved indices (applying two-level indirection)
    println!("\n=== Resolved Vertex Indices ===");
    let resolved_indices = skin_file.get_resolved_indices();
    println!(
        "Resolved indices count: {} (should equal triangles count)",
        resolved_indices.len()
    );

    if resolved_indices.len() >= 12 {
        println!("\nFirst 12 resolved vertex indices:");
        println!("  {:?}", &resolved_indices[..12]);

        // Demonstrate the resolution process
        println!("\n=== Index Resolution Process ===");
        println!("For external skins, we apply two-level indirection:");
        for (i, &tri_value) in raw_triangles.iter().enumerate().take(6) {
            let final_index = if (tri_value as usize) < raw_indices.len() {
                raw_indices[tri_value as usize]
            } else {
                0
            };
            println!(
                "  triangles[{}] = {} → indices[{}] = {} (final vertex index)",
                i, tri_value, tri_value, final_index
            );
        }

        // Verify indices are valid
        let max_index = resolved_indices.iter().max().copied().unwrap_or(0);
        let min_index = resolved_indices.iter().min().copied().unwrap_or(0);
        println!("\n=== Validation ===");
        println!("Resolved index range: {} to {}", min_index, max_index);
        println!("Model vertex count: {}", model.vertices.len());

        if max_index as usize >= model.vertices.len() {
            println!("⚠️ WARNING: Some resolved indices exceed vertex count!");
        } else {
            println!("✅ All resolved indices are within valid vertex range");
        }

        // Check if indices are sequential (which would indicate a problem)
        let sequential = (0..12).map(|i| i as u16).collect::<Vec<_>>();
        if resolved_indices.len() >= 12 && resolved_indices[..12] == sequential {
            println!("\n❌ ERROR: Resolved indices are sequential (0,1,2,3...)");
            println!("   This indicates the indirection is not working correctly!");
        } else if resolved_indices.len() >= 12 {
            println!("\n✅ Resolved indices contain proper vertex data (non-sequential)");

            // Count unique vertices referenced
            let mut unique_verts = std::collections::HashSet::new();
            for &idx in &resolved_indices[..resolved_indices.len().min(100)] {
                unique_verts.insert(idx);
            }
            println!(
                "   First 100 indices reference {} unique vertices",
                unique_verts.len()
            );
        }
    }

    // Display submesh information
    if !submeshes.is_empty() {
        println!("\n=== Submesh Information ===");
        for (i, submesh) in submeshes.iter().enumerate().take(3) {
            println!("\nSubmesh {}:", i);
            println!("  ID: {}", submesh.id);
            println!("  Level: {} (LOD)", submesh.level);
            println!("  Vertex Start: {}", submesh.vertex_start);
            println!("  Vertex Count: {}", submesh.vertex_count);
            println!("  Triangle Start: {}", submesh.triangle_start);
            println!(
                "  Triangle Count: {} ({} triangles)",
                submesh.triangle_count,
                submesh.triangle_count / 3
            );

            // Show resolved indices for this submesh
            let start = submesh.triangle_start as usize;
            let end = (start + 9).min(resolved_indices.len());
            if start < resolved_indices.len() && end > start {
                println!(
                    "  Sample resolved indices: {:?}",
                    &resolved_indices[start..end]
                );
            }
        }
    }

    Ok(())
}
