# WMO CLI Usage Guide 🏰

This guide covers the `warcraft-rs wmo` command and its subcommands for working with World Map Object (WMO) files.

## Overview

The WMO command provides tools for parsing, validating, converting, and manipulating WMO files. WMO files represent buildings, dungeons, and other large structures in World of Warcraft.

## Installation

```bash
# Install from source
cargo install --path warcraft-rs

# Or build and run directly
cargo run --bin warcraft-rs -- wmo <subcommand>
```

## Subcommands

### `info` - Display WMO Information

Get detailed information about a WMO file.

```bash
# Basic info
warcraft-rs wmo info building.wmo

# Detailed info with all chunks
warcraft-rs wmo info building.wmo --detailed

# Example output:
WMO Version: 17
Groups: 5
Materials: 12
Textures: 8
Doodads: 45
Portals: 3
Lights: 7
Flags: HAS_VERTEX_COLORS | INDOOR
Bounding Box: (-100.5, -50.2, 0.0) to (100.5, 50.2, 30.0)
```

### `validate` - Check WMO File Integrity

Validate a WMO file for structural integrity and common issues.

```bash
# Basic validation
warcraft-rs wmo validate building.wmo

# Include warnings
warcraft-rs wmo validate building.wmo --warnings

# Detailed validation with all checks
warcraft-rs wmo validate building.wmo --warnings --detailed

# Example output:
✓ Header validation passed
✓ Material references valid
✓ Texture indices valid
✓ Group information consistent
⚠ Warning: Material 5 uses deprecated blend mode
✓ Overall: VALID (1 warning)
```

### `convert` - Convert Between WMO Versions

Convert WMO files between different WoW expansion formats.

```bash
# Convert to specific version number
warcraft-rs wmo convert classic.wmo modern.wmo --to 21

# Convert from Classic to Cataclysm
warcraft-rs wmo convert classic.wmo cata.wmo --to 21

# Example output:
Converting WMO from version 17 to version 21...
✓ Header updated
✓ Materials converted (added extended format)
✓ Group data updated
✓ Conversion complete: modern.wmo
```

**Supported Versions:**

- 17: Classic through Wrath of the Lich King
- 18: Cataclysm
- 19: Mists of Pandaria
- 20: Warlords of Draenor
- 21: Legion and later

### `tree` - Visualize WMO Structure

Display the hierarchical structure of a WMO file.

```bash
# Basic tree view
warcraft-rs wmo tree building.wmo

# Limit depth
warcraft-rs wmo tree building.wmo --depth 2

# Show references (textures, doodads)
warcraft-rs wmo tree building.wmo --show-refs

# Disable colors
warcraft-rs wmo tree building.wmo --no-color

# Hide metadata
warcraft-rs wmo tree building.wmo --no-metadata

# Compact view
warcraft-rs wmo tree building.wmo --compact

# Example output:
🏰 WMO Root: building.wmo (v17)
├── 📋 Header [MOHD] (64 bytes)
│   ├── Groups: 5
│   ├── Materials: 12
│   └── Flags: INDOOR | HAS_VERTEX_COLORS
├── 🎨 Textures [MOTX] (256 bytes)
│   ├── [0] "world/generic/stone_floor.blp"
│   └── [1] "world/generic/wood_wall.blp"
├── 🎭 Materials [MOMT] (480 bytes)
│   ├── [0] Shader: DIFFUSE | TWO_SIDED
│   └── [1] Shader: SPECULAR | ENV
├── 📁 Groups [MOGI] (160 bytes)
│   ├── [0] "MainHall" (flags: INDOOR | HAS_BSP)
│   └── [1] "Entrance" (flags: OUTDOOR)
└── 💡 Lights [MOLT] (336 bytes)
    ├── [0] Omni (intensity: 1.5)
    └── [1] Spot (intensity: 2.0)
```

### `edit` - Modify WMO Properties

Edit properties of a WMO file.

```bash
# Set flags
warcraft-rs wmo edit building.wmo --set-flag has-fog
warcraft-rs wmo edit building.wmo --unset-flag use-unified-render-path

# Change properties
warcraft-rs wmo edit building.wmo --ambient-color "0.5,0.5,0.7,1.0"

# Example output:
Editing WMO: building.wmo
✓ Set flag: HAS_FOG
✓ Ambient color changed to: RGBA(0.5, 0.5, 0.7, 1.0)
✓ Changes saved to: building.wmo
```

**Available Flags:**

- `do-not-attenuate-vertices`
- `use-unified-render-path`
- `use-liquid-from-dbc`
- `do-not-fix-vertex-color-alpha`
- `lod`
- `has-fog`

### `build` - Create WMO from Configuration

Build a new WMO file from a YAML configuration.

```bash
# Build from config
warcraft-rs wmo build output.wmo --from config.yaml

# Specify version
warcraft-rs wmo build output.wmo --from config.yaml --version 17
```

**Example Configuration (config.yaml):**

