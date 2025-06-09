# CLI Architecture for warcraft-rs

This document outlines the CLI structure for the warcraft-rs project.

## Overview

The `warcraft-rs` CLI provides a unified interface for working with World of
Warcraft file formats through subcommands for each format type.

Currently implemented:

- ✅ **MPQ subcommands** - Full-featured MPQ archive operations with 98.75% StormLib compatibility
  - `list` - List archive contents
  - `extract` - Extract files
  - `info` - Show archive information
  - `verify` - Verify archive integrity
  - `create` - Create new archives
  - `rebuild` - Rebuild archives with format upgrades
  - `compare` - Compare two archives

Planned for future releases:

- 🚧 **WDL subcommands** - Low-res world operations (crate exists, CLI pending)
- 🚧 **DBC subcommands** - Database file operations
- 🚧 **BLP subcommands** - Texture file operations
- 🚧 **M2 subcommands** - Model file operations
- 🚧 **WMO subcommands** - World object operations
- 🚧 **ADT subcommands** - Terrain operations
- 🚧 **WDT subcommands** - Map definition operations

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
│   │   ├── dbc.rs         # DBC subcommands (planned)
│   │   ├── blp.rs         # BLP subcommands (planned)
│   │   ├── m2.rs          # M2 subcommands (planned)
│   │   ├── wmo.rs         # WMO subcommands (planned)
│   │   ├── adt.rs         # ADT subcommands (planned)
│   │   ├── wdt.rs         # WDT subcommands (planned)
│   │   └── wdl.rs         # WDL subcommands (planned)
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
warcraft-rs dbc export items.dbc --format json
warcraft-rs blp convert texture.blp --to png
warcraft-rs m2 info model.m2
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

# Verify archive integrity
warcraft-rs mpq verify archive.mpq [--check-checksums]
```

## Feature Flags

The CLI supports feature flags to include only the formats you need:

```toml
[features]
default = ["mpq"]  # MPQ is included by default
full = ["mpq", "dbc", "blp", "m2", "wmo", "adt", "wdt", "wdl"]
mpq = []
dbc = ["dep:wow-dbc"]
blp = ["dep:wow-blp"]
m2 = ["dep:wow-m2"]
wmo = ["dep:wow-wmo"]
adt = ["dep:wow-adt"]
wdt = ["dep:wow-wdt"]
wdl = ["dep:wow-wdl"]
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
wow-dbc = { path = "../file-formats/database/wow-dbc", optional = true }
# ... other format crates ...

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

    /// Verify integrity of an MPQ archive
    Verify {
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
# Build the CLI with default features (MPQ only)
cd warcraft-rs
cargo build --release

# Build with all features
cargo build --release --features full

# Build with specific features
cargo build --release --features "mpq dbc blp"

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

When implementing additional CLI tools, consider:

1. **Consistency**: Follow the same command patterns as the MPQ CLI
2. **Shared utilities**: Use the `utils` module for common functionality
3. **Error handling**: Use `anyhow` for consistent error reporting
4. **Testing**: Include both unit and integration tests
5. **Documentation**: Update this document and usage guides

## Limitations

- Only MPQ subcommands are fully implemented
- Other format subcommands return "not yet implemented" errors
- No man pages (shell completions are available via `warcraft-rs completions`)
