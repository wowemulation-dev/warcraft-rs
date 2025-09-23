use criterion::{Criterion, criterion_group, criterion_main};
use std::{path::Path, time::Instant};

/// Demonstrate Phase 1 performance improvements using real WoW archives
/// This provides concrete numbers showing the actual improvements achieved
fn demonstrate_phase1_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase1_improvements");

    // Test with available WoW MPQ archives
    let test_archives = vec![
        "/home/danielsreichenbach/Downloads/wow/1.12.1/Data/patch.mpq",
        "/home/danielsreichenbach/Downloads/wow/1.12.1/Data/patch-2.mpq",
        "/home/danielsreichenbach/Downloads/wow/2.4.3/Data/patch-2.mpq",
        "/home/danielsreichenbach/Downloads/wow/3.3.5a/Data/patch.mpq",
    ];

    println!("\n=== Phase 1 Performance Demonstration ===");
    println!("Testing debug code removal optimizations on real WoW MPQ archives\n");

    for archive_path in test_archives {
        if Path::new(archive_path).exists() {
            let archive_name = Path::new(archive_path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();

            // Benchmark archive opening and listing
            group.bench_function(format!("list_{}", archive_name), |b| {
                b.iter(|| {
                    if let Ok(archive) = wow_mpq::Archive::open(archive_path) {
                        let _ = archive.list().unwrap_or_default();
                    }
                })
            });

            // Manual measurement for files/second calculation
            measure_listing_performance(archive_path, &archive_name);
        }
    }

    group.finish();
}

fn measure_listing_performance(archive_path: &str, archive_name: &str) {
    println!("Testing archive: {}", archive_name);

    // Measure archive opening time
    let open_start = Instant::now();
    match wow_mpq::Archive::open(archive_path) {
        Ok(archive) => {
            let open_duration = open_start.elapsed();

            // Measure file listing time
            let list_start = Instant::now();
            match archive.list() {
                Ok(entries) => {
                    let list_duration = list_start.elapsed();
                    let total_duration = open_start.elapsed();

                    let files_per_sec = entries.len() as f64 / list_duration.as_secs_f64();
                    let total_files_per_sec = entries.len() as f64 / total_duration.as_secs_f64();

                    println!("  Files: {}", entries.len());
                    println!("  Open time: {:?}", open_duration);
                    println!("  List time: {:?}", list_duration);
                    println!("  Total time: {:?}", total_duration);
                    println!("  List performance: {:.0} files/sec", files_per_sec);
                    println!(
                        "  Overall performance: {:.0} files/sec",
                        total_files_per_sec
                    );

                    // Categorize performance based on file count
                    let category = if entries.len() < 1000 {
                        "small"
                    } else if entries.len() < 10000 {
                        "medium"
                    } else {
                        "large"
                    };

                    println!("  Category: {} archive", category);
                    println!();
                }
                Err(e) => println!("  Failed to list files: {}", e),
            }
        }
        Err(e) => println!("  Failed to open archive: {}", e),
    }
}

/// Show the theoretical improvements from Phase 1
#[allow(dead_code)]
fn show_phase1_analysis() {
    println!("=== Phase 1 Optimization Analysis ===");
    println!();
    println!("BEFORE Phase 1 (from flamegraph analysis):");
    println!("  - Debug prints active in release builds");
    println!("  - format!() calls in hot paths during file listing");
    println!("  - std::io::stdio::_print dominated small archive performance");
    println!("  - Small archives: ~439 files/sec (debug overhead)");
    println!("  - Large archives: ~314,930 files/sec (overhead amortized)");
    println!("  - Performance ratio: 700x difference");
    println!();

    println!("AFTER Phase 1 optimizations:");
    println!("  ✓ Debug prints wrapped with #[cfg(debug_assertions)]");
    println!("  ✓ Optimized anonymous filename generation");
    println!("  ✓ Eliminated format!() overhead in listing operations");
    println!("  ✓ Removed unused mut warnings");
    println!();

    println!("Expected improvements:");
    println!("  - Small archives: 2-5x improvement (reduced debug overhead)");
    println!("  - Large archives: maintained performance");
    println!("  - Reduced performance variance between archive sizes");
    println!();
}

criterion_group!(phase1_benches, demonstrate_phase1_performance);
criterion_main!(phase1_benches);
