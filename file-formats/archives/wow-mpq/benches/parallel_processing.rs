//! Benchmarks for parallel processing operations
//!
//! These benchmarks compare sequential vs parallel processing of MPQ archives
//! to measure the performance improvements from parallelization.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use tempfile::TempDir;
use wow_mpq::{Archive, ArchiveBuilder, PatchChain, compression::flags};

/// Create test archives for benchmarking
fn create_test_archives(
    count: usize,
    files_per_archive: usize,
) -> (TempDir, Vec<std::path::PathBuf>) {
    let temp_dir = TempDir::new().unwrap();
    let mut paths = Vec::new();

    for i in 0..count {
        let path = temp_dir.path().join(format!("archive_{i}.mpq"));
        let mut builder = ArchiveBuilder::new()
            .block_size(7) // 64KB sectors
            .default_compression(flags::ZLIB);

        // Add files to each archive
        for j in 0..files_per_archive {
            let content = format!(
                "Archive {i} File {j} Content with some padding to make it larger"
            )
            .repeat(100); // Make files reasonably sized
            builder = builder.add_file_data(content.into_bytes(), &format!("file_{j:03}.txt"));
        }

        builder.build(&path).unwrap();
        paths.push(path);
    }

    (temp_dir, paths)
}

/// Benchmark extracting a single file from multiple archives
fn bench_multi_archive_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel/multi_archive_extraction");

    let archive_counts = vec![2, 4, 8, 16];
    let (_temp_dir, archives) = create_test_archives(16, 10);

    for &count in &archive_counts {
        let archives_subset = &archives[..count];

        group.throughput(Throughput::Elements(count as u64));

        // Sequential version
        group.bench_with_input(
            BenchmarkId::new("sequential", count),
            &archives_subset,
            |b, archives| {
                b.iter(|| {
                    let mut results = Vec::new();
                    for path in archives.iter() {
                        let mut archive = Archive::open(black_box(path)).unwrap();
                        let data = archive.read_file("file_005.txt").unwrap();
                        results.push((path.clone(), data));
                    }
                    black_box(results)
                });
            },
        );

        // Parallel version
        group.bench_with_input(
            BenchmarkId::new("parallel", count),
            &archives_subset,
            |b, archives| {
                b.iter(|| {
                    use wow_mpq::parallel::extract_from_multiple_archives;
                    let results =
                        extract_from_multiple_archives(black_box(archives), "file_005.txt")
                            .unwrap();
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark searching for files across multiple archives
fn bench_multi_archive_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel/multi_archive_search");

    let (_temp_dir, archives) = create_test_archives(8, 50);
    let search_pattern = "file_02"; // Will match file_020 through file_029

    group.throughput(Throughput::Elements(archives.len() as u64));

    // Sequential search
    group.bench_function("sequential", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for path in archives.iter() {
                let mut archive = Archive::open(black_box(path)).unwrap();
                let files = archive.list().unwrap();
                let matching: Vec<String> = files
                    .into_iter()
                    .filter(|entry| entry.name.contains(search_pattern))
                    .map(|entry| entry.name)
                    .collect();
                results.push((path.clone(), matching));
            }
            black_box(results)
        });
    });

    // Parallel search
    group.bench_function("parallel", |b| {
        b.iter(|| {
            use wow_mpq::parallel::search_in_multiple_archives;
            let results =
                search_in_multiple_archives(black_box(&archives), search_pattern).unwrap();
            black_box(results)
        });
    });

    group.finish();
}

/// Benchmark extracting multiple files from multiple archives
fn bench_multi_file_multi_archive(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel/multi_file_multi_archive");

    let (_temp_dir, archives) = create_test_archives(4, 20);
    let files_to_extract = vec![
        "file_001.txt",
        "file_005.txt",
        "file_010.txt",
        "file_015.txt",
    ];

    let total_operations = archives.len() * files_to_extract.len();
    group.throughput(Throughput::Elements(total_operations as u64));

    // Sequential extraction
    group.bench_function("sequential", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for path in archives.iter() {
                let mut archive = Archive::open(black_box(path)).unwrap();
                let mut file_results = Vec::new();
                for &file_name in &files_to_extract {
                    let data = archive.read_file(file_name).unwrap();
                    file_results.push((file_name.to_string(), data));
                }
                results.push((path.clone(), file_results));
            }
            black_box(results)
        });
    });

    // Parallel extraction
    group.bench_function("parallel", |b| {
        b.iter(|| {
            use wow_mpq::parallel::extract_multiple_from_multiple_archives;
            let results =
                extract_multiple_from_multiple_archives(black_box(&archives), &files_to_extract)
                    .unwrap();
            black_box(results)
        });
    });

    group.finish();
}

