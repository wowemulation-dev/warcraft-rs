//! Example demonstrating Phase 7 specialized chunks implementation
//!
//! This example shows how to work with the newly implemented specialized chunks:
//! - TXAC: Texture animation chunks for advanced effects
//! - PGD1: Particle geoset data for particle emitter management
//! - DBOC: Unknown purpose chunk (raw data handling)
//! - AFRA: Unknown purpose chunk (raw data handling)
//! - DPIV: Collision mesh for player housing

use std::io::Cursor;
use wow_m2::chunks::infrastructure::{ChunkHeader, ChunkReader};
use wow_m2::chunks::rendering_enhancements::*;
use wow_m2::error::Result;

fn main() -> Result<()> {
    println!("=== Phase 7 M2 Specialized Chunks Demo ===\n");

    // Demo TXAC texture animation chunk
    demo_txac_chunk()?;

    // Demo PGD1 particle geoset data
    demo_pgd1_chunk()?;

    // Demo unknown chunks (DBOC, AFRA)
    demo_unknown_chunks()?;

    // Demo DPIV collision chunk
    demo_dpiv_chunk()?;

    println!("\n=== All Phase 7 chunks demonstrated successfully! ===");
    Ok(())
}

fn demo_txac_chunk() -> Result<()> {
    println!("--- TXAC Texture Animation Chunk ---");

    // Create sample TXAC chunk data
    let mut data = Vec::new();

    // Count of texture animations
    data.extend_from_slice(&1u32.to_le_bytes());

    // Base texture animation data
    data.extend_from_slice(&1u16.to_le_bytes()); // Animation type (Scroll)
    data.extend_from_slice(&0u16.to_le_bytes()); // Padding

    // Add minimal animation block data (5 animation blocks for M2TextureAnimation)
    for _ in 0..5 {
        data.extend_from_slice(&0u16.to_le_bytes()); // Interpolation type
        data.extend_from_slice(&(-1i16).to_le_bytes()); // Global sequence
        data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps count
        data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps offset
        data.extend_from_slice(&0u32.to_le_bytes()); // Values count
        data.extend_from_slice(&0u32.to_le_bytes()); // Values offset
    }

    // Extended properties
    data.extend_from_slice(&1.0f32.to_le_bytes()); // Flow direction X
    data.extend_from_slice(&0.0f32.to_le_bytes()); // Flow direction Y
    data.extend_from_slice(&(-0.5f32).to_le_bytes()); // Flow direction Z (downward)
    data.extend_from_slice(&2.0f32.to_le_bytes()); // Speed multiplier
    data.extend_from_slice(&0.3f32.to_le_bytes()); // Turbulence factor
    data.extend_from_slice(&2u8.to_le_bytes()); // Animation mode (TurbulentFlow)
    data.extend_from_slice(&2u8.to_le_bytes()); // Loop behavior (PingPong)
    data.extend_from_slice(&1u8.to_le_bytes()); // Blend mode (Additive)
    data.extend_from_slice(&0u8.to_le_bytes()); // Padding

    let header = ChunkHeader {
        magic: *b"TXAC",
        size: data.len() as u32,
    };
    let cursor = Cursor::new(data);
    let mut chunk_reader = ChunkReader::new(cursor, header)?;

    let txac = TextureAnimationChunk::parse(&mut chunk_reader)?;

    println!("TXAC chunk parsed successfully!");
    println!(
        "  Texture animations count: {}",
        txac.texture_animations.len()
    );
    if let Some(anim) = txac.texture_animations.first() {
        println!("  First animation:");
        println!("    Base type: {:?}", anim.base_animation.animation_type);
        println!(
            "    Flow direction: {:?}",
            anim.extended_properties.flow_direction
        );
        println!(
            "    Speed multiplier: {}",
            anim.extended_properties.speed_multiplier
        );
        println!(
            "    Animation mode: {:?}",
            anim.extended_properties.animation_mode
        );
        println!(
            "    Loop behavior: {:?}",
            anim.extended_properties.loop_behavior
        );
        println!("    Blend mode: {:?}", anim.extended_properties.blend_mode);
    }

    Ok(())
}

fn demo_pgd1_chunk() -> Result<()> {
    println!("\n--- PGD1 Particle Geoset Data Chunk ---");

    // Create sample PGD1 chunk data
    let data = vec![
        0x01, 0x00, // Geoset 1 (for first particle emitter)
        0x03, 0x00, // Geoset 3 (for second particle emitter)
        0x05, 0x00, // Geoset 5 (for third particle emitter)
        0x00, 0x00, // Geoset 0 (for fourth particle emitter - default/always visible)
    ];

    let header = ChunkHeader {
        magic: *b"PGD1",
        size: data.len() as u32,
    };
    let cursor = Cursor::new(data);
    let mut chunk_reader = ChunkReader::new(cursor, header)?;

    let pgd1 = ParticleGeosetData::parse(&mut chunk_reader)?;

    println!("PGD1 chunk parsed successfully!");
    println!(
        "  Particle emitter count: {}",
        pgd1.geoset_assignments.len()
    );
    for (i, assignment) in pgd1.geoset_assignments.iter().enumerate() {
        println!("    Particle emitter {}: geoset {}", i, assignment.geoset);
    }

    println!("  This allows particle emitters to follow geoset visibility rules");
    println!("  (useful for character models with different armor pieces)");

    Ok(())
}

