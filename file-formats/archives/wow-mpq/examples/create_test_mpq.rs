//! Create test MPQ archives
//!
//! This example replaces the mpq_tools.py create functionality.
//!
//! Usage:
//!     cargo run --example create_test_mpq -- minimal --version 1
//!     cargo run --example create_test_mpq -- compressed --compression zlib
//!     cargo run --example create_test_mpq -- all

use std::env;
use std::path::Path;
use wow_mpq::{ArchiveBuilder, FormatVersion, ListfileOption, compression};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <type> [options]", args[0]);
        eprintln!("\nArchive types:");
        eprintln!("  minimal      - Minimal test archive");
        eprintln!("  compressed   - Archive with compressed files");
        eprintln!("  encrypted    - Archive with encrypted files");
        eprintln!("  edge-cases   - Archive with edge case files");
        eprintln!("  comprehensive - Comprehensive test archive");
        eprintln!("  all          - Create all test archives");
        eprintln!("\nOptions:");
        eprintln!("  --version <1-4>       - MPQ version (for minimal/comprehensive)");
        eprintln!("  --compression <type>  - Compression type (for compressed)");
        eprintln!("  --output-dir <dir>    - Output directory (default: test-data)");
        std::process::exit(1);
    }

    let archive_type = &args[1];
    let mut output_dir = "test-data";
    let mut version = 2; // Default to v2
    let mut compression_type = "zlib";

    // Parse options
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--output-dir" if i + 1 < args.len() => {
                output_dir = &args[i + 1];
                i += 2;
            }
            "--version" if i + 1 < args.len() => {
                version = args[i + 1].parse::<u8>().unwrap_or(2);
                i += 2;
            }
            "--compression" if i + 1 < args.len() => {
                compression_type = &args[i + 1];
                i += 2;
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                std::process::exit(1);
            }
        }
    }

    let output_path = Path::new(output_dir);
    std::fs::create_dir_all(output_path)?;

    if archive_type == "all" {
        println!("Creating all test archives in: {}", output_path.display());

        // Create minimal archives for each version
        for v in 1..=4 {
            create_minimal_archive(output_path, v)?;
        }

        // Create compressed archives
        for comp in ["zlib", "bzip2", "lzma", "sparse"] {
            create_compressed_archive(output_path, comp)?;
        }

        // Create other test archives
        create_encrypted_archive(output_path)?;
        create_edge_cases_archive(output_path)?;
        create_comprehensive_archive(output_path, 2)?;
        create_comprehensive_archive(output_path, 4)?;

        println!("\nAll test archives created successfully!");
    } else {
        match archive_type.as_str() {
            "minimal" => create_minimal_archive(output_path, version)?,
            "compressed" => create_compressed_archive(output_path, compression_type)?,
            "encrypted" => create_encrypted_archive(output_path)?,
            "edge-cases" => create_edge_cases_archive(output_path)?,
            "comprehensive" => create_comprehensive_archive(output_path, version)?,
            _ => {
                eprintln!("Unknown archive type: {archive_type}");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn create_minimal_archive(
    output_path: &Path,
    version: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let format_version = match version {
        1 => FormatVersion::V1,
        2 => FormatVersion::V2,
        3 => FormatVersion::V3,
        4 => FormatVersion::V4,
        _ => {
            eprintln!("Invalid version: {version}. Must be 1-4");
            return Err("Invalid version".into());
        }
    };

    let archive_path = output_path.join(format!("minimal_v{version}.mpq"));
    println!(
        "Creating minimal v{} archive: {}",
        version,
        archive_path.display()
    );

    let mut builder = ArchiveBuilder::new()
        .version(format_version)
        .block_size(3)
        .listfile_option(ListfileOption::Generate); // Auto-generate listfile

    // Add test file
    builder = builder.add_file_data(b"Hello, MPQ!".to_vec(), "test.txt");

    builder.build(&archive_path)?;

    Ok(())
}

fn create_compressed_archive(
    output_path: &Path,
    compression_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let compression_flag = match compression_type {
        "zlib" => compression::flags::ZLIB,
        "bzip2" => compression::flags::BZIP2,
        "lzma" => compression::flags::LZMA,
        "sparse" => compression::flags::SPARSE,
        _ => {
            eprintln!("Unknown compression type: {compression_type}");
            return Err("Unknown compression".into());
        }
    };

    let archive_path = output_path.join(format!("compressed_{compression_type}.mpq"));
    println!(
        "Creating compressed ({}) archive: {}",
        compression_type,
        archive_path.display()
    );

    let test_data =
        b"This is test data that should compress well because it has repeated patterns. "
            .repeat(50);

    let mut builder = ArchiveBuilder::new()
        .version(FormatVersion::V2)
        .block_size(4);

    // Add compressed file
    builder = builder.add_file_data_with_options(
        test_data.clone(),
        "compressed.dat",
        compression_flag,
        false, // encrypt
        0,     // locale
    );

    // Add uncompressed file for comparison
    builder = builder.add_file_data(test_data[..1024].to_vec(), "uncompressed.dat");

    builder.build(&archive_path)?;

    Ok(())
}

fn create_encrypted_archive(output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let archive_path = output_path.join("encrypted.mpq");
    println!("Creating encrypted archive: {}", archive_path.display());

    let mut builder = ArchiveBuilder::new()
        .version(FormatVersion::V2)
        .block_size(3);

    // Add encrypted file
    builder = builder.add_file_data_with_encryption(
        b"This is encrypted data!".to_vec(),
        "secret.dat",
        compression::flags::ZLIB,
        false, // use_fix_key
        0,     // locale
    );

    // Add encrypted file with fix key
    builder = builder.add_file_data_with_encryption(
        b"This uses fix key encryption!".to_vec(),
        "fixed_key.dat",
        0,    // no compression
        true, // use_fix_key
        0,    // locale
    );

    builder.build(&archive_path)?;

    Ok(())
}

fn create_edge_cases_archive(output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let archive_path = output_path.join("edge_cases.mpq");
    println!("Creating edge cases archive: {}", archive_path.display());

    let mut builder = ArchiveBuilder::new()
        .version(FormatVersion::V2)
        .block_size(5)
        .listfile_option(ListfileOption::Generate); // Auto-generate listfile

    // Empty file
    builder = builder.add_file_data(vec![], "empty.txt");

    // Single byte file
    builder = builder.add_file_data(vec![0x42], "single_byte.dat");

    // File with spaces in name
    builder = builder.add_file_data(b"Spaces in filename!".to_vec(), "file with spaces.txt");

    // File with path
    builder = builder.add_file_data(b"Nested file".to_vec(), "folder/subfolder/nested.dat");

    // Large uncompressible file (100KB of random data)
    let mut random_data = vec![0u8; 100 * 1024];
    for (i, byte) in random_data.iter_mut().enumerate() {
        *byte = (i * 7 + 13) as u8;
    }
    builder = builder.add_file_data_with_options(
        random_data,
        "random.bin",
        compression::flags::ZLIB,
        false, // encrypt
        0,     // locale
    );

    builder.build(&archive_path)?;

    Ok(())
}

fn create_comprehensive_archive(
    output_path: &Path,
    version: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let format_version = match version {
        1 => FormatVersion::V1,
        2 => FormatVersion::V2,
        3 => FormatVersion::V3,
        4 => FormatVersion::V4,
        _ => {
            eprintln!("Invalid version: {version}. Must be 1-4");
            return Err("Invalid version".into());
        }
    };

    let archive_path = output_path.join(format!("comprehensive_v{version}.mpq"));
    println!(
        "Creating comprehensive v{} archive: {}",
        version,
        archive_path.display()
    );

    let mut builder = ArchiveBuilder::new()
        .version(format_version)
        .block_size(7) // 64KB sectors
        .listfile_option(ListfileOption::Generate); // Auto-generate listfile

    // Add various files
    builder = builder.add_file_data(
        b"MPQ Archive Test Suite\n\nThis archive contains various test files.".to_vec(),
        "readme.txt",
    );

    builder = builder.add_file_data_with_options(
        b"[Settings]\nversion=1.0\ntest=true".to_vec(),
        "data/config.ini",
        compression::flags::ZLIB,
        false, // encrypt
        0,     // locale
    );

    // Binary pattern
    let mut binary_data = Vec::with_capacity(10 * 1024);
    for i in 0..10 * 1024 {
        binary_data.push((i & 0xFF) as u8);
    }
    builder = builder.add_file_data_with_options(
        binary_data,
        "data/binary.dat",
        compression::flags::BZIP2,
        false, // encrypt
        0,     // locale
    );

    builder = builder.add_file_data_with_encryption(
        b"Secret data".to_vec(),
        "secure/encrypted.bin",
        0, // no compression
        false,
        0,
    );

    // Add large file for v2+
    if version >= 2 {
        let large_data = b"Large file content. ".repeat(50000); // ~1MB
        builder = builder.add_file_data_with_options(
            large_data,
            "large/bigfile.dat",
            compression::flags::LZMA,
            false, // encrypt
            0,     // locale
        );
    }

    // Add (attributes) for v2+
    if version >= 2 {
        // Simple attributes (just version and flags)
        let attributes = [
            100u32.to_le_bytes().to_vec(),  // Version
            0x03u32.to_le_bytes().to_vec(), // Flags (CRC32 + TIMESTAMP)
        ]
        .concat();
        builder = builder.add_file_data(attributes, "(attributes)");
    }

    builder.build(&archive_path)?;

    Ok(())
}
