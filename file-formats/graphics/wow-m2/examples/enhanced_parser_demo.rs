//! Enhanced M2 Parser Demo
//!
//! This example demonstrates how to use the enhanced M2 parser to extract
//! comprehensive model data including vertices, bones, animations, textures,
//! and embedded skin data for vanilla models.

use std::env;
use std::fs;
use std::io::Cursor;

use wow_m2::{M2Error, parse_m2};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_m2_file>", args[0]);
        eprintln!();
        eprintln!("Example:");
        eprintln!("  {} /path/to/HumanMale.m2", args[0]);
        return Ok(());
    }

    let m2_path = &args[1];
    println!("Loading M2 model: {}", m2_path);

    // Load the M2 file
    let m2_data = fs::read(m2_path).map_err(|e| {
        M2Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Could not read M2 file '{}': {}", m2_path, e),
        ))
    })?;

    // Parse the M2 model
    let m2_format = parse_m2(&mut Cursor::new(&m2_data))?;
    let model = m2_format.model();

    println!("Successfully parsed M2 model!");
    println!();

    // Extract all model data using the enhanced parser
    println!("Extracting comprehensive model data...");
    let enhanced_data = model.parse_all_data(&m2_data)?;
    println!("Data extraction complete!");
    println!();

    // Display comprehensive model information
    model.display_info(&enhanced_data);

    // Additional detailed information
    if !enhanced_data.bones.is_empty() {
        println!("=== Detailed Bone Information ===");
        let root_bones: Vec<_> = enhanced_data
            .bones
            .iter()
            .enumerate()
            .filter(|(_, bone_info)| bone_info.parent_index < 0)
            .collect();

        println!("Root bones: {}", root_bones.len());
        for (index, bone_info) in root_bones {
            println!(
                "  Root bone {}: {} (children: {})",
                index,
                bone_info
                    .name
                    .as_deref()
                    .unwrap_or(&format!("Bone_{}", index)),
                bone_info.children.len()
            );
        }
        println!();
    }

    if !enhanced_data.animations.is_empty() {
        println!("=== Animation Details ===");
        for (i, anim_info) in enhanced_data.animations.iter().take(10).enumerate() {
            println!(
                "  Animation {}: {} (ID: {}, Duration: {}ms, Loop: {})",
                i,
                anim_info.name,
                anim_info.animation.animation_id,
                anim_info.duration_ms,
                anim_info.is_looping
            );
        }
        if enhanced_data.animations.len() > 10 {
            println!(
                "  ... and {} more animations",
                enhanced_data.animations.len() - 10
            );
        }
        println!();
    }

    if !enhanced_data.textures.is_empty() {
        println!("=== Texture Details ===");
        for (i, tex_info) in enhanced_data.textures.iter().take(5).enumerate() {
            println!(
                "  Texture {}: {} (Type: {})",
                i,
                tex_info.filename.as_deref().unwrap_or("<unresolved>"),
                tex_info.texture_type
            );
        }
        if enhanced_data.textures.len() > 5 {
            println!(
                "  ... and {} more textures",
                enhanced_data.textures.len() - 5
            );
        }
        println!();
    }

    if !enhanced_data.embedded_skins.is_empty() {
        println!("=== Embedded Skin Analysis ===");
        for (i, skin) in enhanced_data.embedded_skins.iter().enumerate() {
            println!(
                "  Skin {}: {} indices, {} triangles",
                i,
                skin.indices().len(),
                skin.triangles().len()
            );

            for (j, submesh) in skin.submeshes().iter().enumerate() {
                println!(
                    "    Submesh {}: {} vertices, {} triangles",
                    j, submesh.vertex_count, submesh.triangle_count
                );
            }
        }
        println!();
    }

    // Model complexity analysis
    println!("=== Model Complexity Analysis ===");
    let stats = &enhanced_data.stats;
    println!(
        "Vertex density: {:.2} vertices per bone",
        if stats.bone_count > 0 {
            stats.vertex_count as f32 / stats.bone_count as f32
        } else {
            0.0
        }
    );
    println!(
        "Triangle density: {:.2} triangles per vertex",
        if stats.vertex_count > 0 {
            stats.triangle_count as f32 / stats.vertex_count as f32
        } else {
            0.0
        }
    );
    println!(
        "Animation richness: {:.2} animations per bone",
        if stats.bone_count > 0 {
            stats.animation_count as f32 / stats.bone_count as f32
        } else {
            0.0
        }
    );
    println!(
        "Texture coverage: {:.2} textures per submesh",
        if stats.embedded_skin_count > 0 {
            let total_submeshes: usize = enhanced_data
                .embedded_skins
                .iter()
                .map(|skin| skin.submeshes().len())
                .sum();
            if total_submeshes > 0 {
                stats.texture_count as f32 / total_submeshes as f32
            } else {
                0.0
            }
        } else {
            0.0
        }
    );

    println!();
    println!("Enhanced parsing complete! The model has been fully analyzed.");

    Ok(())
}
