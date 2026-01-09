//! Example: Comprehensive API demonstration for accessing M2 model data
//!
//! This example shows how users can access all the various data types in an M2 model,
//! including animations, bones, skins, vertices, and chunked format data.
//!
//! Usage: cargo run --example access_model_data -- <path_to_m2_file>

use std::io::Cursor;
use std::path::Path;
use wow_m2::{AnimFile, AnimMetadata, M2Model, PathResolver, SkinFile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_m2_file>", args[0]);
        eprintln!("Example: {} Character/Human/Male/HumanMale.m2", args[0]);
        std::process::exit(1);
    }

    let m2_path = &args[1];

    // Load the M2 model
    println!("Loading M2 model from: {}", m2_path);
    let m2_format = M2Model::load(m2_path)?;
    let model = m2_format.model();

    // ========================================
    // 1. ACCESS BASIC MODEL DATA
    // ========================================
    println!("\n=== BASIC MODEL DATA ACCESS ===");

    // Model metadata
    println!("Model name: {:?}", model.name);
    println!("Version: {}", model.header.version);
    println!("Global flags: {:#x}", model.header.flags);

    // Direct field access - all fields are public!
    println!("\n--- Direct Data Access ---");
    println!("Vertices: {} items", model.vertices.len());
    println!("Bones: {} items", model.bones.len());
    println!("Animations: {} items", model.animations.len());
    println!("Textures: {} items", model.textures.len());
    println!("Materials: {} items", model.materials.len());

    // ========================================
    // 2. ACCESS VERTEX DATA
    // ========================================
    println!("\n=== VERTEX DATA ACCESS ===");

    if let Some(first_vertex) = model.vertices.first() {
        println!("First vertex:");
        println!(
            "  Position: ({:.2}, {:.2}, {:.2})",
            first_vertex.position.x, first_vertex.position.y, first_vertex.position.z
        );
        println!(
            "  Normal: ({:.2}, {:.2}, {:.2})",
            first_vertex.normal.x, first_vertex.normal.y, first_vertex.normal.z
        );
        println!(
            "  UV coords: ({:.2}, {:.2})",
            first_vertex.tex_coords.x, first_vertex.tex_coords.y
        );
        println!("  Bone weights: {:?}", first_vertex.bone_weights);
        println!("  Bone indices: {:?}", first_vertex.bone_indices);
    }

    // ========================================
    // 3. ACCESS BONE DATA
    // ========================================
    println!("\n=== BONE DATA ACCESS ===");

    for (i, bone) in model.bones.iter().take(3).enumerate() {
        println!("Bone {}:", i);
        println!("  Bone ID: {}", bone.bone_id);
        println!("  Parent bone: {}", bone.parent_bone);
        println!("  Flags: {:#x}", bone.flags);
        println!(
            "  Pivot: ({:.2}, {:.2}, {:.2})",
            bone.pivot.x, bone.pivot.y, bone.pivot.z
        );
    }

    // Access bone lookup table
    println!(
        "\nBone lookup table: {} entries",
        model.key_bone_lookup.len()
    );
    if !model.key_bone_lookup.is_empty() {
        println!(
            "  First 5 lookups: {:?}",
            &model.key_bone_lookup[..5.min(model.key_bone_lookup.len())]
        );
    }

    // ========================================
    // 4. ACCESS ANIMATION DATA
    // ========================================
    println!("\n=== ANIMATION DATA ACCESS ===");

    for (i, anim) in model.animations.iter().take(3).enumerate() {
        println!("Animation {}:", i);
        println!("  Animation ID: {}", anim.animation_id);
        println!("  Sub-animation ID: {}", anim.sub_animation_id);
        let duration = anim
            .end_timestamp
            .unwrap_or(0)
            .saturating_sub(anim.start_timestamp);
        println!("  Duration: {}ms", duration);
        println!("  Move speed: {}", anim.movement_speed);
        println!("  Flags: {:#x}", anim.flags);
        println!("  Frequency: {}", anim.frequency);
        if let Some(replay) = anim.replay {
            println!("  Replay: {}-{}", replay.minimum, replay.maximum);
        }
    }

    // Animation lookup table
    println!(
        "\nAnimation lookup table: {} entries",
        model.animation_lookup.len()
    );

    // ========================================
    // 5. ACCESS CHUNKED FORMAT DATA (Legion+)
    // ========================================
    println!("\n=== CHUNKED FORMAT DATA ACCESS ===");

    if m2_format.is_chunked() {
        println!("Model uses chunked format (MD21)");

        // Skin file IDs
        if let Some(skin_ids) = &model.skin_file_ids {
            println!("\nSkin file IDs: {:?}", skin_ids.ids);
        }

        // Animation file IDs
        if let Some(anim_ids) = &model.animation_file_ids {
            println!("Animation file IDs: {:?}", anim_ids.ids);
        }

        // Texture file IDs
        if let Some(texture_ids) = &model.texture_file_ids {
            println!("Texture file IDs: {:?}", texture_ids.ids);
        }

        // Physics file ID
        if let Some(physics_id) = &model.physics_file_id {
            println!("Physics file ID: {}", physics_id.id);
        }

        // Bone file IDs
        if let Some(bone_ids) = &model.bone_file_ids {
            println!("Bone file IDs: {:?}", bone_ids.ids);
        }

        // Particle geoset data
        if let Some(_particle_geoset) = &model.particle_geoset_data {
            println!("\nParticle geoset data available");
        }

        // Particle emitters (parsed)
        if !model.particle_emitters.is_empty() {
            println!("Particle emitters: {}", model.particle_emitters.len());
            for (i, emitter) in model.particle_emitters.iter().enumerate() {
                println!(
                    "  Emitter {}: type={:?}, bone={}, flags={:?}",
                    i, emitter.emitter_type, emitter.bone_index, emitter.flags
                );
            }
        }

        // Texture animations
        if let Some(_texture_anims) = &model.texture_animation_chunk {
            println!("Texture animation chunks available");
        }
    } else {
        println!("Model uses legacy format (MD20)");
    }

    // ========================================
    // 6. LOAD EXTERNAL FILES (Skin/Anim)
    // ========================================
    println!("\n=== LOADING EXTERNAL FILES ===");

    // Create a file resolver (for resolving relative paths)
    let base_dir = Path::new(m2_path).parent().unwrap_or(Path::new("."));
    let resolver = PathResolver::new(base_dir);

    // Try to load skin files
    let skin_count = model.skin_file_count();
    println!("\nModel has {} skin profiles", skin_count);

    if skin_count > 0 {
        // Load first skin file
        match model.load_skin_file(0, &resolver) {
            Ok(skin_data) => {
                println!("Successfully loaded skin 0 ({} bytes)", skin_data.len());

                // Parse the skin file
                let mut cursor = Cursor::new(&skin_data);
                if let Ok(_skin) = SkinFile::parse(&mut cursor) {
                    println!("Skin 0 statistics:");
                    println!("Skin file parsed successfully");
                    // Access skin data through the parsed structure
                }
            }
            Err(e) => {
                println!("Could not load skin 0: {}", e);
            }
        }
    }

    // Try to load animation files
    let anim_count = model.animation_file_count();
    println!("\nModel has {} external animation files", anim_count);

    if anim_count > 0 {
        // Load first animation file
        match model.load_animation_file(0, &resolver) {
            Ok(anim_data) => {
                println!(
                    "Successfully loaded animation 0 ({} bytes)",
                    anim_data.len()
                );

                // Parse the animation file
                let mut cursor = Cursor::new(&anim_data);
                if let Ok(anim_file) = AnimFile::parse(&mut cursor) {
                    println!("Animation file format: {:?}", anim_file.format);
                    match &anim_file.metadata {
                        AnimMetadata::Legacy {
                            animation_count, ..
                        } => {
                            println!("  Animation count: {}", animation_count);
                        }
                        AnimMetadata::Modern { header, .. } => {
                            println!("  Animation count: {}", header.id_count);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Could not load animation 0: {}", e);
            }
        }
    }

    // ========================================
    // 7. ACCESS RAW DATA (for custom parsing)
    // ========================================
    println!("\n=== RAW DATA ACCESS ===");

    // The model preserves raw data for sections we don't fully parse
    // Raw data is stored in individual fields rather than a single buffer
    println!("Raw data preserved for various sections");

    // You can access specific offsets if you know the format
    // For example, to read custom data at a specific offset:
    // let custom_data = &model.raw_data.data[offset..offset+size];

    // ========================================
    // 8. UTILITY METHODS
    // ========================================
    println!("\n=== UTILITY METHODS ===");

    // Get specific file IDs
    if let Some(skin_ids) = model.get_skin_file_ids() {
        println!(
            "Direct skin ID access: {:?}",
            &skin_ids[..2.min(skin_ids.len())]
        );
    }

    if let Some(anim_ids) = model.get_animation_file_ids() {
        println!(
            "Direct animation ID access: {:?}",
            &anim_ids[..2.min(anim_ids.len())]
        );
    }

    if let Some(bone_ids) = model.get_bone_file_ids() {
        println!(
            "Direct bone ID access: {:?}",
            &bone_ids[..2.min(bone_ids.len())]
        );
    }

    // Check animation blacklisting
    if model.is_animation_blacklisted(0) {
        println!("Animation 0 is blacklisted");
    }

    println!("\n✅ All model data successfully accessed!");

    Ok(())
}
