//! Benchmarks for ADT chunk discovery phase (T113).
//!
//! Tests the performance of the two-pass parser's first phase:
//! chunk enumeration without content parsing.
//!
//! Performance targets:
//! - Discovery phase: <10ms for 5-15 MB files
//! - Memory footprint: <1MB for discovery metadata

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::io::Cursor;
use wow_adt::builder::AdtBuilder;
use wow_adt::chunk_discovery::discover_chunks;
use wow_adt::AdtVersion;

/// Generate a minimal ADT file for benchmarking.
///
/// Creates a Vanilla ADT with:
/// - 1 texture
/// - 1 MCNK chunk with heights
/// - MVER, MHDR, MCIN, MTEX chunks
fn create_minimal_adt(version: AdtVersion) -> Vec<u8> {
    AdtBuilder::new()
        .with_version(version)
        .add_texture("terrain/grass.blp")
        .build()
        .expect("Failed to build test ADT")
        .to_bytes()
        .expect("Failed to serialize test ADT")
}

/// Generate an ADT file with multiple MCNK chunks.
fn create_multi_mcnk_adt(num_chunks: usize, version: AdtVersion) -> Vec<u8> {
    let mut builder = AdtBuilder::new()
        .with_version(version)
        .add_texture("terrain/grass.blp")
        .add_texture("terrain/dirt.blp");

    // Add multiple MCNK chunks (limited by validation)
    for _ in 0..num_chunks.min(256) {
        use wow_adt::chunks::mcnk::{McnkChunk, McnkHeader, McnkFlags};
        use wow_adt::chunks::mcnk::mcvt::McvtChunk;

        let chunk = McnkChunk {
            header: McnkHeader {
                flags: McnkFlags { value: 0 },
                index_x: 0,
                index_y: 0,
                n_layers: 1,
                n_doodad_refs: 0,
                holes_high_res: 0,
                ofs_height: 0,
                ofs_normal: 0,
                ofs_layer: 0,
                ofs_refs: 0,
                ofs_alpha: 0,
                size_alpha: 0,
                ofs_shadow: 0,
                size_shadow: 0,
                area_id: 0,
                n_map_obj_refs: 0,
                holes_low_res: 0,
                unknown_but_used: 0,
                pred_tex: [0; 8],
                no_effect_doodad: [0; 8],
                ofs_snd_emitters: 0,
                n_snd_emitters: 0,
                ofs_liquid: 0,
                size_liquid: 0,
                position: [0.0, 0.0, 0.0],
                ofs_mccv: 0,
                ofs_mclv: 0,
                unused: 0,
            },
            heights: Some(McvtChunk {
                heights: vec![100.0; 145],
            }),
            normals: None,
            layers: None,
            materials: None,
            refs: None,
            doodad_refs: None,
            wmo_refs: None,
            alpha: None,
            shadow: None,
            vertex_colors: None,
            vertex_lighting: None,
            sound_emitters: None,
            liquid: None,
            doodad_disable: None,
            blend_batches: None,
        };

        builder = builder.add_mcnk_chunk(chunk);
    }

    builder
        .build()
        .expect("Failed to build test ADT")
        .to_bytes()
        .expect("Failed to serialize test ADT")
}

/// Benchmark chunk discovery on minimal ADT.
fn bench_discovery_minimal(c: &mut Criterion) {
    let adt_data = create_minimal_adt(AdtVersion::VanillaEarly);

    let mut group = c.benchmark_group("discovery_minimal");
    group.throughput(Throughput::Bytes(adt_data.len() as u64));

    group.bench_function("vanilla", |b| {
        b.iter(|| {
            let mut cursor = Cursor::new(&adt_data);
            discover_chunks(&mut cursor).expect("Discovery failed")
        });
    });

    group.finish();
}

/// Benchmark chunk discovery across WoW versions.
fn bench_discovery_versions(c: &mut Criterion) {
    let versions = vec![
        ("vanilla", AdtVersion::VanillaEarly),
        ("tbc", AdtVersion::TBC),
        ("wotlk", AdtVersion::WotLK),
        ("cataclysm", AdtVersion::Cataclysm),
        ("mop", AdtVersion::MoP),
    ];

    let mut group = c.benchmark_group("discovery_versions");

    for (name, version) in versions {
        let adt_data = create_minimal_adt(version);
        group.throughput(Throughput::Bytes(adt_data.len() as u64));

        group.bench_with_input(BenchmarkId::from_parameter(name), &adt_data, |b, data| {
            b.iter(|| {
                let mut cursor = Cursor::new(data);
                discover_chunks(&mut cursor).expect("Discovery failed")
            });
        });
    }

    group.finish();
}

/// Benchmark discovery with varying MCNK chunk counts.
fn bench_discovery_scaling(c: &mut Criterion) {
    let chunk_counts = vec![1, 10, 50, 100];

    let mut group = c.benchmark_group("discovery_scaling");

    for count in chunk_counts {
        let adt_data = create_multi_mcnk_adt(count, AdtVersion::WotLK);
        group.throughput(Throughput::Bytes(adt_data.len() as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_mcnk", count)),
            &adt_data,
            |b, data| {
                b.iter(|| {
                    let mut cursor = Cursor::new(data);
                    discover_chunks(&mut cursor).expect("Discovery failed")
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory allocation during discovery.
fn bench_discovery_memory(c: &mut Criterion) {
    let adt_data = create_multi_mcnk_adt(100, AdtVersion::WotLK);

    c.bench_function("discovery_100_mcnk", |b| {
        b.iter(|| {
            let mut cursor = Cursor::new(&adt_data);
            let discovery = discover_chunks(&mut cursor).expect("Discovery failed");
            // Force use of discovery to prevent optimization
            std::hint::black_box(discovery.chunks.len())
        });
    });
}

criterion_group!(
    benches,
    bench_discovery_minimal,
    bench_discovery_versions,
    bench_discovery_scaling,
    bench_discovery_memory
);
criterion_main!(benches);