fn demo_unknown_chunks() -> Result<()> {
    println!("\n--- Unknown Chunks (DBOC, AFRA) ---");

    // Demo DBOC chunk (16 bytes observed in some files)
    let dboc_data = vec![
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        0x10,
    ];

    let dboc_header = ChunkHeader {
        magic: *b"DBOC",
        size: dboc_data.len() as u32,
    };
    let dboc_cursor = Cursor::new(dboc_data.clone());
    let mut dboc_reader = ChunkReader::new(dboc_cursor, dboc_header)?;

    let dboc = DbocChunk::parse(&mut dboc_reader)?;

    println!("DBOC chunk parsed successfully!");
    println!("  Raw data length: {} bytes", dboc.data.len());
    println!("  First 8 bytes: {:02X?}", &dboc.data[..8]);
    println!("  Purpose: Currently unknown, preserves raw data for future research");

    // Demo AFRA chunk (not observed in files yet, but structure ready)
    let afra_data = vec![0xFF, 0xAA, 0x55, 0x00]; // Placeholder data

    let afra_header = ChunkHeader {
        magic: *b"AFRA",
        size: afra_data.len() as u32,
    };
    let afra_cursor = Cursor::new(afra_data);
    let mut afra_reader = ChunkReader::new(afra_cursor, afra_header)?;

    let afra = AfraChunk::parse(&mut afra_reader)?;

    println!("\nAFRA chunk parsed successfully!");
    println!("  Raw data length: {} bytes", afra.data.len());
    println!("  Purpose: Unknown, added in Dragonflight but not yet observed");

    Ok(())
}

fn demo_dpiv_chunk() -> Result<()> {
    println!("\n--- DPIV Collision Mesh Chunk ---");

    // Create sample DPIV chunk data (simplified structure)
    let mut data = Vec::new();

    // Header with counts and offsets
    let vertex_count = 4u32;
    let normal_count = 2u32;
    let index_count = 6u32;
    let flags_count = 2u32;

    // Calculate offsets (after header)
    let header_size = 32u32; // 8 * 4 bytes
    let vertex_offset = header_size;
    let normal_offset = vertex_offset + (vertex_count * 12); // 3 floats per vertex
    let index_offset = normal_offset + (normal_count * 12); // 3 floats per normal
    let flags_offset = index_offset + (index_count * 2); // 2 bytes per index

    // Write header
    data.extend_from_slice(&vertex_count.to_le_bytes());
    data.extend_from_slice(&vertex_offset.to_le_bytes());
    data.extend_from_slice(&normal_count.to_le_bytes());
    data.extend_from_slice(&normal_offset.to_le_bytes());
    data.extend_from_slice(&index_count.to_le_bytes());
    data.extend_from_slice(&index_offset.to_le_bytes());
    data.extend_from_slice(&flags_count.to_le_bytes());
    data.extend_from_slice(&flags_offset.to_le_bytes());

    // Write vertex positions (a simple quad)
    let vertices = [
        [-1.0f32, 0.0, -1.0], // Bottom-left
        [1.0f32, 0.0, -1.0],  // Bottom-right
        [1.0f32, 0.0, 1.0],   // Top-right
        [-1.0f32, 0.0, 1.0],  // Top-left
    ];
    for vertex in &vertices {
        for &component in vertex {
            data.extend_from_slice(&component.to_le_bytes());
        }
    }

    // Write face normals (upward facing)
    let normals = [
        [0.0f32, 1.0, 0.0], // Up
        [0.0f32, 1.0, 0.0], // Up
    ];
    for normal in &normals {
        for &component in normal {
            data.extend_from_slice(&component.to_le_bytes());
        }
    }

    // Write indices (two triangles forming a quad)
    let indices = [0u16, 1, 2, 0, 2, 3];
    for &index in &indices {
        data.extend_from_slice(&index.to_le_bytes());
    }

    // Write flags
    let flags = [0x01u16, 0x02u16]; // Example flags
    for &flag in &flags {
        data.extend_from_slice(&flag.to_le_bytes());
    }

    let header = ChunkHeader {
        magic: *b"DPIV",
        size: data.len() as u32,
    };
    let cursor = Cursor::new(data);
    let mut chunk_reader = ChunkReader::new(cursor, header)?;

    let dpiv = DpivChunk::parse(&mut chunk_reader)?;

    println!("DPIV chunk parsed successfully!");
    println!("  Vertex positions: {}", dpiv.vertex_positions.len());
    println!("  Face normals: {}", dpiv.face_normals.len());
    println!("  Indices: {}", dpiv.indices.len());
    println!("  Flags: {}", dpiv.flags.len());

    println!("  Sample vertex: {:?}", dpiv.vertex_positions[0]);
    println!("  Sample normal: {:?}", dpiv.face_normals[0]);
    println!("  Indices: {:?}", dpiv.indices);

    println!("  Purpose: Collision mesh data for player housing furniture");
    println!("  Added in Midnight expansion for housing collision detection");

    Ok(())
}
