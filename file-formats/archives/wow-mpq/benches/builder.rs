//! ArchiveBuilder benchmarks

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use wow_mpq::ArchiveBuilder;

/// Test struct to access the private encrypt_data method
/// (Alternatively, you could make encrypt_data pub(crate) for testing)
struct TestBuilder {
    builder: ArchiveBuilder,
}

impl TestBuilder {
    fn new() -> Self {
        Self {
            builder: ArchiveBuilder::new(),
        }
    }

    /// Wrapper to test the encrypt_data method
    fn encrypt_data(&self, data: &mut [u8], key: u32) {
        // This would call the actual encrypt_data method
        // You might need to make it pub(crate) or create a test module
        self.builder.encrypt_data(data, key);
    }
}

fn bench_encrypt_data(c: &mut Criterion) {
    let mut group = c.benchmark_group("encrypt_data");

    // Test different data sizes
    let sizes = vec![
        ("64B", 64),
        ("512B", 512),
        ("4KB", 4 * 1024),
        ("64KB", 64 * 1024),
        ("1MB", 1024 * 1024),
        ("16MB", 16 * 1024 * 1024),
    ];

    let test_builder = TestBuilder::new();
    let key = 0xDEADBEEF;

    for (name, size) in sizes {
        group.bench_with_input(BenchmarkId::from_parameter(name), &size, |b, &size| {
            // Create data outside the benchmark loop
            let data = vec![0xAB; size];

            b.iter(|| {
                // Clone the data for each iteration to ensure consistent state
                let mut work_data = data.clone();
                test_builder.encrypt_data(black_box(&mut work_data), black_box(key));
                black_box(work_data); // Prevent optimization
            });
        });
    }

    group.finish();
}

fn bench_encrypt_data_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("encrypt_data_patterns");

    let test_builder = TestBuilder::new();
    let key = 0xDEADBEEF;
    let size = 64 * 1024; // 64KB for all pattern tests

    // Different data patterns
    let patterns = vec![
        ("zeros", vec![0u8; size]),
        ("ones", vec![0xFF; size]),
        ("random", {
            let mut data = vec![0u8; size];
            for (i, byte) in data.iter_mut().enumerate() {
                *byte = (i * 7 + 13) as u8; // Pseudo-random pattern
            }
            data
        }),
        (
            "text",
            "The quick brown fox jumps over the lazy dog. "
                .repeat(size / 45)
                .into_bytes(),
        ),
    ];

    for (name, pattern) in patterns {
        group.bench_with_input(BenchmarkId::from_parameter(name), &pattern, |b, pattern| {
            b.iter(|| {
                let mut work_data = pattern.clone();
                test_builder.encrypt_data(black_box(&mut work_data), black_box(key));
                black_box(work_data);
            });
        });
    }

    group.finish();
}

fn bench_encrypt_data_aligned_vs_unaligned(c: &mut Criterion) {
    let mut group = c.benchmark_group("encrypt_data_alignment");

    let test_builder = TestBuilder::new();
    let key = 0xDEADBEEF;

    // Test perfectly aligned vs unaligned data
    let test_cases = vec![
        ("aligned_4KB", 4096),
        ("unaligned_4KB-1", 4095),
        ("unaligned_4KB+1", 4097),
        ("unaligned_4KB+2", 4098),
        ("unaligned_4KB+3", 4099),
    ];

    for (name, size) in test_cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), &size, |b, &size| {
            let data = vec![0xAB; size];

            b.iter(|| {
                let mut work_data = data.clone();
                test_builder.encrypt_data(black_box(&mut work_data), black_box(key));
                black_box(work_data);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_encrypt_data,
    bench_encrypt_data_patterns,
    bench_encrypt_data_aligned_vs_unaligned
);
criterion_main!(benches);
