//! Benchmarks for single archive parallel processing
//!
//! These benchmarks compare sequential vs parallel extraction from a single MPQ archive
//! to measure the performance improvements and overhead of parallelization.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use tempfile::TempDir;
use wow_mpq::single_archive_parallel::{ParallelArchive, ParallelConfig, extract_with_config};
use wow_mpq::{Archive, ArchiveBuilder, compression::flags};

/// Create a test archive with many files for benchmarking
fn create_benchmark_archive(
    num_files: usize,
    file_size_kb: usize,
) -> (TempDir, std::path::PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("benchmark.mpq");

    let mut builder = ArchiveBuilder::new()
        .block_size(7) // 64KB sectors
        .default_compression(flags::ZLIB);

    // Create files with specified size
    let content_template = "0123456789ABCDEF".repeat(64); // 1KB of base content
    let content = content_template.repeat(file_size_kb);

    for i in 0..num_files {
        builder = builder.add_file_data(
            content.clone().into_bytes(),
            &format!("data/file_{:04}.dat", i),
        );
    }

    builder.build(&path).unwrap();
    (temp_dir, path)
}

/// Benchmark extracting multiple files sequentially vs in parallel
fn bench_multiple_file_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_archive/multiple_files");

    let file_counts = vec![10, 50, 100, 200];
    let (_temp_dir, archive_path) = create_benchmark_archive(200, 10); // 200 files, 10KB each

    for &count in &file_counts {
        let files: Vec<&str> = (0..count)
            .map(|i| Box::leak(format!("data/file_{:04}.dat", i).into_boxed_str()) as &str)
            .collect();

        group.throughput(Throughput::Elements(count as u64));

        // Sequential extraction
        group.bench_with_input(BenchmarkId::new("sequential", count), &files, |b, files| {
            b.iter(|| {
                let mut archive = Archive::open(black_box(&archive_path)).unwrap();
                let mut results = Vec::new();
                for &file in files.iter() {
                    let data = archive.read_file(file).unwrap();
                    results.push((file, data));
                }
                black_box(results)
            });
        });

        // Parallel extraction
        group.bench_with_input(BenchmarkId::new("parallel", count), &files, |b, files| {
            let archive = ParallelArchive::open(&archive_path).unwrap();
            b.iter(|| {
                let results = archive.extract_files_parallel(black_box(files)).unwrap();
                black_box(results)
            });
        });

        // Batched parallel extraction
        group.bench_with_input(
            BenchmarkId::new("parallel_batched", count),
            &files,
            |b, files| {
                let archive = ParallelArchive::open(&archive_path).unwrap();
                let batch_size = (count / 10).max(1);
                b.iter(|| {
                    let results = archive
                        .extract_files_batched(black_box(files), batch_size)
                        .unwrap();
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark pattern matching extraction
fn bench_pattern_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_archive/pattern_matching");

    let (_temp_dir, archive_path) = create_benchmark_archive(1000, 5); // 1000 files, 5KB each

    group.throughput(Throughput::Elements(1000));

    // Sequential pattern matching
    group.bench_function("sequential", |b| {
        b.iter(|| {
            let mut archive = Archive::open(black_box(&archive_path)).unwrap();
            let files = archive.list().unwrap();
            let mut results = Vec::new();

            for entry in files {
                if entry.name.contains("50") || entry.name.contains("75") {
                    let data = archive.read_file(&entry.name).unwrap();
                    results.push((entry.name, data));
                }
            }
            black_box(results)
        });
    });

    // Parallel pattern matching
    group.bench_function("parallel", |b| {
        let archive = ParallelArchive::open(&archive_path).unwrap();
        b.iter(|| {
            let results = archive
                .extract_matching_parallel(|name| name.contains("50") || name.contains("75"))
                .unwrap();
            black_box(results)
        });
    });

    group.finish();
}

/// Benchmark different file sizes
fn bench_file_size_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_archive/file_size_impact");

    let file_sizes = vec![(100, 1), (50, 10), (20, 50), (10, 100)]; // (count, size_kb)

    for (count, size_kb) in file_sizes {
        let (_temp_dir, archive_path) = create_benchmark_archive(count, size_kb);
        let files: Vec<&str> = (0..count)
            .map(|i| Box::leak(format!("data/file_{:04}.dat", i).into_boxed_str()) as &str)
            .collect();

        let total_mb = (count * size_kb) as f64 / 1024.0;
        group.throughput(Throughput::Bytes((count * size_kb * 1024) as u64));

        // Sequential
        group.bench_with_input(
            BenchmarkId::new("sequential", format!("{:.1}MB", total_mb)),
            &files,
            |b, files| {
                b.iter(|| {
                    let mut archive = Archive::open(black_box(&archive_path)).unwrap();
                    let mut results = Vec::new();
                    for &file in files.iter() {
                        let data = archive.read_file(file).unwrap();
                        results.push(data);
                    }
                    black_box(results)
                });
            },
        );

        // Parallel
        group.bench_with_input(
            BenchmarkId::new("parallel", format!("{:.1}MB", total_mb)),
            &files,
            |b, files| {
                let archive = ParallelArchive::open(&archive_path).unwrap();
                b.iter(|| {
                    let results = archive.extract_files_parallel(black_box(files)).unwrap();
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark thread pool configuration
fn bench_thread_pool_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_archive/thread_scaling");

    let (_temp_dir, archive_path) = create_benchmark_archive(100, 20); // 100 files, 20KB each
    let files: Vec<&str> = (0..100)
        .map(|i| Box::leak(format!("data/file_{:04}.dat", i).into_boxed_str()) as &str)
        .collect();

    let thread_counts = vec![1, 2, 4, 8, 16];

    for &threads in &thread_counts {
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            &threads,
            |b, &num_threads| {
                let config = ParallelConfig::new().threads(num_threads);
                b.iter(|| {
                    let results =
                        extract_with_config(black_box(&archive_path), &files, config.clone())
                            .unwrap();
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark overhead of parallel setup
fn bench_parallel_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_archive/parallel_overhead");

    // Test with very small files to measure overhead
    let (_temp_dir, archive_path) = create_benchmark_archive(1000, 1); // 1000 files, 1KB each

    // Small extraction (where overhead might dominate)
    let small_files: Vec<&str> = (0..5)
        .map(|i| Box::leak(format!("data/file_{:04}.dat", i).into_boxed_str()) as &str)
        .collect();

    group.bench_function("sequential_small", |b| {
        b.iter(|| {
            let mut archive = Archive::open(black_box(&archive_path)).unwrap();
            let mut results = Vec::new();
            for &file in small_files.iter() {
                let data = archive.read_file(file).unwrap();
                results.push(data);
            }
            black_box(results)
        });
    });

    group.bench_function("parallel_small", |b| {
        let archive = ParallelArchive::open(&archive_path).unwrap();
        b.iter(|| {
            let results = archive
                .extract_files_parallel(black_box(&small_files))
                .unwrap();
            black_box(results)
        });
    });

    // Large extraction (where parallelism should win)
    let large_files: Vec<&str> = (0..100)
        .map(|i| Box::leak(format!("data/file_{:04}.dat", i).into_boxed_str()) as &str)
        .collect();

    group.bench_function("sequential_large", |b| {
        b.iter(|| {
            let mut archive = Archive::open(black_box(&archive_path)).unwrap();
            let mut results = Vec::new();
            for &file in large_files.iter() {
                let data = archive.read_file(file).unwrap();
                results.push(data);
            }
            black_box(results)
        });
    });

    group.bench_function("parallel_large", |b| {
        let archive = ParallelArchive::open(&archive_path).unwrap();
        b.iter(|| {
            let results = archive
                .extract_files_parallel(black_box(&large_files))
                .unwrap();
            black_box(results)
        });
    });

    group.finish();
}

/// Benchmark custom processing functions
fn bench_custom_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_archive/custom_processing");

    let (_temp_dir, archive_path) = create_benchmark_archive(50, 50); // 50 files, 50KB each
    let files: Vec<&str> = (0..50)
        .map(|i| Box::leak(format!("data/file_{:04}.dat", i).into_boxed_str()) as &str)
        .collect();

    // Sequential processing
    group.bench_function("sequential_checksum", |b| {
        b.iter(|| {
            let mut archive = Archive::open(black_box(&archive_path)).unwrap();
            let mut results = Vec::new();

            for &file in files.iter() {
                let data = archive.read_file(file).unwrap();
                // Simple checksum calculation
                let checksum: u32 = data.iter().map(|&b| b as u32).sum();
                results.push((file, checksum));
            }
            black_box(results)
        });
    });

    // Parallel processing
    group.bench_function("parallel_checksum", |b| {
        let archive = ParallelArchive::open(&archive_path).unwrap();
        b.iter(|| {
            let results = archive
                .process_files_parallel(&files, |_name, data| {
                    // Same checksum calculation
                    let checksum: u32 = data.iter().map(|&b| b as u32).sum();
                    Ok(checksum)
                })
                .unwrap();
            black_box(results)
        });
    });

    group.finish();
}

/// Benchmark error handling overhead
fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_archive/error_handling");

    let (_temp_dir, archive_path) = create_benchmark_archive(100, 5);

    // Mix of valid and invalid files
    let mixed_files: Vec<&str> = (0..50)
        .flat_map(|i| {
            vec![
                Box::leak(format!("data/file_{:04}.dat", i).into_boxed_str()) as &str,
                Box::leak(format!("missing_{:04}.dat", i).into_boxed_str()) as &str,
            ]
        })
        .collect();

    // With error skipping
    group.bench_function("skip_errors", |b| {
        let config = ParallelConfig::new().skip_errors(true);
        b.iter(|| {
            let results =
                extract_with_config(black_box(&archive_path), &mixed_files, config.clone())
                    .unwrap();
            black_box(results)
        });
    });

    // Process only valid files
    let valid_files: Vec<&str> = (0..50)
        .map(|i| Box::leak(format!("data/file_{:04}.dat", i).into_boxed_str()) as &str)
        .collect();

    group.bench_function("valid_only", |b| {
        let archive = ParallelArchive::open(&archive_path).unwrap();
        b.iter(|| {
            let results = archive
                .extract_files_parallel(black_box(&valid_files))
                .unwrap();
            black_box(results)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_multiple_file_extraction,
    bench_pattern_matching,
    bench_file_size_impact,
    bench_thread_pool_scaling,
    bench_parallel_overhead,
    bench_custom_processing,
    bench_error_handling
);
criterion_main!(benches);
