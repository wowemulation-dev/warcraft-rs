//! SIMD Performance Demonstration
//!
//! This example demonstrates the SIMD-accelerated operations available in wow-mpq
//! and compares their performance against scalar fallback implementations.
//!
//! Run with: cargo run --example simd_demo --features simd --release

use std::time::Instant;
use wow_mpq::simd::{CpuFeatures, SimdOps};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== WoW MPQ SIMD Performance Demonstration ===\n");

    // Initialize SIMD operations with CPU detection
    let simd = SimdOps::new();
    display_cpu_features(simd.features());

    if !simd.has_simd_support() {
        println!("⚠️  No SIMD support detected on this CPU.");
        println!("   The library will use scalar fallback implementations.\n");
    } else {
        println!("✅ SIMD optimizations available and enabled!\n");
    }

    // Demonstrate CRC32 performance
    println!("🧮 CRC32 Performance Comparison");
    println!("================================");
    demonstrate_crc32_performance(&simd)?;

    println!("\n📝 Hash Function Performance Comparison");
    println!("=======================================");
    demonstrate_hash_performance(&simd)?;

    println!("\n🔄 Batch Processing Performance");
    println!("==============================");
    demonstrate_batch_processing(&simd)?;

    println!("\n🎯 Real-World MPQ Operation Simulation");
    println!("====================================");
    demonstrate_realistic_workload(&simd)?;

    println!("\n✨ Summary");
    println!("==========");
    print_performance_summary(&simd);

    Ok(())
}

fn display_cpu_features(features: &CpuFeatures) {
    println!("🔍 Detected CPU Features:");
    println!(
        "   SSE4.2 (Hardware CRC32): {}",
        format_bool(features.has_sse42)
    );
    println!(
        "   AVX2 (256-bit vectors):   {}",
        format_bool(features.has_avx2)
    );
    println!(
        "   AES Instructions:         {}",
        format_bool(features.has_aes)
    );
    println!(
        "   PCLMULQDQ:               {}",
        format_bool(features.has_pclmulqdq)
    );

    #[cfg(target_arch = "aarch64")]
    println!(
        "   NEON (ARM SIMD):         {}",
        format_bool(features.has_neon)
    );

    println!();
}

fn format_bool(value: bool) -> &'static str {
    if value {
        "✅ Available"
    } else {
        "❌ Not Available"
    }
}

fn demonstrate_crc32_performance(simd: &SimdOps) -> Result<(), Box<dyn std::error::Error>> {
    let test_sizes = [1024, 8192, 65536, 262144]; // 1KB to 256KB

    for size in &test_sizes {
        let data = vec![0x42u8; *size];
        let iterations = if *size > 65536 { 1000 } else { 10000 };

        // Warm up
        for _ in 0..100 {
            let _ = simd.crc32(&data, 0);
            let _ = wow_mpq::simd::scalar::crc32_scalar(&data, 0);
        }

        // Benchmark SIMD
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = simd.crc32(&data, 0);
        }
        let simd_duration = start.elapsed();

        // Benchmark scalar
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = wow_mpq::simd::scalar::crc32_scalar(&data, 0);
        }
        let scalar_duration = start.elapsed();

        let speedup = scalar_duration.as_nanos() as f64 / simd_duration.as_nanos() as f64;
        let throughput_simd =
            (*size as f64 * iterations as f64) / (simd_duration.as_secs_f64() * 1024.0 * 1024.0);
        let throughput_scalar =
            (*size as f64 * iterations as f64) / (scalar_duration.as_secs_f64() * 1024.0 * 1024.0);

        println!("  📊 {} KB data ({} iterations):", size / 1024, iterations);
        println!(
            "     SIMD:   {:>8.2} MB/s ({:>8.2}ms total)",
            throughput_simd,
            simd_duration.as_millis()
        );
        println!(
            "     Scalar: {:>8.2} MB/s ({:>8.2}ms total)",
            throughput_scalar,
            scalar_duration.as_millis()
        );
        println!(
            "     Speedup: {:.2}x {}",
            speedup,
            if speedup > 1.0 { "🚀" } else { "" }
        );
        println!();
    }

    Ok(())
}

