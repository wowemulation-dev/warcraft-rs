# MPQ CLI Usage Guide

The `warcraft-rs` command-line tool provides MPQ archive operations through the
`mpq` subcommand.

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

### Verify Archives

```bash
# Basic verification
warcraft-rs mpq verify archive.mpq
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
warcraft-rs mpq extract patch.warcraft-rs mpq "Interface/Icons/INV_Misc_QuestionMark.blp" --preserve-paths --output ./extracted

# Extract multiple related files
warcraft-rs mpq extract common.warcraft-rs mpq "DBFilesClient/ItemDisplayInfo.dbc" "DBFilesClient/Item.dbc"
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
warcraft-rs mpq list archive.warcraft-rs mpq --filter "*.m2" --long

# Look for specific content
warcraft-rs mpq list archive.warcraft-rs mpq --filter "*Stormwind*"

# Extract database files for analysis
warcraft-rs mpq extract common.warcraft-rs mpq --filter "*.dbc" --output ./dbc_files
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
# Verify archive integrity
warcraft-rs mpq verify archive.mpq

# Extract specific content for analysis
warcraft-rs mpq extract archive.mpq --filter "DBFilesClient/*" --output ./database_files --preserve-paths
warcraft-rs mpq extract archive.mpq --filter "Interface/Icons/*" --output ./icons --preserve-paths
```

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

# 1. Verify original archive integrity
echo "1. Verifying original archive..."
warcraft-rs mpq verify "$original"

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
```

## Performance Tips

1. **Use filters** to reduce processing time when working with large archives
2. **Enable quiet mode** (`-q`) for scripting to reduce output overhead
3. **Extract to SSD** for better performance with many small files
4. **Use specific file names** when possible instead of extracting everything
5. **Enable verbose mode** (`-v`) for debugging when commands fail
