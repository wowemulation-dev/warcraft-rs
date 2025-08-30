use criterion::{Criterion, criterion_group, criterion_main};
use std::path::Path;
use std::time::Instant;
use wow_mpq::{Archive, OpenOptions};

fn find_available_mpq() -> Option<String> {
    let test_paths = [
        "/home/danielsreichenbach/Downloads/wow/1.12.1/Data/patch.MPQ",
        "/home/danielsreichenbach/Downloads/wow/2.4.3/Data/patch.MPQ",
        "/home/danielsreichenbach/Downloads/wow/3.3.5a/Data/patch.MPQ",
        "/home/danielsreichenbach/Downloads/wow/1.12.1/Data/common.MPQ",
        "/home/danielsreichenbach/Downloads/wow/2.4.3/Data/common.MPQ",
    ];

    for path in &test_paths {
        if Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    None
}

fn benchmark_archive_open_lazy(c: &mut Criterion) {
    let mpq_path = match find_available_mpq() {
        Some(path) => path,
        None => {
            println!("No MPQ files found for testing");
            return;
        }
    };

    println!("Testing with: {}", mpq_path);

    c.bench_function("archive_open_lazy", |b| {
        b.iter(|| {
            let archive = Archive::open_with_options(
                std::hint::black_box(&mpq_path),
                OpenOptions::new().load_tables(false),
            )
            .expect("Failed to open archive");
            std::hint::black_box(archive);
        })
    });
}

fn benchmark_archive_open_eager(c: &mut Criterion) {
    let mpq_path = match find_available_mpq() {
        Some(path) => path,
        None => {
            println!("No MPQ files found for testing");
            return;
        }
    };

    c.bench_function("archive_open_eager", |b| {
        b.iter(|| {
            let archive = Archive::open_with_options(
                std::hint::black_box(&mpq_path),
                OpenOptions::new().load_tables(true),
            )
            .expect("Failed to open archive");
            std::hint::black_box(archive);
        })
    });
}

fn benchmark_single_file_access(c: &mut Criterion) {
    let mpq_path = match find_available_mpq() {
        Some(path) => path,
        None => {
            println!("No MPQ files found for testing");
            return;
        }
    };

    // Common files likely to exist in patch archives
    let test_files = [
        "(listfile)",
        "(attributes)",
        "interface\\glues\\models\\ui_mainmenu\\ui_mainmenu.m2",
        "sound\\music\\zones\\tavern_01.mp3",
    ];

    let archive = Archive::open_with_options(&mpq_path, OpenOptions::new().load_tables(false))
        .expect("Failed to open archive");

    c.bench_function("single_file_access_lazy", |b| {
        b.iter(|| {
            for filename in &test_files {
                if let Ok(Some(_file_info)) = archive.find_file(std::hint::black_box(filename)) {
                    break; // Found a file, that's enough for benchmark
                }
            }
        })
    });
}

fn measure_archive_sizes() {
    println!("\n=== Archive Size Analysis ===");

    let test_paths = [
        "/home/danielsreichenbach/Downloads/wow/1.12.1/Data/patch.MPQ",
        "/home/danielsreichenbach/Downloads/wow/2.4.3/Data/patch.MPQ",
        "/home/danielsreichenbach/Downloads/wow/3.3.5a/Data/patch.MPQ",
        "/home/danielsreichenbach/Downloads/wow/1.12.1/Data/common.MPQ",
        "/home/danielsreichenbach/Downloads/wow/2.4.3/Data/common.MPQ",
    ];

    for path in &test_paths {
        if Path::new(path).exists() {
            let start = Instant::now();
            match Archive::open_with_options(path, OpenOptions::new().load_tables(false)) {
                Ok(_archive) => {
                    let lazy_time = start.elapsed();

                    let start = Instant::now();
                    match Archive::open_with_options(path, OpenOptions::new().load_tables(true)) {
                        Ok(_) => {
                            let eager_time = start.elapsed();
                            let file_size = std::fs::metadata(path).unwrap().len();

                            println!(
                                "Archive: {} ({:.2} MB)",
                                path.split('/').next_back().unwrap_or(path),
                                file_size as f64 / 1024.0 / 1024.0
                            );
                            println!("  Lazy open:  {:?}", lazy_time);
                            println!("  Eager open: {:?}", eager_time);
                            println!(
                                "  Speedup:    {:.1}x",
                                eager_time.as_micros() as f64 / lazy_time.as_micros().max(1) as f64
                            );
                            println!();
                        }
                        Err(e) => println!("  Failed eager open: {}", e),
                    }
                }
                Err(e) => println!("  Failed lazy open: {}", e),
            }
        }
    }
}

criterion_group!(
    benches,
    benchmark_archive_open_lazy,
    benchmark_archive_open_eager,
    benchmark_single_file_access
);

criterion_main!(benches);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measure_archive_sizes() {
        measure_archive_sizes();
    }
}
