# wow-m2

A Rust library for parsing, validating, and converting World of Warcraft M2 model files.

<div align="center">

[![Crates.io Version](https://img.shields.io/crates/v/wow-m2)](https://crates.io/crates/wow-m2)
[![docs.rs](https://img.shields.io/docsrs/wow-m2)](https://docs.rs/wow-m2)
[![License](https://img.shields.io/crates/l/wow-mpq.svg)](https://github.com/wowemulation-dev/warcraft-rs#license)

</div>

## Overview

`wow-m2` provides comprehensive support for M2 model files across all World of Warcraft expansions from Classic (1.12.1) through The War Within (11.x). The library handles:

- **M2 Models** (`.m2`/`.mdx`) - 3D character, creature, and object models
- **Skin Files** (`.skin`) - Level-of-detail and submesh information
- **Animation Files** (`.anim`) - External animation sequences
- **BLP Texture References** - Re-exports BLP support from the [wow-blp](https://crates.io/crates/wow-blp) crate

## Features

- ✅ Parse and validate M2 models from all WoW versions
- ✅ Convert models between different game versions
- ✅ **Coordinate system transformations** for Blender, Unity, Unreal Engine
- ✅ Support for all chunk types (bones, animations, textures, etc.)
- ✅ Comprehensive error handling with detailed context
- ✅ Zero-copy parsing where possible for performance
- ✅ Optional serde support for serialization

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
wow-m2 = "0.3.0"
```

Or use cargo add:

```bash
cargo add wow-m2
```

## Usage

### Basic Example

```rust
use wow_m2::{M2Model, M2Version, M2Converter};

// Load a model
let data = std::fs::read("path/to/model.m2")?;
let mut cursor = std::io::Cursor::new(data);
let model = M2Model::parse(&mut cursor)?;

// Print basic information
println!("Model version: {:?}", model.header.version());
println!("Vertices: {}", model.vertices.len());
println!("Bones: {}", model.bones.len());

// Convert to a different version
let converter = M2Converter::new();
let converted = converter.convert(&model, M2Version::WotLK)?;
// Save the converted model
let output_data = converted.write_to_bytes()?;
std::fs::write("path/to/converted.m2", output_data)?;
```

### Working with Skin Files

```rust
use wow_m2::Skin;

// Load a skin file
let data = std::fs::read("path/to/model00.skin")?;
let mut cursor = std::io::Cursor::new(data);
let skin = Skin::parse(&mut cursor)?;

// Access submesh information
for submesh in &skin.submeshes {
    println!("Submesh {}: {} vertices, {} triangles",
        submesh.id, submesh.vertex_count, submesh.triangle_count);
}
```

### Coordinate System Transformations

Transform WoW models for use in common 3D applications:

```rust
use wow_m2::{M2Model, CoordinateSystem, CoordinateTransformer, transform_position};

// Load a model
let model = M2Model::load("character.m2")?;

// Transform for Blender (right-handed: X=right, Y=forward, Z=up)
let transformer = CoordinateTransformer::new(CoordinateSystem::Blender);
for vertex in &model.vertices {
    let blender_pos = transformer.transform_position(vertex.position);
    println!("WoW: {:?} → Blender: {:?}", vertex.position, blender_pos);
}

// Or transform individual coordinates
let wow_pos = model.vertices[0].position;
let unity_pos = transform_position(wow_pos, CoordinateSystem::Unity);
let unreal_pos = transform_position(wow_pos, CoordinateSystem::UnrealEngine);
```

**Why coordinate transformation is needed:** WoW uses X=North, Y=West, Z=Up, which differs from standard 3D applications. Without transformation, models appear rotated or mirrored.

### Version Support

The library supports parsing versions by both numeric format and expansion names:

```rust
use wow_m2::M2Version;

// Using version numbers
let version = M2Version::from_string("3.3.5a")?;  // WotLK

// Using expansion names
let version = M2Version::from_expansion_name("wotlk")?;
let version = M2Version::from_expansion_name("MoP")?;
```

## Supported Versions

| Expansion | Version Range | Support |
|-----------|---------------|---------|
| Classic | 1.12.x | ✅ Full |
| TBC | 2.4.x | ✅ Full |
| WotLK | 3.3.x | ✅ Full |
| Cataclysm | 4.3.x | ✅ Full |
| MoP | 5.4.x | ✅ Full |
| WoD | 6.2.x | ✅ Full |
| Legion | 7.3.x | ✅ Full |
| BfA | 8.3.x | ✅ Full |
| Shadowlands | 9.x | ✅ Full |
| Dragonflight | 10.x | ✅ Full |
| The War Within | 11.x | ✅ Full |

## Examples

See the `examples/` directory for more detailed examples:

- `convert_model.rs` - Convert models between versions
- `analyze_model.rs` - Analyze model structure and contents
- `validate_model.rs` - Validate model integrity

## Documentation

- **[Coordinate Systems Guide](../../../docs/COORDINATE_SYSTEMS.md)** - Comprehensive guide to WoW coordinate systems and transformations
- **[API Documentation](https://docs.rs/wow-m2)** - Complete API reference

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../../../LICENSE-MIT))

at your option.

## Contributing

See [CONTRIBUTING.md](../../../CONTRIBUTING.md) for guidelines.

## References

- [WoWDev.wiki M2 Format](https://wowdev.wiki/M2)
- [WoWDev.wiki SKIN Format](https://wowdev.wiki/M2/.skin)
