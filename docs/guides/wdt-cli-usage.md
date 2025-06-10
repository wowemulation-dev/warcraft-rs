# WDT CLI Usage Guide

The `warcraft-rs` command-line tool provides comprehensive WDT (World Data Table) operations through the `wdt` subcommand, supporting all World of Warcraft versions from Classic through modern expansions.

**Key Features:**

- ✅ **Multi-Version Support** - Works with WDT files from 1.12.1 through 5.4.8+
- ✅ **Validation & Analysis** - Detailed file structure validation and reporting
- ✅ **Version Conversion** - Convert WDT files between different WoW versions
- ✅ **Tile Discovery** - Find all existing ADT tiles efficiently
- ✅ **Format Export** - Output data in text, JSON, or CSV formats

## Installation

```bash
# Build from source
cd warcraft-rs
cargo build --release

# Or install globally
cargo install --path .

# The binary will be available as 'warcraft-rs'
```

## Basic Commands

### Display WDT Information

```bash
# Basic information about a WDT file
warcraft-rs wdt info Azeroth.wdt

# Specify WoW version for accurate parsing
warcraft-rs wdt info Azeroth.wdt --version 3.3.5a

# Show detailed chunk information
warcraft-rs wdt info Azeroth.wdt --detailed
```

Example output:

```
WDT File Information
===================

File: Azeroth.wdt
Version: 18
Type: Terrain map

MPHD Flags:
  ✓ 0x0002 - ADTs have vertex colors
  ✓ 0x0004 - ADTs use big alpha
  ✓ 0x0040 - Universal flag (4.3.4+)
  Raw value: 0x00000046

ADT Tiles: 1337 / 4096 tiles
```

### Validate WDT Files

```bash
# Basic validation
warcraft-rs wdt validate Azeroth.wdt --version 3.3.5a

# Show all warnings (not just errors)
warcraft-rs wdt validate Azeroth.wdt --version 3.3.5a --warnings
```

Example output:

```
Validating WDT File
==================

✓ File is valid!
```

### List ADT Tiles

```bash
# List all tiles in text format
warcraft-rs wdt tiles Azeroth.wdt

# Export as JSON
warcraft-rs wdt tiles Azeroth.wdt --format json

# Export as CSV for spreadsheet analysis
warcraft-rs wdt tiles Azeroth.wdt --format csv > azeroth_tiles.csv
```

Text output:

```
Existing ADT Tiles
==================

Total: 1337 tiles

  [29,29] - Area ID: 12
  [29,30] - Area ID: 12
  [30,29] - Area ID: 12
  [30,30] - Area ID: 12
  ...
```

JSON output:

```json
[
  {
    "x": 29,
    "y": 29,
    "area_id": 12
  },
  {
    "x": 29,
    "y": 30,
    "area_id": 12
  }
]
```

### Convert Between Versions

```bash
# Preview conversion changes
warcraft-rs wdt convert input.wdt output.wdt \
    --from-version 1.12.1 \
    --to-version 3.3.5a \
    --preview

# Perform actual conversion
warcraft-rs wdt convert classic_map.wdt wotlk_map.wdt \
    --from-version 1.12.1 \
    --to-version 3.3.5a
```

Example conversion output:

```
WDT Version Conversion
=====================

Converting: Classic → WotLK
Input: classic_map.wdt
Output: wotlk_map.wdt

Changes to be made:
  • MODF UniqueID: 0xFFFFFFFF → 0x00000000
  • MODF Scale: 0 → 1024 (1.0x scale)
  • Update version compatibility flags

✓ Conversion complete!
```

### Tree Visualization

Visualize the internal structure of WDT files using the tree command:

```bash
# Basic tree view showing chunk structure
warcraft-rs wdt tree Azeroth.wdt

# Limit depth for focused view
warcraft-rs wdt tree Azeroth.wdt --depth 2

# Show external ADT file references
warcraft-rs wdt tree Azeroth.wdt --show-refs

# Compact mode for overview
warcraft-rs wdt tree Azeroth.wdt --compact

# No color output (for piping/redirecting)
warcraft-rs wdt tree Azeroth.wdt --no-color
```

