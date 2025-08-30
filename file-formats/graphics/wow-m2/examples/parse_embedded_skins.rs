//! Example: Parsing embedded skin data from pre-WotLK M2 models
//!
//! This example demonstrates how to extract and parse skin profile data that is
//! embedded directly within pre-WotLK (versions 256-260) M2 model files.
//!
//! Usage: cargo run --example parse_embedded_skins -- <path_to_pre_wotlk_m2_file>

use std::fs;
use std::io::Cursor;
use std::path::Path;
use wow_m2::parse_m2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_pre_wotlk_m2_file>", args[0]);
        eprintln!("Example: {} Character/Human/Male/HumanMale.m2", args[0]);
        eprintln!("\nNote: This example requires a pre-WotLK M2 file (version 256-260)");
        std::process::exit(1);
    }

    let m2_path = &args[1];

    // Check if file exists
    if !Path::new(m2_path).exists() {
        eprintln!("Error: File not found: {}", m2_path);
        std::process::exit(1);
    }

    // Read the complete M2 file data
    println!("Loading M2 file: {}", m2_path);
    let m2_data = fs::read(m2_path)?;
    println!("Loaded {} bytes", m2_data.len());

    // Parse the M2 model
    let mut cursor = Cursor::new(&m2_data);
    let m2_format = parse_m2(&mut cursor)?;
    let model = m2_format.model();

    // Display basic model information
    println!("\n=== Model Information ===");
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

    // Check if this model has embedded skins
    if model.has_embedded_skins() {
        println!("\n=== Embedded Skin Profiles ===");
        println!("This is a pre-WotLK model with embedded skin data!");

        let skin_count = model.embedded_skin_count().unwrap();
        println!("Number of embedded skin profiles: {}", skin_count);

        // Parse each embedded skin profile
        for i in 0..skin_count {
            println!("\n--- Skin Profile {} ---", i);

            match model.parse_embedded_skin(&m2_data, i as usize) {
                Ok(skin) => {
                    // Display skin statistics
                    println!("Successfully parsed embedded skin {}", i);

                    let indices = skin.indices();
                    let triangles = skin.triangles();
                    let submeshes = skin.submeshes();

                    println!("  Indices: {}", indices.len());
                    println!("  Triangles: {}", triangles.len());
                    println!("  Submeshes: {}", submeshes.len());

                    // Verify triangle indices contain actual mesh data
                    if indices.len() >= 12 {
                        println!("\n  Triangle Indices (first 12): {:?}", &indices[..12]);

                        // Check if they're sequential (0,1,2,3...) which would indicate bad data
                        let sequential = (0..12).map(|i| i as u16).collect::<Vec<_>>();
                        if indices[..12] == sequential {
                            println!(
                                "  ❌ ERROR: Indices are sequential - geometry will be broken!"
                            );
                            println!(
                                "     This indicates the embedded skin data is not being parsed correctly."
                            );
                        } else {
                            println!("  ✅ Indices contain proper triangle data (non-sequential)");

                            // Analyze the triangle pattern
                            let mut unique_verts = std::collections::HashSet::new();
                            for idx in &indices[..indices.len().min(30)] {
                                unique_verts.insert(*idx);
                            }
                            println!(
                                "     First 30 indices reference {} unique vertices",
                                unique_verts.len()
                            );

                            // Check for reasonable vertex index range
                            let max_idx = indices.iter().max().copied().unwrap_or(0);
                            let min_idx = indices.iter().min().copied().unwrap_or(0);
                            println!(
                                "     Index range: {} to {} (model has {} vertices)",
                                min_idx,
                                max_idx,
                                model.vertices.len()
                            );

                            if max_idx as usize >= model.vertices.len() {
                                println!("  ⚠️  WARNING: Some indices exceed vertex count!");
                            }
                        }
                    }

                    // Check triangles array (vertex references)
                    if triangles.len() >= 12 {
                        println!(
                            "\n  Triangles/Vertex refs (first 12): {:?}",
                            &triangles[..12]
                        );
                    }

                    // Display submesh details with triangle validation
                    if !submeshes.is_empty() {
                        println!("\n  Submesh Details:");
                        for (j, submesh) in submeshes.iter().enumerate().take(3) {
                            println!("    Submesh {}:", j);
                            println!("      ID: {}", submesh.id);
                            println!("      Level: {} (LOD)", submesh.level);
                            println!("      Vertex Start: {}", submesh.vertex_start);
                            println!("      Vertex Count: {}", submesh.vertex_count);
                            println!("      Triangle Start: {}", submesh.triangle_start);
                            println!(
                                "      Triangle Count: {} ({} triangles)",
                                submesh.triangle_count,
                                submesh.triangle_count / 3
                            );

                            // Sample triangle indices from this submesh
                            let start = submesh.triangle_start as usize;
                            let end = (start + 9).min(indices.len());
                            if start < indices.len() && end > start {
                                println!("      Sample indices: {:?}", &indices[start..end]);
                            }
                        }

                        if submeshes.len() > 3 {
                            println!("    ... and {} more submeshes", submeshes.len() - 3);
                        }
                    }

                    // Calculate triangle statistics (use u32 to avoid overflow)
                    let total_triangles: u32 =
                        submeshes.iter().map(|s| s.triangle_count as u32 / 3).sum();
                    println!(
                        "\n  Total triangles across all submeshes: {}",
                        total_triangles
                    );

                    // Analyze LOD distribution
                    let mut lod_counts = [0u32; 4];
                    for submesh in submeshes {
                        if submesh.level < 4 {
                            lod_counts[submesh.level as usize] += 1;
                        }
                    }

                    println!("\n  LOD Distribution:");
                    for (level, count) in lod_counts.iter().enumerate() {
                        if *count > 0 {
                            println!("    LOD {}: {} submeshes", level, count);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse embedded skin {}: {}", i, e);
                }
            }
        }

        // Try to parse all skins at once
        println!("\n=== Batch Processing ===");
        match model.parse_all_embedded_skins(&m2_data) {
            Ok(all_skins) => {
                println!(
                    "Successfully parsed all {} embedded skins at once",
                    all_skins.len()
                );

                // Calculate total statistics
                let total_indices: usize = all_skins.iter().map(|s| s.indices().len()).sum();
                let total_submeshes: usize = all_skins.iter().map(|s| s.submeshes().len()).sum();

                println!("Total indices across all skins: {}", total_indices);
                println!("Total submeshes across all skins: {}", total_submeshes);
            }
            Err(e) => {
                eprintln!("Failed to parse all embedded skins: {}", e);
            }
        }
    } else if model.header.version > 260 {
        println!("\n=== External Skin Files ===");
        println!(
            "This is a WotLK+ model (version {}) that uses external .skin files",
            model.header.version
        );
        println!("Embedded skin parsing is not applicable for this model.");

        // Show information about external skins instead
        if let Some(skin_ids) = &model.skin_file_ids {
            println!("\nExternal skin file IDs: {:?}", skin_ids.ids);
        } else {
            println!(
                "\nThis model uses {} external skin profiles",
                model.header.num_skin_profiles.unwrap_or(0)
            );
            println!("Skin files would be named:");
            let base_name = Path::new(m2_path)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();
            for i in 0..model.header.num_skin_profiles.unwrap_or(0) {
                println!("  {}{:02}.skin", base_name, i);
            }
        }
    } else {
        println!("\n⚠️ This model has no skin profiles defined");
    }

    // Additional model statistics relevant to skins
    println!("\n=== Related Model Data ===");
    println!(
        "Vertices: {} (shared by all skin profiles)",
        model.vertices.len()
    );
    println!("Bones: {} (for vertex weighting)", model.bones.len());
    println!("Materials: {} (for rendering)", model.materials.len());
    println!("Textures: {} (for material mapping)", model.textures.len());

    // Show bone data that affects skinning
    if !model.bones.is_empty() {
        println!("\n=== Bone Information (First 5) ===");
        for (i, bone) in model.bones.iter().enumerate().take(5) {
            println!(
                "Bone {}: ID={}, Parent={}, Flags={:#x}",
                i, bone.bone_id, bone.parent_bone, bone.flags
            );
        }
    }

    Ok(())
}
