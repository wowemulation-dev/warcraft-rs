# 🎮 Basic Usage

Learn the fundamental patterns for using `warcraft-rs` with World of Warcraft files.

**Current Support Status:**
- ✅ **MPQ Archives** - Fully implemented with 98.75% StormLib compatibility
- ✅ **WDL Format** - Low-resolution terrain heightmaps (basic implementation)
- 🚧 **ADT Format** - Terrain data (planned)
- 🚧 **WDT Format** - World tables (planned)
- 🚧 **BLP Format** - Textures (planned)
- 🚧 **M2 Format** - Models (planned)
- 🚧 **WMO Format** - World objects (planned)
- 🚧 **DBC Format** - Databases (planned)

## Core Concepts

### File Loading Pattern

Most warcraft-rs types follow a consistent API:

```rust
use warcraft_rs::{Format, Error};

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
use warcraft_rs::Error;

fn load_model(path: &str) -> Result<Model, Error> {
    match Model::open(path) {
        Ok(model) => Ok(model),
        Err(Error::FileNotFound(path)) => {
            eprintln!("File not found: {}", path);
            Err(Error::FileNotFound(path))
        }
        Err(Error::InvalidFormat(msg)) => {
            eprintln!("Invalid format: {}", msg);
            Err(Error::InvalidFormat(msg))
        }
        Err(e) => Err(e),
    }
}
```

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

// List all files (requires listfile)
if let Ok(entries) = archive.list() {
    for entry in entries {
        println!("{}: {} bytes (compressed: {} bytes)",
            entry.name,
            entry.size,
            entry.compressed_size
        );
    }
}
```

### Extracting Archives

```rust
use wow_mpq::{Archive, path::mpq_path_to_system};
use std::path::Path;
use std::fs;

let mut archive = Archive::open("Data/art.mpq")?;

