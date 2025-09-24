use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::io::Cursor;
// Commented out until these modules are implemented
// use wow_wmo::chunk::ChunkHeader;
// use wow_wmo::converter::WmoConverter;
// use wow_wmo::parser::chunks;
// use wow_wmo::validator::WmoValidator;
// use wow_wmo::writer::WmoWriter;
// use wow_wmo::*; // Commented until needed

#[allow(dead_code)]
fn generate_test_wmo(_size: usize) -> Vec<u8> {
    let mut buffer = Vec::new();

    // MVER chunk
    buffer.extend_from_slice(b"REVM"); // MVER reversed
    buffer.extend_from_slice(&4u32.to_le_bytes());
    buffer.extend_from_slice(&[17, 0, 0, 0]); // Version 17 (Classic)

    // MOHD chunk (header)
    buffer.extend_from_slice(b"DHOM"); // MOHD reversed
    buffer.extend_from_slice(&64u32.to_le_bytes());

    // n_materials, n_groups, etc.
    let n_materials = 10u32;
    let n_groups = 5u32;
    let n_portals = 3u32;
    let n_lights = 8u32;
    let n_doodad_names = 20u32;
    let n_doodad_defs = 15u32;
    let n_doodad_sets = 2u32;

    // Write header content
    buffer.extend_from_slice(&n_materials.to_le_bytes());
    buffer.extend_from_slice(&n_groups.to_le_bytes());
    buffer.extend_from_slice(&n_portals.to_le_bytes());
    buffer.extend_from_slice(&n_lights.to_le_bytes());
    buffer.extend_from_slice(&n_doodad_names.to_le_bytes());
    buffer.extend_from_slice(&n_doodad_defs.to_le_bytes());
    buffer.extend_from_slice(&n_doodad_sets.to_le_bytes());

    // ambient_color and flags
    buffer.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]); // White color
    buffer.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // Flags (has vertex colors)

    // Bounding box
    buffer.extend_from_slice(&(-100.0f32).to_le_bytes());
    buffer.extend_from_slice(&(-100.0f32).to_le_bytes());
    buffer.extend_from_slice(&(-100.0f32).to_le_bytes());
    buffer.extend_from_slice(&(100.0f32).to_le_bytes());
    buffer.extend_from_slice(&(100.0f32).to_le_bytes());
    buffer.extend_from_slice(&(100.0f32).to_le_bytes());

    // Some padding
    buffer.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0]);

    // Add some fake materials (MOMT chunk)
    buffer.extend_from_slice(b"TMOM"); // MOMT reversed
    buffer.extend_from_slice(&(n_materials * 40).to_le_bytes()); // 40 bytes per material

    for _ in 0..n_materials {
        // Basic material data
        let material_data = [
            0x01, 0x00, 0x00, 0x00, // flags
            0x00, 0x00, 0x00, 0x00, // shader
            0x00, 0x00, 0x00, 0x00, // blend mode
            0x00, 0x00, 0x00, 0x00, // texture1
            0xFF, 0xFF, 0xFF, 0xFF, // emissive_color
            0xFF, 0xFF, 0xFF, 0xFF, // sidn_color
            0xFF, 0xFF, 0xFF, 0xFF, // framebuffer_blend
            0x00, 0x00, 0x00, 0x00, // texture2
            0xFF, 0xFF, 0xFF, 0xFF, // diffuse_color
            0x00, 0x00, 0x00, 0x00, // ground_type
        ];
        buffer.extend_from_slice(&material_data);
    }

    // Add texture names (MOTX chunk)
    let textures = [
        "textures/stone1.blp\0",
        "textures/stone2.blp\0",
        "textures/wood1.blp\0",
        "textures/roof1.blp\0",
        "textures/wall1.blp\0",
    ];

    let motx_size = textures.iter().map(|s| s.len()).sum::<usize>();
    buffer.extend_from_slice(b"XTOM"); // MOTX reversed
    buffer.extend_from_slice(&(motx_size as u32).to_le_bytes());

    for texture in &textures {
        buffer.extend_from_slice(texture.as_bytes());
    }

    // Add group names (MOGN chunk)
    let group_names = [
        "Group_0\0",
        "Group_1\0",
        "Group_2\0",
        "Group_3\0",
        "Group_4\0",
    ];

    let mogn_size = group_names.iter().map(|s| s.len()).sum::<usize>();
    buffer.extend_from_slice(b"NGOM"); // MOGN reversed
    buffer.extend_from_slice(&(mogn_size as u32).to_le_bytes());

    for name in &group_names {
        buffer.extend_from_slice(name.as_bytes());
    }

    // Add group info (MOGI chunk)
    buffer.extend_from_slice(b"IGOM"); // MOGI reversed
    buffer.extend_from_slice(&(n_groups * 32).to_le_bytes()); // 32 bytes per group

    for i in 0..n_groups {
        // flags
        buffer.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);

        // bounding box
        buffer.extend_from_slice(&(-50.0f32).to_le_bytes());
        buffer.extend_from_slice(&(-50.0f32).to_le_bytes());
        buffer.extend_from_slice(&(-50.0f32).to_le_bytes());
        buffer.extend_from_slice(&(50.0f32).to_le_bytes());
        buffer.extend_from_slice(&(50.0f32).to_le_bytes());
        buffer.extend_from_slice(&(50.0f32).to_le_bytes());

        // name offset
        let offset = i * 8; // Simple offset calculation based on name lengths
        buffer.extend_from_slice(&offset.to_le_bytes());
    }

    // Add portal vertices (MOPV chunk)
    let n_portal_vertices = 12u32; // 4 vertices per portal
    buffer.extend_from_slice(b"VPOM"); // MOPV reversed
    buffer.extend_from_slice(&(n_portal_vertices * 12).to_le_bytes()); // 12 bytes per vertex

    for _ in 0..n_portal_vertices {
        // x, y, z
        buffer.extend_from_slice(&(0.0f32).to_le_bytes());
        buffer.extend_from_slice(&(0.0f32).to_le_bytes());
        buffer.extend_from_slice(&(0.0f32).to_le_bytes());
    }

    // Add portal info (MOPT chunk)
    buffer.extend_from_slice(b"TPOM"); // MOPT reversed
    buffer.extend_from_slice(&(n_portals * 20).to_le_bytes()); // 20 bytes per portal

    for i in 0..n_portals {
        // vertex index and count
        let vertex_index = (i * 4) as u16;
        buffer.extend_from_slice(&vertex_index.to_le_bytes());
        buffer.extend_from_slice(&4u16.to_le_bytes()); // 4 vertices per portal
        buffer.extend_from_slice(&0u16.to_le_bytes());

        // normal and plane distance
        buffer.extend_from_slice(&(1.0f32).to_le_bytes());
        buffer.extend_from_slice(&(0.0f32).to_le_bytes());
        buffer.extend_from_slice(&(0.0f32).to_le_bytes());
        buffer.extend_from_slice(&(0.0f32).to_le_bytes());
    }

    // Add portal references (MOPR chunk)
    let n_portal_refs = 6u32; // 2 refs per portal
    buffer.extend_from_slice(b"RPOM"); // MOPR reversed
    buffer.extend_from_slice(&(n_portal_refs * 8).to_le_bytes()); // 8 bytes per ref

    for i in 0..n_portal_refs {
        let portal_index = (i / 2) as u16;
        let group_index = (i % n_groups) as u16;
        let side = (i % 2) as u16;

        buffer.extend_from_slice(&portal_index.to_le_bytes());
        buffer.extend_from_slice(&group_index.to_le_bytes());
        buffer.extend_from_slice(&side.to_le_bytes());
        buffer.extend_from_slice(&0u16.to_le_bytes()); // padding
    }

    // Add lights (MOLT chunk)
    buffer.extend_from_slice(b"TLOM"); // MOLT reversed
    buffer.extend_from_slice(&(n_lights * 48).to_le_bytes()); // 48 bytes per light

    for i in 0..n_lights {
        // light type and padding
        let light_type = (i % 4) as u8; // 0-3 for different light types
        buffer.extend_from_slice(&[light_type, 0, 0, 0]);

        // use_attenuation
        buffer.extend_from_slice(&1u32.to_le_bytes());
        buffer.extend_from_slice(&0u32.to_le_bytes());
        buffer.extend_from_slice(&0u32.to_le_bytes());
        buffer.extend_from_slice(&0u32.to_le_bytes());

        // position
        let x = (i as f32 * 10.0) - 40.0;
        buffer.extend_from_slice(&x.to_le_bytes());
        buffer.extend_from_slice(&0.0f32.to_le_bytes());
        buffer.extend_from_slice(&0.0f32.to_le_bytes());

        // intensity
        buffer.extend_from_slice(&1.0f32.to_le_bytes());

        // color
        buffer.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes());

        // attenuation_start, attenuation_end
        buffer.extend_from_slice(&5.0f32.to_le_bytes());
        buffer.extend_from_slice(&100.0f32.to_le_bytes());

        // Additional fields based on light type
        buffer.extend_from_slice([0u8; 20].as_slice());
    }

    buffer
}

