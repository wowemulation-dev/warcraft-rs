//! Archive creation benchmarks
//!
//! These benchmarks measure the performance of creating MPQ archives
//! with different configurations to track performance regressions.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use tempfile::TempDir;
use wow_mpq::{ArchiveBuilder, FormatVersion, compression::flags};

/// Generate test data with specified characteristics
fn generate_test_data(size: usize, compressibility: &str) -> Vec<u8> {
    match compressibility {
        "high" => vec![b'A'; size], // Highly compressible
        "medium" => {
            // Repeating pattern
            let pattern = b"The quick brown fox jumps over the lazy dog. ";
            let mut data = Vec::with_capacity(size);
            while data.len() < size {
                let remaining = size - data.len();
                let to_copy = remaining.min(pattern.len());
                data.extend_from_slice(&pattern[..to_copy]);
            }
            data
        }
        "low" => {
            // Pseudo-random data
            let mut data = Vec::with_capacity(size);
            let mut seed = 0x12345678u32;
            for _ in 0..size {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                data.push((seed >> 16) as u8);
            }
            data
        }
        _ => panic!("Unknown compressibility level"),
    }
}

/// Benchmark creating archives with single files of different sizes
fn bench_single_file_archive(c: &mut Criterion) {
    let mut group = c.benchmark_group("archive_creation/single_file");

    // Test different file sizes
    let sizes = vec![
        ("1KB", 1024),
        ("64KB", 64 * 1024),
        ("1MB", 1024 * 1024),
        ("10MB", 10 * 1024 * 1024),
    ];

    for (name, size) in sizes {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), &size, |b, &size| {
            let data = generate_test_data(size, "medium");
            let temp_dir = TempDir::new().unwrap();

            b.iter(|| {
                let output_path = temp_dir.path().join("archive.mpq");
                ArchiveBuilder::new()
                    .version(FormatVersion::V2)
                    .block_size(16) // 64KB sectors
                    .add_file_data(data.clone(), "test_file.dat")
                    .build(&output_path)
                    .unwrap();
            });
        });
    }

    group.finish();
}