// Extract all files
if let Ok(entries) = archive.list() {
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

*Note: BLP support is planned but not yet implemented in the current release.*

### Basic Texture Loading (Coming Soon)

```rust
// Future API preview
use wow_blp::Blp;

// Load BLP texture
let blp = Blp::open("Textures/Minimap/MinimapMask.blp")?;

println!("Texture info:");
println!("  Size: {}x{}", blp.width(), blp.height());
println!("  Format: {:?}", blp.format());
println!("  Mipmap levels: {}", blp.mipmap_count());

// Get RGBA data for use with graphics APIs
let rgba_data = blp.to_rgba8(0)?; // mipmap level 0
```

## Loading Models (M2)

*Note: M2 support is planned but not yet implemented in the current release.*

### Basic Model Loading

```rust
use warcraft_rs::m2::{M2Model, M2Skin};

// Load model and skin
let model = M2Model::open("Creature/Murloc/Murloc.m2")?;
let skin = M2Skin::open("Creature/Murloc/Murloc00.skin")?;

// Apply skin to model
model.set_skin(skin)?;

println!("Model info:");
println!("  Name: {}", model.name());
println!("  Vertices: {}", model.vertex_count());
println!("  Animations: {}", model.animation_count());
println!("  Bones: {}", model.bone_count());

// Get model bounds
let bounds = model.bounding_box();
println!("  Bounds: {:?} to {:?}", bounds.min, bounds.max);
```

### Playing Animations

```rust
use warcraft_rs::m2::{AnimationId, AnimationState};

// List available animations
for (id, anim) in model.animations() {
    println!("Animation {}: {} ({}ms)",
        id, anim.name(), anim.duration_ms());
}

// Create animation state
let mut anim_state = AnimationState::new();

// Play stand animation
anim_state.play(AnimationId::Stand, true)?; // true = loop

// Update animation (call each frame)
let delta_ms = 16; // 60 FPS
anim_state.update(delta_ms);

// Get bone transforms for current frame
let bones = model.calculate_bones(&anim_state)?;

// Apply to vertices for rendering
let animated_vertices = model.apply_skinning(&bones)?;
```

## Loading World Data

### Working with Terrain (ADT)

```rust
use warcraft_rs::adt::{Adt, ChunkPos};

// Load terrain tile
let adt = Adt::open("World/Maps/Azeroth/Azeroth_32_48.adt")?;

// Get height at world position
let world_x = 100.0;
let world_y = 200.0;
let height = adt.get_height_at(world_x, world_y)?;

println!("Terrain height at ({}, {}): {}", world_x, world_y, height);

// Iterate chunks
for chunk in adt.chunks() {
    let pos = chunk.position();
    println!("Chunk [{}, {}]", pos.x, pos.y);

    // Get chunk data
    let heights = chunk.height_map();
    let normals = chunk.normal_map();
    let textures = chunk.texture_layers();
}

// Find objects on terrain
for doodad in adt.doodads() {
    println!("M2 Model: {} at {:?}", doodad.model_path, doodad.position);
}

for wmo in adt.wmos() {
    println!("WMO: {} at {:?}", wmo.model_path, wmo.position);
}
```

### Loading Maps (WDT)

```rust
use warcraft_rs::wdt::Wdt;

// Load world definition
let wdt = Wdt::open("World/Maps/Azeroth/Azeroth.wdt")?;

// Check which tiles exist
for y in 0..64 {
    for x in 0..64 {
        if wdt.has_adt(x, y) {
            println!("ADT exists at [{}, {}]", x, y);
        }
    }
}

// Get map info
if let Some(map_info) = wdt.map_info() {
    println!("Map name: {}", map_info.name);
    println!("Map ID: {}", map_info.id);
}
```

## Reading Databases (DBC)

### Basic DBC Reading

```rust
use warcraft_rs::dbc::{Dbc, StringRef};

// Open DBC file
let item_dbc = Dbc::open("DBFilesClient/Item.dbc")?;

println!("Item database:");
println!("  Records: {}", item_dbc.record_count());
println!("  Fields per record: {}", item_dbc.field_count());

// Read records generically
for record in item_dbc.records() {
    let id = record.get_u32(0)?; // First field is usually ID
    let name = record.get_string(1)?; // String reference

    println!("Item {}: {}", id, name);
}
```

### Typed DBC Access

```rust
use warcraft_rs::dbc::{DbcTable, ItemRecord};

// Load with schema
let items = DbcTable::<ItemRecord>::open("DBFilesClient/Item.dbc")?;

// Find specific item
if let Some(thunderfury) = items.find_by_id(19019) {
    println!("Thunderfury:");
    println!("  Quality: {:?}", thunderfury.quality);
    println!("  Item Level: {}", thunderfury.item_level);
}

// Query items
let epic_swords = items
    .iter()
    .filter(|item| {
        item.quality == ItemQuality::Epic &&
        item.class == ItemClass::Weapon &&
        item.subclass == WeaponSubclass::Sword
    })
    .collect::<Vec<_>>();

println!("Found {} epic swords", epic_swords.len());
```

## Best Practices

### Memory Management

```rust
// Use Arc for shared data
use std::sync::Arc;

let model = Arc::new(M2Model::open("expensive_model.m2")?);

// Clone is cheap - just increments reference count
let model_ref = Arc::clone(&model);

// For large collections, use streaming
use warcraft_rs::mpq::FileStream;

let mut archive = Archive::open("huge.mpq")?;
let mut stream = archive.open_file_stream("large_file.dat")?;

let mut buffer = vec![0u8; 4096];
while let Ok(bytes_read) = stream.read(&mut buffer) {
    if bytes_read == 0 { break; }
    // Process chunk...
}
```

### Error Recovery

```rust
use warcraft_rs::{Error, m2::M2Model};

fn load_model_with_fallback(path: &str, fallback: &str) -> Result<M2Model, Error> {
    match M2Model::open(path) {
        Ok(model) => Ok(model),
        Err(Error::FileNotFound(_)) => {
            eprintln!("Model {} not found, using fallback", path);
            M2Model::open(fallback)
        }
        Err(e) => Err(e),
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

let textures: Vec<_> = texture_paths
    .par_iter()
    .filter_map(|path| Blp::open(path).ok())
    .collect();
```

## Next Steps

- Explore [File Format Reference](../formats/README.md)
- Read format-specific guides in [Guides](../guides/)
- Check [API Documentation](../api/)
- See [Example Projects](https://github.com/wowemulation-dev/warcraft-rs/tree/main/examples)
