//! Benchmarks for the ADT parser

use criterion::{Criterion, criterion_group, criterion_main};
use std::io::Cursor;
use wow_adt::{AdtBuilder, AdtVersion, ChunkHeader, create_flat_terrain};

fn bench_create_terrain(c: &mut Criterion) {
    c.bench_function("create_flat_terrain_wotlk", |b| {
        b.iter(|| create_flat_terrain(AdtVersion::WotLK, 100.0))
    });

    c.bench_function("create_flat_terrain_cata", |b| {
        b.iter(|| create_flat_terrain(AdtVersion::Cataclysm, 150.0))
    });
}

fn bench_builder(c: &mut Criterion) {
    c.bench_function("adt_builder_create", |b| {
        b.iter(|| AdtBuilder::new(AdtVersion::WotLK).build())
    });
}

fn bench_chunk_header(c: &mut Criterion) {
    // Benchmark chunk header parsing
    c.bench_function("parse_chunk_header", |b| {
        let data = vec![0x52, 0x45, 0x56, 0x4D, 0x04, 0x00, 0x00, 0x00];
        b.iter(|| {
            let mut cursor = Cursor::new(&data);
            ChunkHeader::read(&mut cursor).unwrap()
        })
    });
}

criterion_group!(
    benches,
    bench_create_terrain,
    bench_builder,
    bench_chunk_header
);
criterion_main!(benches);
