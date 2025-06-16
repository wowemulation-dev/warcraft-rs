# CLI Architecture for warcraft-rs

This document outlines the CLI structure for the warcraft-rs project.

## Overview

The `warcraft-rs` CLI provides a unified interface for working with World of
Warcraft file formats through subcommands for each format type.

Currently implemented:

- ✅ **MPQ subcommands** - Full-featured MPQ archive operations with 100% StormLib compatibility
  - `list` - List archive contents
  - `extract` - Extract files
  - `info` - Show archive information
  - `validate` - Validate archive integrity
  - `create` - Create new archives
  - `rebuild` - Rebuild archives with format upgrades
  - `compare` - Compare two archives
- ✅ **DBC subcommands** - Database file operations
  - `info` - Display information about a DBC file
  - `validate` - Validate a DBC file against a schema
  - `list` - List records in a DBC file
  - `export` - Export DBC data to JSON/CSV formats
  - `analyze` - Analyze DBC file performance and structure
  - `discover` - Discover the schema of a DBC file through analysis
- ✅ **DBD subcommands** - Database definition operations
  - `convert` - Convert a DBD file to YAML schemas
- ✅ **BLP subcommands** - Texture file operations
  - `info` - Display information about a BLP file
  - `validate` - Validate BLP file integrity
  - `convert` - Convert BLP files to/from other image formats (PNG, JPEG, etc.)
- ✅ **M2 subcommands** - Model file operations (basic functionality)
  - `info` - Display information about an M2 model file
  - `validate` - Validate an M2 model file
  - `convert` - Convert an M2 model to a different version
  - `tree` - Display M2 file structure as a tree
  - `skin-info` - Display information about a Skin file
  - `skin-convert` - Convert a Skin file to a different version
  - `anim-info` - Display information about an ANIM file
  - `anim-convert` - Convert an ANIM file to a different version
  - `blp-info` - Display information about a BLP texture file
- ✅ **WMO subcommands** - World object operations
  - `info` - Show information about a WMO file
  - `validate` - Validate a WMO file
  - `convert` - Convert WMO between different WoW versions
  - `list` - List WMO components (groups, materials, doodads, etc.)
  - `tree` - Visualize WMO structure as a tree
  - `export` - Export WMO data (not yet implemented)
  - `extract-groups` - Extract WMO groups (not yet implemented)
- ✅ **ADT subcommands** - Terrain operations
  - `info` - Show information about an ADT file
  - `validate` - Validate an ADT file
  - `convert` - Convert ADT between different WoW versions
  - `extract` - Extract data from ADT files (requires 'extract' feature)
  - `tree` - Visualize ADT structure as a tree
  - `batch` - Batch process multiple ADT files (requires 'parallel' feature)
- ✅ **WDL subcommands** - Low-res world operations
  - `validate` - Validate WDL file format
  - `info` - Show WDL file information
  - `convert` - Convert between WDL versions
- ✅ **WDT subcommands** - Map definition operations
  - `info` - Display WDT file information
  - `validate` - Validate WDT file structure
  - `convert` - Convert between WDT versions
  - `tiles` - List tiles with ADT data

## Project Structure

The CLI is implemented in the warcraft-rs crate:

```text
warcraft-rs/
├── Cargo.toml
├── src/
│   ├── main.rs            # Entry point
│   ├── cli.rs             # Root CLI structure
│   ├── commands/          # Format-specific commands
│   │   ├── mod.rs
│   │   ├── mpq.rs         # MPQ subcommands (implemented)
│   │   ├── dbc.rs         # DBC subcommands (implemented)
│   │   ├── dbd.rs         # DBD subcommands (implemented)
│   │   ├── blp.rs         # BLP subcommands (implemented)
│   │   ├── m2.rs          # M2 subcommands (implemented)
│   │   ├── wmo.rs         # WMO subcommands (implemented)
│   │   ├── adt.rs         # ADT subcommands (implemented)
│   │   ├── wdl.rs         # WDL subcommands (implemented)
│   │   └── wdt.rs         # WDT subcommands (implemented)
│   └── utils/             # Shared utilities
│       ├── mod.rs
│       ├── progress.rs    # Progress bars
│       ├── table.rs       # Table formatting
│       ├── format.rs      # Byte/time formatting
│       └── io.rs          # File I/O helpers
```

## Command Pattern

All format subcommands follow a consistent pattern:

```rust
warcraft-rs <format> <action> [options]
```

Examples:

```bash
warcraft-rs mpq list archive.mpq
warcraft-rs dbc info items.dbc
warcraft-rs dbc export items.dbc --format json
warcraft-rs dbd convert definitions.dbd --output schemas/
warcraft-rs blp convert texture.blp --to png
warcraft-rs m2 info model.m2
warcraft-rs wmo tree worldobject.wmo
warcraft-rs adt batch process --input maps/ --output processed/
warcraft-rs wdl convert old.wdl new.wdl --to wotlk
warcraft-rs wdt tiles map.wdt
```

## Command Structure

### MPQ Subcommands

The MPQ subcommands currently support:

```bash
# List files in archive
warcraft-rs mpq list archive.mpq [--long] [--filter pattern]

# Extract files
warcraft-rs mpq extract archive.mpq [files...] [--output dir] [--preserve-paths]

# Create new archive
warcraft-rs mpq create new.mpq --add files... [--version v2] [--compression zlib]

# Show archive information
warcraft-rs mpq info archive.mpq [--show-hash-table] [--show-block-table]

# Validate archive integrity
warcraft-rs mpq validate archive.mpq [--check-checksums]
```

## Feature Flags

