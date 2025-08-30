use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use wow_mpq::simd::{CpuFeatures, SimdOps};

/// Benchmark SIMD CRC32 performance vs scalar implementation
fn bench_crc32_performance(c: &mut Criterion) {
    let simd = SimdOps::new();

    // Test data of various sizes
    let sizes = [64, 256, 1024, 4096, 16384, 65536, 262144]; // 64B to 256KB

    let mut group = c.benchmark_group("crc32_performance");

    for size in sizes {
        let data = vec![0x42u8; size];

        group.throughput(Throughput::Bytes(size as u64));

        // Benchmark SIMD implementation
        group.bench_with_input(BenchmarkId::new("simd", size), &data, |b, data| {
            b.iter(|| simd.crc32(black_box(data), black_box(0)))
        });

        // Benchmark scalar fallback for comparison
        group.bench_with_input(BenchmarkId::new("scalar", size), &data, |b, data| {
            b.iter(|| wow_mpq::simd::scalar::crc32_scalar(black_box(data), black_box(0)))
        });
    }

    group.finish();
}

/// Benchmark SIMD hash performance vs scalar implementation
fn bench_hash_performance(c: &mut Criterion) {
    let simd = SimdOps::new();

    // Test different filename lengths
    let filenames = [
        "a.txt",
        "file.mdx",
        "Units\\Human\\Footman.mdx",
        "Interface\\Glue\\MainMenu\\MainMenuBackgroundPandaria.blp",
        &"very_long_filename_with_many_characters_".repeat(4),
        &"x".repeat(200), // Very long filename
    ];

    let mut group = c.benchmark_group("hash_performance");

    for filename in &filenames {
        let data = filename.as_bytes();

        group.throughput(Throughput::Bytes(data.len() as u64));

        // Benchmark SIMD implementation
        group.bench_with_input(BenchmarkId::new("simd", data.len()), &data, |b, data| {
            b.iter(|| simd.hash_string_simd(black_box(data), black_box(0)))
        });

        // Benchmark scalar fallback for comparison
        group.bench_with_input(BenchmarkId::new("scalar", data.len()), &data, |b, data| {
            b.iter(|| wow_mpq::simd::scalar::hash_string_scalar(black_box(data), black_box(0)))
        });
    }

    group.finish();
}

/// Benchmark Jenkins hash batch processing performance
fn bench_jenkins_batch_performance(c: &mut Criterion) {
    let simd = SimdOps::new();

    // Different batch sizes
    let batch_sizes = [1, 4, 16, 64, 256];

    let mut group = c.benchmark_group("jenkins_batch_performance");

    for batch_size in batch_sizes {
        // Create test filenames
        let filenames: Vec<String> = (0..batch_size)
            .map(|i| format!("Units\\Human\\Footman_{:04}.mdx", i))
            .collect();
        let filename_refs: Vec<&str> = filenames.iter().map(|s| s.as_str()).collect();

        group.throughput(Throughput::Elements(batch_size as u64));

        // Benchmark SIMD batch processing
        group.bench_with_input(
            BenchmarkId::new("simd_batch", batch_size),
            &filename_refs,
            |b, filenames| b.iter(|| simd.jenkins_hash_batch(black_box(filenames))),
        );

        // Benchmark individual processing for comparison
        group.bench_with_input(
            BenchmarkId::new("individual", batch_size),
            &filename_refs,
            |b, filenames| {
                b.iter(|| {
                    let mut results = Vec::with_capacity(filenames.len());
                    for filename in filenames {
                        results.push(wow_mpq::simd::scalar::jenkins_hash_scalar(black_box(
                            filename,
                        )));
                    }
                    results
                })
            },
        );
    }

    group.finish();
}

