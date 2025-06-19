//! Benchmarks for the WDT parser

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn bench_coordinate_conversion(c: &mut Criterion) {
    c.bench_function("world_to_tile_coords", |b| {
        b.iter(|| {
            let world_x = 17066.0;
            let world_y = 17066.0;
            let tile_x = ((32.0 - (world_x / 533.333_3)) as i32).clamp(0, 63);
            let tile_y = ((32.0 - (world_y / 533.333_3)) as i32).clamp(0, 63);
            black_box((tile_x, tile_y))
        })
    });

    c.bench_function("tile_to_world_coords", |b| {
        b.iter(|| {
            let tile_x = 32;
            let tile_y = 32;
            let world_x = (32.0 - tile_x as f32) * 533.333_3;
            let world_y = (32.0 - tile_y as f32) * 533.333_3;
            black_box((world_x, world_y))
        })
    });
}

fn bench_tile_calculations(c: &mut Criterion) {
    c.bench_function("tile_index_calculation", |b| {
        b.iter(|| {
            let x = 32;
            let y = 32;
            let index = y * 64 + x;
            black_box(index)
        })
    });
}

criterion_group!(
    benches,
    bench_coordinate_conversion,
    bench_tile_calculations
);
criterion_main!(benches);
