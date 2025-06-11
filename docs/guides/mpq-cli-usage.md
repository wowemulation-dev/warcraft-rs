# MPQ CLI Usage Guide

The `warcraft-rs` command-line tool provides MPQ archive operations through the
`mpq` subcommand with excellent compatibility for all World of Warcraft MPQ archives.

**Key Features:**

- ✅ **100% StormLib Compatibility** - Works with archives from both implementations
- ✅ **Full Blizzard Support** - Handles all official WoW archives (1.12.1 - 5.4.8)
- ✅ **Automatic Path Conversion** - Cross-platform path handling
- ✅ **Archive Rebuild & Compare** - Advanced archive management capabilities

## Installation

```bash
# Build from source
cd warcraft-rs
cargo build --release

# Or install globally
cargo install --path .

# The binary will be available as 'warcraft-rs'
```

## Basic Commands

### List Archive Contents

```bash
# Simple listing
warcraft-rs mpq list archive.mpq

# Detailed listing with sizes and compression ratios
warcraft-rs mpq list archive.mpq --long

# Filter by pattern (supports wildcards)
warcraft-rs mpq list archive.mpq --filter "*.m2"
warcraft-rs mpq list archive.mpq --filter "*Interface*" --long
```

### Extract Files

```bash
# Extract all files
warcraft-rs mpq extract archive.mpq

# Extract to specific directory
warcraft-rs mpq extract archive.mpq --output ./extracted

# Extract specific files
warcraft-rs mpq extract archive.mpq file1.txt file2.dat

# Preserve directory structure
warcraft-rs mpq extract archive.mpq --preserve-paths
```

### Archive Information

```bash
# Basic information
warcraft-rs mpq info archive.mpq

# Show hash table details
warcraft-rs mpq info archive.mpq --show-hash-table

# Show block table details
warcraft-rs mpq info archive.mpq --show-block-table
```

### Validate Archives

```bash
# Basic validation
warcraft-rs mpq validate archive.mpq
```

### Create Archives

```bash
# Create new archive with files
warcraft-rs mpq create new.mpq --add file1.txt --add file2.dat

# Specify format version
warcraft-rs mpq create new.mpq --version v2 --add "data/*"

# Choose compression method
warcraft-rs mpq create new.mpq --compression zlib --add file1.txt
warcraft-rs mpq create new.mpq --compression bzip2 --add largefile.dat
warcraft-rs mpq create new.mpq --compression none --add already_compressed.zip

# Include (listfile) for better compatibility
warcraft-rs mpq create new.mpq --with-listfile --add file1.txt --add file2.txt
```

### Rebuild Archives

Rebuild MPQ archives 1:1 while preserving original structure and optionally
upgrading format:

```bash
# Basic rebuild (preserves original format)
warcraft-rs mpq rebuild source.mpq target.mpq

# Upgrade to specific format version
warcraft-rs mpq rebuild old.mpq new.mpq --upgrade-to v4

# Skip encrypted files during rebuild
warcraft-rs mpq rebuild source.mpq target.mpq --skip-encrypted

# Skip digital signatures (default: true)
warcraft-rs mpq rebuild source.mpq target.mpq --skip-signatures

# Verify rebuilt archive matches original
warcraft-rs mpq rebuild source.mpq target.mpq --verify

# Override compression method for all files
warcraft-rs mpq rebuild source.mpq target.mpq --compression zlib

# Override block size
warcraft-rs mpq rebuild source.mpq target.mpq --block-size 4

# Dry run - list files that would be processed
warcraft-rs mpq rebuild source.mpq target.mpq --list-only
```

### Compare Archives

Compare two MPQ archives to identify differences in metadata, file lists, and content:

```bash
# Basic comparison with table output
warcraft-rs mpq compare source.mpq target.mpq

# Summary output format
warcraft-rs mpq compare source.mpq target.mpq --output summary

# Detailed file-by-file comparison
warcraft-rs mpq compare source.mpq target.mpq --detailed

# Compare actual file contents (slower but thorough)
warcraft-rs mpq compare source.mpq target.mpq --content-check

# Only compare archive metadata
warcraft-rs mpq compare source.mpq target.mpq --metadata-only

# Filter comparison to specific files
warcraft-rs mpq compare source.mpq target.mpq --filter "*.dbc"
warcraft-rs mpq compare source.mpq target.mpq --filter "*Interface*"

# JSON output for scripting
warcraft-rs mpq compare source.mpq target.mpq --output json
```

