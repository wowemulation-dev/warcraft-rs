//! Benchmarks for ADT full parsing (T114).
//!
//! Tests the performance of complete two-pass ADT parsing including
//! chunk discovery and content extraction.
//!
//! Performance targets:
//! - Full parse: <50ms per file
//! - Batch parse: 100 ADT files in <5 seconds
//! - Throughput: >20 MB/s

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::io::Cursor;
use wow_adt::AdtVersion;
use wow_adt::api::parse_adt;
use wow_adt::builder::AdtBuilder;
use wow_adt::chunks::mcnk::mcvt::McvtChunk;
use wow_adt::chunks::mcnk::{McnkChunk, McnkFlags, McnkHeader};

/// Generate a minimal ADT for parsing benchmarks.
fn create_test_adt(version: AdtVersion, num_mcnk: usize) -> Vec<u8> {
    let mut builder = AdtBuilder::new()
        .with_version(version)
        .add_texture("terrain/grass.blp")
        .add_texture("terrain/dirt.blp")
        .add_texture("terrain/rock.blp");

    // Add MCNK chunks with heights
    for i in 0..num_mcnk.min(256) {
        let chunk = McnkChunk {
            header: McnkHeader {
                flags: McnkFlags { value: 0 },
                index_x: (i % 16) as u32,
                index_y: (i / 16) as u32,
                n_layers: 1,
                n_doodad_refs: 0,
                multipurpose_field: McnkHeader::multipurpose_from_offsets(0, 0),
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
                unknown_8bytes: [0; 8],
                ofs_snd_emitters: 0,
                n_snd_emitters: 0,
                ofs_liquid: 0,
                size_liquid: 0,
                position: [0.0, 0.0, 0.0],
                ofs_mccv: 0,
                ofs_mclv: 0,
                unused: 0,
                _padding: [0; 8],
            },
            heights: Some(McvtChunk {
                heights: vec![(i as f32) * 10.0; 145],
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

/// Benchmark full ADT parsing for different WoW versions.
fn bench_parse_versions(c: &mut Criterion) {
    let versions = vec![
        ("vanilla", AdtVersion::VanillaEarly),
        ("tbc", AdtVersion::TBC),
        ("wotlk", AdtVersion::WotLK),
        ("cataclysm", AdtVersion::Cataclysm),
        ("mop", AdtVersion::MoP),
    ];

    let mut group = c.benchmark_group("parse_versions");

    for (name, version) in versions {
        let adt_data = create_test_adt(version, 1);
        group.throughput(Throughput::Bytes(adt_data.len() as u64));

        group.bench_with_input(BenchmarkId::from_parameter(name), &adt_data, |b, data| {
            b.iter(|| {
                let mut cursor = Cursor::new(data);
                parse_adt(&mut cursor).expect("Parse failed")
            });
        });
    }

    group.finish();
}

/// Benchmark parsing with varying MCNK chunk counts.
fn bench_parse_scaling(c: &mut Criterion) {
    let chunk_counts = vec![1, 10, 50, 100, 200];

    let mut group = c.benchmark_group("parse_scaling");

    for count in chunk_counts {
        let adt_data = create_test_adt(AdtVersion::WotLK, count);
        group.throughput(Throughput::Bytes(adt_data.len() as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_mcnk", count)),
            &adt_data,
            |b, data| {
                b.iter(|| {
                    let mut cursor = Cursor::new(data);
                    parse_adt(&mut cursor).expect("Parse failed")
                });
            },
        );
    }

    group.finish();
}

/// Benchmark batch parsing (simulates 100 ADT files).
fn bench_batch_parse(c: &mut Criterion) {
    // Create 100 small ADT files
    let adt_files: Vec<Vec<u8>> = (0..100)
        .map(|i| {
            let version = match i % 5 {
                0 => AdtVersion::VanillaEarly,
                1 => AdtVersion::TBC,
                2 => AdtVersion::WotLK,
                3 => AdtVersion::Cataclysm,
                _ => AdtVersion::MoP,
            };
            create_test_adt(version, 1)
        })
        .collect();

    let total_bytes: usize = adt_files.iter().map(|f| f.len()).sum();

    let mut group = c.benchmark_group("batch_parse");
    group.throughput(Throughput::Bytes(total_bytes as u64));
    group.sample_size(10); // Reduce sample size for batch operations

    group.bench_function("100_files", |b| {
        b.iter(|| {
            for data in &adt_files {
                let mut cursor = Cursor::new(data);
                let _ = parse_adt(&mut cursor).expect("Parse failed");
            }
        });
    });

    group.finish();
}

/// Benchmark parsing throughput (MB/s).
fn bench_parse_throughput(c: &mut Criterion) {
    let adt_data = create_test_adt(AdtVersion::WotLK, 100);
    let size_mb = adt_data.len() as f64 / (1024.0 * 1024.0);

    let mut group = c.benchmark_group("parse_throughput");
    group.throughput(Throughput::Bytes(adt_data.len() as u64));

    group.bench_function(format!("wotlk_{:.2}mb", size_mb), |b| {
        b.iter(|| {
            let mut cursor = Cursor::new(&adt_data);
            parse_adt(&mut cursor).expect("Parse failed")
        });
    });

    group.finish();
}

/// Benchmark parsing with round-trip (parse → build → serialize → parse).
fn bench_round_trip(c: &mut Criterion) {
    let adt_data = create_test_adt(AdtVersion::WotLK, 10);

    c.bench_function("round_trip_10_mcnk", |b| {
        b.iter(|| {
            // Parse
            let mut cursor = Cursor::new(&adt_data);
            let parsed = parse_adt(&mut cursor).expect("Parse failed");

            // Convert to builder
            let root = match parsed {
                wow_adt::api::ParsedAdt::Root(r) => r,
                _ => panic!("Expected root ADT"),
            };

            // Build
            let built = AdtBuilder::from_parsed(*root)
                .build()
                .expect("Build failed");

            // Serialize
            let bytes = built.to_bytes().expect("Serialize failed");

            // Parse again
            let mut cursor2 = Cursor::new(bytes);
            parse_adt(&mut cursor2).expect("Second parse failed")
        });
    });
}

criterion_group!(
    benches,
    bench_parse_versions,
    bench_parse_scaling,
    bench_batch_parse,
    bench_parse_throughput,
    bench_round_trip
);
criterion_main!(benches);