The tree view shows:

- 📦 WDT file structure with all chunks
- 🗂️ Chunk hierarchy (MVER, MPHD, MAIN, etc.)
- 📊 Chunk sizes and metadata
- 🗺️ ADT tile references with coordinates
- 🏛️ WMO references for object-only maps

## Advanced Usage

### Batch Processing

Process multiple WDT files using shell scripting:

```bash
#!/bin/bash
# Validate all WDT files in a directory
for wdt_file in *.wdt; do
    echo "Validating: $wdt_file"
    warcraft-rs wdt validate "$wdt_file" --version 3.3.5a
done
```

```bash
#!/bin/bash
# Export tile lists for all maps
for wdt_file in *.wdt; do
    map_name=$(basename "$wdt_file" .wdt)
    warcraft-rs wdt tiles "$wdt_file" --format csv > "${map_name}_tiles.csv"
done
```

### Integration with Other Tools

Use WDT CLI output in data pipelines:

```bash
# Find maps with many tiles (large outdoor zones)
warcraft-rs wdt tiles *.wdt --format csv | \
    awk -F',' 'END {print "Tiles:", NR-1}' | \
    sort -n

# Extract tile data for specific coordinate ranges
warcraft-rs wdt tiles Azeroth.wdt --format csv | \
    awk -F',' '$1 >= 30 && $1 <= 35 && $2 >= 30 && $2 <= 35'
```

## Version-Specific Examples

### Classic (1.12.1)

```bash
# Classic WDT files often have empty MWMO chunks
warcraft-rs wdt info StormwindCity.wdt --version 1.12.1 --detailed
```

Classic maps characteristics:

- MODF UniqueID is 0xFFFFFFFF
- MODF Scale is 0 (not 1024)
- Terrain maps have empty MWMO chunks

### Wrath of the Lich King (3.3.5a)

```bash
# WotLK introduces many new flags
warcraft-rs wdt info Icecrown.wdt --version 3.3.5a --detailed
```

WotLK improvements:

- Extensive use of vertex colors (0x0002 flag)
- Big alpha blending (0x0004 flag)
- Sorted doodad references (0x0008 flag)

### Cataclysm (4.3.4)

```bash
# Cataclysm removes MWMO from terrain maps
warcraft-rs wdt info Deepholm.wdt --version 4.3.4 --detailed
```

Cataclysm breaking changes:

- Terrain maps have NO MWMO chunk
- Universal 0x0040 flag on all maps
- Improved terrain rendering capabilities

### Mists of Pandaria (5.4.8)

```bash
# MoP introduces height texturing
warcraft-rs wdt info Pandaria.wdt --version 5.4.8 --detailed
```

MoP enhancements:

- Height texturing flag (0x0080) becomes active
- Scenario support (small instanced content)
- Pet battle arenas as dedicated maps

## Common Use Cases

### Map Development

```bash
# Check if your custom map has valid structure
warcraft-rs wdt validate MyCustomMap.wdt --version 3.3.5a --warnings

# List tiles to verify terrain coverage
warcraft-rs wdt tiles MyCustomMap.wdt --format text
```

### Data Mining

```bash
# Export all tile data for analysis
for map in *.wdt; do
    warcraft-rs wdt tiles "$map" --format json > "data/$(basename "$map" .wdt).json"
done

# Find WMO-only maps (dungeons, instances)
warcraft-rs wdt info *.wdt | grep -B2 "WMO-only"
```

### Quality Assurance

```bash
# Validate entire map collection
find . -name "*.wdt" -exec warcraft-rs wdt validate {} --version 3.3.5a \;

# Check for version consistency
warcraft-rs wdt info *.wdt --version 3.3.5a | grep "✗"
```

### Archive Analysis