/// Benchmark real-world MPQ operations with SIMD
fn bench_realistic_workload(c: &mut Criterion) {
    let simd = SimdOps::new();

    // Simulate realistic MPQ workloads
    let test_cases = [
        ("small_archive", generate_small_archive_filenames()),
        ("medium_archive", generate_medium_archive_filenames()),
        ("large_archive", generate_large_archive_filenames()),
    ];

    let mut group = c.benchmark_group("realistic_workload");

    for (name, filenames) in &test_cases {
        let filename_refs: Vec<&str> = filenames.iter().map(|s| s.as_str()).collect();
        let filename_bytes: Vec<&[u8]> = filenames.iter().map(|s| s.as_bytes()).collect();

        group.throughput(Throughput::Elements(filenames.len() as u64));

        // Benchmark combined hash + CRC operations (simulating file processing)
        group.bench_with_input(
            BenchmarkId::new("simd_combined", name),
            &(filename_bytes.as_slice(), filename_refs.as_slice()),
            |b, (bytes, refs)| {
                b.iter(|| {
                    // Hash computation for file lookup
                    let mut hash_results = Vec::with_capacity(bytes.len());
                    for filename in *bytes {
                        hash_results.push(simd.hash_string_simd(black_box(filename), 0));
                    }

                    // Jenkins hash for HET/BET tables
                    let jenkins_results = simd.jenkins_hash_batch(black_box(*refs));

                    // CRC32 for file verification
                    let mut crc_results = Vec::with_capacity(bytes.len());
                    for filename in *bytes {
                        crc_results.push(simd.crc32(black_box(filename), 0));
                    }

                    (hash_results, jenkins_results, crc_results)
                })
            },
        );

        // Benchmark scalar equivalent
        group.bench_with_input(
            BenchmarkId::new("scalar_combined", name),
            &(filename_bytes.as_slice(), filename_refs.as_slice()),
            |b, (bytes, refs)| {
                b.iter(|| {
                    // Hash computation
                    let mut hash_results = Vec::with_capacity(bytes.len());
                    for filename in *bytes {
                        hash_results.push(wow_mpq::simd::scalar::hash_string_scalar(
                            black_box(filename),
                            0,
                        ));
                    }

                    // Jenkins hash
                    let mut jenkins_results = Vec::with_capacity(refs.len());
                    for filename in *refs {
                        jenkins_results.push(wow_mpq::simd::scalar::jenkins_hash_scalar(
                            black_box(filename),
                        ));
                    }

                    // CRC32
                    let mut crc_results = Vec::with_capacity(bytes.len());
                    for filename in *bytes {
                        crc_results
                            .push(wow_mpq::simd::scalar::crc32_scalar(black_box(filename), 0));
                    }

                    (hash_results, jenkins_results, crc_results)
                })
            },
        );
    }

    group.finish();
}

/// Display detected CPU features and expected performance
fn bench_cpu_feature_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("cpu_detection");

    // Benchmark CPU feature detection overhead
    group.bench_function("feature_detection", |b| {
        b.iter(|| {
            let features = CpuFeatures::default();
            black_box(features)
        })
    });

    // Display detected features
    let simd = SimdOps::new();
    let features = simd.features();

    println!("\n=== Detected CPU Features ===");
    println!("SSE4.2 (CRC32): {}", features.has_sse42);
    println!("AVX2: {}", features.has_avx2);
    println!("AES: {}", features.has_aes);
    println!("PCLMULQDQ: {}", features.has_pclmulqdq);

    #[cfg(target_arch = "aarch64")]
    println!("NEON: {}", features.has_neon);

    println!("SIMD Support Available: {}", simd.has_simd_support());
    println!("==============================\n");

    group.finish();
}

/// Generate test data for small archive (typical for older games)
fn generate_small_archive_filenames() -> Vec<String> {
    let mut filenames = Vec::new();

    // Typical small MPQ contents
    for i in 0..50 {
        filenames.push("war3map.w3e".to_string());
        filenames.push("war3map.w3i".to_string());
        filenames.push(format!("Scripts\\war3map_{}.j", i));
    }

    filenames
}

/// Generate test data for medium archive (WoW patches)  
fn generate_medium_archive_filenames() -> Vec<String> {
    let mut filenames = Vec::new();

    // Typical WoW patch contents
    let categories = ["Units", "Interface", "Textures", "Models", "Sound"];
    let subcategories = ["Human", "Orc", "NightElf", "Undead", "Neutral"];
    let extensions = [".mdx", ".blp", ".wav", ".m2"];

    for category in &categories {
        for subcategory in &subcategories {
            for i in 0..100 {
                for ext in &extensions {
                    filenames.push(format!(
                        "{}\\{}\\File_{:04}{}",
                        category, subcategory, i, ext
                    ));
                }
            }
        }
    }

    filenames
}

/// Generate test data for large archive (Cataclysm/MoP era)
fn generate_large_archive_filenames() -> Vec<String> {
    let mut filenames = Vec::new();

    // Very large archive simulation
    let zones = [
        "Stormwind",
        "Orgrimmar",
        "Ironforge",
        "Undercity",
        "Darnassus",
        "ThunderBluff",
    ];
    let types = [
        "Terrain",
        "Buildings",
        "Creatures",
        "Effects",
        "Textures",
        "Audio",
    ];
    let formats = [".adt", ".m2", ".wmo", ".blp", ".ogg", ".mp3"];

    for zone in &zones {
        for typ in &types {
            for i in 0..500 {
                for format in &formats {
                    filenames.push(format!(
                        "World\\Maps\\{}\\{}\\{}_{:06}{}",
                        zone, typ, typ, i, format
                    ));
                }
            }
        }
    }

    filenames
}

criterion_group!(
    benches,
    bench_cpu_feature_detection,
    bench_crc32_performance,
    bench_hash_performance,
    bench_jenkins_batch_performance,
    bench_realistic_workload
);

criterion_main!(benches);