/// Benchmark custom processing across multiple archives
fn bench_custom_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel/custom_processing");

    let (_temp_dir, archives) = create_test_archives(8, 30);

    group.throughput(Throughput::Elements(archives.len() as u64));

    // Sequential file counting
    group.bench_function("sequential_count", |b| {
        b.iter(|| {
            let mut counts = Vec::new();
            for path in archives.iter() {
                let mut archive = Archive::open(black_box(path)).unwrap();
                let count = archive.list().unwrap().len();
                counts.push(count);
            }
            black_box(counts)
        });
    });

    // Parallel file counting
    group.bench_function("parallel_count", |b| {
        b.iter(|| {
            use wow_mpq::parallel::process_archives_parallel;
            let counts = process_archives_parallel(black_box(&archives), |mut archive| {
                Ok(archive.list()?.len())
            })
            .unwrap();
            black_box(counts)
        });
    });

    group.finish();
}

/// Benchmark patch chain loading
fn bench_patch_chain_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel/patch_chain_loading");

    let chain_sizes = vec![2, 4, 8, 16];
    let (_temp_dir, all_archives) = create_test_archives(16, 20);

    for &count in &chain_sizes {
        let archives: Vec<_> = all_archives[..count]
            .iter()
            .enumerate()
            .map(|(i, path)| (path.clone(), (i * 100) as i32))
            .collect();

        group.throughput(Throughput::Elements(count as u64));

        // Sequential loading
        group.bench_with_input(
            BenchmarkId::new("sequential", count),
            &archives,
            |b, archives| {
                b.iter(|| {
                    let mut chain = PatchChain::new();
                    for (path, priority) in archives {
                        chain.add_archive(black_box(path), *priority).unwrap();
                    }
                    black_box(chain)
                });
            },
        );

        // Parallel loading
        group.bench_with_input(
            BenchmarkId::new("parallel", count),
            &archives,
            |b, archives| {
                b.iter(|| {
                    let chain =
                        PatchChain::from_archives_parallel(black_box(archives.clone())).unwrap();
                    black_box(chain)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark patch chain file operations
fn bench_patch_chain_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel/patch_chain_operations");

    let (temp_dir, archives) = create_test_archives(8, 50);

    // Create patch chain with overlapping files
    let archive_configs: Vec<_> = archives
        .iter()
        .enumerate()
        .map(|(i, path)| (path.clone(), (i * 100) as i32))
        .collect();

    // Keep temp_dir alive for the duration
    let _temp_dir = temp_dir;

    // Benchmark file reading from chain
    group.bench_function("read_file", |b| {
        // Create chain inside the benchmark
        let mut chain = PatchChain::new();
        for (path, priority) in &archive_configs {
            chain.add_archive(path, *priority).unwrap();
        }

        b.iter(|| {
            let data = chain.read_file("file_025.txt").unwrap();
            black_box(data)
        });
    });

    // Benchmark listing all files
    group.bench_function("list_all", |b| {
        // Create chain inside the benchmark
        let mut chain = PatchChain::new();
        for (path, priority) in &archive_configs {
            chain.add_archive(path, *priority).unwrap();
        }

        b.iter(|| {
            let files = chain.list().unwrap();
            black_box(files)
        });
    });

    // Benchmark finding which archive contains a file
    group.bench_function("find_file_archive", |b| {
        // Create chain inside the benchmark
        let mut chain = PatchChain::new();
        for (path, priority) in &archive_configs {
            chain.add_archive(path, *priority).unwrap();
        }

        b.iter(|| {
            let archive = chain.find_file_archive("file_025.txt").unwrap();
            black_box(archive)
        });
    });

    group.finish();
}

/// Benchmark parallel processing with different CPU core counts
fn bench_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel/scalability");

    let (_temp_dir, archives) = create_test_archives(32, 10);
    let file_name = "file_005.txt";

    // Test with different thread pool sizes
    let thread_counts = vec![1, 2, 4, 8];

    for &threads in &thread_counts {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{threads}_threads")),
            &threads,
            |b, &num_threads| {
                b.iter(|| {
                    use rayon::ThreadPoolBuilder;
                    use wow_mpq::parallel::extract_from_multiple_archives;

                    let pool = ThreadPoolBuilder::new()
                        .num_threads(num_threads)
                        .build()
                        .unwrap();

                    pool.install(|| {
                        let results =
                            extract_from_multiple_archives(black_box(&archives), file_name)
                                .unwrap();
                        black_box(results)
                    })
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_multi_archive_extraction,
    bench_multi_archive_search,
    bench_multi_file_multi_archive,
    bench_custom_processing,
    bench_patch_chain_loading,
    bench_patch_chain_operations,
    bench_scalability
);
criterion_main!(benches);