**Note**: Archive modification features (add/remove files to existing archives)
are planned for future releases.

## Advanced Usage

### Working with World of Warcraft Archives

```bash
# List models in a patch archive
warcraft-rs mpq list "patch-4.mpq" --filter "*.m2" --long

# Extract specific files with preserved paths
warcraft-rs mpq extract patch.mpq "Interface/Icons/INV_Misc_QuestionMark.blp" --preserve-paths --output ./extracted

# Extract multiple related files
warcraft-rs mpq extract common.mpq "DBFilesClient/ItemDisplayInfo.dbc" "DBFilesClient/Item.dbc"

# Note: Forward slashes in paths are automatically converted to backslashes for MPQ compatibility
# Both of these work identically:
warcraft-rs mpq extract patch.mpq "Units/Human/Footman.mdx"
warcraft-rs mpq extract patch.mpq "Units\\Human\\Footman.mdx"
```

### Batch Operations

```bash
# Extract from multiple archives
for archive in *.mpq; do
    echo "Processing $archive..."
    warcraft-rs mpq extract "$archive" --output "./${archive%.mpq}_extracted"
done

# List all textures across archives
for archive in *.mpq; do
    echo "=== $archive ==="
    warcraft-rs mpq list "$archive" --filter "*.blp" | head -10
done

# Get archive statistics
for archive in *.mpq; do
    echo "=== $archive ==="
    warcraft-rs mpq info "$archive"
    echo
done
```

### Analysis Tasks

```bash
# Find all model files
warcraft-rs mpq list archive.mpq --filter "*.m2" --long

# Look for specific content
warcraft-rs mpq list archive.mpq --filter "*Stormwind*"

# Extract database files for analysis
warcraft-rs mpq extract common.mpq --filter "*.dbc" --output ./dbc_files
```

## Global Options

### Verbosity Control

```bash
# Quiet mode (errors only)
warcraft-rs mpq --quiet list archive.mpq

# Verbose output
warcraft-rs mpq -v list archive.mpq

# Very verbose (debug output)
warcraft-rs mpq -vv info archive.mpq

# Maximum verbosity (trace output)
warcraft-rs mpq -vvv info archive.mpq
```

## Environment Variables

```bash
# Set default log level
export RUST_LOG=info
warcraft-rs mpq list archive.mpq

# For debug logging
export RUST_LOG=debug
warcraft-rs mpq extract archive.mpq
```

## Common Workflows

### Archive Analysis

```bash
# Get archive statistics
warcraft-rs mpq info archive.mpq

# List all files with details
warcraft-rs mpq list archive.mpq --long

# Find largest files (using external tools)
warcraft-rs mpq list archive.mpq --long | sort -k3 -n -r | head -20

# Find specific file types
warcraft-rs mpq list archive.mpq --filter "*.m2"
warcraft-rs mpq list archive.mpq --filter "*.dbc"
warcraft-rs mpq list archive.mpq --filter "*Interface*"
```

### Archive Exploration

```bash
# Validate archive integrity
warcraft-rs mpq validate archive.mpq

# Extract specific content for analysis
warcraft-rs mpq extract archive.mpq --filter "DBFilesClient/*" --output ./database_files --preserve-paths
warcraft-rs mpq extract archive.mpq --filter "Interface/Icons/*" --output ./icons --preserve-paths
```

### Tree Visualization

Visualize the structure of MPQ archives using the tree command:

```bash
# Basic tree view
warcraft-rs mpq tree archive.mpq

# Limit depth for large archives
warcraft-rs mpq tree archive.mpq --depth 3

# Compact mode without metadata
warcraft-rs mpq tree archive.mpq --compact

# Show external file references
warcraft-rs mpq tree archive.mpq --show-refs

# No color output for piping
warcraft-rs mpq tree archive.mpq --no-color

# Hide file sizes and metadata
warcraft-rs mpq tree archive.mpq --no-metadata
```