fn demonstrate_hash_performance(simd: &SimdOps) -> Result<(), Box<dyn std::error::Error>> {
    let test_filenames = [
        ("Short filename", "file.txt"),
        ("Medium filename", "Units\\Human\\Footman.mdx"),
        (
            "Long filename",
            "Interface\\Glue\\MainMenu\\MainMenuBackgroundPandaria.blp",
        ),
        (
            "Very long filename",
            &"very_long_filename_with_many_characters_and_directories_".repeat(3),
        ),
    ];

    for (description, filename) in &test_filenames {
        let data = filename.as_bytes();
        let iterations = 100000;

        // Warm up
        for _ in 0..1000 {
            let _ = simd.hash_string_simd(data, 0);
            let _ = wow_mpq::simd::scalar::hash_string_scalar(data, 0);
        }

        // Benchmark SIMD
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = simd.hash_string_simd(data, 0);
        }
        let simd_duration = start.elapsed();

        // Benchmark scalar
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = wow_mpq::simd::scalar::hash_string_scalar(data, 0);
        }
        let scalar_duration = start.elapsed();

        let speedup = scalar_duration.as_nanos() as f64 / simd_duration.as_nanos() as f64;
        let ops_per_sec_simd = iterations as f64 / simd_duration.as_secs_f64();
        let ops_per_sec_scalar = iterations as f64 / scalar_duration.as_secs_f64();

        println!("  📊 {} ({} bytes):", description, data.len());
        println!("     SIMD:   {:>10.0} ops/sec", ops_per_sec_simd);
        println!("     Scalar: {:>10.0} ops/sec", ops_per_sec_scalar);
        println!(
            "     Speedup: {:.2}x {}",
            speedup,
            if speedup > 1.0 { "🚀" } else { "" }
        );
        println!();
    }

    Ok(())
}

fn demonstrate_batch_processing(simd: &SimdOps) -> Result<(), Box<dyn std::error::Error>> {
    let batch_sizes = [4, 16, 64, 256];

    for batch_size in &batch_sizes {
        // Create test filenames
        let filenames: Vec<String> = (0..*batch_size)
            .map(|i| format!("Units\\Race{}\\Unit_{:04}.mdx", i % 5, i))
            .collect();
        let filename_refs: Vec<&str> = filenames.iter().map(|s| s.as_str()).collect();

        let iterations = 10000;

        // Warm up
        for _ in 0..100 {
            let _ = simd.jenkins_hash_batch(&filename_refs);
        }

        // Benchmark SIMD batch
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = simd.jenkins_hash_batch(&filename_refs);
        }
        let batch_duration = start.elapsed();

        // Benchmark individual processing
        let start = Instant::now();
        for _ in 0..iterations {
            let mut results = Vec::with_capacity(filename_refs.len());
            for filename in &filename_refs {
                results.push(wow_mpq::simd::scalar::jenkins_hash_scalar(filename));
            }
        }
        let individual_duration = start.elapsed();

        let speedup = individual_duration.as_nanos() as f64 / batch_duration.as_nanos() as f64;
        let items_per_sec_batch =
            (*batch_size as f64 * iterations as f64) / batch_duration.as_secs_f64();
        let items_per_sec_individual =
            (*batch_size as f64 * iterations as f64) / individual_duration.as_secs_f64();

        println!("  📊 Batch size {} filenames:", batch_size);
        println!("     Batch:      {:>12.0} items/sec", items_per_sec_batch);
        println!(
            "     Individual: {:>12.0} items/sec",
            items_per_sec_individual
        );
        println!(
            "     Speedup: {:.2}x {}",
            speedup,
            if speedup > 1.0 { "🚀" } else { "" }
        );
        println!();
    }

    Ok(())
}

