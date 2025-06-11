# warcraft-rs

A collection of crates handling World of Warcraft file formats for WoW 1.12.1,
2.4.3, 3.3.5a, 4.3.4 and 5.4.8 (from Vanilla to Mists of Pandaria).

<div align="center">

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![CI Status](https://github.com/wowemulation-dev/warcraft-rs/workflows/CI/badge.svg)](https://github.com/wowemulation-dev/warcraft-rs/actions)
[![codecov](https://img.shields.io/codecov/c/github/wowemulation-dev/warcraft-rs?logo=codecov&style=flat-square&token=BAQ8SOKEST&color=C43AC3)](https://codecov.io/gh/wowemulation-dev/warcraft-rs)
[![Awesome WoW Rust](https://awesome.re/badge.svg)](https://github.com/arlyon/awesome-wow-rust)

</div>

`warcraft-rs` is part of the Rust WoW community. See
[awesome-wow-rust](https://github.com/arlyon/awesome-wow-rust) for other
projects and a link to the WoW Rust Discord.

## Features

### 📦 Format Support

- **MPQ Archives** - Read, write, modify, rebuild and compare MPQ archive files (v1-v4)
  - ✅ **StormLib Compatibility** - Bidirectional compatibility with reference implementation
  - ✅ **WoW Version Support** - Tested with versions 1.12.1 through 5.4.8
  - ✏️ **Archive Modification** - Add, remove, and rename files with automatic listfile/attributes updates
  - 🔄 **Archive Rebuilding** - Recreate archives with format upgrades and optimization
  - 🔍 **Archive Comparison** - Compare archives for differences in metadata, files, and content
  - 🔐 **Digital Signatures** - Generate and verify archive signatures
  - 🎮 **Official WoW Archive Support** - Handles Blizzard-specific quirks and format variations
- **DBC Database** - Parse client database files
- **BLP Textures** - Handle texture files
- **M2 Models** - Work with character and creature models
- **WMO Objects** - Process world map objects (buildings, structures)
  - ✅ **Format Support** - Parse and write root and group files
  - ✅ **WoW Versions** - Supports v17 (Classic) through v27 (The War Within)
  - 🔄 **Version Conversion** - Convert between expansions (Classic → Cataclysm, etc.)
  - 🏗️ **Builder API** - Create WMO files programmatically
  - 🔍 **Validation** - Field-level and structural checks
- **ADT Terrain** - Parse terrain and map data
- **WDT Maps** - World map definitions and tile layouts
- **WDL Maps** - Low-resolution terrain heightmaps

### 🛠️ Command-Line Tools

CLI tools for each format:

```bash
# MPQ archive manipulation
warcraft-rs mpq list archive.mpq
warcraft-rs mpq extract archive.mpq --output ./extracted
warcraft-rs mpq create new.mpq --add file1.txt --add file2.dat
warcraft-rs mpq info archive.mpq
warcraft-rs mpq tree archive.mpq  # Visualize archive structure

# Archive rebuild and comparison
warcraft-rs mpq rebuild original.mpq rebuilt.mpq --upgrade-to v4
warcraft-rs mpq compare original.mpq rebuilt.mpq --content-check

# WDL terrain data manipulation
warcraft-rs wdl validate terrain.wdl
warcraft-rs wdl info terrain.wdl
warcraft-rs wdl convert terrain.wdl terrain_new.wdl --to wotlk
warcraft-rs wdl tree terrain.wdl  # Visualize WDL structure

# WDT map operations
warcraft-rs wdt info map.wdt
warcraft-rs wdt validate map.wdt
warcraft-rs wdt convert map.wdt map_new.wdt --to wotlk
warcraft-rs wdt tiles map.wdt
warcraft-rs wdt tree map.wdt  # Visualize WDT structure

# ADT terrain operations
warcraft-rs adt info terrain.adt
warcraft-rs adt validate terrain.adt --level strict
warcraft-rs adt convert classic.adt cata.adt --to cataclysm
warcraft-rs adt tree terrain.adt  # Visualize ADT structure

# WMO object operations
warcraft-rs wmo info building.wmo
warcraft-rs wmo validate building.wmo --warnings
warcraft-rs wmo convert classic.wmo modern.wmo --to 21
warcraft-rs wmo tree building.wmo  # Visualize WMO structure
warcraft-rs wmo edit building.wmo --set-flag has-fog
warcraft-rs wmo build new.wmo --from config.yaml

# Other tools (when implemented)
warcraft-rs dbc list items.dbc
warcraft-rs blp convert texture.blp --format png
warcraft-rs m2 info model.m2
```

### 📚 Library Usage

Using formats as Rust libraries:

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

```rust
// ADT terrain parsing
use wow_adt::{Adt, ValidationLevel};
use std::fs::File;

let file = File::open("terrain.adt")?;
let adt = Adt::from_reader(file)?;
println!("ADT version: {:?}", adt.version);
println!("Terrain chunks: {}", adt.mcnk_chunks().len());

// Validate the ADT file
let report = adt.validate_with_report(ValidationLevel::Standard)?;
println!("Validation passed with {} warnings", report.warnings.len());
```

```rust
// WMO object parsing and manipulation
use wow_wmo::{WmoParser, WmoWriter, WmoVersion, WmoConverter};
use std::fs;
use std::io::Cursor;

// Parse WMO root file
let data = fs::read("building.wmo")?;
let mut cursor = Cursor::new(&data);
let parser = WmoParser::new();
let wmo = parser.parse_root(&mut cursor)?;
println!("WMO version: v{}", wmo.version.to_raw());
println!("Groups: {}", wmo.groups.len());
println!("Materials: {}", wmo.materials.len());

// Convert to a different version
let converter = WmoConverter::new();
let converted_wmo = converter.convert(&wmo, WmoVersion::Cataclysm)?;

// Save the converted file
let writer = WmoWriter::new();
let mut output = Vec::new();
writer.write_root(&mut output, &converted_wmo, WmoVersion::Cataclysm)?;
fs::write("building_cata.wmo", output)?;
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
wow-adt = "0.1"
wow-wdt = "0.1"
wow-wdl = "0.1"
wow-wmo = "0.1"
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
warcraft-rs mpq tree patch.mpq --depth 3  # Visualize archive structure

# Extract files
warcraft-rs mpq extract patch.mpq --output ./extracted --preserve-paths

# Rebuild an archive
warcraft-rs mpq rebuild old.mpq modern.mpq --upgrade-to v4 --compression lzma

# Verify the rebuild
warcraft-rs mpq compare old.mpq modern.mpq --content-check --output summary

# Visualize file structures
warcraft-rs wdt tree Azeroth.wdt --show-refs  # See ADT tile references
warcraft-rs wdl tree Azeroth.wdl --compact     # Compact view of WDL chunks
warcraft-rs adt tree terrain.adt --show-refs  # See terrain chunk structure
warcraft-rs wmo tree building.wmo --show-refs # See WMO structure with references
```

## Documentation

Documentation in the `docs/` directory:

- [Getting Started](docs/getting-started/quick-start.md)
- [Format Documentation](docs/formats/)
- [API Reference](docs/api/)
- [Examples and Guides](docs/guides/)
  - **[📦 MPQ CLI Usage Guide](docs/guides/mpq-cli-usage.md)** - CLI
    reference with examples
  - **[📦 MPQ Archives Guide](docs/guides/mpq-archives.md)** - Programming guide
    with rebuild and comparison APIs
  - **[🔍 StormLib vs wow-mpq](docs/guides/stormlib-differences.md)** - Technical
    comparison with the reference implementation
  - **[🏰 WMO CLI Usage Guide](docs/guides/wmo-cli-usage.md)** - Guide
    for working with World Map Objects

## 🤝 Contributing

See the [Contributing Guide](CONTRIBUTING.md) for:

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

Thanks to all [contributors](CONTRIBUTORS.md).

---

*This project represents the collective knowledge of the WoW modding community
and is based on reverse engineering efforts. Blizzard Entertainment has not
officially documented any formats handled by this project.*