The tree view shows:

- 📦 Archive structure with header, tables, and files
- 📁 Directory hierarchy
- 📄 Individual files with sizes
- 🔗 External file references (e.g., M2 models referencing .skin files)
- 🎨 Color-coded output for better readability

### Data Extraction Workflow

```bash
# Extract database files
warcraft-rs mpq extract common.mpq --filter "*.dbc" --output ./dbc_analysis --preserve-paths

# Extract models and textures
warcraft-rs mpq extract model.mpq --filter "*.m2" --output ./models --preserve-paths
warcraft-rs mpq extract texture.mpq --filter "*.blp" --output ./textures --preserve-paths

# Extract UI resources
warcraft-rs mpq extract interface.mpq --filter "Interface/*" --output ./ui_resources --preserve-paths
```

### Archive Rebuild and Verification Workflow

Complete workflow for rebuilding and verifying MPQ archives:

```bash
# 1. Analyze original archive
warcraft-rs mpq info original.mpq
warcraft-rs mpq list original.mpq --long | head -20

# 2. Rebuild with format preservation
warcraft-rs mpq rebuild original.mpq rebuilt.mpq

# 3. Verify rebuild accuracy
warcraft-rs mpq compare original.mpq rebuilt.mpq --content-check

# 4. Upgrade to modern format
warcraft-rs mpq rebuild original.mpq modern.mpq --upgrade-to v4

# 5. Compare format differences
warcraft-rs mpq compare original.mpq modern.mpq --metadata-only

# 6. Verify file integrity after upgrade
warcraft-rs mpq compare original.mpq modern.mpq --content-check --filter "*.dbc"
```

### Archive Migration and Optimization

```bash
# Upgrade old archives to V4 format with better compression
warcraft-rs mpq rebuild old_v1.mpq optimized.mpq \
    --upgrade-to v4 \
    --compression lzma \
    --block-size 6 \
    --verify

# Compare before and after optimization
warcraft-rs mpq compare old_v1.mpq optimized.mpq --output summary

# Batch upgrade multiple archives
for archive in *.mpq; do
    echo "Upgrading $archive..."
    warcraft-rs mpq rebuild "$archive" "v4_${archive}" --upgrade-to v4
    warcraft-rs mpq compare "$archive" "v4_${archive}" --metadata-only
done
```

## WDL Subcommand Usage

> **Note**: WDL support requires enabling the WDL feature when building or running the CLI:
>
> ```bash
> # Build with WDL support
> cargo build --features wdl
> cargo build --features full  # Includes all format features
>
> # Run with WDL support
> cargo run --features wdl -- wdl validate Azeroth.wdl
> cargo run --features full -- wdl validate Azeroth.wdl
> ```

The `warcraft-rs` tool also provides WDL (World of Warcraft Low-resolution terrain) file operations:

### WDL Validation

```bash
# Validate WDL file structure
warcraft-rs wdl validate Azeroth.wdl

# Validate with verbose output
warcraft-rs wdl validate Azeroth.wdl --verbose

# Validate multiple files
warcraft-rs wdl validate *.wdl
```

### WDL Information

```bash
# Show basic WDL information
warcraft-rs wdl info Azeroth.wdl

# Detailed information with tile data
warcraft-rs wdl info Azeroth.wdl --detailed

# Show version and format details
warcraft-rs wdl info Azeroth.wdl --show-version
```

### WDL Conversion

```bash
# Convert WDL to heightmap image
warcraft-rs wdl convert Azeroth.wdl --output azeroth_heightmap.png

# Convert to grayscale heightmap
warcraft-rs wdl convert Azeroth.wdl --format grayscale --output heightmap.png

# Convert to colorized heightmap
warcraft-rs wdl convert Azeroth.wdl --format color --output colorized.png

# Specify custom output directory
warcraft-rs wdl convert Azeroth.wdl --output-dir ./heightmaps/
```

### WDL Tree Visualization

