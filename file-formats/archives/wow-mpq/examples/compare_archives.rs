use std::error::Error;
use std::fs;
use wow_mpq::Archive;

fn hexdump(data: &[u8], offset: usize, max_len: usize) {
    let len = data.len().min(max_len);
    println!("Hexdump at offset 0x{offset:04X}:");
    for i in (0..len).step_by(16) {
        print!("{:04X}: ", offset + i);

        // Hex bytes
        for j in 0..16 {
            if i + j < len {
                print!("{:02X} ", data[i + j]);
            } else {
                print!("   ");
            }
            if j == 7 {
                print!(" ");
            }
        }

        print!(" |");
        // ASCII representation
        for j in 0..16 {
            if i + j < len {
                let c = data[i + j];
                if (0x20..0x7F).contains(&c) {
                    print!("{}", c as char);
                } else {
                    print!(".");
                }
            }
        }
        println!("|");
    }
    println!();
}

fn compare_archives(stormlib_path: &str, wowmpq_path: &str) -> Result<(), Box<dyn Error>> {
    println!("\n=== Comparing {stormlib_path} vs {wowmpq_path} ===");

    // Read raw files
    let stormlib_data = fs::read(stormlib_path)?;
    let wowmpq_data = fs::read(wowmpq_path)?;

    println!("StormLib size: {} bytes", stormlib_data.len());
    println!("wow-mpq size:  {} bytes", wowmpq_data.len());
    println!(
        "Difference:    {} bytes",
        (stormlib_data.len() as i64 - wowmpq_data.len() as i64).abs()
    );

    // Show first 512 bytes of each
    println!("\nStormLib header:");
    hexdump(&stormlib_data, 0, 512);

    println!("wow-mpq header:");
    hexdump(&wowmpq_data, 0, 512);

    // Try to open both archives and compare contents
    println!("Opening archives to compare contents...");

    let mut storm_archive = Archive::open(stormlib_path)?;
    let mut wow_archive = Archive::open(wowmpq_path)?;

    // Get archive info
    let storm_info = storm_archive.get_info()?;
    let wow_info = wow_archive.get_info()?;

    println!("\nArchive Info Comparison:");
    println!(
        "Format version: StormLib={:?}, wow-mpq={:?}",
        storm_info.format_version, wow_info.format_version
    );
    println!(
        "Number of files: StormLib={}, wow-mpq={}",
        storm_info.file_count, wow_info.file_count
    );
    println!(
        "Has listfile: StormLib={}, wow-mpq={}",
        storm_info.has_listfile, wow_info.has_listfile
    );
    println!(
        "Has attributes: StormLib={}, wow-mpq={}",
        storm_info.has_attributes, wow_info.has_attributes
    );

    // List files
    let storm_files = storm_archive.list_all()?;
    let wow_files = wow_archive.list_all()?;

    println!("\nFile listing:");
    println!("StormLib files: {} files", storm_files.len());
    for f in &storm_files {
        println!("  - {}", f.name);
    }
    println!("wow-mpq files: {} files", wow_files.len());
    for f in &wow_files {
        println!("  - {}", f.name);
    }

    // Compare file contents
    println!("\nFile content comparison:");
    for file_entry in &storm_files {
        let filename = &file_entry.name;
        if filename == "(listfile)" || filename == "(attributes)" {
            continue; // Skip special files for now
        }

        let storm_data = storm_archive.read_file(filename)?;
        let wow_data = wow_archive.read_file(filename)?;

        if storm_data == wow_data {
            println!("{}: IDENTICAL ({} bytes)", filename, storm_data.len());
        } else {
            println!(
                "{}: DIFFERENT! StormLib={} bytes, wow-mpq={} bytes",
                filename,
                storm_data.len(),
                wow_data.len()
            );
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== MPQ Archive Comparison Tool ===");

    // Compare each version
    compare_archives("stormlib_v1.mpq", "wowmpq_v1.mpq")?;
    compare_archives("stormlib_v2.mpq", "wowmpq_v2.mpq")?;
    compare_archives("stormlib_v3.mpq", "wowmpq_v3.mpq")?;
    compare_archives("stormlib_v4.mpq", "wowmpq_v4.mpq")?;

    Ok(())
}
