# wow-wmo

A comprehensive Rust library for parsing, editing, validating, and converting World of Warcraft WMO (World Model Object) files.

<div align="center">

[![Crates.io Version](https://img.shields.io/crates/v/wow-wmo)](https://crates.io/crates/wow-wmo)
[![docs.rs](https://img.shields.io/docsrs/wow-wmo)](https://docs.rs/wow-wmo)
[![License](https://img.shields.io/crates/l/wow-wmo.svg)](https://github.com/wowemulation-dev/warcraft-rs#license)

</div>

## Status

✅ **Production Ready** - Comprehensive WMO parser with full format support

## Overview

WMO files represent buildings, dungeons, and other large structures in World of Warcraft. They consist of a root file containing metadata and multiple group files containing geometry data.

## Features

### Core Functionality

- **Parse WMO files** from all World of Warcraft versions (Classic through The War Within)
- **Parse WMO group files** with full geometry and rendering data
- **Validate WMO files** with detailed error reporting
- **Convert WMO files** between different versions (upgrading and downgrading)
- **Edit WMO files** programmatically
- **Write WMO files** with proper chunk formatting
- **Builder API** for creating WMO files from scratch
- **Tree visualization** for inspecting WMO structure

### Supported Chunks

- Root file chunks: MVER, MOHD, MOTX, MOMT, MOGN, MOGI, MOSB, MOPV, MOPT, MOPR, MOVV, MOVB, MOLT, MODS, MODN, MODD, MFOG, MCVP, GFID
- Group file chunks: MOGP, MOPY, MOVI, MOVT, MONR, MOTV, MOBA, MOLR, MODR, MOBN, MOBR, MOCV, MLIQ, MORI, MORB, MOTA, MOBS

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
wow-wmo = "0.2.0"
```

Or use cargo add:

```bash
cargo add wow-wmo
```

## Usage

### Parsing a WMO file

```rust
use wow_wmo::{WmoParser, WmoGroupParser};
use std::fs::File;
use std::io::BufReader;

// Parse root file
let file = File::open("building.wmo")?;
let mut reader = BufReader::new(file);
let wmo = WmoParser::new().parse_root(&mut reader)?;

println!("WMO Version: v{}", wmo.version.to_raw());
println!("Groups: {}", wmo.groups.len());
println!("Materials: {}", wmo.materials.len());
println!("Textures: {:?}", wmo.textures);

// Parse group file
let group_file = File::open("building_000.wmo")?;
let mut group_reader = BufReader::new(group_file);
let group = WmoGroupParser::new().parse_group(&mut group_reader, 0)?;

println!("Vertices: {}", group.vertices.len());
println!("Triangles: {}", group.indices.len() / 3);
```

### Validating a WMO file

```rust
use wow_wmo::{WmoValidator, WmoParser};
use std::fs::File;
use std::io::BufReader;

let file = File::open("building.wmo")?;
let mut reader = BufReader::new(file);
let wmo = WmoParser::new().parse_root(&mut reader)?;

// Validate the WMO
let validator = WmoValidator::new();
let report = validator.validate_root(&wmo)?;

if !report.errors.is_empty() {
    for error in &report.errors {
        println!("Error: {:?}", error);
    }
}

for warning in &report.warnings {
    println!("Warning: {:?}", warning);
}
```

### Converting between versions

```rust
use wow_wmo::{WmoConverter, WmoVersion, WmoParser, WmoWriter};
use std::fs::File;
use std::io::{BufReader, Cursor};

let file = File::open("classic_building.wmo")?;
let mut reader = BufReader::new(file);
let mut wmo = WmoParser::new().parse_root(&mut reader)?;

// Convert from Classic to Cataclysm
let converter = WmoConverter::new();
converter.convert_root(&mut wmo, WmoVersion::Cataclysm)?;

// Write the converted file
let writer = WmoWriter::new();
let mut output = Vec::new();
let mut cursor = Cursor::new(&mut output);
writer.write_root(&mut cursor, &wmo, WmoVersion::Cataclysm)?;
std::fs::write("cata_building.wmo", output)?;
```

### Building a WMO programmatically

```rust
use wow_wmo::{WmoRoot, WmoMaterial, WmoMaterialFlags, WmoVersion, WmoWriter, Vec3, Color, WmoHeader, WmoFlags, BoundingBox};
use std::io::Cursor;

// Create a simple WMO structure
let wmo = WmoRoot {
    version: WmoVersion::Wotlk,
    materials: vec![WmoMaterial {
        flags: WmoMaterialFlags::UNLIT,
        shader: 0,
        blend_mode: 0,
        texture1: 0,
        emissive_color: Color::default(),
        sidn_color: Color::default(),
        framebuffer_blend: Color::default(),
        texture2: u32::MAX,
        diffuse_color: Color::default(),
        ground_type: 0,
    }],
    groups: vec![],
    portals: vec![],
    portal_references: vec![],
    visible_block_lists: vec![],
    lights: vec![],
    doodad_defs: vec![],
    doodad_sets: vec![],
    bounding_box: BoundingBox {
        min: Vec3 { x: -50.0, y: -50.0, z: 0.0 },
        max: Vec3 { x: 50.0, y: 50.0, z: 30.0 },
    },
    textures: vec!["world/generic/stone_floor.blp".to_string()],
    header: WmoHeader {
        n_materials: 1,
        n_groups: 0,
        n_portals: 0,
        n_lights: 0,
        n_doodad_names: 0,
        n_doodad_defs: 0,
        n_doodad_sets: 0,
        flags: WmoFlags::empty(),
        ambient_color: Color { r: 128, g: 128, b: 128, a: 255 },
    },
    skybox: None,
};

// Write to file
let writer = WmoWriter::new();
let mut output = Vec::new();
let mut cursor = Cursor::new(&mut output);
writer.write_root(&mut cursor, &wmo, WmoVersion::Wotlk)?;
std::fs::write("custom.wmo", output)?;
```

## CLI Integration

WMO functionality is integrated into the main `warcraft-rs` CLI:

```bash
# Get information about a WMO
warcraft-rs wmo info building.wmo --detailed

# Validate WMO structure
warcraft-rs wmo validate building.wmo --warnings

# Convert between versions
warcraft-rs wmo convert classic.wmo modern.wmo --to 21

# Visualize WMO structure
warcraft-rs wmo tree building.wmo --show-refs

# Edit WMO properties
warcraft-rs wmo edit building.wmo --set-flag has-fog

# Build from configuration
warcraft-rs wmo build output.wmo --from config.yaml
```

## Supported Versions

| Version | Expansion | Status |
|---------|-----------|--------|
| 17 | Classic - Wrath of the Lich King | ✅ Fully Supported |
| 18 | Cataclysm | ✅ Fully Supported |
| 19 | Mists of Pandaria | ✅ Fully Supported |
| 20 | Warlords of Draenor | ✅ Fully Supported |
| 21 | Legion | ✅ Fully Supported |
| 22 | Battle for Azeroth | ✅ Fully Supported |
| 23 | Battle for Azeroth (8.1+) | ✅ Fully Supported |
| 24 | Shadowlands | ✅ Fully Supported |
| 25 | Shadowlands (9.1+) | ✅ Fully Supported |
| 26 | Dragonflight | ✅ Fully Supported |
| 27 | The War Within | ✅ Fully Supported |

## Performance

- **Parsing**: ~5-50ms for typical WMO files
- **Validation**: ~1-10ms depending on strictness level
- **Conversion**: ~10-100ms depending on version gap
- **Writing**: ~5-20ms for typical files

## Examples

The crate includes several examples:

- `parse_wmo` - Basic WMO parsing example
- `validate_wmo` - Validation with different strictness levels
- `convert_wmo` - Version conversion example
- `build_wmo` - Creating WMO files programmatically

Run examples with:

```bash
cargo run --example parse_wmo
```

## Known Issues

1. **Parser Overflow**: Fixed - Group name parsing now handles pointer arithmetic correctly
2. **Header Size**: Fixed - MOHD chunk size corrected to 60 bytes
3. **Texture Validation**: Fixed - Special marker values (0xFF000000+) are now handled
4. **Light Types**: Fixed - Unknown light types default to Omni
5. **Doodad Structure**: Fixed - Always uses 40 bytes for proper round-trip conversion

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](../../../LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](../../../LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
