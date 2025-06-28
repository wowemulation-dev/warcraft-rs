use std::error::Error;
use wow_mpq::Archive;

fn test_archive(path: &str, expected_files: &[(&str, usize)]) -> Result<(), Box<dyn Error>> {
    println!("\nTesting archive: {path}");

    let mut archive = Archive::open(path)?;
    let info = archive.get_info()?;

    println!("Format: {:?}", info.format_version);
    println!("Files: {}", info.file_count);
    println!("Has listfile: {}", info.has_listfile);
    println!("Has attributes: {}", info.has_attributes);

    // Test reading each expected file
    for (filename, expected_size) in expected_files {
        match archive.read_file(filename) {
            Ok(data) => {
                if data.len() == *expected_size {
                    println!("✓ {}: {} bytes (correct)", filename, data.len());
                } else {
                    println!(
                        "✗ {}: {} bytes (expected {})",
                        filename,
                        data.len(),
                        expected_size
                    );
                }
            }
            Err(e) => {
                println!("✗ {filename}: Error - {e}");
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Cross-Compatibility Test ===");
    println!("Testing if wow-mpq can read StormLib archives and vice versa");

    let expected_files = [
        ("readme.txt", 54),
        ("data/binary.dat", 1024),
        ("assets/large.bin", 102400),
    ];

    // Also try with backslashes
    let expected_files_backslash = [
        ("readme.txt", 54),
        ("data\\binary.dat", 1024),
        ("assets\\large.bin", 102400),
    ];

    // Test StormLib archives with wow-mpq
    println!("\n--- wow-mpq reading StormLib archives (forward slashes) ---");
    test_archive("stormlib_v1.mpq", &expected_files)?;
    test_archive("stormlib_v2.mpq", &expected_files)?;
    test_archive("stormlib_v3.mpq", &expected_files)?;
    test_archive("stormlib_v4.mpq", &expected_files)?;

    println!("\n--- wow-mpq reading StormLib archives (backslashes) ---");
    test_archive("stormlib_v1.mpq", &expected_files_backslash)?;
    test_archive("stormlib_v2.mpq", &expected_files_backslash)?;
    test_archive("stormlib_v3.mpq", &expected_files_backslash)?;
    test_archive("stormlib_v4.mpq", &expected_files_backslash)?;

    // Test wow-mpq archives with wow-mpq (control)
    println!("\n--- wow-mpq reading wow-mpq archives ---");
    test_archive("wowmpq_v1.mpq", &expected_files)?;
    test_archive("wowmpq_v2.mpq", &expected_files)?;
    test_archive("wowmpq_v3.mpq", &expected_files)?;
    test_archive("wowmpq_v4.mpq", &expected_files)?;

    Ok(())
}