/// Benchmark creating archives with multiple files
fn bench_multi_file_archive(c: &mut Criterion) {
    let mut group = c.benchmark_group("archive_creation/multi_file");

    // Test different file counts
    let configurations = vec![
        ("10_files_100KB", 10, 100 * 1024),
        ("100_files_10KB", 100, 10 * 1024),
        ("1000_files_1KB", 1000, 1024),
    ];

    for (name, file_count, file_size) in configurations {
        let total_size = file_count * file_size;
        group.throughput(Throughput::Bytes(total_size as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &(file_count, file_size),
            |b, &(count, size)| {
                let files: Vec<(String, Vec<u8>)> = (0..count)
                    .map(|i| {
                        let filename = format!("file_{:04}.dat", i);
                        let data = generate_test_data(size, "medium");
                        (filename, data)
                    })
                    .collect();

                let temp_dir = TempDir::new().unwrap();

                b.iter(|| {
                    let output_path = temp_dir.path().join("archive.mpq");
                    let mut builder = ArchiveBuilder::new()
                        .version(FormatVersion::V2)
                        .block_size(16); // 64KB sectors

                    for (filename, data) in &files {
                        builder = builder.add_file_data(data.clone(), filename);
                    }

                    builder.build(&output_path).unwrap();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark different compression methods
fn bench_compression_methods(c: &mut Criterion) {
    let mut group = c.benchmark_group("archive_creation/compression");

    let data_size = 1024 * 1024; // 1MB
    let compressions = vec![
        ("none", 0), // No compression
        ("zlib", flags::ZLIB),
        ("bzip2", flags::BZIP2),
        ("lzma", flags::LZMA),
    ];

    // Test each compression on different data types
    let data_types = vec![
        ("high_compress", "high"),
        ("medium_compress", "medium"),
        ("low_compress", "low"),
    ];

    for (compression_name, compression) in &compressions {
        for (data_name, compressibility) in &data_types {
            let data = generate_test_data(data_size, compressibility);
            let bench_name = format!("{}/{}", compression_name, data_name);

            group.throughput(Throughput::Bytes(data_size as u64));
            group.bench_with_input(
                BenchmarkId::from_parameter(&bench_name),
                &(&data, compression),
                |b, (data, compression)| {
                    let temp_dir = TempDir::new().unwrap();

                    b.iter(|| {
                        let output_path = temp_dir.path().join("archive.mpq");
                        ArchiveBuilder::new()
                            .version(FormatVersion::V2)
                            .default_compression(**compression)
                            .add_file_data((*data).clone(), "test.dat")
                            .build(&output_path)
                            .unwrap();
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark different archive versions
fn bench_archive_versions(c: &mut Criterion) {
    let mut group = c.benchmark_group("archive_creation/versions");

    let data_size = 512 * 1024; // 512KB
    let data = generate_test_data(data_size, "medium");

    let versions = vec![
        ("v1", FormatVersion::V1),
        ("v2", FormatVersion::V2),
        ("v3_het_bet", FormatVersion::V3),
        ("v4_het_bet", FormatVersion::V4),
    ];

    for (name, version) in versions {
        group.throughput(Throughput::Bytes(data_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &version,
            |b, &version| {
                let temp_dir = TempDir::new().unwrap();

                b.iter(|| {
                    let output_path = temp_dir.path().join("archive.mpq");
                    ArchiveBuilder::new()
                        .version(version)
                        .add_file_data(data.clone(), "test.dat")
                        .build(&output_path)
                        .unwrap();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark encrypted archive creation
fn bench_encrypted_archives(c: &mut Criterion) {
    let mut group = c.benchmark_group("archive_creation/encrypted");

    let data_size = 256 * 1024; // 256KB
    let data = generate_test_data(data_size, "medium");

    group.throughput(Throughput::Bytes(data_size as u64));

    // Non-encrypted baseline
    group.bench_function("no_encryption", |b| {
        let temp_dir = TempDir::new().unwrap();

        b.iter(|| {
            let output_path = temp_dir.path().join("archive.mpq");
            ArchiveBuilder::new()
                .version(FormatVersion::V2)
                .add_file_data(data.clone(), "test.dat")
                .build(&output_path)
                .unwrap();
        });
    });

    // With encryption
    group.bench_function("with_encryption", |b| {
        let temp_dir = TempDir::new().unwrap();

        b.iter(|| {
            let output_path = temp_dir.path().join("archive.mpq");
            ArchiveBuilder::new()
                .version(FormatVersion::V2)
                .add_file_data_with_encryption(
                    data.clone(),
                    "test.dat",
                    flags::ZLIB,
                    false, // use_fix_key
                    0,     // locale
                )
                .build(&output_path)
                .unwrap();
        });
    });

    group.finish();
}

/// Benchmark different block sizes
fn bench_block_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("archive_creation/block_sizes");

    let data_size = 4 * 1024 * 1024; // 4MB
    let data = generate_test_data(data_size, "medium");

    let block_sizes = vec![
        ("4KB", 3),   // 512 * 2^3 = 4KB
        ("16KB", 5),  // 512 * 2^5 = 16KB
        ("64KB", 7),  // 512 * 2^7 = 64KB
        ("256KB", 9), // 512 * 2^9 = 256KB
        ("1MB", 11),  // 512 * 2^11 = 1MB
    ];

    for (name, block_size) in block_sizes {
        group.throughput(Throughput::Bytes(data_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &block_size,
            |b, &block_size| {
                let temp_dir = TempDir::new().unwrap();

                b.iter(|| {
                    let output_path = temp_dir.path().join("archive.mpq");
                    ArchiveBuilder::new()
                        .version(FormatVersion::V2)
                        .block_size(block_size)
                        .add_file_data(data.clone(), "test.dat")
                        .build(&output_path)
                        .unwrap();
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_single_file_archive,
    bench_multi_file_archive,
    bench_compression_methods,
    bench_archive_versions,
    bench_encrypted_archives,
    bench_block_sizes
);
criterion_main!(benches);
