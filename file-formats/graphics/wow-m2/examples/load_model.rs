//! Example: Loading and inspecting an M2 model
//!
//! This example demonstrates how to load an M2 model file and print basic information about it.
//!
//! Usage: cargo run --example load_model -- <path_to_m2_file>

use wow_m2::M2Model;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    todo!()
    // // Get the file path from command line arguments
    // let args: Vec<String> = std::env::args().collect();
    // if args.len() < 2 {
    //     eprintln!("Usage: {} <path_to_m2_file>", args[0]);
    //     std::process::exit(1);
    // }
    //
    // let path = &args[1];
    //
    // // Load the model
    // println!("Loading model from: {path}");
    // let model = M2Model::load(path)?;
    //
    // // Print basic information
    // println!("\n=== Model Information ===");
    // println!("Name: {:?}", model.name);
    // println!("Version: {:?}", model.header.version());
    // println!("Magic: {:?}", std::str::from_utf8(&model.header.magic)?);
    //
    // // Print counts
    // println!("\n=== Model Statistics ===");
    // println!("Vertices: {}", model.vertices.len());
    // println!("Bones: {}", model.bones.len());
    // println!("Textures: {}", model.textures.len());
    // println!("Materials: {}", model.materials.len());
    // println!("Animations: {}", model.animations.len());
    // println!("Animation Lookups: {}", model.animation_lookup.len());
    //
    // // Print texture information
    // if !model.textures.is_empty() {
    //     println!("\n=== Textures ===");
    //     for (i, texture) in model.textures.iter().enumerate() {
    //         println!(
    //             "Texture {}: Type={:?}, Flags={:?}",
    //             i, texture.texture_type, texture.flags
    //         );
    //     }
    // }
    //
    // // Print animations
    // if !model.animations.is_empty() {
    //     println!("\n=== Animations ===");
    //     for (i, anim) in model.animations.iter().take(5).enumerate() {
    //         println!(
    //             "Animation {}: ID={}, SubID={}, Flags={:#x}",
    //             i, anim.animation_id, anim.sub_animation_id, anim.flags
    //         );
    //     }
    //     if model.animations.len() > 5 {
    //         println!("... and {} more animations", model.animations.len() - 5);
    //     }
    // }
    //
    // // Check model bounds
    // println!("\n=== Model Bounds ===");
    // println!("Bounding Box Min: {:?}", model.header.bounding_box_min);
    // println!("Bounding Box Max: {:?}", model.header.bounding_box_max);
    // println!(
    //     "Bounding Sphere Radius: {}",
    //     model.header.bounding_sphere_radius
    // );
    //
    // Ok(())
}
