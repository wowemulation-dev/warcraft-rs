//! Benchmarks for the BLP parser

use criterion::{Criterion, criterion_group, criterion_main};
use image::{ImageBuffer, Rgba};
use std::hint::black_box;

fn create_test_image(size: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    ImageBuffer::from_fn(size, size, |x, y| {
        let r = ((x * 255) / size) as u8;
        let g = ((y * 255) / size) as u8;
        let b = (((x + y) * 255) / (size * 2)) as u8;
        let a = 255;
        Rgba([r, g, b, a])
    })
}

fn bench_image_creation(c: &mut Criterion) {
    c.bench_function("create_test_image_256", |b| {
        b.iter(|| create_test_image(256))
    });

    c.bench_function("create_test_image_512", |b| {
        b.iter(|| create_test_image(512))
    });

    c.bench_function("create_test_image_1024", |b| {
        b.iter(|| create_test_image(1024))
    });
}

fn bench_image_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("image_sizes");

    for size in [64, 128, 256, 512].iter() {
        group.bench_function(format!("size_{size}"), |b| {
            b.iter(|| {
                let img = create_test_image(*size);
                black_box(img.as_raw().len())
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_image_creation, bench_image_sizes);
criterion_main!(benches);
