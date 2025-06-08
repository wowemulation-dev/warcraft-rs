//! Archive extraction benchmarks
//!
//! These benchmarks measure the performance of extracting files from MPQ archives
//! with different configurations to track performance regressions.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::path::Path;
use tempfile::TempDir;
use wow_mpq::{Archive, ArchiveBuilder, FormatVersion, compression::flags};

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

/// Create a test archive and return its path
fn create_test_archive(
    temp_dir: &Path,
    name: &str,
    files: Vec<(&str, Vec<u8>)>,
    compression: u8,
    version: FormatVersion,
) -> std::path::PathBuf {
    let archive_path = temp_dir.join(format!("{}.mpq", name));
    let mut builder = ArchiveBuilder::new()
        .version(version)
        .default_compression(compression)
        .block_size(7); // 64KB sectors

    for (filename, data) in files {
        builder = builder.add_file_data(data, filename);
    }

    builder.build(&archive_path).unwrap();
    archive_path
}

/// Benchmark extracting single files of different sizes
fn bench_single_file_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("archive_extraction/single_file");

    let sizes = vec![
        ("1KB", 1024),
        ("64KB", 64 * 1024),
        ("1MB", 1024 * 1024),
        ("10MB", 10 * 1024 * 1024),
    ];

    let temp_dir = TempDir::new().unwrap();

    for (name, size) in sizes {
        let data = generate_test_data(size, "medium");
        let archive_path = create_test_archive(
            temp_dir.path(),
            name,
            vec![("test_file.dat", data.clone())],
            0, // No compression
            FormatVersion::V2,
        );

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &archive_path,
            |b, path| {
                b.iter(|| {
                    let mut archive = Archive::open(black_box(path)).unwrap();
                    let extracted = archive.read_file("test_file.dat").unwrap();
                    black_box(extracted);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark extracting from compressed archives
fn bench_compressed_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("archive_extraction/compressed");

    let data_size = 1024 * 1024; // 1MB
    let compressions = vec![
        ("none", 0),
        ("zlib", flags::ZLIB),
        ("bzip2", flags::BZIP2),
        ("lzma", flags::LZMA),
    ];

    let temp_dir = TempDir::new().unwrap();

    for (compression_name, compression) in compressions {
        let data = generate_test_data(data_size, "high"); // Highly compressible
        let archive_path = create_test_archive(
            temp_dir.path(),
            compression_name,
            vec![("test.dat", data)],
            compression,
            FormatVersion::V2,
        );

        group.throughput(Throughput::Bytes(data_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(compression_name),
            &archive_path,
            |b, path| {
                b.iter(|| {
                    let mut archive = Archive::open(black_box(path)).unwrap();
                    let extracted = archive.read_file("test.dat").unwrap();
                    black_box(extracted);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark listing files in archives
fn bench_file_listing(c: &mut Criterion) {
    let mut group = c.benchmark_group("archive_extraction/listing");

    let file_counts = vec![
        ("10_files", 10),
        ("100_files", 100),
        ("1000_files", 1000),
        ("10000_files", 10000),
    ];

    let temp_dir = TempDir::new().unwrap();

    for (name, count) in file_counts {
        let files: Vec<(&str, Vec<u8>)> = (0..count)
            .map(|i| {
                let filename = Box::leak(format!("file_{:05}.dat", i).into_boxed_str()) as &str;
                let data = vec![0u8; 100]; // Small files
                (filename, data)
            })
            .collect();

        let archive_path = create_test_archive(
            temp_dir.path(),
            name,
            files,
            0, // No compression
            FormatVersion::V2,
        );

        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &archive_path,
            |b, path| {
                b.iter(|| {
                    let mut archive = Archive::open(black_box(path)).unwrap();
                    let files = archive.list().unwrap();
                    black_box(files);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark random access patterns
fn bench_random_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("archive_extraction/random_access");

    let temp_dir = TempDir::new().unwrap();
    let file_count = 100;
    let file_size = 10 * 1024; // 10KB each

    let files: Vec<(&str, Vec<u8>)> = (0..file_count)
        .map(|i| {
            let filename = Box::leak(format!("file_{:03}.dat", i).into_boxed_str()) as &str;
            let data = generate_test_data(file_size, "medium");
            (filename, data)
        })
        .collect();

    let archive_path = create_test_archive(
        temp_dir.path(),
        "random_access",
        files,
        flags::ZLIB,
        FormatVersion::V2,
    );

    // Generate access patterns
    let sequential: Vec<String> = (0..file_count)
        .map(|i| format!("file_{:03}.dat", i))
        .collect();
    let random = {
        let mut indices: Vec<usize> = (0..file_count).collect();
        // Simple shuffle
        for i in 0..file_count {
            let j = (i * 7 + 13) % file_count;
            indices.swap(i, j);
        }
        indices
            .iter()
            .map(|&i| format!("file_{:03}.dat", i))
            .collect::<Vec<_>>()
    };

    group.throughput(Throughput::Elements(file_count as u64));

    group.bench_with_input(
        BenchmarkId::from_parameter("sequential"),
        &(&archive_path, &sequential),
        |b, (path, filenames)| {
            b.iter(|| {
                let mut archive = Archive::open(black_box(path)).unwrap();
                for filename in filenames.iter() {
                    let data = archive.read_file(filename).unwrap();
                    black_box(data);
                }
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::from_parameter("random"),
        &(&archive_path, &random),
        |b, (path, filenames)| {
            b.iter(|| {
                let mut archive = Archive::open(black_box(path)).unwrap();
                for filename in filenames.iter() {
                    let data = archive.read_file(filename).unwrap();
                    black_box(data);
                }
            });
        },
    );

    group.finish();
}

/// Benchmark extracting from different archive versions
fn bench_version_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("archive_extraction/versions");

    let data_size = 512 * 1024; // 512KB
    let data = generate_test_data(data_size, "medium");
    let temp_dir = TempDir::new().unwrap();

    let versions = vec![
        ("v1", FormatVersion::V1),
        ("v2", FormatVersion::V2),
        ("v3_het_bet", FormatVersion::V3),
        ("v4_het_bet", FormatVersion::V4),
    ];

    for (name, version) in versions {
        let archive_path = create_test_archive(
            temp_dir.path(),
            name,
            vec![("test.dat", data.clone())],
            flags::ZLIB,
            version,
        );

        group.throughput(Throughput::Bytes(data_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &archive_path,
            |b, path| {
                b.iter(|| {
                    let mut archive = Archive::open(black_box(path)).unwrap();
                    let extracted = archive.read_file("test.dat").unwrap();
                    black_box(extracted);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark parallel extraction (simulated)
fn bench_parallel_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("archive_extraction/parallel");

    let temp_dir = TempDir::new().unwrap();
    let file_count = 50;
    let file_size = 100 * 1024; // 100KB each

    let files: Vec<(&str, Vec<u8>)> = (0..file_count)
        .map(|i| {
            let filename = Box::leak(format!("file_{:02}.dat", i).into_boxed_str()) as &str;
            let data = generate_test_data(file_size, "medium");
            (filename, data)
        })
        .collect();

    let archive_path = create_test_archive(
        temp_dir.path(),
        "parallel",
        files,
        flags::ZLIB,
        FormatVersion::V2,
    );

    let total_size = file_count * file_size;
    group.throughput(Throughput::Bytes(total_size as u64));

    // Single-threaded baseline
    group.bench_function("single_thread", |b| {
        b.iter(|| {
            let mut archive = Archive::open(black_box(&archive_path)).unwrap();
            for i in 0..file_count {
                let filename = format!("file_{:02}.dat", i);
                let data = archive.read_file(&filename).unwrap();
                black_box(data);
            }
        });
    });

    // Simulated parallel access (multiple archive handles)
    group.bench_function("multi_handle", |b| {
        b.iter(|| {
            // In real parallel code, these would be in different threads
            let mut archives: Vec<_> = (0..4)
                .map(|_| Archive::open(&archive_path).unwrap())
                .collect();

            for (i, archive) in archives.iter_mut().enumerate() {
                for j in (i..file_count).step_by(4) {
                    let filename = format!("file_{:02}.dat", j);
                    let data = archive.read_file(&filename).unwrap();
                    black_box(data);
                }
            }
        });
    });

    group.finish();
}

/// Benchmark metadata operations
fn bench_metadata_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("archive_extraction/metadata");

    let temp_dir = TempDir::new().unwrap();
    let file_count = 1000;

    let files: Vec<(&str, Vec<u8>)> = (0..file_count)
        .map(|i| {
            let filename = Box::leak(
                format!(
                    "folder{}/subfolder{}/file_{:04}.dat",
                    i / 100,
                    (i / 10) % 10,
                    i
                )
                .into_boxed_str(),
            ) as &str;
            let data = vec![0u8; 1024]; // 1KB files
            (filename, data)
        })
        .collect();

    let archive_path = create_test_archive(
        temp_dir.path(),
        "metadata",
        files,
        0, // No compression
        FormatVersion::V2,
    );

    group.bench_function("file_exists", |b| {
        let archive = Archive::open(&archive_path).unwrap();
        b.iter(|| {
            for i in 0..100 {
                let filename = format!(
                    "folder{}/subfolder{}/file_{:04}.dat",
                    i / 100,
                    (i / 10) % 10,
                    i
                );
                let exists = archive.find_file(&filename).unwrap().is_some();
                black_box(exists);
            }
        });
    });

    group.bench_function("file_info", |b| {
        let archive = Archive::open(&archive_path).unwrap();
        b.iter(|| {
            for i in 0..100 {
                let filename = format!(
                    "folder{}/subfolder{}/file_{:04}.dat",
                    i / 100,
                    (i / 10) % 10,
                    i
                );
                if let Ok(Some(entry)) = archive.find_file(&filename) {
                    black_box(entry.file_size);
                    black_box(entry.compressed_size);
                    black_box(entry.flags);
                }
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_single_file_extraction,
    bench_compressed_extraction,
    bench_file_listing,
    bench_random_access,
    bench_version_extraction,
    bench_parallel_extraction,
    bench_metadata_operations
);
criterion_main!(benches);
