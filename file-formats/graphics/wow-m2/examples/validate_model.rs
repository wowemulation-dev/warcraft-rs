//! Example: Validating M2 model files
//!
//! This example demonstrates how to validate an M2 model file for correctness and integrity.
//!
//! Usage: cargo run --example validate_model -- <path_to_m2_file>

use std::path::Path;
use wow_m2::M2Model;

fn validate_model(model: &M2Model) -> Vec<String> {
    let mut issues = Vec::new();

    // Check header magic
    if &model.header.magic != b"MD20" {
        issues.push(format!("Invalid magic: {:?}", model.header.magic));
    }

    // Check version
    let version = model.header.version();
    if version.is_none() {
        issues.push(format!("Unknown version: {}", model.header.version));
    }

    // Validate bounds
    let bbox_min = &model.header.bounding_box_min;
    let bbox_max = &model.header.bounding_box_max;
    if bbox_min[0] > bbox_max[0] || bbox_min[1] > bbox_max[1] || bbox_min[2] > bbox_max[2] {
        issues.push("Invalid bounding box: min > max".to_string());
    }

    if model.header.bounding_sphere_radius <= 0.0 {
        issues.push("Invalid bounding sphere radius".to_string());
    }

    // Check vertices
    if model.vertices.is_empty() {
        issues.push("No vertices found".to_string());
    } else {
        for (i, vertex) in model.vertices.iter().enumerate() {
            // Check bone weights sum to 1.0 (with some tolerance)
            let weight_sum: u8 = vertex.bone_weights.iter().sum();
            if weight_sum != 255 && weight_sum != 0 {
                issues.push(format!(
                    "Vertex {i}: bone weights don't sum to 1.0 (sum={weight_sum})"
                ));
            }

            // Check bone indices are valid
            for (j, &bone_idx) in vertex.bone_indices.iter().enumerate() {
                if vertex.bone_weights[j] > 0 && bone_idx as usize >= model.bones.len() {
                    issues.push(format!("Vertex {i}: invalid bone index {bone_idx}"));
                }
            }
        }
    }

    // Check textures
    for (i, texture) in model.textures.iter().enumerate() {
        if texture.filename.is_empty() {
            issues.push(format!("Texture {i}: empty filename"));
        }
    }

    // Check materials have valid flags and blend modes
    for (i, material) in model.materials.iter().enumerate() {
        // Materials don't directly reference textures in this model format
        // The texture-material mapping is handled differently
        if material.flags.is_empty() && material.blend_mode.is_empty() {
            issues.push(format!("Material {i}: has no flags or blend mode set"));
        }
    }

    // Check animations
    // for (i, anim) in model.animations.iter().enumerate() {
    //     // For Classic format, check if end_timestamp > start_timestamp
    //     if let Some(end_ts) = anim.end_timestamp {
    //         if end_ts <= anim.start_timestamp {
    //             issues.push(format!("Animation {i}: end timestamp <= start timestamp"));
    //         }
    //     } else {
    //         // For BC+ format, start_timestamp contains duration
    //         if anim.start_timestamp == 0 {
    //             issues.push(format!("Animation {i}: zero duration"));
    //         }
    //     }
    // }

    // Check bone hierarchy
    for (i, bone) in model.bones.iter().enumerate() {
        if bone.parent_bone != -1 {
            if bone.parent_bone < 0 || bone.parent_bone as usize >= model.bones.len() {
                issues.push(format!(
                    "Bone {}: invalid parent bone index {}",
                    i, bone.parent_bone
                ));
            } else if bone.parent_bone as usize == i {
                issues.push(format!("Bone {i}: references itself as parent"));
            }
        }
    }

    issues
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_m2_file>", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];

    // Check if file exists
    if !Path::new(path).exists() {
        eprintln!("Error: File not found: {path}");
        std::process::exit(1);
    }

    // Load the model
    println!("Loading model from: {path}");
    let model = match M2Model::load(path) {
        Ok(model) => model,
        Err(e) => {
            eprintln!("Failed to load model: {e}");
            std::process::exit(1);
        }
    };

    println!("Model loaded successfully!");
    println!("Version: {:?}", model.header.version());

    // Validate the model
    println!("\nValidating model...");
    let issues = validate_model(&model);

    if issues.is_empty() {
        println!("✅ Model validation passed! No issues found.");
    } else {
        println!("❌ Model validation found {} issue(s):", issues.len());
        for issue in &issues {
            println!("  - {issue}");
        }
    }

    // Print statistics even if validation fails
    println!("\n=== Model Statistics ===");
    println!("Vertices: {}", model.vertices.len());
    println!("Bones: {}", model.bones.len());
    println!("Textures: {}", model.textures.len());
    println!("Materials: {}", model.materials.len());
    println!("Animations: {}", model.animations.len());
    println!("Animation Lookups: {}", model.animation_lookup.len());

    if !issues.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}
