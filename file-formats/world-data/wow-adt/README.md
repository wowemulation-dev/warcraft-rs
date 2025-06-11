# wow-adt

[![Crates.io](https://img.shields.io/crates/v/wow-adt.svg)](https://crates.io/crates/wow-adt)
[![Documentation](https://docs.rs/wow-adt/badge.svg)](https://docs.rs/wow-adt)
[![License](https://img.shields.io/crates/l/wow-adt.svg)](https://github.com/wowemulation-dev/warcraft-rs#license)

Parser for World of Warcraft ADT (terrain) files.

## Status

✅ **Production Ready** - Full parsing and validation support for ADT terrain files.

See [STATUS.md](STATUS.md) for detailed implementation status.

## Overview

ADT files contain terrain and object information for WoW map tiles. Each map in
World of Warcraft is divided into 64x64 tiles, with each tile stored as an ADT
file.

## Features

- **Full ADT Parsing** - Read and parse all chunk types
- **Version Support** - Classic through Cataclysm+
- **Validation** - Comprehensive validation with multiple strictness levels
- **Version Conversion** - Convert between different WoW versions
- **Split File Support** - Handle Cataclysm+ split ADT files (_tex0,_obj0, etc.)
- **Tree Visualization** - Visualize ADT structure hierarchically
- **Extract Support** (optional) - Extract heightmaps, textures, and model references
- **Parallel Processing** (optional) - Batch process multiple ADT files

## Supported Versions

- ✅ Classic (1.12.1)
- ✅ The Burning Crusade (2.4.3)
- ✅ Wrath of the Lich King (3.3.5a)
- ✅ Cataclysm (4.3.4)
- ⚠️  Mists of Pandaria (5.4.8) - Basic support, may need updates

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
wow-adt = "0.1"
```

### Basic Parsing

```rust
use wow_adt::{Adt, AdtVersion};

// Parse an ADT file
let adt = Adt::from_path("path/to/terrain.adt")?;

// Get version information
println!("ADT Version: {:?}", adt.version());

// Access terrain chunks
println!("Terrain chunks: {}", adt.mcnk_chunks().len());

// Check for water data
if let Some(water) = adt.mh2o() {
    println!("Contains water data");
}
```

### Validation

```rust
use wow_adt::{Adt, ValidationLevel};

let adt = Adt::from_path("terrain.adt")?;

// Basic validation
adt.validate()?;

// Detailed validation with report
let report = adt.validate_with_report(ValidationLevel::Strict)?;
if !report.errors.is_empty() {
    for error in &report.errors {
        eprintln!("Error: {}", error);
    }
}
```

### Version Conversion

```rust
use wow_adt::{Adt, AdtVersion};

let adt = Adt::from_path("vanilla_terrain.adt")?;

// Convert to Cataclysm format
let cata_adt = adt.to_version(AdtVersion::Cataclysm)?;

// Write to file
use std::fs::File;
use std::io::BufWriter;

let file = File::create("cata_terrain.adt")?;
let mut writer = BufWriter::new(file);
cata_adt.write(&mut writer)?;
```

### CLI Usage

The ADT functionality is integrated into the `warcraft-rs` CLI:

```bash
# Show ADT file information
warcraft-rs adt info terrain.adt

# Validate an ADT file
warcraft-rs adt validate terrain.adt --level strict

# Convert between versions
warcraft-rs adt convert input.adt output.adt --to cataclysm

# Visualize ADT structure
warcraft-rs adt tree terrain.adt --show-refs
```

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](../../LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
