use std::error::Error;
use wow_mpq::{ArchiveBuilder, AttributesOption, FormatVersion, ListfileOption, compression};

/// Generate test files with predictable content (matching StormLib test)
fn generate_test_files() -> Vec<(&'static str, Vec<u8>)> {
    let mut files = Vec::new();

    // Small text file
    files.push((
        "readme.txt",
        b"This is a test readme file for MPQ archive comparison.".to_vec(),
    ));

    // Binary file with pattern
    let mut binary_data = vec![0u8; 1024];
    for i in 0..binary_data.len() {
        binary_data[i] = (i & 0xFF) as u8;
    }
    files.push(("data/binary.dat", binary_data));

    // Larger file (100KB)
    let mut large_data = vec![0u8; 100 * 1024];
    for i in 0..large_data.len() {
        large_data[i] = ((i * 7) & 0xFF) as u8;
    }
    files.push(("assets/large.bin", large_data));

    files
}

fn create_archive_v1(filename: &str, files: &[(&str, Vec<u8>)]) -> Result<(), Box<dyn Error>> {
    println!("Creating V1 archive: {}", filename);

    let mut builder = ArchiveBuilder::new()
        .version(FormatVersion::V1)
        .listfile_option(ListfileOption::Generate)
        .attributes_option(AttributesOption::GenerateFull) // Generate attributes file with CRC32 + MD5 + timestamp
        .default_compression(compression::flags::ZLIB);

    for (name, data) in files {
        builder = builder.add_file_data(data.clone(), name);
    }

    builder.build(filename)?;
    println!("V1 archive created successfully");
    Ok(())
}

fn create_archive_v2(filename: &str, files: &[(&str, Vec<u8>)]) -> Result<(), Box<dyn Error>> {
    println!("Creating V2 archive: {}", filename);

    let mut builder = ArchiveBuilder::new()
        .version(FormatVersion::V2)
        .listfile_option(ListfileOption::Generate)
        .attributes_option(AttributesOption::GenerateFull) // Generate attributes file with CRC32 + MD5 + timestamp
        .default_compression(compression::flags::ZLIB);

    for (name, data) in files {
        builder = builder.add_file_data(data.clone(), name);
    }

    builder.build(filename)?;
    println!("V2 archive created successfully");
    Ok(())
}

fn create_archive_v3(filename: &str, files: &[(&str, Vec<u8>)]) -> Result<(), Box<dyn Error>> {
    println!("Creating V3 archive: {}", filename);

    let mut builder = ArchiveBuilder::new()
        .version(FormatVersion::V3)
        .listfile_option(ListfileOption::Generate)
        .attributes_option(AttributesOption::GenerateFull) // Generate attributes file with CRC32 + MD5 + timestamp
        .default_compression(compression::flags::ZLIB);

    for (name, data) in files {
        builder = builder.add_file_data(data.clone(), name);
    }

    builder.build(filename)?;
    println!("V3 archive created successfully");
    Ok(())
}

fn create_archive_v4(filename: &str, files: &[(&str, Vec<u8>)]) -> Result<(), Box<dyn Error>> {
    println!("Creating V4 archive: {}", filename);

    let mut builder = ArchiveBuilder::new()
        .version(FormatVersion::V4)
        .listfile_option(ListfileOption::Generate)
        .attributes_option(AttributesOption::GenerateFull) // Generate attributes file with CRC32 + MD5 + timestamp
        .default_compression(compression::flags::ZLIB);

    for (name, data) in files {
        builder = builder.add_file_data(data.clone(), name);
    }

    builder.build(filename)?;
    println!("V4 archive created successfully");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== wow-mpq Archive Creation Test ===");

    // Generate test files
    let test_files = generate_test_files();
    println!("Generated {} test files", test_files.len());

    // Create archives of each version
    create_archive_v1("wowmpq_v1.mpq", &test_files)?;
    create_archive_v2("wowmpq_v2.mpq", &test_files)?;
    create_archive_v3("wowmpq_v3.mpq", &test_files)?;
    create_archive_v4("wowmpq_v4.mpq", &test_files)?;

    println!("\nwow-mpq archives created successfully!");

    Ok(())
}
