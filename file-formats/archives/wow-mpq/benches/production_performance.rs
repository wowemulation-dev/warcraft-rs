//! Production Performance Benchmarks for MPQ Extraction System
//!
//! This benchmark suite validates that the MPQ extraction system meets production
//! performance requirements and scales appropriately under various conditions.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use wow_mpq::single_archive_parallel::{ParallelArchive, ParallelConfig, extract_with_config};
use wow_mpq::{Archive, ArchiveBuilder, compression::flags};

/// Performance metrics collector
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub files_extracted: usize,
    pub total_bytes: u64,
    pub duration_ms: u64,
    pub files_per_second: f64,
    pub mb_per_second: f64,
    pub threads_used: usize,
    pub batch_size: Option<usize>,
}

impl PerformanceMetrics {
    pub fn new(
        files: usize,
        bytes: u64,
        duration: Duration,
        threads: usize,
        batch_size: Option<usize>,
    ) -> Self {
        let duration_ms = duration.as_millis() as u64;
        let duration_sec = duration.as_secs_f64();

        Self {
            files_extracted: files,
            total_bytes: bytes,
            duration_ms,
            files_per_second: files as f64 / duration_sec,
            mb_per_second: (bytes as f64 / (1024.0 * 1024.0)) / duration_sec,
            threads_used: threads,
            batch_size,
        }
    }

    pub fn efficiency_score(&self) -> f64 {
        // Efficiency score based on files/sec per thread
        self.files_per_second / self.threads_used as f64
    }
}

/// Create realistic test archives of various sizes
fn create_test_archive_realistic(
    name: &str,
    file_count: usize,
    avg_file_size_kb: usize,
    size_variation: f64,
) -> (TempDir, std::path::PathBuf, u64) {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join(format!("{name}.mpq"));

    let mut builder = ArchiveBuilder::new()
        .block_size(6) // 32KB sectors for realistic performance
        .default_compression(flags::ZLIB);

    let mut total_uncompressed_bytes = 0u64;

    // Create files with realistic size distribution
    for i in 0..file_count {
        // Vary file sizes realistically
        let size_factor = 1.0 + (i as f64 / file_count as f64 - 0.5) * size_variation;
        let file_size = (avg_file_size_kb as f64 * size_factor) as usize;

        // Create realistic file content (not just repeating patterns)
        let mut content = Vec::new();
        for j in 0..(file_size * 1024 / 16) {
            content.extend_from_slice(&(i * 1000 + j).to_le_bytes());
            content.extend_from_slice(&j.to_le_bytes());
            content.extend_from_slice(&[0u8; 8]);
        }

        // Pad to exact size
        content.resize(file_size * 1024, 0);
        total_uncompressed_bytes += content.len() as u64;

        // Use realistic file paths like a game would
        let file_path = match i % 10 {
            0..=3 => format!("Interface/Icons/icon_{i:04}.blp"),
            4..=6 => format!("Sound/Music/track_{i:04}.wav"),
            7..=8 => format!("World/Maps/map_{i:04}.adt"),
            _ => format!("Data/files/data_{i:04}.dat"),
        };

        builder = builder.add_file_data(content, &file_path);
    }

    builder.build(&path).unwrap();
    (temp_dir, path, total_uncompressed_bytes)
}

