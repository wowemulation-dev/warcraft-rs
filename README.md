# warcraft-rs

A collection of crates handling World of Warcraft file formats for WoW 1.12.1,
2.4.3, 3.3.5a, 4.3.4 an 5.4.8 (from Vanilla to Mists of Pandaria).

<div align="center">

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![CI Status](https://github.com/wowemulation-dev/warcraft-rs/workflows/CI/badge.svg)](https://github.com/wowemulation-dev/warcraft-rs/actions)
[![codecov](https://img.shields.io/codecov/c/github/wowemulation-dev/warcraft-rs?logo=codecov&style=flat-square&token=BAQ8SOKEST&color=C43AC3)](https://codecov.io/gh/wowemulation-dev/warcraft-rs)

</div>

## Features

### 📦 Format Support

- **MPQ Archives** - Read, write, modify, rebuild and compare MPQ archive files (v1-v4)
  - ✅ **Full StormLib Compatibility** - 100% bidirectional compatibility with reference implementation
  - ✅ **100% WoW Version Support** - Tested with all versions from 1.12.1 through 5.4.8
  - ✏️ **Archive Modification** - Add, remove, and rename files with automatic listfile/attributes updates
  - 🔄 **Archive Rebuilding** - Recreate archives 1:1 with format upgrades and optimization
  - 🔍 **Archive Comparison** - Compare archives for differences in metadata, files, and content
  - 🔐 **Digital Signatures** - Generate and verify archive signatures for integrity protection
  - 🎮 **Official WoW Archive Support** - Handles all Blizzard-specific quirks and format variations
- **DBC Database** - Parse client database files
- **BLP Textures** - Handle texture files
- **M2 Models** - Work with character and creature models
- **WMO Objects** - Process world map objects
- **ADT Terrain** - Parse terrain and map data
- **WDT Maps** - Handle world map definitions
- **WDL Maps** - Low-resolution terrain heightmaps

### 🛠️ Command-Line Tools

Each format comes with its own CLI tool for common operations:

```bash
# MPQ archive manipulation
warcraft-rs mpq list archive.mpq
warcraft-rs mpq extract archive.mpq --output ./extracted
warcraft-rs mpq create new.mpq --add file1.txt --add file2.dat
warcraft-rs mpq info archive.mpq

# Archive rebuild and comparison (NEW!)
warcraft-rs mpq rebuild original.mpq rebuilt.mpq --upgrade-to v4
warcraft-rs mpq compare original.mpq rebuilt.mpq --content-check

# WDL terrain data manipulation
warcraft-rs wdl validate terrain.wdl
warcraft-rs wdl info terrain.wdl
warcraft-rs wdl convert terrain.wdl terrain_new.wdl --to wotlk

# Other tools (when implemented)
warcraft-rs dbc list items.dbc
warcraft-rs blp convert texture.blp --format png
warcraft-rs m2 info model.m2
```

### 📚 Library Usage

All formats can also be used as Rust libraries:

```rust
use wow_mpq::{Archive, ArchiveBuilder, MutableArchive, AddFileOptions};

// Open and read from MPQ
let mut archive = Archive::open("patch.mpq")?;
let data = archive.read_file("path/to/file.txt")?;

// Create new MPQ
ArchiveBuilder::new()
    .add_file_data(b"Hello, WoW!".to_vec(), "greeting.txt")
    .build("output.mpq")?;

// Modify existing MPQ
let mut mutable = MutableArchive::open("existing.mpq")?;
mutable.add_file_data(b"New content".as_ref(), "new_file.txt", AddFileOptions::default())?;
mutable.remove_file("old_file.txt")?;
mutable.rename_file("file.txt", "renamed.txt")?;
mutable.flush()?; // Save changes
```

## Installation

### CLI Tools

```bash
# Install individual tools
cargo install --path file-formats/archives/wow-mpq --features cli --bin mpq

# Install from source with all features
git clone https://github.com/wowemulation-dev/warcraft-rs
cd warcraft-rs
cargo build --release --features cli
```

### Library Usage

Add the crates you need to your `Cargo.toml`:

```toml
[dependencies]
wow-mpq = "0.1"
wow-dbc = "0.1"
wow-blp = "0.1"
```

## Quick Start Example

```bash
# Install warcraft-rs
cargo install --path .

# Analyze an MPQ archive
warcraft-rs mpq info patch.mpq
warcraft-rs mpq list patch.mpq --filter "*.dbc" --long

# Extract files
warcraft-rs mpq extract patch.mpq --output ./extracted --preserve-paths

# Rebuild and modernize an archive
warcraft-rs mpq rebuild old.mpq modern.mpq --upgrade-to v4 --compression lzma

# Verify the rebuild
warcraft-rs mpq compare old.mpq modern.mpq --content-check --output summary
```

## Documentation

Comprehensive documentation is available in the `docs/` directory:

- [Getting Started](docs/getting-started/quick-start.md)
- [Format Documentation](docs/formats/)
- [API Reference](docs/api/)
- [Examples and Guides](docs/guides/)
  - **[📦 MPQ CLI Usage Guide](docs/guides/mpq-cli-usage.md)** - Complete CLI
    reference with rebuild and compare examples
  - **[📦 MPQ Archives Guide](docs/guides/mpq-archives.md)** - Programming guide
    with rebuild and comparison APIs
  - **[🔍 StormLib vs wow-mpq](docs/guides/stormlib-differences.md)** - Technical
    comparison with the reference implementation

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for
details on:

- Setting up your development environment
- Finding issues to work on
- Submitting pull requests
- Coding standards

### CI/CD Pipeline

This project uses GitHub Actions for continuous integration:

- **Automatic testing** on all pull requests
- **Cross-platform builds** for Linux, Windows, and macOS
- **Security audits** for dependencies
- **Performance benchmarks** with automatic PR comments
- **Code coverage** reporting

### Contributors

Special thanks to all our [contributors](CONTRIBUTORS.md)!

---

*This project represents the collective knowledge of the WoW modding community
and is based on reverse engineering efforts. Blizzard Entertainment has not
officially documented any formats handled by this project.*
