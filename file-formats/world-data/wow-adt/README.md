# wow-adt

Parser for World of Warcraft ADT (terrain) files.

<div align="center">

[![Crates.io Version](https://img.shields.io/crates/v/wow-adt)](https://crates.io/crates/wow-adt)
[![docs.rs](https://img.shields.io/docsrs/wow-adt)](https://docs.rs/wow-adt)
[![License](https://img.shields.io/crates/l/wow-adt.svg)](https://github.com/wowemulation-dev/warcraft-rs#license)

</div>

## Status

**Production Ready** - Full parsing and validation support for ADT terrain files.

## Overview

ADT files contain terrain and object information for WoW map tiles. Each map in
World of Warcraft is divided into 64x64 tiles, with each tile stored as an ADT
file.

## Features

- **Full ADT Parsing** - Read and parse all chunk types
- **Version Support** - Classic through Cataclysm+
- **Validation** - Multiple strictness levels
- **Version Conversion** - Convert between different WoW versions
- **Split File Support** - Handle Cataclysm+ split ADT files (_tex0,_obj0, etc.)
- **Tree Visualization** - Visualize ADT structure hierarchically
- **Extract Support** (optional) - Extract heightmaps, textures, and model references
- **Parallel Processing** (optional) - Batch process multiple ADT files

## Supported Versions

- Classic (1.12.1)
- The Burning Crusade (2.4.3)
- Wrath of the Lich King (3.3.5a)
- Cataclysm (4.3.4)
- Mists of Pandaria (5.4.8) - Basic support, may need updates

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
wow-adt = "0.6"
```

Or use cargo add:

```bash
cargo add wow-adt
```

## Usage

### Basic Parsing

```rust
use wow_adt::{parse_adt, ParsedAdt};
use std::fs::File;
use std::io::BufReader;

// Parse an ADT file
let file = File::open("path/to/terrain.adt")?;
let mut reader = BufReader::new(file);
let parsed = parse_adt(&mut reader)?;

// ParsedAdt is an enum - check what type was parsed
match &parsed {
    ParsedAdt::Root(root) => {
        println!("ADT Version: {}", root.version);
        println!("Terrain chunks: {}", root.mcnk_chunks.len());
        println!("Textures: {}", root.textures.len());

        if root.water_data.is_some() {
            println!("Contains water data");
        }
        if root.flight_bounds.is_some() {
            println!("Contains flight bounds (TBC+)");
        }
    }
    ParsedAdt::Tex0(_) => println!("Split texture file (_tex0.adt)"),
    ParsedAdt::Obj0(_) => println!("Split object file (_obj0.adt)"),
    _ => println!("File type: {:?}", parsed.file_type()),
}
```

### Version Conversion

```rust
use wow_adt::{parse_adt, ParsedAdt, AdtVersion};
use wow_adt::builder::BuiltAdt;
use std::fs::File;
use std::io::BufReader;

let file = File::open("vanilla_terrain.adt")?;
let mut reader = BufReader::new(file);
let parsed = parse_adt(&mut reader)?;

if let ParsedAdt::Root(root) = parsed {
    // Convert to Cataclysm format
    let built = BuiltAdt::from_root_adt(*root, Some(AdtVersion::Cataclysm));
    built.write_to_file("cata_terrain.adt")?;
}
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
