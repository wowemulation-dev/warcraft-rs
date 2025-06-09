use std::env;
use wow_mpq::Archive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <archive.mpq>", args[0]);
        std::process::exit(1);
    }

    let archive_path = &args[1];
    println!("=== Path Separator Testing ===");
    println!("Archive: {}", archive_path);

    let mut archive = Archive::open(archive_path)?;
    println!("Archive opened successfully!");

    // Test files with different path separators
    let test_paths = [
        // Backslashes (MPQ native format)
        ("data\\binary.dat", "backslash path"),
        ("assets\\large.bin", "backslash path"),
        // Forward slashes (Unix-style)
        ("data/binary.dat", "forward slash path"),
        ("assets/large.bin", "forward slash path"),
        // Mixed separators
        ("data/binary.dat", "mixed separators"),
        ("assets\\large.bin", "mixed separators"),
    ];

    for (path, description) in &test_paths {
        println!("\nTrying to read: {} ({})", path, description);
        match archive.read_file(path) {
            Ok(data) => {
                println!("  ✓ Successfully read {} bytes", data.len());
            }
            Err(e) => {
                println!("  ✗ Failed to read: {}", e);
            }
        }
    }

    // Also test some files that should definitely exist
    println!("\n=== Testing known files ===");
    let known_files = ["readme.txt", "(listfile)", "(attributes)"];

    for filename in &known_files {
        println!("\nTrying to read: {}", filename);
        match archive.read_file(filename) {
            Ok(data) => {
                println!("  ✓ Successfully read {} bytes", data.len());
            }
            Err(e) => {
                println!("  ✗ Failed to read: {}", e);
            }
        }
    }

    Ok(())
}
