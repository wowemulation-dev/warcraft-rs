use std::fs;
use std::path::Path;
use wow_mpq::compression::flags as compression_flags;
use wow_mpq::{ArchiveBuilder, FormatVersion};

fn main() -> wow_mpq::Result<()> {
    let source_dir = "test-data/raw-data/game_assets";

    if !Path::new(source_dir).exists() {
        eprintln!("Source directory {} does not exist", source_dir);
        eprintln!("Please run 'cargo run --example generate_test_data -- all' first");
        return Ok(());
    }

    // Create v3 archive with compressed HET/BET tables using zlib
    println!("Creating v3 archive with compressed HET/BET tables (zlib)...");
    let mut builder = ArchiveBuilder::new()
        .version(FormatVersion::V3)
        .compress_tables(true)
        .table_compression(compression_flags::ZLIB);

    // Add all files from game_assets directory
    add_directory_files(&mut builder, source_dir, "")?;

    builder.build("test_archives/v3_compressed_tables_zlib.mpq")?;
    println!("✓ Created test_archives/v3_compressed_tables_zlib.mpq");

    // Create v3 archive with compressed HET/BET tables using bzip2
    println!("Creating v3 archive with compressed HET/BET tables (bzip2)...");
    let mut builder = ArchiveBuilder::new()
        .version(FormatVersion::V3)
        .compress_tables(true)
        .table_compression(compression_flags::BZIP2);

    add_directory_files(&mut builder, source_dir, "")?;

    builder.build("test_archives/v3_compressed_tables_bzip2.mpq")?;
    println!("✓ Created test_archives/v3_compressed_tables_bzip2.mpq");

    // Create v3 archive with compressed HET/BET tables using LZMA
    println!("Creating v3 archive with compressed HET/BET tables (LZMA)...");
    let mut builder = ArchiveBuilder::new()
        .version(FormatVersion::V3)
        .compress_tables(true)
        .table_compression(compression_flags::LZMA);

    add_directory_files(&mut builder, source_dir, "")?;

    builder.build("test_archives/v3_compressed_tables_lzma.mpq")?;
    println!("✓ Created test_archives/v3_compressed_tables_lzma.mpq");

    println!("\nAll test archives created successfully!");
    Ok(())
}

fn add_directory_files(
    builder: &mut ArchiveBuilder,
    dir: &str,
    prefix: &str,
) -> wow_mpq::Result<()> {
    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_file() {
            let archive_path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", prefix, name)
            };

            *builder = std::mem::take(builder).add_file(&path, &archive_path);
            println!("  Added: {}", archive_path);
        } else if path.is_dir() {
            let new_prefix = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", prefix, name)
            };
            add_directory_files(builder, &path.to_string_lossy(), &new_prefix)?;
        }
    }

    Ok(())
}