The CLI supports feature flags to include only the formats you need:

```toml
[features]
default = ["mpq", "dbc", "blp", "m2", "wmo", "adt", "wdt", "wdl"]  # All formats included by default
full = ["mpq", "dbc", "blp", "m2", "wmo", "adt", "wdt", "wdl", "serde", "extract", "parallel", "yaml"]
mpq = []  # MPQ is always included (no optional dependency)
dbc = ["dep:wow-cdbc"]
blp = ["dep:wow-blp", "dep:image"]
m2 = ["dep:wow-m2"]
wmo = ["dep:wow-wmo"]
adt = ["dep:wow-adt"]
wdt = ["dep:wow-wdt", "serde"]
wdl = ["dep:wow-wdl"]
serde = ["dep:serde", "dep:serde_json"]
extract = ["wow-adt?/extract"]
parallel = ["wow-adt?/parallel", "dep:rayon"]
yaml = ["dbc", "serde", "dep:serde_yaml_ng"]
```

## Implementation Guidelines

### 1. Dependencies

The CLI uses these dependencies:

```toml
# From warcraft-rs/Cargo.toml
[dependencies]
# CLI framework
clap = { version = "4.5", features = ["derive", "cargo", "env"] }
clap_complete = "4.5"

# File format crates
wow-mpq = { path = "../file-formats/archives/wow-mpq" }
wow-cdbc = { path = "../file-formats/database/wow-cdbc", optional = true }
wow-blp = { path = "../file-formats/graphics/wow-blp", optional = true }
wow-m2 = { path = "../file-formats/graphics/wow-m2", optional = true }
wow-wmo = { path = "../file-formats/graphics/wow-wmo", optional = true }
wow-adt = { path = "../file-formats/world-data/wow-adt", optional = true }
wow-wdt = { path = "../file-formats/world-data/wow-wdt", optional = true }
wow-wdl = { path = "../file-formats/world-data/wow-wdl", optional = true }

# Error handling and utilities
anyhow = "1.0"
log = "0.4"
env_logger = "0.11"
indicatif = "0.17"
prettytable-rs = "0.10"
```

### 2. Shared Utilities

The `utils` module provides common functionality:

```rust
// Available utilities in warcraft-rs/src/utils/:
- format_bytes()          // Human-readable file sizes
- format_timestamp()      // Time formatting
- format_percentage()     // Percentage formatting
- format_compression_ratio() // Compression ratio display
- create_progress_bar()   // Progress indicators
- create_spinner()        // Indeterminate progress
- create_table()         // Formatted table output
- add_table_row()        // Add rows to tables
- truncate_path()        // Path truncation for display
- matches_pattern()      // Wildcard matching
```

### 3. CLI Structure Example

Example CLI structure (from MPQ implementation):

```rust
// Simplified version of the actual CLI structure
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mpq")]
#[command(about = "MPQ archive manipulation tool for World of Warcraft")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Verbosity level (can be repeated)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress all output except errors
    #[arg(short, long)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List files in an MPQ archive
    List {
        /// Path to the MPQ archive
        archive: String,

        /// Show detailed information (size, compression ratio)
        #[arg(short, long)]
        long: bool,

        /// Filter files by pattern (supports wildcards)
        #[arg(short, long)]
        filter: Option<String>,
    },

    /// Extract files from an MPQ archive
    Extract {
        /// Path to the MPQ archive
        archive: String,

        /// Specific files to extract (extracts all if not specified)
        files: Vec<String>,

        /// Output directory
        #[arg(short, long, default_value = ".")]
        output: String,

        /// Preserve directory structure
        #[arg(short, long)]
        preserve_paths: bool,
    },

    /// Show information about an MPQ archive
    Info {
        /// Path to the MPQ archive
        archive: String,

        /// Show hash table details
        #[arg(long)]
        show_hash_table: bool,

        /// Show block table details
        #[arg(long)]
        show_block_table: bool,
    },

    /// Validate integrity of an MPQ archive
    Validate {
        /// Path to the MPQ archive
        archive: String,
    },

    // Future commands (not yet implemented):
    // Create, Add, Remove, Repair
}
```

## Installation

The CLI can be built and used as follows:

```bash
# Build the CLI with default features (all formats)
cd warcraft-rs
cargo build --release

# Build with all features including extras (serialization, parallel processing, etc.)
cargo build --release --features full

# Build with only specific features
cargo build --release --no-default-features --features "mpq dbc blp"

# Install globally
cargo install --path . --features full

# Example usage
warcraft-rs mpq list archive.mpq
warcraft-rs mpq info archive.mpq
warcraft-rs mpq extract archive.mpq --output ./extracted
```

## Testing

The CLI includes integration tests:

```bash
# Run CLI tests
cd warcraft-rs
cargo test

# Test with actual files
warcraft-rs mpq info /path/to/test.mpq
warcraft-rs mpq list /path/to/archive.mpq --filter "*.dbc"
```

## Future Considerations

When implementing additional CLI tools or enhancing existing ones, consider:

1. **Consistency**: Follow the same command patterns as existing CLIs
2. **Shared utilities**: Use the `utils` module for common functionality
3. **Error handling**: Use `anyhow` for consistent error reporting
4. **Testing**: Include both unit and integration tests
5. **Documentation**: Update this document and usage guides
6. **Feature flags**: Consider adding feature flags for optional functionality

## Current Limitations

- Some M2 subcommands have limited functionality due to API constraints
- WMO `export` and `extract-groups` subcommands are not yet implemented
- ADT `extract` and `batch` commands require additional feature flags
- No man pages (shell completions are generated but not yet exposed via CLI)
