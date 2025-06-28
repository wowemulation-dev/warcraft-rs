//! Compression benchmarks

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use wow_mpq::compression::{compress, decompress, flags};

fn create_test_data(size: usize, pattern: &str) -> Vec<u8> {
    let mut data = Vec::with_capacity(size);
    let pattern_bytes = pattern.as_bytes();

    while data.len() < size {
        let remaining = size - data.len();
        let to_copy = remaining.min(pattern_bytes.len());
        data.extend_from_slice(&pattern_bytes[..to_copy]);
    }

    data
}

fn bench_zlib_compression(c: &mut Criterion) {
    let sizes = vec![1024, 4096, 16384, 65536]; // 1KB, 4KB, 16KB, 64KB
    let data_types = vec![
        ("text", "The quick brown fox jumps over the lazy dog. "),
        (
            "binary",
            "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F",
        ),
        ("zeros", "\x00\x00\x00\x00\x00\x00\x00\x00"),
    ];

    let mut group = c.benchmark_group("zlib_compression");

    for (data_name, pattern) in &data_types {
        for &size in &sizes {
            let data = create_test_data(size, pattern);

            group.bench_with_input(
                BenchmarkId::new(*data_name, format!("{size}B")),
                &data,
                |b, test_data| {
                    b.iter(|| compress(black_box(test_data), flags::ZLIB));
                },
            );
        }
    }

    group.finish();
}

fn bench_zlib_decompression(c: &mut Criterion) {
    let sizes = vec![1024, 4096, 16384, 65536];
    let pattern = "The quick brown fox jumps over the lazy dog. ";

    let mut group = c.benchmark_group("zlib_decompression");

    for &size in &sizes {
        let data = create_test_data(size, pattern);
        let compressed = compress(&data, flags::ZLIB).expect("Compression failed");

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{size}B")),
            &(compressed, size),
            |b, (compressed_data, original_size)| {
                b.iter(|| decompress(black_box(compressed_data), flags::ZLIB, *original_size));
            },
        );
    }

    group.finish();
}

fn bench_bzip2_compression(c: &mut Criterion) {
    let sizes = vec![1024, 4096, 16384, 65536];
    let pattern = "The quick brown fox jumps over the lazy dog. ";

    let mut group = c.benchmark_group("bzip2_compression");

    for &size in &sizes {
        let data = create_test_data(size, pattern);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{size}B")),
            &data,
            |b, test_data| {
                b.iter(|| compress(black_box(test_data), flags::BZIP2));
            },
        );
    }

    group.finish();
}

fn bench_sparse_decompression(c: &mut Criterion) {
    // Create different sparse patterns
    let patterns = vec![
        // Small sparse data
        (
            "small",
            vec![
                5, b'H', b'e', b'l', b'l', b'o', 0x8A, // 10 zeros
                5, b'W', b'o', b'r', b'l', b'd', 0xFF,
            ],
            20,
        ),
        // Medium sparse data with multiple runs
        (
            "medium",
            {
                let mut data = Vec::new();
                for _ in 0..10 {
                    data.extend_from_slice(&[4, b'T', b'e', b's', b't']);
                    data.push(0x90); // 16 zeros
                }
                data.push(0xFF);
                data
            },
            200,
        ),
    ];

    let mut group = c.benchmark_group("sparse_decompression");

    for (name, compressed, decompressed_size) in &patterns {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &(compressed, *decompressed_size),
            |b, (data, size)| {
                b.iter(|| decompress(black_box(data), flags::SPARSE, *size));
            },
        );
    }

    group.finish();
}

fn bench_compression_comparison(c: &mut Criterion) {
    // Compare different compression methods on the same data
    let data = create_test_data(16384, "The quick brown fox jumps over the lazy dog. ");

    let mut group = c.benchmark_group("compression_comparison");

    group.bench_function("zlib", |b| {
        b.iter(|| compress(black_box(&data), flags::ZLIB));
    });

    group.bench_function("bzip2", |b| {
        b.iter(|| compress(black_box(&data), flags::BZIP2));
    });

    group.bench_function("none", |b| {
        b.iter(|| compress(black_box(&data), 0));
    });

    group.finish();
}

fn bench_round_trip(c: &mut Criterion) {
    let data = create_test_data(4096, "Mixed content with some repetition. ");

    c.bench_function("zlib_round_trip", |b| {
        b.iter(|| {
            let compressed = compress(black_box(&data), flags::ZLIB).unwrap();
            decompress(black_box(&compressed), flags::ZLIB, data.len())
        });
    });
}

criterion_group!(
    benches,
    bench_zlib_compression,
    bench_zlib_decompression,
    bench_bzip2_compression,
    bench_sparse_decompression,
    bench_compression_comparison,
    bench_round_trip
);
criterion_main!(benches);
