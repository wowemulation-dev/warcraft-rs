# warcraft-rs

A comprehensive Rust library and CLI toolset for parsing, manipulating, and creating World of Warcraft file formats.

<div align="center">

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![CI Status](https://github.com/wowemulation-dev/warcraft-rs/workflows/CI/badge.svg)](https://github.com/wowemulation-dev/warcraft-rs/actions)
[![codecov](https://img.shields.io/codecov/c/github/wowemulation-dev/warcraft-rs?logo=codecov&style=flat-square&token=BAQ8SOKEST&color=C43AC3)](https://codecov.io/gh/wowemulation-dev/warcraft-rs)
[![Awesome WoW Rust](https://awesome.re/badge.svg)](https://github.com/arlyon/awesome-wow-rust)

</div>

Part of the [awesome-wow-rust](https://github.com/arlyon/awesome-wow-rust) community.

## WoW Version Support

Supports World of Warcraft versions **1.12.1 through 5.4.8**:

| Version | Expansion | Status |
|---------|-----------|--------|
| 1.12.1 | Vanilla | ✅ Full Support |
| 2.4.3 | Burning Crusade | ✅ Full Support |
| 3.3.5a | Wrath of the Lich King | ✅ Full Support |
| 4.3.4 | Cataclysm | ✅ Full Support |
| 5.4.8 | Mists of Pandaria | ✅ Full Support |

## Supported File Formats

- **MPQ Archives** - Game data archives with StormLib compatibility
- **DBC/DB2** - Client database files containing game data
- **BLP Textures** - Compressed texture format with DXT and palette support
- **M2 Models** - Character, creature, and object 3D models
- **WMO Objects** - World map objects (buildings, structures)
- **ADT Terrain** - Terrain chunks with heightmaps and textures
- **WDT Maps** - World definitions and tile layouts
- **WDL Maps** - Low-resolution terrain heightmaps

## Quick Start

### Command-Line Usage

```bash
# Extract files from MPQ archives
warcraft-rs mpq extract patch.mpq --output ./extracted

# Get information about any file format
warcraft-rs mpq info archive.mpq
warcraft-rs blp info texture.blp
warcraft-rs wdt info map.wdt

# Convert between formats and versions
warcraft-rs blp convert texture.blp texture.png
warcraft-rs wmo convert classic.wmo modern.wmo --to cataclysm
```

### Library Usage

```rust
use wow_mpq::Archive;
use wow_blp::parser::load_blp;

// Read files from MPQ archives
let mut archive = Archive::open("patch.mpq")?;
let file_data = archive.read_file("Interface/Icons/spell.blp")?;

// Parse BLP textures
let blp_image = load_blp("texture.blp")?;
println!("Texture: {}x{}", blp_image.header.width, blp_image.header.height);
```

Add to your `Cargo.toml`:

```toml
[dependencies]
wow-mpq = "0.1"
wow-blp = "0.1"
wow-adt = "0.1"
# ... other formats as needed
```

## Installation

```bash
# Install CLI tools
git clone https://github.com/wowemulation-dev/warcraft-rs
cd warcraft-rs
cargo install --path .

# Or build from source
cargo build --release
```

## Documentation

- [Getting Started Guide](docs/getting-started/quick-start.md)
- [Format Documentation](docs/formats/)
- [API Reference](docs/api/)
- [Usage Examples](docs/guides/)

## Contributing

See the [Contributing Guide](CONTRIBUTING.md) for development setup and guidelines.

Thanks to all [contributors](CONTRIBUTORS.md).

---

*This project represents the collective knowledge of the WoW modding community
and is based on reverse engineering efforts. Blizzard Entertainment has not
officially documented any formats handled by this project.*
