# ADT CLI Usage Guide 🏔️

This guide covers the ADT (terrain) commands available in the warcraft-rs CLI tool.

## Overview

The ADT command provides tools for working with World of Warcraft terrain files, including parsing, validation, conversion, and visualization.

## Installation

Ensure you have the ADT feature enabled when building warcraft-rs:

```bash
cargo install warcraft-rs --features adt
# Or for all features:
cargo install warcraft-rs --features full
```

## Available Commands

### Info Command

Display detailed information about an ADT file:

```bash
# Basic information
warcraft-rs adt info terrain.adt

# Detailed chunk information
warcraft-rs adt info terrain.adt --detailed
```

Output includes:

- File version and format
- Terrain chunk count and height range
- Texture references
- Model and WMO placements
- Water information
- Split file detection (Cataclysm+)

### Validate Command

Check ADT files for errors and inconsistencies:

```bash
# Standard validation
warcraft-rs adt validate terrain.adt

# Strict validation with warnings
warcraft-rs adt validate terrain.adt --level strict --warnings

# Basic validation (faster)
warcraft-rs adt validate terrain.adt --level basic
```

Validation levels:

- **basic**: Essential structure checks
- **standard**: Comprehensive validation (default)
- **strict**: All checks including best practices

### Convert Command

Convert ADT files between different WoW versions:

```bash
# Convert to Cataclysm format
warcraft-rs adt convert vanilla.adt cataclysm.adt --to cataclysm

# Convert to WotLK format
warcraft-rs adt convert cata.adt wotlk.adt --to wotlk

# Supported versions: classic, tbc, wotlk, cataclysm
```

Version conversion handles:

- Chunk format changes
- Water system updates (MCLQ → MH2O)
- Version-specific features
- Data preservation where possible

### Tree Command

Visualize ADT structure hierarchically:

```bash
# Basic tree view
warcraft-rs adt tree terrain.adt

# Show external file references
warcraft-rs adt tree terrain.adt --show-refs

# Compact view
warcraft-rs adt tree terrain.adt --compact

# Limit depth
warcraft-rs adt tree terrain.adt --depth 2

# No colors (for piping)
warcraft-rs adt tree terrain.adt --no-color
```

Tree view shows:

- 🏔️ Root ADT file
- 📋 Header chunks (MHDR, MCIN)
- 🌍 Terrain chunks (MCNK)
- 🎨 Texture references (MTEX)
- 🌲 Model data (MMDX/MMID, MDDF)
- 🏛️ WMO data (MWMO/MWID, MODF)
- 💧 Water chunks (MH2O)

### Extract Command (Optional Feature)

Extract data from ADT files (requires `extract` feature):

```bash
# Extract heightmap
warcraft-rs adt extract terrain.adt --heightmap --output ./extracted

# Extract with specific format
warcraft-rs adt extract terrain.adt --heightmap --heightmap-format png

# Extract texture information
warcraft-rs adt extract terrain.adt --textures

# Extract model placements
warcraft-rs adt extract terrain.adt --models

# Extract everything
warcraft-rs adt extract terrain.adt --all
```

Supported heightmap formats:

- **pgm**: Portable GrayMap (default)
- **png**: PNG image
- **tiff**: TIFF image
- **raw**: Raw float data

### Batch Command (Optional Feature)

Process multiple ADT files (requires `parallel` feature):

```bash
# Validate all ADT files
warcraft-rs adt batch "World/Maps/Azeroth/*.adt" --output ./results --operation validate

# Convert multiple files
warcraft-rs adt batch "*.adt" --output ./converted --operation convert --to cataclysm

# Use specific thread count
warcraft-rs adt batch "**/*.adt" --output ./output --operation validate --threads 8
```

## Working with Split Files

Cataclysm+ uses split ADT files:

```bash
# Main terrain file
warcraft-rs adt info Azeroth_32_48.adt

# Texture data
warcraft-rs adt info Azeroth_32_48_tex0.adt

# Object placement
warcraft-rs adt info Azeroth_32_48_obj0.adt
```

The tool automatically detects and reports related split files.

## Examples

### Analyzing a Zone

```bash
# Get overview of Elwynn Forest tile
warcraft-rs adt info World/Maps/Azeroth/Azeroth_32_48.adt

# Check for issues
warcraft-rs adt validate World/Maps/Azeroth/Azeroth_32_48.adt --warnings

# Visualize structure
warcraft-rs adt tree World/Maps/Azeroth/Azeroth_32_48.adt --show-refs
```

### Version Migration

```bash
# Convert Classic ADT to Cataclysm format
warcraft-rs adt convert classic_terrain.adt cata_terrain.adt --to cataclysm

# Batch convert entire zone
warcraft-rs adt batch "Kalimdor/*.adt" --output ./cata_kalimdor --operation convert --to cataclysm
```

### Data Extraction

```bash
# Extract heightmap for terrain editing
warcraft-rs adt extract terrain.adt --heightmap --heightmap-format tiff

# Extract all texture references
warcraft-rs adt extract terrain.adt --textures --output ./texture_data

# Get model placement data
warcraft-rs adt extract terrain.adt --models --output ./model_data
```

## Tips and Best Practices

1. **Validation First**: Always validate ADT files before conversion
2. **Backup Files**: Keep original files when converting versions
3. **Check Split Files**: For Cataclysm+, ensure all split files are present
4. **Use Appropriate Formats**: Choose heightmap format based on your editing tool
5. **Batch Operations**: Use glob patterns for processing multiple files
6. **Thread Count**: For batch operations, use thread count = CPU cores - 1

## Troubleshooting

### Common Issues

**"Failed to parse ADT file"**

- Check file is not corrupted
- Ensure it's a valid ADT file (not WDT/WDL)
- Try basic validation first

**"Version conversion failed"**

- Some features can't be converted backwards
- Check source and target version compatibility
- Validate source file first

**"Extract command not available"**

- Install with `--features extract` or `--features full`
- Check feature is enabled in your build

### Performance Tips

- Use `--compact` for tree view on large files
- Limit `--depth` for quick structure overview
- Use `parallel` feature for batch operations
- Process files from local disk, not network drives

## See Also

- [ADT Format Documentation](../formats/world-data/adt.md)
- [ADT Rendering Guide](adt-rendering.md)
- [Coordinate System](../resources/coordinates.md)
- [MPQ Archive Usage](mpq-cli-usage.md) - Extract ADT files from game data
