# File Format Reference

This section contains detailed documentation for all World of Warcraft file
formats supported by `warcraft-rs`.

## Categories

### Archives

[Archive formats](archives/) for storing and compressing game assets.

- **[MPQ](archives/mpq.md)** - Blizzard's main archive format

### 🌍 [World Data](world-data/)

Formats for terrain, maps, and world geometry.

- **[ADT](world-data/adt.md)** - Terrain tiles with height, textures, and objects
- **[WDL](world-data/wdl.md)** - Low-resolution terrain for distant views
- **[WDT](world-data/wdt.md)** - World definition tables

### 🎨 [Graphics & Models](graphics/)

3D models, textures, and visual assets.

- **[BLP](graphics/blp.md)** - Texture format with DXT compression
- **[M2](graphics/m2.md)** - Animated 3D models (characters, creatures, props)
- **[WMO](graphics/wmo.md)** - Large static world objects (buildings, dungeons)

### 📊 [Database](database/)

Client-side data storage.

- **[DBC](database/dbc.md)** - Database files with game data tables

## Format Overview

| Format | Type | Description | Typical Size |
|--------|------|-------------|--------------|
| MPQ | Archive | Compressed archive containing other files | 10MB - 4GB |
| ADT | Terrain | Map tile with terrain mesh and textures | 1-5MB |
| WDL | Terrain | Low-detail world map | 100-500KB |
| WDT | Map Info | Map configuration and ADT references | 10-50KB |
| BLP | Texture | 2D images with compression and mipmaps | 10KB - 2MB |
| M2 | Model | 3D models with animations | 50KB - 10MB |
| WMO | Model | Large world objects | 100KB - 50MB |
| DBC | Database | Tabular data storage | 1KB - 10MB |

## Version Compatibility

Different WoW versions use different format versions:

- **Classic (1.12.x)**: Original formats
- **TBC (2.4.3)**: Some format updates, new DBC columns
- **WotLK (3.3.5)**: Major M2 updates, new terrain features
- **Cataclysm (4.3.4)**: Terrain streaming, updated formats
- **MoP (5.4.8)**: Latest supported version

## Quick Reference

### Reading Files

```rust
// Most formats follow this pattern
let file = FileFormat::open("path/to/file.ext")?;

// Access data
for item in file.items() {
    // Process item
}
```

### Common Traits

All formats implement these traits where applicable:

```rust
trait FileFormat {
    fn open(path: &str) -> Result<Self, Error>;
    fn version(&self) -> u32;
    fn validate(&self) -> Result<(), Error>;
}
```

## Tools & Utilities

`warcraft-rs` provides command-line tools for each format:

```bash
# Extract MPQ archive
warcraft-mpq extract archive.mpq output/

# Convert BLP to PNG
warcraft-blp convert texture.blp texture.png

# Export DBC as CSV
warcraft-rs dbc export Spell.dbc --format csv
```
