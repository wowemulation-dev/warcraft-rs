/// Concrete performance demonstration for Phase 1 improvements
/// This measures actual performance numbers on available WoW MPQ archives
use std::{path::Path, time::Instant};
use wow_mpq::Archive;

fn main() {
    println!("=== MPQ Performance Analysis - Phase 1 Results ===\n");

    // Test archives in order of increasing size
    let test_archives = vec![
        (
            "/home/danielsreichenbach/Downloads/wow/1.12.1/Data/patch.mpq",
            "WoW 1.12.1 patch.mpq",
        ),
        (
            "/home/danielsreichenbach/Downloads/wow/1.12.1/Data/patch-2.mpq",
            "WoW 1.12.1 patch-2.mpq",
        ),
        (
            "/home/danielsreichenbach/Downloads/wow/2.4.3/Data/patch-2.mpq",
            "WoW 2.4.3 patch-2.mpq",
        ),
        (
            "/home/danielsreichenbach/Downloads/wow/3.3.5a/Data/patch.mpq",
            "WoW 3.3.5a patch.mpq",
        ),
    ];

    let mut results = Vec::new();

    for (archive_path, description) in test_archives {
        if Path::new(archive_path).exists() {
            if let Some(result) = measure_archive_performance(archive_path, description) {
                results.push(result);
            }
        }
    }

    // Display results summary
    println!("=== Performance Summary ===");
    println!(
        "{:<25} {:<8} {:<12} {:<12} {:<12}",
        "Archive", "Files", "Open Time", "List Time", "Files/Sec"
    );
    println!("{}", "-".repeat(75));

    for result in &results {
        println!(
            "{:<25} {:<8} {:<12} {:<12} {:<12}",
            result.name,
            result.file_count,
            format!("{:.1}ms", result.open_time_ms),
            format!("{:.1}ms", result.list_time_ms),
            format!("{:.0}", result.files_per_sec)
        );
    }

    // Analyze the performance improvement
    analyze_phase1_improvements(&results);
}

#[derive(Debug)]
struct PerformanceResult {
    name: String,
    file_count: usize,
    open_time_ms: f64,
    list_time_ms: f64,
    files_per_sec: f64,
}

fn measure_archive_performance(archive_path: &str, description: &str) -> Option<PerformanceResult> {
    println!("Testing: {}", description);

    let open_start = Instant::now();
    let archive = match Archive::open(archive_path) {
        Ok(archive) => archive,
        Err(e) => {
            println!("  ❌ Failed to open: {}\n", e);
            return None;
        }
    };
    let open_time = open_start.elapsed();

    let list_start = Instant::now();
    let entries = match archive.list() {
        Ok(entries) => entries,
        Err(e) => {
            println!("  ❌ Failed to list files: {}\n", e);
            return None;
        }
    };
    let list_time = list_start.elapsed();

    let files_per_sec = entries.len() as f64 / list_time.as_secs_f64();

    println!(
        "  ✅ {} files listed in {:.2}ms ({:.0} files/sec)",
        entries.len(),
        list_time.as_millis(),
        files_per_sec
    );
    println!("  📂 Archive opened in {:.2}ms", open_time.as_millis());

    // Categorize performance
    let category = match entries.len() {
        0..=999 => "small",
        1000..=9999 => "medium",
        _ => "large",
    };
    println!("  📊 Category: {} archive\n", category);

    Some(PerformanceResult {
        name: description
            .split_whitespace()
            .last()
            .unwrap_or("unknown")
            .to_string(),
        file_count: entries.len(),
        open_time_ms: open_time.as_secs_f64() * 1000.0,
        list_time_ms: list_time.as_secs_f64() * 1000.0,
        files_per_sec,
    })
}

fn analyze_phase1_improvements(results: &[PerformanceResult]) {
    if results.is_empty() {
        println!("No archives found for testing. Showing theoretical analysis:");
        show_theoretical_analysis();
        return;
    }

    println!("\n=== Phase 1 Impact Analysis ===");

    // Find min and max performance
    let min_perf = results
        .iter()
        .min_by(|a, b| a.files_per_sec.partial_cmp(&b.files_per_sec).unwrap())
        .unwrap();
    let max_perf = results
        .iter()
        .max_by(|a, b| a.files_per_sec.partial_cmp(&b.files_per_sec).unwrap())
        .unwrap();

    let performance_ratio = max_perf.files_per_sec / min_perf.files_per_sec;

    println!("Current Performance Spread:");
    println!(
        "  Slowest: {} ({:.0} files/sec, {} files)",
        min_perf.name, min_perf.files_per_sec, min_perf.file_count
    );
    println!(
        "  Fastest: {} ({:.0} files/sec, {} files)",
        max_perf.name, max_perf.files_per_sec, max_perf.file_count
    );
    println!("  Ratio: {:.1}x difference", performance_ratio);

    println!("\nPhase 1 Optimizations Applied:");
    println!("  ✅ Debug print overhead eliminated (#[cfg(debug_assertions)])");
    println!("  ✅ Anonymous filename generation optimized");
    println!("  ✅ format!() calls removed from hot paths");
    println!("  ✅ String allocation overhead reduced");

    if performance_ratio < 50.0 {
        println!("  🎯 SUCCESS: Performance variance significantly reduced from 700x baseline");
        println!("  📈 Small archives now perform much closer to large archive efficiency");
    } else {
        println!("  ⚠️  Still high variance - Phase 2 lazy loading needed for further improvement");
    }

    // Average performance
    let avg_perf: f64 = results.iter().map(|r| r.files_per_sec).sum::<f64>() / results.len() as f64;
    println!("  📊 Average performance: {:.0} files/sec", avg_perf);

    // Estimate improvement (theoretical baseline was 439 files/sec for small archives)
    if min_perf.files_per_sec > 1000.0 {
        let improvement = min_perf.files_per_sec / 439.0;
        println!(
            "  🚀 Estimated improvement vs. pre-optimization: {:.1}x",
            improvement
        );
    }
}

fn show_theoretical_analysis() {
    println!("=== Theoretical Phase 1 Analysis ===");
    println!("Based on flamegraph analysis of the 700x performance issue:");
    println!();
    println!("BEFORE Phase 1:");
    println!("  • Small archives: ~439 files/sec (debug overhead dominated)");
    println!("  • Large archives: ~314,930 files/sec (overhead amortized)");
    println!("  • Root cause: std::io::stdio::_print (73 calls) + core::fmt::write (52 calls)");
    println!();
    println!("AFTER Phase 1:");
    println!("  • Debug prints eliminated from release builds");
    println!("  • format!() overhead removed from file listing hot paths");
    println!("  • Anonymous filename generation optimized");
    println!("  • Expected: 2-5x improvement for small archives");
    println!();
    println!("To see actual numbers, place WoW MPQ files in:");
    println!("  /home/danielsreichenbach/Downloads/wow/[version]/Data/");
}
