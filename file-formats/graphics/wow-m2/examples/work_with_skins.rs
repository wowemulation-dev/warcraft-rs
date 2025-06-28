//! Example: Working with M2 skin files
//!
//! This example shows how to load and inspect skin files (.skin) associated with M2 models.
//! Skin files contain mesh and LOD (Level of Detail) information.
//!
//! Usage: cargo run --example work_with_skins -- <path_to_skin_file>

use std::path::Path;
use wow_m2::Skin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_skin_file>", args[0]);
        eprintln!("Example: {} model00.skin", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];

    // Check if file exists
    if !Path::new(path).exists() {
        eprintln!("Error: File not found: {path}");
        std::process::exit(1);
    }

    // Load the skin file
    println!("Loading skin file from: {path}");
    let skin = Skin::load(path)?;

    // Print header information
    println!("\n=== Skin Header ===");
    println!("Magic: {:?}", std::str::from_utf8(&skin.header.magic)?);
    println!("Version: {}", skin.header.version);

    // Print statistics
    println!("\n=== Skin Statistics ===");
    println!("Vertices: {}", skin.header.vertex_count);
    println!("Indices: {}", skin.header.indices.count);
    println!("Submeshes: {}", skin.submeshes.len());

    // Print submesh information
    if !skin.submeshes.is_empty() {
        println!("\n=== Submeshes ===");
        for (i, submesh) in skin.submeshes.iter().enumerate() {
            println!("\nSubmesh {i}:");
            println!("  ID: {}", submesh.id);
            println!("  Level: {}", submesh.level);
            println!("  Start Triangle: {}", submesh.triangle_start);
            println!("  Triangle Count: {}", submesh.triangle_count);
            println!("  Start Vertex: {}", submesh.vertex_start);
            println!("  Vertex Count: {}", submesh.vertex_count);
            println!("  Bone Count: {}", submesh.bone_count);
            println!("  Bone Start: {}", submesh.bone_start);

            // Calculate approximate triangle count
            let approx_triangles = submesh.triangle_count / 3;
            println!("  Approximate Triangles: {approx_triangles}");
        }
    }

    // Analyze LOD distribution
    println!("\n=== LOD Analysis ===");

    // Determine LOD level based on submesh levels
    let max_lod_level = skin.submeshes.iter().map(|s| s.level).max().unwrap_or(0);

    println!("Maximum LOD level in submeshes: {max_lod_level}");

    let total_triangles: u16 = skin.submeshes.iter().map(|s| s.triangle_count / 3).sum();
    println!("Total triangles (approx): {total_triangles}");

    // Provide recommendations based on LOD level
    match max_lod_level {
        0 => println!("This skin contains highest detail (LOD 0) submeshes"),
        1 => println!("This skin contains medium detail (LOD 1) submeshes"),
        2 => println!("This skin contains low detail (LOD 2) submeshes"),
        3 => println!("This skin contains very low detail (LOD 3) submeshes"),
        _ => println!(
            "This skin contains ultra-low detail (LOD {max_lod_level}) submeshes"
        ),
    }

    // Check for potential issues
    println!("\n=== Validation ===");
    let mut issues = Vec::new();

    if skin.header.vertex_count == 0 {
        issues.push("No vertices defined".to_string());
    }

    if skin.header.indices.count == 0 {
        issues.push("No indices defined".to_string());
    }

    if skin.submeshes.is_empty() {
        issues.push("No submeshes defined".to_string());
    }

    for (i, submesh) in skin.submeshes.iter().enumerate() {
        if submesh.triangle_count == 0 {
            issues.push(format!("Submesh {i} has no triangles"));
        }
        if submesh.vertex_count == 0 {
            issues.push(format!("Submesh {i} has no vertices"));
        }
    }

    if issues.is_empty() {
        println!("✅ Skin file validation passed!");
    } else {
        println!("❌ Found {} issue(s):", issues.len());
        for issue in &issues {
            println!("  - {issue}");
        }
    }

    Ok(())
}