#[allow(dead_code)]
fn generate_test_wmo_group(_size: usize) -> Vec<u8> {
    let mut buffer = Vec::new();

    // MVER chunk
    buffer.extend_from_slice(b"REVM"); // MVER reversed
    buffer.extend_from_slice(&4u32.to_le_bytes());
    buffer.extend_from_slice(&[17, 0, 0, 0]); // Version 17 (Classic)

    // MOGP chunk (group header)
    buffer.extend_from_slice(b"PGOM"); // MOGP reversed
    buffer.extend_from_slice(&68u32.to_le_bytes()); // Header size

    // Group header fields
    buffer.extend_from_slice(&[0, 0, 0, 0]); // Name offset
    buffer.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // Flags (has normals)

    // Bounding box
    buffer.extend_from_slice(&(-50.0f32).to_le_bytes());
    buffer.extend_from_slice(&(-50.0f32).to_le_bytes());
    buffer.extend_from_slice(&(-50.0f32).to_le_bytes());
    buffer.extend_from_slice(&(50.0f32).to_le_bytes());
    buffer.extend_from_slice(&(50.0f32).to_le_bytes());
    buffer.extend_from_slice(&(50.0f32).to_le_bytes());

    // Flags2 and index
    buffer.extend_from_slice(&[0, 0]); // Flags2
    buffer.extend_from_slice(&[0, 0]); // Group index

    // Add vertices (MOVT chunk)
    let n_vertices = 100u32;
    buffer.extend_from_slice(b"TVOM"); // MOVT reversed
    buffer.extend_from_slice(&(n_vertices * 12).to_le_bytes()); // 12 bytes per vertex

    for i in 0..n_vertices {
        let x = ((i % 10) as f32 - 5.0) * 10.0;
        let y = ((i / 10) as f32 - 5.0) * 10.0;
        let z = 0.0f32;

        buffer.extend_from_slice(&x.to_le_bytes());
        buffer.extend_from_slice(&y.to_le_bytes());
        buffer.extend_from_slice(&z.to_le_bytes());
    }

    // Add normals (MONR chunk)
    buffer.extend_from_slice(b"RNOM"); // MONR reversed
    buffer.extend_from_slice(&(n_vertices * 12).to_le_bytes()); // 12 bytes per normal

    for _ in 0..n_vertices {
        buffer.extend_from_slice(&(0.0f32).to_le_bytes());
        buffer.extend_from_slice(&(0.0f32).to_le_bytes());
        buffer.extend_from_slice(&(1.0f32).to_le_bytes()); // All normals pointing up
    }

    // Add texture coordinates (MOTV chunk)
    buffer.extend_from_slice(b"VTOM"); // MOTV reversed
    buffer.extend_from_slice(&(n_vertices * 8).to_le_bytes()); // 8 bytes per tex coord

    for i in 0..n_vertices {
        let u = (i % 10) as f32 / 10.0;
        let v = (i / 10) as f32 / 10.0;

        buffer.extend_from_slice(&u.to_le_bytes());
        buffer.extend_from_slice(&v.to_le_bytes());
    }

    // Add indices (MOVI chunk)
    let n_indices = 200u32; // For triangles
    buffer.extend_from_slice(b"IVOM"); // MOVI reversed
    buffer.extend_from_slice(&(n_indices * 2).to_le_bytes()); // 2 bytes per index

    for i in 0..n_indices {
        let idx = (i % n_vertices) as u16;
        buffer.extend_from_slice(&idx.to_le_bytes());
    }

    // Add batches (MOBA chunk)
    let n_batches = 5u32;
    buffer.extend_from_slice(b"ABOM"); // MOBA reversed
    buffer.extend_from_slice(&(n_batches * 24).to_le_bytes()); // 24 bytes per batch

    for i in 0..n_batches {
        let idx_per_batch = n_indices / n_batches;
        let start_idx = i * idx_per_batch;

        buffer.extend_from_slice(&[0, 0]); // Flags + padding
        buffer.extend_from_slice(&[0, 0]); // Material ID
        buffer.extend_from_slice(&start_idx.to_le_bytes()); // Start index
        buffer.extend_from_slice(&(idx_per_batch as u16).to_le_bytes()); // Count
        buffer.extend_from_slice(&[0, 0]); // Start vertex
        buffer.extend_from_slice(&(n_vertices as u16).to_le_bytes()); // End vertex
        buffer.extend_from_slice(&[0, 0]); // Padding
        buffer.extend_from_slice(&(0.0f32).to_le_bytes()); // Position X
        buffer.extend_from_slice(&(0.0f32).to_le_bytes()); // Position Y
    }

    buffer
}

