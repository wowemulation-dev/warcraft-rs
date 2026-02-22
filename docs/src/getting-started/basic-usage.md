# Basic Usage

Learn the fundamental patterns for using `warcraft-rs` with World of Warcraft files.

**Current Support Status:**

- ✅ **MPQ Archives** - Fully implemented with 100% StormLib compatibility
- ✅ **DBC Format** - Client databases (full implementation with JSON/CSV export)
- ✅ **BLP Format** - Textures (full implementation)
- ✅ **M2 Format** - Models (full implementation)
- ✅ **WMO Format** - World objects (full implementation)
- ✅ **ADT Format** - Terrain data (full implementation)
- ✅ **WDT Format** - World table files (full implementation)
- ✅ **WDL Format** - Low-resolution terrain heightmaps (full implementation)

## Core Concepts

### File Loading Pattern

Each crate has its own parsing approach. There is no shared loading trait:

```rust
// wow-mpq: static open method
use wow_mpq::Archive;
let mut archive = Archive::open("archive.mpq")?;

// wow-blp: load function
use wow_blp::parser::load_blp;
let blp = load_blp("texture.blp")?;

// wow-wdt: reader struct
use wow_wdt::{WdtReader, version::WowVersion};
let reader = WdtReader::new(BufReader::new(file), WowVersion::WotLK);
let wdt = reader.read()?;

// wow-m2: parse from cursor
use wow_m2::M2Model;
let model = M2Model::parse(&mut cursor)?;

// wow-adt: standalone function
use wow_adt::api::parse_adt;
let adt = parse_adt(&mut reader)?;
```

### Error Handling

Each crate defines its own error type using `thiserror`:

```rust
use wow_mpq::Archive;

// Errors propagate with the ? operator
fn extract_file(path: &str) -> Result<Vec<u8>, wow_mpq::error::Error> {
    let mut archive = Archive::open(path)?;
    archive.read_file("Interface/FrameXML/UIParent.lua")
}
```

See [Error Handling](../api/error-handling.md) for the full list of error types.

## Working with Archives (MPQ)

### Opening and Reading Files

```rust
use wow_mpq::Archive;

// Open an MPQ archive
let mut archive = Archive::open("Data/patch.mpq")?;

// Read file data (both path styles work - auto-converted)
let data = archive.read_file("Interface/FrameXML/UIParent.lua")?;
// or: archive.read_file("Interface\\FrameXML\\UIParent.lua")?;
println!("File size: {} bytes", data.len());

// Check if file exists
if let Ok(Some(file_info)) = archive.find_file("Interface/FrameXML/UIParent.lua") {
    println!("File found: {} bytes", file_info.file_size);
}

// List files from listfile
if let Ok(entries) = archive.list() {
    for entry in entries {
        println!("{}: {} bytes (compressed: {} bytes)",
            entry.name,
            entry.size,
            entry.compressed_size
        );
    }
}

// Or list ALL files by scanning tables (includes files not in listfile)
if let Ok(entries) = archive.list_all() {
    for entry in entries {
        println!("{}: {} bytes", entry.name, entry.size);
    }
}
```

### Extracting Archives

```rust
use wow_mpq::{Archive, path::mpq_path_to_system};
use std::path::Path;
use std::fs;

let mut archive = Archive::open("Data/art.mpq")?;

// Extract all files (use list_all() to include files not in listfile)
if let Ok(entries) = archive.list_all() {
    for entry in entries {
        if let Ok(data) = archive.read_file(&entry.name) {
            // Convert MPQ path to system path
            let system_path = mpq_path_to_system(&entry.name);
            let output_path = Path::new("output").join(&system_path);

            // Create directories
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Write file
            fs::write(output_path, data)?;
            println!("Extracted: {}", entry.name);
        }
    }
}

// Extract specific files with proper path handling
let files = vec![
    "Character/Human/Male/HumanMale.m2",
    "Character/Human/Male/HumanMaleSkin00.skin",
];

for file in files {
    if let Ok(data) = archive.read_file(file) {
        let system_path = mpq_path_to_system(file);
        let output_path = Path::new("extracted").join(&system_path);
        fs::create_dir_all(output_path.parent().unwrap())?;
        fs::write(output_path, data)?;
    }
}
```

## Loading Textures (BLP)

### Basic Texture Loading

```rust
use wow_blp::{parser::load_blp, convert::blp_to_image};

// Load BLP texture
let blp = load_blp("Textures/Minimap/MinimapMask.blp")?;

println!("Texture info:");
println!("  Size: {}x{}", blp.header.width, blp.header.height);
println!("  Version: {:?}", blp.header.version);
println!("  Has mipmaps: {}", blp.header.has_mipmaps());

// Convert to standard image format
let image = blp_to_image(&blp, 0)?; // mipmap level 0

// Save as PNG
image.save("minimap_mask.png")?;
```

## Working with World Data (WDL)

### Basic WDL Operations

```rust
use wow_wdl::parser::WdlParser;
use wow_wdl::version::WdlVersion;
use std::io::Cursor;

// Parse a WDL file
let data = std::fs::read("World/Maps/Azeroth/Azeroth.wdl")?;
let parser = WdlParser::new(WdlVersion::WotLK);
let wdl = parser.parse(&mut Cursor::new(data))?;

// The WdlFile struct contains the parsed chunk data
println!("WDL version: {:?}", wdl.version);
```

## Loading Models (M2)


### Basic Model Loading

