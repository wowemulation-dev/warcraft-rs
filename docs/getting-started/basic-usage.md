# 🎮 Basic Usage

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

Most warcraft-rs types follow a consistent API:

```rust
// Each format has its own crate
use wow_mpq::Archive;
use wow_blp::parser::load_blp;
use wow_adt::reader::AdtReader;
// etc.

// Standard loading pattern
let file = Format::open("path/to/file.ext")?;

// From bytes
let data = std::fs::read("file.ext")?;
let file = Format::from_bytes(&data)?;

// With options
let file = Format::open_with_options("file.ext", FormatOptions {
    validate: true,
    strict_mode: false,
})?;
```

### Error Handling

warcraft-rs uses a unified error type:

```rust
// Each crate has its own error type
use wow_mpq::error::Error as MpqError;
use wow_blp::parser::error::Error as BlpError;

fn handle_mpq_error(e: MpqError) {
    match e {
        MpqError::FileNotFound(name) => eprintln!("File not found: {}", name),
        MpqError::InvalidArchive => eprintln!("Invalid MPQ archive"),
        _ => eprintln!("MPQ error: {}", e),
    }
}
```

## Working with Archives (MPQ)

### Opening and Reading Files

```rust
use wow_mpq::Archive;

// Open an MPQ archive
let archive = Archive::open("Data/patch.mpq")?;

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

let archive = Archive::open("Data/art.mpq")?;

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
use wow_wdl::Wdl;

// Load a WDL file
let wdl = Wdl::from_file("World/Maps/Azeroth/Azeroth.wdl")?;

// Get basic information
println!("WDL version: {}", wdl.version());
println!("Map size: {}x{} tiles", wdl.map_width(), wdl.map_height());

// Access heightmap data
for y in 0..64 {
    for x in 0..64 {
        if let Some(height_data) = wdl.get_tile(x, y) {
            println!("Tile [{}, {}] has {} height values",
                x, y, height_data.len());
        }
    }
}

// Export to heightmap image
let heightmap = wdl.to_heightmap()?;
heightmap.save("azeroth_heightmap.png")?;
```

## Loading Models (M2)


### Basic Model Loading

```rust
use wow_m2::{Model, version::M2Version};

// Load M2 model
let data = std::fs::read("Creature/Murloc/Murloc.m2")?;
let mut cursor = std::io::Cursor::new(data);
let model = Model::parse(&mut cursor)?;

println!("Model info:");
println!("  Name: {}", model.header.name());
println!("  Version: {:?}", model.header.version());
println!("  Sequences: {}", model.sequences.len());
println!("  Bones: {}", model.bones.len());
println!("  Textures: {}", model.textures.len());

// Load associated skin file
let skin_data = std::fs::read("Creature/Murloc/Murloc00.skin")?;
let mut skin_cursor = std::io::Cursor::new(skin_data);
let skin = wow_m2::skin::SkinFile::parse(&mut skin_cursor)?;
println!("Skin vertices: {}", skin.submeshes.len());
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
use wow_adt::{reader::AdtReader, version::WowVersion};
use std::io::Cursor;

// Load terrain tile
let data = std::fs::read("World/Maps/Azeroth/Azeroth_32_48.adt")?;
let mut reader = AdtReader::new(Cursor::new(data), WowVersion::WotLK);
let adt = reader.read()?;

// Access chunk information
for y in 0..16 {
    for x in 0..16 {
        let chunk_index = y * 16 + x;
        if let Some(mcnk) = adt.get_chunk(chunk_index) {
            println!("Chunk [{}, {}] - Area ID: {}", x, y, mcnk.area_id);
            println!("  Height range: {:.2} to {:.2}",
                mcnk.get_min_height(), mcnk.get_max_height());
            println!("  Has water: {}", mcnk.has_water());
        }
    }
}

// Find doodads (M2 models) on terrain
if let Some(mddf) = &adt.mddf {
    for doodad in &mddf.entries {
        println!("M2 Model ID {} at position {:?}",
            doodad.name_id, doodad.position);
    }
}

// Find WMOs on terrain
if let Some(modf) = &adt.modf {
    for wmo in &modf.entries {
        println!("WMO ID {} at position {:?}",
            wmo.name_id, wmo.position);
    }
}
```

### Working with World Objects (WMO)

```rust
use wow_wmo::{reader::WmoReader, version::WmoVersion};
use std::io::BufReader;
use std::fs::File;

// Load WMO root file
let file = File::open("World/wmo/Dungeon/KL_Orgrimmar/Orgrimmar.wmo")?;
let mut reader = WmoReader::new(BufReader::new(file), WmoVersion::WotLK);
let wmo = reader.read_root()?;

// Get WMO information
println!("WMO: {}", wmo.mohd.name());
println!("Groups: {}", wmo.mohd.group_count);
println!("Materials: {}", wmo.momt.materials.len());
println!("Doodad sets: {}", wmo.mods.sets.len());

// Load WMO group
let group_file = File::open("World/wmo/Dungeon/KL_Orgrimmar/Orgrimmar_000.wmo")?;
let mut group_reader = WmoReader::new(BufReader::new(group_file), WmoVersion::WotLK);
let group = group_reader.read_group()?;

println!("Group vertices: {}", group.movt.vertices.len());
println!("Group triangles: {}", group.movi.indices.len() / 3);
```

## Reading Databases (DBC)


### Basic DBC Reading

```rust
use wow_cdbc::parser::parse_dbc_file;
use wow_cdbc::schema::Schema;

// Parse DBC file
let dbc = parse_dbc_file("DBFilesClient/Item.dbc")?;

println!("Item database:");
println!("  Records: {}", dbc.record_count());
println!("  Fields per record: {}", dbc.field_count());

// Export to JSON
let json = dbc.to_json()?;
std::fs::write("items.json", json)?;

// The CLI provides extensive functionality:
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

let archive = Archive::open("huge.mpq")?;

// Read file when needed
let data = archive.read_file("large_file.dat")?;

// Process in chunks if needed
for chunk in data.chunks(4096) {
    // Process chunk...
}
```

### Error Recovery

```rust
use wow_blp::{parser::load_blp, error::Error};
use wow_blp::types::image::BlpImage;

fn load_texture_with_fallback(path: &str, fallback: &str) -> Result<BlpImage, Error> {
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
- See [Example Projects](https://github.com/wowemulation-dev/warcraft-rs/tree/main/examples)
