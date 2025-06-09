use wow_mpq::Archive;

fn test_archive(path: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Testing wow-mpq reading {} ===", name);

    match Archive::open(path) {
        Ok(mut archive) => {
            println!("Archive opened successfully!");

            // List all files from listfile
            if let Ok(listfile) = archive.read_file("(listfile)") {
                let listfile_content = String::from_utf8_lossy(&listfile);
                println!("Files in archive:");
                for line in listfile_content.lines() {
                    if !line.trim().is_empty() {
                        println!("  {}", line.trim());
                    }
                }
            }

            // Try to read attributes file directly
            match archive.read_file("(attributes)") {
                Ok(data) => {
                    println!("Successfully read (attributes) file: {} bytes", data.len());
                    println!("First 24 bytes: {:?}", &data[..data.len().min(24)]);
                }
                Err(e) => {
                    println!("Failed to read (attributes) file: {}", e);
                }
            }

            // Try to read a regular file
            match archive.read_file("readme.txt") {
                Ok(data) => {
                    let content = String::from_utf8_lossy(&data);
                    println!("Successfully read readme.txt: {}", content.trim());
                }
                Err(e) => {
                    println!("Failed to read readme.txt: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to open archive: {}", e);
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing wow-mpq reading StormLib archives ===");

    test_archive("tests/stormlib_comparison/stormlib_v1.mpq", "StormLib V1")?;
    test_archive("tests/stormlib_comparison/stormlib_v2.mpq", "StormLib V2")?;
    test_archive("tests/stormlib_comparison/stormlib_v3.mpq", "StormLib V3")?;
    test_archive("tests/stormlib_comparison/stormlib_v4.mpq", "StormLib V4")?;

    println!("\nAll tests completed!");

    Ok(())
}