fn bench_parse_wmo(c: &mut Criterion) {
    let test_wmo = generate_test_wmo(10000);

    c.bench_function("parse_wmo_medium", |b| {
        b.iter(|| {
            let mut cursor = Cursor::new(black_box(&test_wmo));
            let _ = wow_wmo::parse_wmo(&mut cursor).unwrap();
        })
    });
}

// TODO: Reimplement after API refactoring
// fn bench_parse_wmo_group(c: &mut Criterion) {
//     let test_wmo_group = generate_test_wmo_group(10000);
//
//     c.bench_function("parse_wmo_group", |b| {
//         b.iter(|| {
//             let mut cursor = Cursor::new(black_box(&test_wmo_group));
//             let _ = wow_wmo::parse_wmo_group(&mut cursor, 0).unwrap();
//         })
//     });
// }

// fn bench_validate_wmo(c: &mut Criterion) {
//     let test_wmo = generate_test_wmo(10000);
//
//     c.bench_function("validate_wmo", |b| {
//         b.iter(|| {
//             let mut cursor = Cursor::new(black_box(&test_wmo));
//             let _ = wow_wmo::validate_wmo(&mut cursor).unwrap();
//         })
//     });
// }

// TODO: Fix after API refactoring - needs to extract Root from ParsedWmo
// fn bench_validate_wmo_detailed(c: &mut Criterion) {
//     let test_wmo = generate_test_wmo(10000);
//
//     c.bench_function("validate_wmo_detailed", |b| {
//         b.iter(|| {
//             let mut cursor = Cursor::new(black_box(&test_wmo));
//             let wmo = wow_wmo::parse_wmo(&mut cursor).unwrap();
//             let validator = WmoValidator::new();
//             let _ = validator.validate_root(&wmo).unwrap();
//         })
//     });
// }