```yaml
version: 17
header:
  ambient_color: [0.5, 0.5, 0.5, 1.0]
  flags:
    - indoor
    - has_vertex_colors
  bounding_box:
    min: [-50.0, -50.0, 0.0]
    max: [50.0, 50.0, 30.0]

textures:
  - "world/generic/stone_floor.blp"
  - "world/generic/wood_wall.blp"

materials:
  - texture: 0
    flags: ["diffuse", "two_sided"]
    blend_mode: 0
  - texture: 1
    flags: ["specular"]
    blend_mode: 1

groups:
  - name: "MainHall"
    flags: ["indoor", "has_bsp"]
    bounding_box:
      min: [-50.0, -50.0, 0.0]
      max: [50.0, 50.0, 30.0]

lights:
  - type: "omni"
    position: [0.0, 0.0, 15.0]
    color: [1.0, 1.0, 0.8, 1.0]
    intensity: 1.5
    attenuation: [5.0, 25.0]

doodad_sets:
  - name: "Furniture"
    doodads:
      - model: "world/generic/chair.m2"
        position: [10.0, 10.0, 0.0]
        rotation: [0.0, 0.0, 0.0, 1.0]
        scale: 1.0
```

## Working with Group Files

WMO files consist of a root file and multiple group files (_000.wmo to_999.wmo).

```bash
# Info for specific group
warcraft-rs wmo info building_000.wmo

# Validate all groups
for i in {000..004}; do
    warcraft-rs wmo validate "building_${i}.wmo"
done
```

## Common Use Cases

### Extracting WMO Files from MPQ

```bash
# Extract a WMO and all its groups
warcraft-rs mpq extract patch.mpq "World/wmo/Azeroth/Buildings/Stormwind/*" --output ./

# List all WMO files in an archive
warcraft-rs mpq list patch.mpq --filter "*.wmo"
```

### Batch Processing

```bash
# Validate all WMO files in a directory
find . -name "*.wmo" -not -name "*_[0-9][0-9][0-9].wmo" | while read wmo; do
    echo "Validating: $wmo"
    warcraft-rs wmo validate "$wmo"
done

# Convert all Classic WMOs to Cataclysm format
for wmo in *.wmo; do
    if [[ ! "$wmo" =~ _[0-9]{3}\.wmo$ ]]; then
        warcraft-rs wmo convert "$wmo" "converted/${wmo}" --to 18
    fi
done
```

### Analyzing WMO Structure

```bash
# Get a quick overview of all WMOs
for wmo in *.wmo; do
    if [[ ! "$wmo" =~ _[0-9]{3}\.wmo$ ]]; then
        echo "=== $wmo ==="
        warcraft-rs wmo info "$wmo" | grep -E "Version:|Groups:|Materials:"
    fi
done

# Find WMOs with specific features
warcraft-rs wmo info building.wmo --detailed | grep -i "skybox"
```

### Debugging Rendering Issues

```bash
# Check for missing textures
warcraft-rs wmo validate building.wmo --warnings --detailed | grep -i "texture"

# Verify material setup
warcraft-rs wmo tree building.wmo --show-refs | grep -A5 "Materials"

# Check group flags
warcraft-rs wmo info building.wmo --detailed | grep -A10 "Group Information"
```

## Tips and Tricks

### Performance Optimization

1. **Use `--compact` with tree command** for large WMOs to reduce output
2. **Validate before converting** to catch issues early
3. **Process root files separately** from group files when batch processing

### Common Issues

1. **Missing Group Files**: Ensure all _XXX.wmo files are present
2. **Texture Path Issues**: WoW uses backslashes; the tool handles conversion
3. **Version Compatibility**: Not all features convert perfectly between versions

### Integration with Other Tools

```bash
# Extract WMO, convert it, then re-import
warcraft-rs mpq extract archive.mpq "path/to/building.wmo" --output temp/
warcraft-rs wmo convert temp/building.wmo converted/building.wmo --to 21
warcraft-rs mpq create new_archive.mpq --add converted/building.wmo

# Validate WMOs after ADT modification
warcraft-rs adt info map.adt | grep -i "wmo" | while read line; do
    wmo_file=$(echo $line | awk '{print $2}')
    warcraft-rs wmo validate "$wmo_file"
done
```

## Error Messages

### Common Errors and Solutions

- **"Failed to parse WMO header"**: File is corrupted or not a valid WMO
- **"Invalid texture index"**: Material references non-existent texture
- **"Missing group file"**: Expected group file not found (e.g., _001.wmo)
- **"Unsupported version"**: WMO version not supported for this operation
- **"Invalid chunk size"**: File corruption or incorrect write operation

## See Also

- [WMO Format Documentation](../formats/graphics/wmo.md)
- [MPQ CLI Usage](mpq-cli-usage.md) - For extracting WMO files
- [ADT CLI Usage](adt-cli-usage.md) - For terrain that contains WMOs
- [WMO Rendering Guide](wmo-rendering.md) - For displaying WMOs