```bash
# Combined with MPQ extraction
warcraft-rs mpq extract patch.mpq --filter "*.wdt" --output wdt_files/
cd wdt_files
warcraft-rs wdt tiles World/Maps/*/\*.wdt --format csv > all_tiles.csv
```

## Output Formats

### Text Format (Default)

Human-readable output suitable for terminal viewing and basic scripting.

### JSON Format

Structured data perfect for web applications and modern data processing:

```bash
warcraft-rs wdt tiles Azeroth.wdt --format json | jq '.[] | select(.area_id == 12)'
```

### CSV Format

Tabular data ideal for spreadsheet analysis and database import:

```bash
warcraft-rs wdt tiles *.wdt --format csv | sqlite3 :memory: \
    "CREATE TABLE tiles(x,y,area_id); .import /dev/stdin tiles; SELECT area_id, COUNT(*) FROM tiles GROUP BY area_id;"
```

## Error Handling

### Common Issues

**File Not Found:**

```bash
warcraft-rs wdt info missing.wdt
# Error: Failed to open WDT file: No such file or directory
```

**Invalid Version:**

```bash
warcraft-rs wdt info map.wdt --version 99.99.99
# Error: Invalid version string
```

**Corrupted File:**

```bash
warcraft-rs wdt validate corrupted.wdt --version 3.3.5a
# ✗ 3 error(s) found:
#   • Invalid WDT version: expected 18, found 0
#   • Missing required chunk: MPHD
#   • Missing required chunk: MAIN
```

### Exit Codes

- `0`: Success
- `1`: File not found or permission error
- `2`: Invalid command line arguments
- `3`: File parsing error
- `4`: Validation failed

## Performance Tips

### Large Archives

For processing many files:

```bash
# Use shell built-ins for better performance
shopt -s nullglob
files=(*.wdt)
printf '%s\n' "${files[@]}" | xargs -P4 -I{} warcraft-rs wdt info {}
```

### Memory Usage

The WDT parser is memory-efficient and suitable for batch processing:

- Typical WDT file: < 1MB memory usage
- Large WDT with MAID: < 5MB memory usage
- No memory leaks in long-running scripts

## Integration Examples

### Python Integration

```python
import subprocess
import json

def get_wdt_tiles(wdt_path):
    result = subprocess.run([
        'warcraft-rs', 'wdt', 'tiles', wdt_path, '--format', 'json'
    ], capture_output=True, text=True)

    if result.returncode == 0:
        return json.loads(result.stdout)
    else:
        raise Exception(f"WDT parsing failed: {result.stderr}")

tiles = get_wdt_tiles("Azeroth.wdt")
print(f"Found {len(tiles)} tiles")
```

### Node.js Integration

```javascript
const { spawn } = require('child_process');

function getWdtInfo(wdtPath) {
    return new Promise((resolve, reject) => {
        const proc = spawn('warcraft-rs', ['wdt', 'tiles', wdtPath, '--format', 'json']);
        let output = '';

        proc.stdout.on('data', (data) => output += data);
        proc.on('close', (code) => {
            if (code === 0) {
                resolve(JSON.parse(output));
            } else {
                reject(new Error(`Process exited with code ${code}`));
            }
        });
    });
}
```

## Related Guides

- [📦 Working with MPQ Archives](./mpq-archives.md) - Extract WDT files from game archives
- [🌍 ADT Rendering Guide](./adt-rendering.md) - Use WDT data to load terrain tiles
- [🏛️ WMO Rendering Guide](./wmo-rendering.md) - Handle WMO-only maps from WDT data
- [📊 DBC Data Extraction](./dbc-extraction.md) - Cross-reference Area IDs from WDT tiles

## References

- [WDT Format Documentation](../formats/world-data/wdt.md) - Complete file format specification
- [WoW Version Support](../resources/version-support.md) - Supported WoW versions and compatibility
- [Map IDs Reference](../resources/map-ids.md) - Map naming conventions and IDs