```rust
use wow_m2::M2Model;

// Load M2 model
let data = std::fs::read("Creature/Murloc/Murloc.m2")?;
let mut cursor = std::io::Cursor::new(data);
let model = M2Model::parse(&mut cursor)?;

// Access model data through the header and parsed fields
println!("Model version: {:?}", model.header.version);

// Load associated skin file
use wow_m2::skin::SkinFile;
let skin_data = std::fs::read("Creature/Murloc/Murloc00.skin")?;
let mut skin_cursor = std::io::Cursor::new(skin_data);
let skin = SkinFile::parse(&mut skin_cursor)?;
```

## Loading World Data

### Working with World Tables (WDT)

```rust
use wow_wdt::{WdtReader, version::WowVersion};
use std::io::BufReader;
use std::fs::File;

// Load a WDT file to see which ADT tiles exist
let file = File::open("World/Maps/Azeroth/Azeroth.wdt")?;
let mut reader = WdtReader::new(BufReader::new(file), WowVersion::WotLK);
let wdt = reader.read()?;

// Check map properties
println!("Map info:");
println!("  Is WMO-only: {}", wdt.is_wmo_only());
println!("  Existing tiles: {}", wdt.count_existing_tiles());

// Check which ADT tiles exist
for y in 0..64 {
    for x in 0..64 {
        if let Some(tile_info) = wdt.get_tile(x, y) {
            if tile_info.has_adt {
                println!("ADT tile exists at [{}, {}] - Area ID: {}",
                    x, y, tile_info.area_id);
            }
        }
    }
}

// For WMO-only maps (like dungeons)
if wdt.is_wmo_only() {
    if let Some(ref mwmo) = wdt.mwmo {
        println!("Global WMO: {}", mwmo.filename());
    }
    if let Some(ref modf) = wdt.modf {
        for placement in modf.entries() {
            println!("WMO placed at: {:?}", placement.position);
        }
    }
}

// Convert coordinates between systems
use wow_wdt::{tile_to_world, world_to_tile};

// Convert tile coordinates to world coordinates
let (world_x, world_y) = tile_to_world(32, 32); // Map center
println!("Tile [32, 32] is at world coordinates ({}, {})", world_x, world_y);

// Convert world coordinates back to tile coordinates
let (tile_x, tile_y) = world_to_tile(world_x, world_y);
println!("World coords ({}, {}) is tile [{}, {}]", world_x, world_y, tile_x, tile_y);
```

### Working with Terrain (ADT)

```rust
use wow_adt::api::parse_adt;
use std::io::Cursor;

// Load and parse terrain tile
let data = std::fs::read("World/Maps/Azeroth/Azeroth_32_48.adt")?;
let adt = parse_adt(&mut Cursor::new(data))?;

// The ParsedAdt struct contains all parsed chunk data
// Access MCNK terrain chunks, MDDF doodad placements, MODF WMO placements, etc.
```

### Working with World Objects (WMO)

```rust
use wow_wmo::api::parse_wmo;
use std::io::Cursor;

// Load and parse WMO root file
let data = std::fs::read("World/wmo/Dungeon/KL_Orgrimmar/Orgrimmar.wmo")?;
let wmo = parse_wmo(&mut Cursor::new(data))?;

// The ParsedWmo struct contains all parsed WMO data:
// header, materials, group info, doodad sets, etc.
```

## Reading Databases (DBC)


### Basic DBC Reading

```rust
use wow_cdbc::parser::DbcParser;
use std::io::Cursor;

// Parse DBC file
let data = std::fs::read("DBFilesClient/Item.dbc")?;
let dbc = DbcParser::parse(&mut Cursor::new(data))?;

// Access header information
println!("Records: {}", dbc.record_count());
println!("Fields per record: {}", dbc.field_count());

// The CLI provides additional functionality:
// - Schema discovery and validation
// - Export to JSON/CSV formats
// - Performance analysis
// Use `warcraft-rs dbc --help` for CLI commands.
```

## Best Practices

### Memory Management

```rust
// Use Arc for shared data
use std::sync::Arc;
use wow_blp::parser::load_blp;

let texture = Arc::new(load_blp("expensive_texture.blp")?);

// Clone is cheap - just increments reference count
let texture_ref = Arc::clone(&texture);

// For MPQ archives, read files on demand
use wow_mpq::Archive;

let mut archive = Archive::open("huge.mpq")?;

// Read file when needed
let data = archive.read_file("large_file.dat")?;

// Process in chunks if needed
for chunk in data.chunks(4096) {
    // Process chunk...
}
```

### Error Recovery

```rust
use wow_blp::parser::{load_blp, error::LoadError};
use wow_blp::BlpImage;

fn load_texture_with_fallback(path: &str, fallback: &str) -> Result<BlpImage, LoadError> {
    match load_blp(path) {
        Ok(texture) => Ok(texture),
        Err(_) => {
            eprintln!("Texture {} not found, using fallback", path);
            load_blp(fallback)
        }
    }
}
```

### Performance Tips

```rust
// Batch operations
let files_to_extract = vec!["file1.blp", "file2.blp", "file3.blp"];
let mut results = Vec::new();

for file in files_to_extract {
    if let Ok(data) = archive.read_file(file) {
        results.push((file, data));
    }
}

// Use parallel processing for CPU-intensive tasks
use rayon::prelude::*;
use wow_blp::parser::load_blp;

let textures: Vec<_> = texture_paths
    .par_iter()
    .filter_map(|path| load_blp(path).ok())
    .collect();
```

## Next Steps

- Explore [File Format Reference](../formats/README.md)
- Read format-specific guides in [Guides](../guides/)
- Check [API Documentation](../api/)
- See [Example Projects](https://github.com/wowemulation-dev/warcraft-rs/tree/main/warcraft-rs/examples)