Visualize the structure of WDL files:

```bash
# Basic tree view showing chunk structure
warcraft-rs wdl tree Azeroth.wdl

# Limit depth for focused view
warcraft-rs wdl tree Azeroth.wdl --depth 2

# Show external file references (WMO models)
warcraft-rs wdl tree Azeroth.wdl --show-refs

# Compact mode for quick overview
warcraft-rs wdl tree Azeroth.wdl --compact

# No color output (for piping)
warcraft-rs wdl tree Azeroth.wdl --no-color
```

The tree view shows:

- 📦 WDL file structure with all chunks
- 🗂️ Chunk hierarchy (MVER, MAOF, MARE, etc.)
- 📊 Chunk sizes and metadata
- 🏛️ WMO model references if present
- 🎨 Color-coded output for readability

### WDL Batch Operations

```bash
# Convert all WDL files in a directory
for wdl in *.wdl; do
    echo "Converting $wdl..."
    warcraft-rs wdl convert "$wdl" --output "${wdl%.wdl}_heightmap.png"
done

# Validate all WDL files and generate report
echo "WDL Validation Report" > wdl_report.txt
for wdl in *.wdl; do
    echo "=== $wdl ===" >> wdl_report.txt
    warcraft-rs wdl validate "$wdl" >> wdl_report.txt 2>&1
    echo >> wdl_report.txt
done

# Extract WDL files from MPQ and convert
warcraft-rs mpq extract world.mpq --filter "*.wdl" --output ./extracted_wdl/
for wdl in ./extracted_wdl/*.wdl; do
    warcraft-rs wdl convert "$wdl" --output "${wdl%.wdl}_heightmap.png"
done
```

### Archive Comparison and Analysis

```bash
# Compare different game versions
warcraft-rs mpq compare wow_1.12.1_dbc.mpq wow_2.4.3_dbc.mpq --detailed

# Find differences in specific content
warcraft-rs mpq compare original.mpq modified.mpq --filter "*.dbc" --content-check

# Quick metadata comparison for multiple archives
for target in rebuilt_*.mpq; do
    original="${target/rebuilt_/}"
    if [[ -f "$original" ]]; then
        echo "=== Comparing $original vs $target ==="
        warcraft-rs mpq compare "$original" "$target" --metadata-only
    fi
done
```

### Quality Assurance Workflow

```bash
# Complete QA workflow for archive processing
original="source.mpq"
rebuilt="rebuilt.mpq"

echo "=== Quality Assurance Workflow ==="

# 1. Validate original archive integrity
echo "1. Validating original archive..."
warcraft-rs mpq validate "$original"

# 2. Rebuild archive
echo "2. Rebuilding archive..."
warcraft-rs mpq rebuild "$original" "$rebuilt" --verify

# 3. Compare archives thoroughly
echo "3. Comparing archives..."
warcraft-rs mpq compare "$original" "$rebuilt" --content-check --output summary

# 4. Spot check random files
echo "4. Spot checking files..."
files=$(warcraft-rs mpq list "$original" | shuf | head -5)
for file in $files; do
    echo "Checking: $file"
    # Extract and compare individual files if needed
done

echo "=== QA Complete ==="
```

## Error Handling

Common errors and solutions:

```bash
# File not found
Error: Failed to open archive: nonexistent.mpq
# Solution: Check file path and permissions

# No listfile warning
Warning: No (listfile) found - cannot enumerate files
# Note: You can still extract specific files if you know their names

# Archive format errors
Error: Invalid MPQ header
# Solution: Check if file is actually an MPQ archive

# Blizzard archive warnings (informational only)
Warning: Attributes file size mismatch: ... difference=-28 (tolerating for compatibility)
# Note: This is normal for all official WoW archives - they work perfectly
```

## Performance Tips

1. **Use filters** to reduce processing time when working with large archives
2. **Enable quiet mode** (`-q`) for scripting to reduce output overhead
3. **Extract to SSD** for better performance with many small files
4. **Use specific file names** when possible instead of extracting everything
5. **Enable verbose mode** (`-v`) for debugging when commands fail
