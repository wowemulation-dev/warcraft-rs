# wow-wmo Examples

This directory contains examples for working with WMO (World Map Object) files.

## Running Examples

To run any example:

```bash
cargo run --example <example_name> [arguments]
```

## Available Examples

*Note: This crate is still in development. Examples will be added as features are implemented.*

### Planned Examples

- **`parse_wmo.rs`** - Parse and display WMO file information
- **`extract_groups.rs`** - Extract WMO group files
- **`list_textures.rs`** - List all textures used by a WMO
- **`validate_wmo.rs`** - Validate WMO file structure
- **`convert_version.rs`** - Convert WMO between game versions

## WMO File Structure

WMO files represent large static geometry in World of Warcraft:

- Buildings, caves, and other structures
- Split into root file (.wmo) and group files (_000.wmo,_001.wmo, etc.)
- Contains materials, textures, and lighting information

## Example Usage (Coming Soon)

```rust
use wow_wmo::WmoRoot;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load a WMO root file
    let wmo = WmoRoot::load("path/to/building.wmo")?;

    // Display basic information
    println!("WMO: {}", wmo.name);
    println!("Groups: {}", wmo.group_count);
    println!("Materials: {}", wmo.materials.len());

    Ok(())
}
```

## Test Data

WMO files can be extracted from MPQ archives using the wow-mpq crate.
Common WMO files include:

- Buildings in cities
- Dungeons and raids
- Cave systems
