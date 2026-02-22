# wow-wmo Examples

This directory contains examples for working with WMO (World Map Object) files.

## Running Examples

To run any example:

```bash
cargo run --example <example_name> [arguments]
```

## Available Examples

### wmo_app.rs - WMO Manipulation Application

A WMO (World Map Object) manipulation tool that demonstrates:

- Parsing WMO root files and group files
- Editing WMO properties and data
- Visualizing WMO structure and contents
- Converting between different WMO versions
- Validating WMO file integrity

```bash
cargo run --example wmo_app -- <command> [arguments]
```

Available commands:
- `parse <file.wmo>` - Parse and display WMO information
- `edit <file.wmo>` - Interactive WMO editing
- `visualize <file.wmo>` - Generate WMO visualization
- `convert <input.wmo> <output.wmo> <version>` - Convert between versions
- `validate <file.wmo>` - Check WMO integrity

### Planned Examples

- **`extract_groups.rs`** - Extract WMO group files
- **`list_textures.rs`** - List all textures used by a WMO
- **`export_obj.rs`** - Export WMO geometry to OBJ format

## WMO File Structure

WMO files represent large static geometry in World of Warcraft:

- Buildings, caves, and other structures
- Split into root file (.wmo) and group files (_000.wmo,_001.wmo, etc.)
- Contains materials, textures, and lighting information

## Example Usage

```bash
# Parse a WMO file and display information
cargo run --example wmo_app -- parse path/to/building.wmo

# Validate WMO structure
cargo run --example wmo_app -- validate path/to/building.wmo

# Convert WMO to different version
cargo run --example wmo_app -- convert old.wmo new.wmo wotlk

# Visualize WMO structure
cargo run --example wmo_app -- visualize path/to/building.wmo
```

## Test Data

WMO files can be extracted from MPQ archives using the wow-mpq crate.
Common WMO files include:

- Buildings in cities
- Dungeons and raids
- Cave systems
