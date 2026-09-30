//! Example demonstrating P2 Extended File Dependencies
//!
//! This example shows how to work with PFID, SKID/BFID, and LDV1 chunks
//! for physics files, skeleton/bone files, and LOD data.

use std::io::Cursor;
use wow_m2::parse_m2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("M2 Extended File Dependencies Example");
    println!("=====================================");

    // Create a mock chunked M2 file with the new chunks
    let data = create_mock_chunked_m2()?;
    let mut cursor = Cursor::new(data);

    // Parse the M2 file
    let m2_format = parse_m2(&mut cursor)?;
    let model = m2_format.model();

    println!(
        "Model format: {}",
        if m2_format.is_chunked() {
            "Chunked (MD21)"
        } else {
            "Legacy (MD20)"
        }
    );
    println!("Has external files: {}", model.has_external_files());
    println!("Physics file ID: {:?}", model.get_physics_file_id());
    println!("Skeleton file ID: {:?}", model.get_skeleton_file_id());
    println!("Bone file count: {}", model.bone_file_count());
    println!("Has LOD data: {}", model.has_lod_data());

    // Demonstrate physics file support (PFID)
    if let Some(physics_id) = model.get_physics_file_id() {
        println!("\nPhysics File Support:");
        println!("  Physics file ID: {}", physics_id);

        // In a real application, you would load the physics file using a FileResolver:
        // let physics_data = model.load_physics(&resolver)?;
        println!("  Would load physics data from .phys file");
    }

    // Demonstrate skeleton file support (SKID)
    if let Some(skeleton_id) = model.get_skeleton_file_id() {
        println!("\nSkeleton File Support:");
        println!("  Skeleton file ID: {}", skeleton_id);
        println!("  Would load skeleton hierarchy from .skel file");
    }

    // Demonstrate bone file support (BFID)
    if let Some(bone_ids) = model.get_bone_file_ids() {
        println!("\nBone File Support:");
        println!("  Number of bone files: {}", bone_ids.len());
        for (index, &bone_id) in bone_ids.iter().enumerate() {
            println!("    Bone file {}: ID {}", index, bone_id);
        }
        println!("  Would load bone animation data from .bone files");
    }

    // Demonstrate LOD support (LDV1)
    if let Some(lod_data) = model.get_lod_data() {
        println!("\nLevel of Detail Support:");
        println!("  Number of LOD levels: {}", lod_data.lod_count);
        println!("  Max LOD index: {:?}", lod_data.max_lod());
        println!("  Particle bone LOD: {:?}", lod_data.particle_bone_lod);
    }

    println!("\nNote: This example demonstrates the parsing of extended file reference chunks.");
    println!("In a real application, you would use a FileResolver to load the actual");
    println!("physics, skeleton, and bone files using their FileDataIDs.");

    Ok(())
}

fn create_mock_chunked_m2() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut data = Vec::new();

    // MD21 file starts with MD21 magic
    data.extend_from_slice(b"MD21"); // File magic
    data.extend_from_slice(&8u32.to_le_bytes()); // File chunk size placeholder

    // MD21 chunk with minimal M2 data
    data.extend_from_slice(b"MD21"); // Chunk magic
    data.extend_from_slice(&400u32.to_le_bytes()); // Chunk size

    // MD20 data within the chunk (simplified)
    data.extend_from_slice(b"MD20"); // Legacy magic within chunk
    data.extend_from_slice(&276u32.to_le_bytes()); // Legion version
    // Add minimal header data (zeros for simplicity)
    for _ in 0..98 {
        // Simplified header padding
        data.extend_from_slice(&0u32.to_le_bytes());
    }

    // PFID chunk - Physics File Reference
    data.extend_from_slice(b"PFID");
    data.extend_from_slice(&4u32.to_le_bytes()); // Chunk size: 4 bytes
    data.extend_from_slice(&123456u32.to_le_bytes()); // Physics file ID

    // SKID chunk - Skeleton File Reference
    data.extend_from_slice(b"SKID");
    data.extend_from_slice(&4u32.to_le_bytes()); // Chunk size: 4 bytes
    data.extend_from_slice(&789012u32.to_le_bytes()); // Skeleton file ID

    // BFID chunk - Bone File References
    data.extend_from_slice(b"BFID");
    data.extend_from_slice(&12u32.to_le_bytes()); // Chunk size: 12 bytes (3 IDs)
    data.extend_from_slice(&111111u32.to_le_bytes()); // Bone file ID 1
    data.extend_from_slice(&111112u32.to_le_bytes()); // Bone file ID 2
    data.extend_from_slice(&111113u32.to_le_bytes()); // Bone file ID 3

    // LDV1 chunk - Level of Detail
    data.extend_from_slice(b"LDV1");
    data.extend_from_slice(&16u32.to_le_bytes()); // Chunk size: 16 bytes (single record)
    data.extend_from_slice(&0u16.to_le_bytes()); // unknown_0
    data.extend_from_slice(&2u16.to_le_bytes()); // lod_count
    data.extend_from_slice(&10.0f32.to_le_bytes()); // unknown_2
    data.extend_from_slice(&[0, 1, 1, 1]); // particle_bone_lod
    data.extend_from_slice(&0u32.to_le_bytes()); // unknown_4

    Ok(data)
}