// fn bench_convert_wmo(c: &mut Criterion) {
//     let test_wmo = generate_test_wmo(10000);
//
//     c.bench_function("convert_wmo", |b| {
//         b.iter(|| {
//             let mut cursor = Cursor::new(black_box(&test_wmo));
//             let mut wmo = wow_wmo::parse_wmo(&mut cursor).unwrap();
//             let converter = WmoConverter::new();
//             converter.convert_root(&mut wmo, WmoVersion::Tbc).unwrap();
//         })
//     });
// }

// fn bench_write_wmo(c: &mut Criterion) {
//     let test_wmo = generate_test_wmo(10000);
//
//     c.bench_function("write_wmo", |b| {
//         b.iter(|| {
//             let mut in_cursor = Cursor::new(black_box(&test_wmo));
//             let wmo = wow_wmo::parse_wmo(&mut in_cursor).unwrap();
//             let mut out_buffer = Vec::new();
//             let mut cursor = Cursor::new(&mut out_buffer);
//             let writer = WmoWriter::new();
//             writer.write_root(&mut cursor, &wmo, wmo.version).unwrap();
//         })
//     });
// }

criterion_group!(
    benches,
    bench_parse_wmo // TODO: Re-enable after fixing API usage
                    // bench_parse_wmo_group,
                    // bench_validate_wmo,
                    // bench_validate_wmo_detailed,
                    // bench_convert_wmo,
                    // bench_write_wmo
);
criterion_main!(benches);
