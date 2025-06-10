# 📦 Working with MPQ Archives

## Overview

MPQ (Mo'PaQ) archives are Blizzard's proprietary archive format used extensively
in World of Warcraft to store game assets. This guide covers how to work with
MPQ archives using `warcraft-rs`, including reading, extracting, and managing
files within these archives.

**Key Features:**
- ✅ **100% StormLib Compatibility** - Full cross-implementation support
- ✅ **Full Blizzard Archive Support** - Handles all official WoW archives (1.12.1 - 5.4.8)
- ✅ **Bidirectional Compatibility** - Archives created by either implementation can be read by both
- ✅ **Automatic Path Conversion** - Forward slashes automatically converted to backslashes

## Prerequisites

Before working with MPQ archives, ensure you have:

- Basic understanding of Rust programming
- `warcraft-rs` installed with the `mpq` feature enabled
- Access to World of Warcraft MPQ files (from game installation)
- Understanding of file I/O operations in Rust

## Understanding MPQ Archives

### What are MPQ Archives?

MPQ archives are compressed file containers that can store:

- Game textures (BLP files)
- Models (M2, WMO files)
- Database files (DBC)
- Audio files
- UI resources
- Scripts and configuration files

### Key Features

- **Compression**: Multiple compression algorithms (PKWARE, zlib, bzip2)
- **Encryption**: Optional file encryption
- **Listfiles**: Internal file listings (though not always present)
- **Patches**: Support for incremental updates
- **Multi-locale**: Language-specific file variations

## Step-by-Step Instructions

### 1. Opening an MPQ Archive

```rust
use wow_mpq::{Archive, OpenOptions};

fn open_mpq_archive() -> Result<Archive, Box<dyn std::error::Error>> {
    // Open an MPQ archive for reading
    let mut archive = Archive::open("Data/common.MPQ")?;

    // Open with specific options
    let options = OpenOptions::new()
        .load_tables(true);  // Load hash and block tables
    let archive = Archive::open_with_options("Data/patch.MPQ", options)?;

    Ok(archive)
}
```

### 2. Listing Files in an Archive

```rust
use wow_mpq::Archive;

fn list_archive_contents(archive: &mut Archive) -> Result<(), Box<dyn std::error::Error>> {
    // List files (requires listfile to be present)
    match archive.list() {
        Ok(entries) => {
            println!("Archive contains {} files:", entries.len());

            for entry in entries {
                println!("  - {} ({} bytes)", entry.name, entry.size);

                // Check file attributes using the flags field
                if entry.compressed_size < entry.size {
                    println!("    Compressed: {} -> {} bytes",
                        entry.compressed_size, entry.size);
                }
                if entry.flags != 0 {
                    println!("    Flags: 0x{:08X}", entry.flags);
                }
            }
        }
        Err(_) => {
            println!("No listfile found in archive");
            // You'll need to know exact filenames to extract without a listfile
        }
    }

    Ok(())
}
```

### 3. Extracting Files

```rust
use wow_mpq::Archive;
use std::fs::File;
use std::io::Write;

fn extract_file(archive: &mut Archive, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Extract a single file
    let data = archive.read_file(filename)?;

    // Save to disk
    let mut file = File::create(filename)?;
    file.write_all(&data)?;

    println!("Extracted {} ({} bytes)", filename, data.len());

    Ok(())
}

fn extract_all_files(archive: &mut Archive, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use std::path::Path;

    // Create output directory
    fs::create_dir_all(output_dir)?;

    // Get file list (requires listfile)
    let entries = archive.list()?;

    for entry in entries {
        let filename = &entry.name;
        let output_path = Path::new(output_dir).join(filename);

        // Create subdirectories if needed
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Extract and save
        match archive.read_file(filename) {
            Ok(data) => {
                let mut file = File::create(output_path)?;
                file.write_all(&data)?;
                println!("Extracted: {}", filename);
            }
            Err(e) => {
                eprintln!("Failed to extract {}: {}", filename, e);
            }
        }
    }

    Ok(())
}
```

### 4. Working with Multiple Archives using PatchChain

The `PatchChain` struct provides automatic priority-based file resolution across
multiple MPQ archives, mimicking how World of Warcraft handles patches.

```rust
use wow_mpq::{PatchChain, Archive};
use std::path::PathBuf;

fn work_with_patch_chain() -> Result<(), Box<dyn std::error::Error>> {
    // Create a patch chain
    let mut chain = PatchChain::new();

    // Add archives with priority (higher numbers override lower)
    chain.add_archive(PathBuf::from("Data/common.MPQ"), 0)?;       // Base content
    chain.add_archive(PathBuf::from("Data/expansion.MPQ"), 100)?;  // Expansion content
    chain.add_archive(PathBuf::from("Data/patch.MPQ"), 200)?;      // Patch 1
    chain.add_archive(PathBuf::from("Data/patch-2.MPQ"), 300)?;    // Patch 2
    chain.add_archive(PathBuf::from("Data/patch-3.MPQ"), 400)?;    // Patch 3 (highest priority)

    // Extract a file - automatically uses the highest priority version
    let filename = "Interface/Icons/INV_Misc_QuestionMark.blp";
    let data = chain.read_file(filename)?;
    println!("Extracted {} ({} bytes)", filename, data.len());

    // Find which archive contains a specific file
    if let Some(archive_path) = chain.find_file_archive(filename) {
        println!("File found in: {}", archive_path.display());
    }

    // List all unique files across all archives
    let all_files = chain.list()?;
    println!("Total unique files: {}", all_files.len());

    // Get information about all archives in the chain
    let chain_info = chain.get_chain_info();
    for info in &chain_info {
        println!("{} (priority {}): {} files",
            info.path.display(),
            info.priority,
            info.file_count
        );
    }

    Ok(())
}
```

#### Manual Archive Management (Legacy Approach)

```rust
use wow_mpq::Archive;

fn work_with_multiple_archives_manual() -> Result<(), Box<dyn std::error::Error>> {
    // Open archives individually
    let mut base = Archive::open("Data/common.MPQ")?;
    let mut patch = Archive::open("Data/patch.MPQ")?;
    let mut patch2 = Archive::open("Data/patch-2.MPQ")?;

    // Search for a file across archives (manual priority handling)
    let filename = "Interface/Icons/INV_Misc_QuestionMark.blp";

    // Try patch archives first (highest priority)
    let data = if let Ok(data) = patch2.read_file(filename) {
        println!("Found {} in patch-2.MPQ", filename);
        data
    } else if let Ok(data) = patch.read_file(filename) {
        println!("Found {} in patch.MPQ", filename);
        data
    } else {
        println!("Found {} in common.MPQ", filename);
        base.read_file(filename)?
    };

    println!("Extracted {} ({} bytes)", filename, data.len());

    Ok(())
}
```

### 5. Creating New Archives

```rust
use wow_mpq::{ArchiveBuilder, FormatVersion, ListfileOption};

fn create_simple_archive() -> Result<(), Box<dyn std::error::Error>> {
    // Create a basic archive
    ArchiveBuilder::new()
        .add_file("readme.txt", "README.txt")
        .add_file_data(b"Hello World".to_vec(), "hello.txt")
        .build("simple.mpq")?;

    Ok(())
}

fn create_advanced_archive() -> Result<(), Box<dyn std::error::Error>> {
    use wow_mpq::compression::flags;

    ArchiveBuilder::new()
        // Configure archive settings
        .version(FormatVersion::V2)
        .block_size(7)  // 64KB sectors
        .default_compression(flags::ZLIB)
        .listfile_option(ListfileOption::Generate)

        // Add files with different options
        .add_file("data/texture.blp", "Textures/MyTexture.blp")
        .add_file_data_with_options(
            b"Important data".to_vec(),
            "Data/config.ini",
            flags::BZIP2,  // Better compression
            false,  // no encryption
            0,      // default locale
        )
        .add_file_data_with_options(
            b"Secret data".to_vec(),
            "Keys/secret.key",
            flags::ZLIB,
            true,   // encrypt
            0,      // locale
        )

        // Build the archive
        .build("advanced.mpq")?;

    Ok(())
}
```

### 6. Rebuilding and Comparing Archives

The `warcraft-rs` CLI provides powerful tools for rebuilding MPQ archives and
comparing them for differences.

#### Rebuilding Archives

Archive rebuilding allows you to recreate MPQ archives 1:1 while optionally
upgrading formats or changing compression:

```rust
use wow_mpq::{rebuild_archive, RebuildOptions, FormatVersion};

fn rebuild_archive_example() -> Result<(), Box<dyn std::error::Error>> {
    // Basic rebuild with format preservation
    let options = RebuildOptions {
        preserve_format: true,
        target_format: None,
        preserve_order: true,
        skip_encrypted: false,
        skip_signatures: true,
        verify: false,
        override_compression: None,
        override_block_size: None,
        list_only: false,
    };

    rebuild_archive(
        "original.mpq",
        "rebuilt.mpq",
        options,
        None  // No progress callback
    )?;

    println!("Archive rebuilt successfully");
    Ok(())
}

fn rebuild_with_upgrade() -> Result<(), Box<dyn std::error::Error>> {
    // Rebuild with format upgrade and verification
    let options = RebuildOptions {
        preserve_format: false,
        target_format: Some(FormatVersion::V4),
        preserve_order: true,
        skip_encrypted: false,
        skip_signatures: true,
        verify: true,
        override_compression: Some(wow_mpq::compression::flags::LZMA),
        override_block_size: Some(6), // 32KB sectors
        list_only: false,
    };

    let summary = rebuild_archive(
        "old_v1.mpq",
        "modern_v4.mpq",
        options,
        Some(&|current, total, file| {
            if current % 100 == 0 {
                println!("Processing [{}/{}]: {}", current, total, file);
            }
        })
    )?;

    println!("Rebuild completed:");
    println!("  Source files: {}", summary.source_files);
    println!("  Extracted files: {}", summary.extracted_files);
    println!("  Skipped files: {}", summary.skipped_files);
    println!("  Target format: {:?}", summary.target_format);
    println!("  Verified: {}", summary.verified);

    Ok(())
}
```

#### Comparing Archives

Archive comparison helps verify rebuilds and analyze differences between archives:

```rust
use wow_mpq::{compare_archives, CompareOptions};

fn compare_archives_example() -> Result<(), Box<dyn std::error::Error>> {
    // Basic comparison
    let result = compare_archives(
        "original.mpq",
        "rebuilt.mpq",
        false,  // not detailed
        false,  // no content check
        false,  // not metadata only
        true,   // ignore order
        None    // no filter
    )?;

    if result.identical {
        println!("✓ Archives are identical");
    } else {
        println!("✗ Archives differ");

        // Show metadata differences
        if !result.metadata.matches {
            println!("Metadata differences:");
            if result.metadata.format_version.0 != result.metadata.format_version.1 {
                println!("  Format: {:?} → {:?}",
                    result.metadata.format_version.0,
                    result.metadata.format_version.1);
            }
            if result.metadata.file_count.0 != result.metadata.file_count.1 {
                println!("  File count: {} → {}",
                    result.metadata.file_count.0,
                    result.metadata.file_count.1);
            }
        }

        // Show file differences
        if let Some(files) = &result.files {
            if !files.source_only.is_empty() {
                println!("Files only in source ({}): {:?}",
                    files.source_only.len(),
                    &files.source_only[..files.source_only.len().min(5)]);
            }
            if !files.target_only.is_empty() {
                println!("Files only in target ({}): {:?}",
                    files.target_only.len(),
                    &files.target_only[..files.target_only.len().min(5)]);
            }
            if !files.size_differences.is_empty() {
                println!("Files with size differences: {}", files.size_differences.len());
            }
        }
    }

    Ok(())
}

fn compare_with_content_verification() -> Result<(), Box<dyn std::error::Error>> {
    // Thorough comparison with content verification
    let result = compare_archives(
        "original.mpq",
        "rebuilt.mpq",
        true,   // detailed
        true,   // content check
        false,  // not metadata only
        true,   // ignore order
        Some("*.dbc".to_string()) // only compare DBC files
    )?;

    if let Some(files) = &result.files {
        if !files.content_differences.is_empty() {
            println!("⚠ Content differences found:");
            for file in &files.content_differences {
                println!("  - {}", file);
            }
        } else {
            println!("✓ All file contents match");
        }
    }

    Ok(())
}
```

#### CLI Workflow Examples

```bash
# Complete rebuild and verification workflow
echo "=== Archive Rebuild and Verification ==="

# 1. Analyze original archive
warcraft-rs mpq info original.mpq
warcraft-rs mpq list original.mpq --long | head -10

# 2. Rebuild archive preserving format
warcraft-rs mpq rebuild original.mpq rebuilt.mpq

# 3. Compare archives
warcraft-rs mpq compare original.mpq rebuilt.mpq --output summary

# 4. Verify content integrity
warcraft-rs mpq compare original.mpq rebuilt.mpq --content-check

# 5. Upgrade to modern format
warcraft-rs mpq rebuild original.mpq modern.mpq --upgrade-to v4 --compression lzma

# 6. Compare format differences
warcraft-rs mpq compare original.mpq modern.mpq --metadata-only
```

### 7. Searching for Files

```rust
use wow_mpq::Archive;
use regex::Regex;

fn search_files(archive: &Archive, pattern: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let re = Regex::new(pattern)?;

    // Get listfile (required for file enumeration)
    let entries = archive.list()?;

    let matches: Vec<String> = entries
        .iter()
        .filter(|entry| re.is_match(&entry.name))
        .map(|entry| entry.name.clone())
        .collect();

    println!("Found {} files matching '{}':", matches.len(), pattern);
    for filename in &matches {
        println!("  - {}", filename);
    }

    Ok(matches)
}

// Example: Find all BLP textures
fn find_textures(archive: &Archive) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    search_files(archive, r"\.blp$")
}

// Example: Find a specific file if you know part of the name
fn find_specific_file(archive: &Archive, partial_name: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let entries = archive.list()?;

    for entry in entries {
        if entry.name.contains(partial_name) {
            return Ok(Some(entry.name));
        }
    }

    Ok(None)
}
```

## Code Examples

### Complete Example: MPQ Explorer

```rust
use wow_mpq::Archive;
use std::io::{self, Write};

struct MpqExplorer {
    archive: Archive,
}

impl MpqExplorer {
    fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let archive = Archive::open(path)?;
        Ok(Self { archive })
    }

    fn info(&self) -> Result<(), Box<dyn std::error::Error>> {
        let info = self.archive.get_info();
        println!("Archive Information:");
        println!("  Path: {}", info.path.display());
        println!("  Format Version: {:?}", info.format_version);
        println!("  File Count: {}", info.file_count);
        println!("  Archive Size: {:.2} MB", info.archive_size as f64 / 1024.0 / 1024.0);
        println!("  Block Size: {} bytes", info.block_size);
        Ok(())
    }

    fn list(&self, filter: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let entries = self.archive.list()?;

        for entry in entries {
            let filename = &entry.name;

            if let Some(filter) = filter {
                if !filename.contains(filter) {
                    continue;
                }
            }

            println!("{} ({} bytes)", filename, entry.size);
        }

        Ok(())
    }

    fn extract(&self, filename: &str, output: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let data = self.archive.read_file(filename)?;
        let output_path = output.unwrap_or(filename);

        use std::fs::File;
        let mut file = File::create(output_path)?;
        file.write_all(&data)?;

        println!("Extracted {} to {} ({} bytes)", filename, output_path, data.len());
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let explorer = MpqExplorer::new("Data/common.MPQ")?;

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "info" => explorer.info()?,
            "list" => explorer.list(parts.get(1).copied())?,
            "extract" => {
                if let Some(filename) = parts.get(1) {
                    explorer.extract(filename, parts.get(2).copied())?;
                } else {
                    println!("Usage: extract <filename> [output]");
                }
            }
            "quit" => break,
            _ => println!("Unknown command. Available: info, list, extract, quit"),
        }
    }

    Ok(())
}
```

## Best Practices

### 1. Memory Management

```rust
// For large files, consider the file size before extraction
use wow_mpq::Archive;

fn extract_with_size_check(archive: &Archive, filename: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Get file list and check size
    let entries = archive.list()?;
    for entry in &entries {
        if entry.name == filename {
            if entry.size > 100 * 1024 * 1024 { // 100MB
                println!("Warning: File {} is large ({} bytes)", filename, entry.size);
            }
            break;
        }
    }

    // Extract the file
    archive.read_file(filename)
}
```

### 2. Error Handling

```rust
use wow_mpq::{Archive, Error};

fn safe_extract(archive: &Archive, filename: &str) -> Result<Vec<u8>, String> {
    match archive.read_file(filename) {
        Ok(data) => Ok(data),
        Err(Error::FileNotFound(_)) => {
            Err(format!("File '{}' not found in archive", filename))
        }
        Err(Error::InvalidFormat(msg)) => {
            Err(format!("File '{}' has invalid format: {}", filename, msg))
        }
        Err(Error::Io(e)) => {
            Err(format!("I/O error reading '{}': {}", filename, e))
        }
        Err(e) => Err(format!("Error reading '{}': {}", filename, e)),
    }
}
```

### 3. Caching Extracted Files

```rust
use std::collections::HashMap;
use wow_mpq::Archive;

struct CachedArchive {
    archive: Archive,
    cache: HashMap<String, Vec<u8>>,
}

impl CachedArchive {
    fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            archive: Archive::open(path)?,
            cache: HashMap::new(),
        })
    }

    fn get(&mut self, filename: &str) -> Result<&[u8], Box<dyn std::error::Error>> {
        if !self.cache.contains_key(filename) {
            let data = self.archive.read_file(filename)?;
            self.cache.insert(filename.to_string(), data);
        }

        Ok(&self.cache[filename])
    }
}
```

## Common Issues and Solutions

### Issue: File Not Found

**Problem**: `archive.read_file()` returns `FileNotFound` error.

**Solutions**:

1. Check the exact filename (case-sensitive)
2. Check if archive has a listfile (use `archive.list()`)
3. Check if using correct patch archive
4. For archives without listfiles, you need to know exact filenames

```rust
// Debug file lookup
fn debug_file_lookup(archive: &Archive, partial_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let entries = archive.list()?;

    println!("Files containing '{}':", partial_name);
    for entry in entries {
        if entry.name.contains(partial_name) {
            println!("  - {}", entry.name);
        }
    }

    Ok(())
}

// Check if file exists before extraction
fn safe_file_check(archive: &Archive, filename: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let entries = archive.list()?;
    for entry in &entries {
        if entry.name == filename {
            return Ok(true);
        }
    }
    Ok(false)
}
```

### Issue: Missing Listfile

**Problem**: Cannot enumerate files without listfile.

**Solutions**:

1. Add a listfile to the archive manually
2. Extract files by their exact known names
3. Some archives don't have listfiles - this is normal

```rust
fn handle_missing_listfile(archive: &Archive) -> Result<(), Box<dyn std::error::Error>> {
    match archive.list() {
        Ok(entries) => {
            println!("Found {} files with listfile:", entries.len());
            for entry in entries.iter().take(10) {
                println!("  - {}", entry.name);
            }
        }
        Err(_) => {
            println!("No listfile found in archive");
            println!("You can:");
            println!("1. Add a listfile to the archive");
            println!("2. Extract specific files by exact name");
            println!("3. Use external listfile references");
        }
    }

    Ok(())
}
```

### Issue: Archive Integrity

**Problem**: Want to verify archive is not corrupted.

**Solutions**:

1. Check archive information
2. Try to read a few files to test basic functionality

```rust
fn basic_archive_test(archive: &Archive) -> Result<(), Box<dyn std::error::Error>> {
    let info = archive.get_info();
    println!("Archive format: {:?}", info.format_version);
    println!("File count: {}", info.file_count);

    // Try to list files as a basic integrity test
    match archive.list() {
        Ok(entries) => println!("Listfile found with {} entries", entries.len()),
        Err(_) => println!("No listfile found - cannot enumerate files without exact names"),
    }

    Ok(())
}
```

### Issue: Blizzard Archive Warnings

**Problem**: Getting "-28 byte attributes file size mismatch" warnings with official WoW archives.

**Solution**: This is normal and expected behavior. All Blizzard MPQ archives have exactly 28 extra zero bytes at the end of their attributes files. The warning is informational only - the archives work perfectly.

```rust
// The warning looks like:
// "Attributes file size mismatch: actual=X, expected=Y, difference=-28 (tolerating for compatibility)"

// This is handled automatically by wow-mpq and doesn't affect functionality
let archive = Archive::open("Data/patch.mpq")?;  // Works despite warning
```

## Patch Chain Management

### Understanding Patch Chains

World of Warcraft uses a patch chain system where newer patches override files
in older archives. The `PatchChain` struct automates this process, ensuring you
always get the most recent version of a file.

### Critical Loading Order Rules

Based on TrinityCore's implementation and the official WoW client behavior,
archives **must** be loaded in a specific order:

1. **Base Archives First**: Common game data (common.MPQ, common-2.MPQ)
2. **Expansion Archives**: Each expansion adds its archives (expansion.MPQ, lichking.MPQ)
3. **Locale Archives**: Language-specific content that overrides base content
4. **General Patches**: Numbered patches (patch.MPQ, patch-2.MPQ, patch-3.MPQ)
5. **Locale Patches**: Language-specific patches (patch-enUS.MPQ, patch-enUS-2.MPQ)

**Important principles:**

- Files in later-loaded archives override files with the same path in earlier archives
- Locale-specific files always override their generic counterparts
- Patches are loaded in numerical order (patch-2 overrides patch)
- Custom patches should use higher numbers (patch-4.MPQ+) or letters (patch-x.MPQ)

### Advanced PatchChain Usage

```rust
use wow_mpq::{PatchChain, ChainInfo};
use std::path::PathBuf;

fn advanced_patch_chain_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut chain = PatchChain::new();

    // Add archives with descriptive priorities
    const BASE_PRIORITY: i32 = 0;
    const EXPANSION_PRIORITY: i32 = 1000;
    const PATCH_PRIORITY_BASE: i32 = 2000;

    chain.add_archive(PathBuf::from("Data/common.MPQ"), BASE_PRIORITY)?;
    chain.add_archive(PathBuf::from("Data/expansion.MPQ"), EXPANSION_PRIORITY)?;

    // Add patches in order
    for (i, patch_file) in vec!["patch.MPQ", "patch-2.MPQ", "patch-3.MPQ"].iter().enumerate() {
        let path = PathBuf::from(format!("Data/{}", patch_file));
        let priority = PATCH_PRIORITY_BASE + (i as i32 * 100);
        chain.add_archive(path, priority)?;
    }

    // Extract multiple files efficiently
    let files_to_extract = vec![
        "Interface/Icons/INV_Misc_QuestionMark.blp",
        "DBFilesClient/Item.dbc",
        "DBFilesClient/Spell.dbc",
    ];

    for filename in &files_to_extract {
        match chain.read_file(filename) {
            Ok(data) => println!("Extracted {}: {} bytes", filename, data.len()),
            Err(e) => eprintln!("Failed to extract {}: {}", filename, e),
        }
    }

    // Get chain information
    let chain_info = chain.get_chain_info();
    for info in &chain_info {
        println!("Archive: {} (priority: {})", info.path.display(), info.priority);
    }

    Ok(())
}
```

### Patch Chain for Different WoW Versions

⚠️ **Important**: The loading order below matches the exact order used by the WoW
client, as documented by TrinityCore. Archives must be loaded in this specific
order for correct file resolution.

```rust
use wow_mpq::PatchChain;
use std::path::Path;

/// Setup patch chain for WoW 3.3.5a following TrinityCore's definitive loading order
fn setup_wotlk_3_3_5a(data_path: &Path, locale: &str) -> Result<PatchChain, Box<dyn std::error::Error>> {
    let mut chain = PatchChain::new();

    // The exact loading order from TrinityCore:
    // 1-4: Base and expansion archives
    chain.add_archive(data_path.join("common.MPQ").to_path_buf(), 0)?;
    chain.add_archive(data_path.join("common-2.MPQ").to_path_buf(), 1)?;
    chain.add_archive(data_path.join("expansion.MPQ").to_path_buf(), 2)?;
    chain.add_archive(data_path.join("lichking.MPQ").to_path_buf(), 3)?;

    // 5-10: Locale and speech archives
    chain.add_archive(data_path.join(format!("locale-{}.MPQ", locale)).to_path_buf(), 4)?;
    chain.add_archive(data_path.join(format!("speech-{}.MPQ", locale)).to_path_buf(), 5)?;
    chain.add_archive(data_path.join(format!("expansion-locale-{}.MPQ", locale)).to_path_buf(), 6)?;
    chain.add_archive(data_path.join(format!("lichking-locale-{}.MPQ", locale)).to_path_buf(), 7)?;
    chain.add_archive(data_path.join(format!("expansion-speech-{}.MPQ", locale)).to_path_buf(), 8)?;
    chain.add_archive(data_path.join(format!("lichking-speech-{}.MPQ", locale)).to_path_buf(), 9)?;

    // 11-13: General patches
    chain.add_archive(data_path.join("patch.MPQ").to_path_buf(), 10)?;
    chain.add_archive(data_path.join("patch-2.MPQ").to_path_buf(), 11)?;
    chain.add_archive(data_path.join("patch-3.MPQ").to_path_buf(), 12)?;

    // 14-16: Locale patches (in locale subdirectory)
    let locale_path = data_path.join(locale);
    chain.add_archive(locale_path.join(format!("patch-{}.MPQ", locale)).to_path_buf(), 13)?;
    chain.add_archive(locale_path.join(format!("patch-{}-2.MPQ", locale)).to_path_buf(), 14)?;
    chain.add_archive(locale_path.join(format!("patch-{}-3.MPQ", locale)).to_path_buf(), 15)?;

    Ok(chain)
}

/// Setup patch chain for different WoW versions
fn setup_wow_patch_chain(wow_path: &Path, version: &str, locale: &str) -> Result<PatchChain, Box<dyn std::error::Error>> {
    let mut chain = PatchChain::new();
    let data_path = wow_path.join("Data");

    match version {
        "1.12.1" => {
            // Vanilla WoW uses categorized archives
            let base_priority = 0;
            chain.add_archive(data_path.join("dbc.MPQ"), base_priority)?;
            chain.add_archive(data_path.join("fonts.MPQ"), base_priority)?;
            chain.add_archive(data_path.join("interface.MPQ"), base_priority)?;
            chain.add_archive(data_path.join("misc.MPQ"), base_priority)?;
            chain.add_archive(data_path.join("model.MPQ"), base_priority)?;
            chain.add_archive(data_path.join("sound.MPQ"), base_priority)?;
            chain.add_archive(data_path.join("speech.MPQ"), base_priority)?;
            chain.add_archive(data_path.join("terrain.MPQ"), base_priority)?;
            chain.add_archive(data_path.join("texture.MPQ"), base_priority)?;
            chain.add_archive(data_path.join("wmo.MPQ"), base_priority)?;

            // Patches override everything
            chain.add_archive(data_path.join("patch.MPQ"), 1000)?;
            chain.add_archive(data_path.join("patch-2.MPQ"), 1001)?;
        }
        "2.4.3" => {
            // TBC introduced common.MPQ structure
            // Base archives
            chain.add_archive(data_path.join("common.MPQ"), 0)?;
            chain.add_archive(data_path.join("common-2.MPQ"), 1)?;
            chain.add_archive(data_path.join("expansion.MPQ"), 2)?;

            // Locale archives (override base)
            let locale_path = data_path.join(locale);
            chain.add_archive(locale_path.join(format!("locale-{}.MPQ", locale)), 100)?;
            chain.add_archive(locale_path.join(format!("speech-{}.MPQ", locale)), 101)?;
            chain.add_archive(locale_path.join(format!("expansion-locale-{}.MPQ", locale)), 102)?;
            chain.add_archive(locale_path.join(format!("expansion-speech-{}.MPQ", locale)), 103)?;

            // General patches
            chain.add_archive(data_path.join("patch.MPQ"), 1000)?;
            chain.add_archive(data_path.join("patch-2.MPQ"), 1001)?;

            // Locale patches (highest priority)
            chain.add_archive(locale_path.join(format!("patch-{}.MPQ", locale)), 2000)?;
            chain.add_archive(locale_path.join(format!("patch-{}-2.MPQ", locale)), 2001)?;
        }
        "3.3.5a" => {
            // Use the definitive loading order function
            return setup_wotlk_3_3_5a(&data_path, locale);
        }
        _ => {
            return Err(format!("Unsupported WoW version: {}", version).into());
        }
    }

    Ok(chain)
}
```

### Searching Across Patch Chains

```rust
use wow_mpq::PatchChain;
use regex::Regex;

fn search_patch_chain(chain: &mut PatchChain, pattern: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let re = Regex::new(pattern)?;
    let all_files = chain.list()?;

    let matches: Vec<String> = all_files
        .into_iter()
        .filter(|entry| re.is_match(&entry.name))
        .map(|entry| entry.name)
        .collect();

    Ok(matches)
}

// Example: Find all spell icons
fn find_spell_icons(chain: &mut PatchChain) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    search_patch_chain(chain, r"Interface/Icons/Spell_.*\.blp")
}
```

## Performance Tips

### 1. Batch Operations

```rust
// Extract multiple files efficiently
fn batch_extract(archive: &Archive, filenames: &[&str]) -> Result<Vec<(String, Vec<u8>)>, Box<dyn std::error::Error>> {
    let mut results = Vec::with_capacity(filenames.len());

    for &filename in filenames {
        match archive.read_file(filename) {
            Ok(data) => results.push((filename.to_string(), data)),
            Err(e) => eprintln!("Failed to extract {}: {}", filename, e),
        }
    }

    Ok(results)
}
```

### 2. Efficient File Listing

```rust
use wow_mpq::Archive;

fn efficient_file_search(archive: &Archive, pattern: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Get file list once and reuse it
    let entries = archive.list()?;

    let matches: Vec<String> = entries
        .into_iter()
        .filter(|entry| entry.name.contains(pattern))
        .map(|entry| entry.name)
        .collect();

    Ok(matches)
}
```

### 3. Reuse Archive Objects

```rust
use wow_mpq::Archive;

struct ArchivePool {
    archives: Vec<Archive>,
}

impl ArchivePool {
    fn new(paths: &[&str]) -> Result<Self, Box<dyn std::error::Error>> {
        let archives = paths.iter()
            .map(|path| Archive::open(path))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { archives })
    }

    fn find_and_extract(&self, filename: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        for archive in &self.archives {
            if let Ok(data) = archive.read_file(filename) {
                return Ok(data);
            }
        }

        Err(format!("File '{}' not found in any archive", filename).into())
    }
}
```

### 4. Check File Existence Before Extraction

```rust
use wow_mpq::Archive;

fn smart_extract(archive: &Archive, filename: &str) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
    // Check if file exists first (cheaper than attempting extraction)
    match archive.find_file(filename)? {
        Some(file_info) => {
            println!("File {} exists ({} bytes), extracting...", filename, file_info.file_size);
            Ok(Some(archive.read_file(filename)?))
        }
        None => {
            println!("File {} not found", filename);
            Ok(None)
        }
    }
}
```

## Related Guides

- [📝 DBC Data Extraction](./dbc-extraction.md) - Extract and parse DBC files from
  MPQ archives
- [🖼️ Texture Loading Guide](./texture-loading.md) - Load BLP textures from MPQ
  archives
- [🎭 Loading M2 Models](./m2-models.md) - Extract and load M2 model files
- [🏛️ WMO Rendering Guide](./wmo-rendering.md) - Extract and render WMO files
- [📦 WoW Patch Chain Summary](./wow-patch-chain-summary.md) - Comprehensive guide
  to patch chaining across all WoW versions

## References

- [MPQ Format Documentation](https://wowdev.wiki/MPQ) - Detailed MPQ format specification
- [StormLib](https://github.com/ladislav-zezula/StormLib) - Reference C++ implementation
- [Blizzard Archive Formats](http://www.zezula.net/en/mpq/mpqformat.html) - Technical
  details
- [WoW File Formats](https://wowdev.wiki/) - Complete WoW file format documentation
