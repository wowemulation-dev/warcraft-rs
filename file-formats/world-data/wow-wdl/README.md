# wow-wdl

Parser for World of Warcraft WDL (World Detail Level) files - low-resolution terrain
data for continents.

<div align="center">

[![Crates.io Version](https://img.shields.io/crates/v/wow-wdl)](https://crates.io/crates/wow-wdl)
[![docs.rs](https://img.shields.io/docsrs/wow-wdl)](https://docs.rs/wow-wdl)
[![License](https://img.shields.io/crates/l/wow-wdl.svg)](https://github.com/wowemulation-dev/warcraft-rs#license)

</div>

## Status

✅ **Production Ready** - Complete WDL parser with support for all WoW versions

## Overview

WDL files contain low-resolution heightmap data for entire WoW continents. They provide:

- **Low-resolution terrain heights** - 17x17 height points per ADT tile
- **Terrain hole information** - Which chunks have holes/gaps
- **World object placements** - Low-detail WMO positions (pre-Legion)
- **Model placements** - M2 and WMO positions (Legion+)

These files enable:

- Efficient rendering of distant terrain
- World map and minimap generation
- Level-of-detail (LoD) terrain switching
- Memory optimization by using low-detail data for distant areas

## Features

- ✅ Parse all WDL chunk types (MVER, MAOF, MARE, MAHO, etc.)
- ✅ Support for all WoW versions (Classic through Legion+)
- ✅ Height data extraction and interpolation
- ✅ Hole detection and manipulation
- ✅ World object placement data
- ✅ Version conversion between formats
- ✅ Validation and error handling

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
wow-wdl = "0.2.0"
```

Or use cargo add:

```bash
cargo add wow-wdl
```

## Quick Start

```rust,no_run
use wow_wdl::parser::WdlParser;
use std::fs::File;
use std::io::BufReader;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
// Load a WDL file
let file = File::open("World/Maps/Azeroth/Azeroth.wdl")?;
let mut reader = BufReader::new(file);

// Parse the file
let parser = WdlParser::new();
let wdl = parser.parse(&mut reader)?;

// Access heightmap data
if let Some(tile) = wdl.heightmap_tiles.get(&(32, 48)) {
    println!("Tile (32,48) has {} height values", tile.outer_values.len());
}

// Check for holes
if let Some(holes) = wdl.holes_data.get(&(32, 48)) {
    if holes.has_hole(8, 8) {
        println!("Chunk (8,8) has a hole!");
    }
}
# Ok(())
# }
```

## File Format

WDL files use a chunk-based structure:

| Chunk | Description | Required |
|-------|-------------|----------|
| **MVER** | Version number | ✅ |
| **MAOF** | Map area offset table (64x64 grid) | ✅ |
| **MARE** | Map area heightmap data (545 heights per tile) | ✅ |
| **MAHO** | Map area hole bitmasks | ❌ |
| **MWMO** | WMO filename list | ❌ |
| **MWID** | WMO filename offsets | ❌ |
| **MODF** | WMO placement data | ❌ |
| **ML*** | Legion+ model chunks | ❌ |

## Supported Versions

- ✅ **Classic** (1.12.1) - Version 18
- ✅ **The Burning Crusade** (2.4.3) - Version 18
- ✅ **Wrath of the Lich King** (3.3.5a) - Version 18
- ✅ **Cataclysm** (4.3.4) - Version 18
- ✅ **Mists of Pandaria** (5.4.8) - Version 18
- ✅ **Legion+** (7.0+) - Version 18 with ML** chunks

## Examples

### Height Interpolation

Height interpolation features are planned for future implementation:

```text
These features will be available in a future version:
let height = wdl.get_height_at(1234.5, 5678.9)?;
let (dx, dy) = wdl.get_height_gradient(1234.5, 5678.9)?;
```

### Hole Manipulation

```rust,no_run
# use wow_wdl::types::WdlFile;
# let mut wdl = WdlFile::new();
# let tile_x = 32u32;
# let tile_y = 48u32;
# let chunk_x = 8;
# let chunk_y = 8;
// Check if a specific chunk has a hole
let has_hole = wdl.holes_data
    .get(&(tile_x, tile_y))
    .map(|h| h.has_hole(chunk_x, chunk_y))
    .unwrap_or(false);

// Modify hole data
if let Some(holes) = wdl.holes_data.get_mut(&(32, 48)) {
    holes.set_hole(8, 8, true); // Create a hole
    holes.set_hole(9, 9, false); // Remove a hole
}
```

### Version Conversion

```rust,no_run
use wow_wdl::version::WdlVersion;
use wow_wdl::conversion::convert_wdl_file;
use wow_wdl::parser::WdlParser;
use std::fs::File;
use std::io::BufWriter;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
# use wow_wdl::types::WdlFile;
# let wdl = WdlFile::new();
// Convert pre-Legion WDL to Legion format
let legion_wdl = convert_wdl_file(&wdl, WdlVersion::Legion)?;

// Save the converted file
let output = File::create("converted.wdl")?;
let mut writer = BufWriter::new(output);
WdlParser::with_version(WdlVersion::Legion)
    .write(&mut writer, &legion_wdl)?;
# Ok(())
# }
```

## Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| MVER parsing | ✅ | All versions supported |
| MAOF parsing | ✅ | 64x64 offset grid |
| MARE parsing | ✅ | 17x17 + 16x16 heights |
| MAHO parsing | ✅ | Hole bitmasks |
| WMO chunks | ✅ | Pre-Legion support |
| ML** chunks | ✅ | Legion+ support |
| Version conversion | ✅ | Between all formats |
| Data validation | ✅ | Comprehensive validation |
| Error handling | ✅ | Robust error types |
| Height interpolation | 🚧 | Planned |
| Coordinate conversion | 🚧 | Planned |
| Minimap generation | 🚧 | Planned |

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](../../LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
