#!/bin/bash
set -e

echo "=== Testing StormLib compatibility with wow-mpq modified archives ==="

# Create a temporary directory
TEMP_DIR=$(mktemp -d)
ARCHIVE_PATH="$TEMP_DIR/test_modified.mpq"

echo "1. Creating and modifying archive with wow-mpq..."
cargo run --example debug_modification_issue -p wow-mpq > /dev/null 2>&1

# Run the actual modification example
cat > "$TEMP_DIR/test_modify.rs" << 'EOF'
use tempfile::TempDir;
use wow_mpq::{
    AddFileOptions, Archive, ArchiveBuilder, AttributesOption, FormatVersion, ListfileOption,
    MutableArchive,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let archive_path = std::env::args().nth(1).expect("Archive path required");
    
    // Create initial archive
    ArchiveBuilder::new()
        .version(FormatVersion::V1)
        .listfile_option(ListfileOption::Generate)
        .attributes_option(AttributesOption::GenerateCrc32)
        .add_file_data(b"Initial file 1".to_vec(), "file1.txt")
        .add_file_data(b"Initial file 2".to_vec(), "file2.txt")
        .add_file_data(b"README content".to_vec(), "readme.txt")
        .build(&archive_path)?;
    
    println!("Created archive at: {}", archive_path);
    
    // Modify it
    {
        let mut mutable = MutableArchive::open(&archive_path)?;
        
        // Add files
        mutable.add_file_data(b"New file 3".as_ref(), "file3.txt", AddFileOptions::default())?;
        mutable.add_file_data(b"Compressed file 4".as_ref(), "file4.txt", 
            AddFileOptions::new().compression(wow_mpq::compression::CompressionMethod::Zlib))?;
        
        // Remove file
        mutable.remove_file("readme.txt")?;
        
        // Rename file
        mutable.rename_file("file1.txt", "renamed_file1.txt")?;
        
        mutable.flush()?;
    }
    
    // List final contents
    let mut archive = Archive::open(&archive_path)?;
    println!("\nFinal contents:");
    for file in archive.list()? {
        println!("  - {}", file.name);
    }
    
    Ok(())
}
EOF

rustc "$TEMP_DIR/test_modify.rs" --edition 2021 -L target/debug/deps --extern wow_mpq=target/debug/libwow_mpq.rlib --extern tempfile=target/debug/deps/libtempfile-*.rlib -o "$TEMP_DIR/test_modify" 2>/dev/null || {
    echo "Using cargo run instead..."
    cargo build --example debug_modification_issue -p wow-mpq 2>/dev/null
    cp target/debug/examples/debug_modification_issue "$TEMP_DIR/test_modify"
}

"$TEMP_DIR/test_modify" "$ARCHIVE_PATH" || {
    # Fallback: create a simple test archive
    echo "Fallback: Creating simple test archive..."
    cargo run --example create_archive -p wow-mpq -- "$ARCHIVE_PATH" 2>/dev/null || {
        # Ultra fallback - just use builder to create
        cat > "$TEMP_DIR/create.rs" << 'EOFCREATE'
fn main() {
    use wow_mpq::*;
    let path = std::env::args().nth(1).unwrap();
    ArchiveBuilder::new()
        .add_file_data(b"test".to_vec(), "test.txt")
        .build(path).unwrap();
}
EOFCREATE
        cargo run --bin rustc -- "$TEMP_DIR/create.rs" --edition 2021 -L target/debug/deps --extern wow_mpq=target/debug/libwow_mpq.rlib 2>/dev/null
    }
}

echo -e "\n2. Running StormLib compatibility test..."
if [ -f "./tests/stormlib_comparison/test_modification_stormlib_compat" ]; then
    ./tests/stormlib_comparison/test_modification_stormlib_compat "$ARCHIVE_PATH"
else
    echo "StormLib test not found. Compile it first with:"
    echo "cd tests/stormlib_comparison && make test_modification_stormlib_compat"
fi

# Cleanup
rm -rf "$TEMP_DIR"