/// Benchmark extraction throughput across different archive sizes
fn bench_throughput_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("production/throughput_scaling");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    // Test different archive sizes to understand scaling
    let test_configs = vec![
        ("small", 100, 50, 0.3),     // 100 files, 50KB avg, 30% size variation
        ("medium", 1000, 100, 0.5),  // 1K files, 100KB avg, 50% variation
        ("large", 5000, 200, 0.8),   // 5K files, 200KB avg, 80% variation
        ("xlarge", 15000, 150, 1.0), // 15K files, 150KB avg, 100% variation
    ];

    for (name, file_count, avg_size_kb, variation) in test_configs {
        let (_temp_dir, archive_path, total_bytes) =
            create_test_archive_realistic(name, file_count, avg_size_kb, variation);

        group.throughput(Throughput::Bytes(total_bytes));

        // Generate file list for extraction (extract all files)
        let files: Vec<String> = {
            let mut archive = Archive::open(&archive_path).unwrap();
            archive
                .list()
                .unwrap()
                .into_iter()
                .map(|e| e.name)
                .collect()
        };
        let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();

        // Measure sequential baseline
        group.bench_with_input(
            BenchmarkId::new("sequential", name),
            &(&archive_path, &file_refs, total_bytes),
            |b, (path, files, _bytes)| {
                b.iter(|| {
                    let mut archive = Archive::open(black_box(path)).unwrap();
                    let mut results = Vec::new();
                    for &file in files.iter() {
                        let data = archive.read_file(file).unwrap();
                        results.push(data);
                    }
                    black_box(results)
                });
            },
        );

        // Measure parallel performance
        group.bench_with_input(
            BenchmarkId::new("parallel", name),
            &(&archive_path, &file_refs, total_bytes),
            |b, (path, files, _bytes)| {
                let archive = ParallelArchive::open(path).unwrap();
                b.iter(|| {
                    let results = archive.extract_files_parallel(black_box(files)).unwrap();
                    black_box(results)
                });
            },
        );

        // Measure batched parallel performance
        group.bench_with_input(
            BenchmarkId::new("parallel_batched", name),
            &(&archive_path, &file_refs, total_bytes),
            |b, (path, files, _bytes)| {
                let config = ParallelConfig::new().batch_size(50);
                b.iter(|| {
                    let results =
                        extract_with_config(black_box(path), files, config.clone()).unwrap();
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark threading scalability with different thread counts
fn bench_threading_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("production/threading_scalability");
    group.sample_size(15);
    group.measurement_time(Duration::from_secs(20));

    // Create a substantial test archive
    let (_temp_dir, archive_path, total_bytes) =
        create_test_archive_realistic("threading_test", 2000, 100, 0.6);

    let files: Vec<String> = {
        let mut archive = Archive::open(&archive_path).unwrap();
        archive
            .list()
            .unwrap()
            .into_iter()
            .map(|e| e.name)
            .collect()
    };
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();

    group.throughput(Throughput::Bytes(total_bytes));

    // Test with different thread counts
    let thread_counts = vec![1, 2, 4, 8, 12, 16];

    for &threads in &thread_counts {
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            &threads,
            |b, &num_threads| {
                let config = ParallelConfig::new().threads(num_threads).batch_size(25);

                b.iter(|| {
                    let results =
                        extract_with_config(black_box(&archive_path), &file_refs, config.clone())
                            .unwrap();
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark batch size optimization
fn bench_batch_size_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("production/batch_size_optimization");
    group.sample_size(12);

    let (_temp_dir, archive_path, _) = create_test_archive_realistic("batch_test", 1500, 80, 0.4);

    let files: Vec<String> = {
        let mut archive = Archive::open(&archive_path).unwrap();
        archive
            .list()
            .unwrap()
            .into_iter()
            .map(|e| e.name)
            .collect()
    };
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();

    // Test different batch sizes
    let batch_sizes = vec![1, 5, 10, 25, 50, 100, 200];

    for &batch_size in &batch_sizes {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, &size| {
                let config = ParallelConfig::new().batch_size(size).threads(8);

                b.iter(|| {
                    let results =
                        extract_with_config(black_box(&archive_path), &file_refs, config.clone())
                            .unwrap();
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// Test system resource utilization patterns
fn bench_resource_utilization(c: &mut Criterion) {
    let mut group = c.benchmark_group("production/resource_utilization");
    group.sample_size(8);
    group.measurement_time(Duration::from_secs(25));

    // Create a large archive to stress resource usage
    let (_temp_dir, archive_path, total_bytes) =
        create_test_archive_realistic("resource_test", 10000, 120, 0.7);

    let files: Vec<String> = {
        let mut archive = Archive::open(&archive_path).unwrap();
        archive
            .list()
            .unwrap()
            .into_iter()
            .map(|e| e.name)
            .collect()
    };
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();

    group.throughput(Throughput::Bytes(total_bytes));

    // Test resource-conscious batched approach
    group.bench_function("resource_conscious", |b| {
        let config = ParallelConfig::new().threads(8).batch_size(75);

        b.iter(|| {
            let results =
                extract_with_config(black_box(&archive_path), &file_refs, config.clone()).unwrap();
            black_box(results)
        });
    });

    // Test with smaller batches (more file handles)
    group.bench_function("many_handles", |b| {
        let config = ParallelConfig::new().threads(16).batch_size(10);

        b.iter(|| {
            let results =
                extract_with_config(black_box(&archive_path), &file_refs, config.clone()).unwrap();
            black_box(results)
        });
    });

    group.finish();
}

/// Benchmark with real WoW archive sizes if available
fn bench_realistic_wow_archive(c: &mut Criterion) {
    let mut group = c.benchmark_group("production/wow_realistic");
    group.sample_size(5);
    group.measurement_time(Duration::from_secs(45));

    // Try to use real WoW data if available
    let wow_paths = vec![
        "/home/danielsreichenbach/Downloads/wow/1.12.1/Data/",
        "/home/danielsreichenbach/Downloads/wow/3.3.5a/Data/",
    ];

    for wow_path in wow_paths {
        let wow_dir = std::path::Path::new(wow_path);
        if !wow_dir.exists() {
            continue;
        }

        // Find the first MPQ archive
        if let Ok(entries) = std::fs::read_dir(wow_dir) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext.to_string_lossy().to_lowercase() == "mpq" {
                        let mpq_path = entry.path();

                        // Try to open and get file count
                        if let Ok(mut archive) = Archive::open(&mpq_path) {
                            if let Ok(file_list) = archive.list() {
                                let file_count = file_list.len();

                                // Only test with reasonably sized archives
                                if file_count > 100 && file_count < 30000 {
                                    // Extract a sample of files
                                    let sample_size = (file_count / 20).clamp(50, 500);
                                    let sample_files: Vec<String> = file_list
                                        .into_iter()
                                        .step_by(file_count / sample_size)
                                        .map(|e| e.name)
                                        .collect();
                                    let file_refs: Vec<&str> =
                                        sample_files.iter().map(|s| s.as_str()).collect();

                                    let archive_name =
                                        mpq_path.file_stem().unwrap_or_default().to_string_lossy();

                                    group.bench_with_input(
                                        BenchmarkId::new("wow_archive", &*archive_name),
                                        &(&mpq_path, &file_refs),
                                        |b, (path, files)| {
                                            let config = ParallelConfig::new()
                                                .threads(8)
                                                .batch_size(40)
                                                .skip_errors(true);

                                            b.iter(|| {
                                                let results = extract_with_config(
                                                    black_box(path),
                                                    files,
                                                    config.clone(),
                                                )
                                                .unwrap();
                                                black_box(results)
                                            });
                                        },
                                    );

                                    // Only test first valid archive per directory
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    group.finish();
}

/// Stress test to validate hanging issue resolution
fn bench_stress_no_hanging(c: &mut Criterion) {
    let mut group = c.benchmark_group("production/stress_no_hanging");
    group.sample_size(3);
    group.measurement_time(Duration::from_secs(60));

    // Create a very large archive that previously would cause hanging
    let (_temp_dir, archive_path, total_bytes) = create_test_archive_realistic(
        "stress_test",
        25000,
        100,
        1.2, // 25K files with high size variation
    );

    let files: Vec<String> = {
        let mut archive = Archive::open(&archive_path).unwrap();
        archive
            .list()
            .unwrap()
            .into_iter()
            .map(|e| e.name)
            .collect()
    };
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();

    group.throughput(Throughput::Bytes(total_bytes));

    group.bench_function("large_extraction_no_hang", |b| {
        let config = ParallelConfig::new()
            .threads(12)
            .batch_size(100)
            .skip_errors(false);

        b.iter(|| {
            let start = Instant::now();
            let results =
                extract_with_config(black_box(&archive_path), &file_refs, config.clone()).unwrap();
            let duration = start.elapsed();

            // Validate we didn't hang (should complete in reasonable time)
            assert!(
                duration < Duration::from_secs(300),
                "Extraction took too long: {}s",
                duration.as_secs()
            );
            assert_eq!(results.len(), files.len(), "Not all files were extracted");

            black_box(results)
        });
    });

    group.finish();
}

/// Performance comparison with individual file extraction
fn bench_individual_vs_bulk(c: &mut Criterion) {
    let mut group = c.benchmark_group("production/individual_vs_bulk");
    group.sample_size(10);

    let (_temp_dir, archive_path, _) = create_test_archive_realistic("comparison", 500, 150, 0.4);

    let files: Vec<String> = {
        let mut archive = Archive::open(&archive_path).unwrap();
        archive
            .list()
            .unwrap()
            .into_iter()
            .take(100)
            .map(|e| e.name)
            .collect()
    };
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();

    // Individual file extraction (one archive handle per file)
    group.bench_function("individual_extraction", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for &file in file_refs.iter() {
                let mut archive = Archive::open(black_box(&archive_path)).unwrap();
                let data = archive.read_file(file).unwrap();
                results.push(data);
            }
            black_box(results)
        });
    });

    // Sequential bulk extraction (one handle, multiple files)
    group.bench_function("sequential_bulk", |b| {
        b.iter(|| {
            let mut archive = Archive::open(black_box(&archive_path)).unwrap();
            let mut results = Vec::new();
            for &file in file_refs.iter() {
                let data = archive.read_file(file).unwrap();
                results.push(data);
            }
            black_box(results)
        });
    });

    // Parallel bulk extraction
    group.bench_function("parallel_bulk", |b| {
        let archive = ParallelArchive::open(&archive_path).unwrap();
        b.iter(|| {
            let results = archive
                .extract_files_parallel(black_box(&file_refs))
                .unwrap();
            black_box(results)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_throughput_scaling,
    bench_threading_scalability,
    bench_batch_size_optimization,
    bench_resource_utilization,
    bench_realistic_wow_archive,
    bench_stress_no_hanging,
    bench_individual_vs_bulk
);
criterion_main!(benches);