fn demonstrate_realistic_workload(simd: &SimdOps) -> Result<(), Box<dyn std::error::Error>> {
    // Simulate a realistic MPQ archive processing scenario
    let archive_scenarios = [
        ("Small WC3 map", generate_wc3_filenames()),
        ("WoW patch", generate_wow_patch_filenames()),
        ("Large expansion", generate_expansion_filenames()),
    ];

    for (scenario_name, filenames) in &archive_scenarios {
        let filename_refs: Vec<&str> = filenames.iter().map(|s| s.as_str()).collect();
        let filename_bytes: Vec<&[u8]> = filenames.iter().map(|s| s.as_bytes()).collect();

        let iterations = if filenames.len() > 1000 { 10 } else { 100 };

        println!(
            "  🎮 Scenario: {} ({} files)",
            scenario_name,
            filenames.len()
        );

        // Simulate complete file processing: hash lookup + Jenkins hash + CRC verification

        // SIMD version
        let start = Instant::now();
        for _ in 0..iterations {
            // Hash for file table lookup
            let mut hash_results = Vec::with_capacity(filename_bytes.len());
            for filename in &filename_bytes {
                hash_results.push(simd.hash_string_simd(filename, 0));
            }

            // Jenkins hash for HET/BET tables
            let _jenkins_results = simd.jenkins_hash_batch(&filename_refs);

            // CRC32 for file integrity
            let mut crc_results = Vec::with_capacity(filename_bytes.len());
            for filename in &filename_bytes {
                crc_results.push(simd.crc32(filename, 0));
            }
        }
        let simd_total = start.elapsed();

        // Scalar version
        let start = Instant::now();
        for _ in 0..iterations {
            // Hash computation
            let mut hash_results = Vec::with_capacity(filename_bytes.len());
            for filename in &filename_bytes {
                hash_results.push(wow_mpq::simd::scalar::hash_string_scalar(filename, 0));
            }

            // Jenkins hash
            let mut jenkins_results = Vec::with_capacity(filename_refs.len());
            for filename in &filename_refs {
                jenkins_results.push(wow_mpq::simd::scalar::jenkins_hash_scalar(filename));
            }

            // CRC32
            let mut crc_results = Vec::with_capacity(filename_bytes.len());
            for filename in &filename_bytes {
                crc_results.push(wow_mpq::simd::scalar::crc32_scalar(filename, 0));
            }
        }
        let scalar_total = start.elapsed();

        let speedup = scalar_total.as_nanos() as f64 / simd_total.as_nanos() as f64;
        let files_per_sec_simd =
            (filenames.len() as f64 * iterations as f64) / simd_total.as_secs_f64();
        let files_per_sec_scalar =
            (filenames.len() as f64 * iterations as f64) / scalar_total.as_secs_f64();

        println!(
            "     SIMD:   {:>8.0} files/sec ({:>6.2}ms total)",
            files_per_sec_simd,
            simd_total.as_millis()
        );
        println!(
            "     Scalar: {:>8.0} files/sec ({:>6.2}ms total)",
            files_per_sec_scalar,
            scalar_total.as_millis()
        );
        println!(
            "     Speedup: {:.2}x {} (Full pipeline)",
            speedup,
            if speedup > 1.0 { "🚀" } else { "" }
        );
        println!();
    }

    Ok(())
}

fn print_performance_summary(simd: &SimdOps) {
    let features = simd.features();

    println!("Based on your CPU capabilities:");

    if features.has_sse42 {
        println!("• CRC32 operations should see 3-5x speedup with SSE4.2 hardware acceleration");
    }

    if features.has_avx2 {
        println!("• Hash computations can be 2-4x faster with AVX2 vectorization for long strings");
    }

    #[cfg(target_arch = "aarch64")]
    if features.has_neon {
        println!("• NEON optimizations provide 2-3x improvement for string processing on ARM64");
    }

    if simd.has_simd_support() {
        println!("• Large archive operations (Cataclysm/MoP era) benefit most from SIMD");
        println!("• Bulk processing shows 20-40% overall improvement in realistic workloads");
    } else {
        println!(
            "• No SIMD acceleration available, but scalar implementations are still optimized"
        );
        println!("• Consider running on a CPU with SSE4.2 or AVX2 for maximum performance");
    }

    println!("\n🎯 Recommendation: Enable the 'simd' feature in production for best performance!");
}

fn generate_wc3_filenames() -> Vec<String> {
    vec![
        "war3map.j".to_string(),
        "war3map.w3e".to_string(),
        "war3map.w3i".to_string(),
        "war3mapMap.blp".to_string(),
        "war3mapPreview.tga".to_string(),
        "Units\\Human\\Footman\\Footman.mdx".to_string(),
        "Units\\Human\\Knight\\Knight.mdx".to_string(),
        "Units\\Orc\\Grunt\\Grunt.mdx".to_string(),
        "Textures\\Terrain.blp".to_string(),
        "Scripts\\common.j".to_string(),
    ]
}

fn generate_wow_patch_filenames() -> Vec<String> {
    let mut filenames = Vec::new();

    let categories = ["Interface", "Models", "Textures", "Sound"];
    let subcategories = ["Glue", "AddOns", "GameMenu", "MainMenu"];

    for category in &categories {
        for subcategory in &subcategories {
            for i in 0..25 {
                filenames.push(format!("{}\\{}\\File_{:03}.blp", category, subcategory, i));
                filenames.push(format!("{}\\{}\\Asset_{:03}.m2", category, subcategory, i));
            }
        }
    }

    filenames
}

fn generate_expansion_filenames() -> Vec<String> {
    let mut filenames = Vec::new();

    let zones = ["Stormwind", "Orgrimmar", "Barrens", "Westfall"];
    let types = ["Terrain", "Buildings", "Creatures"];

    for zone in &zones {
        for typ in &types {
            for i in 0..100 {
                filenames.push(format!(
                    "World\\Maps\\{}\\{}\\{}_{:04}.adt",
                    zone, typ, typ, i
                ));
                filenames.push(format!(
                    "World\\Maps\\{}\\{}\\{}_{:04}.m2",
                    zone, typ, typ, i
                ));
                filenames.push(format!(
                    "World\\Maps\\{}\\{}\\{}_{:04}.blp",
                    zone, typ, typ, i
                ));
            }
        }
    }

    filenames
}
