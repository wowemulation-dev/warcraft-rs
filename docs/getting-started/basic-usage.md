# 🎮 Basic Usage

Learn the fundamental patterns for using `warcraft-rs` with World of Warcraft files.

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
use warcraft_rs::mpq::Archive;

// Open an MPQ archive
let mut archive = Archive::open("Data/patch.mpq")?;

// Check if file exists
if archive.has_file("Interface\\FrameXML\\UIParent.lua") {
    // Read file data
    let data = archive.read_file("Interface\\FrameXML\\UIParent.lua")?;
    println!("File size: {} bytes", data.len());
}

// List all files
for file_info in archive.list_files() {
    println!("{}: {} bytes (compressed: {} bytes)",
        file_info.name,
        file_info.uncompressed_size,
        file_info.compressed_size
    );
}
```

### Extracting Archives

```rust
use warcraft_rs::mpq::{Archive, ExtractionProgress};
use std::path::Path;

let mut archive = Archive::open("Data/art.mpq")?;

// Extract all files
archive.extract_all("output/directory")?;

// Extract with progress callback
archive.extract_all_with_progress("output/", |progress| {
    match progress {
        ExtractionProgress::FileStart(name) => {
            println!("Extracting: {}", name);
        }
        ExtractionProgress::FileComplete(name, size) => {
            println!("  Done: {} ({} bytes)", name, size);
        }
        ExtractionProgress::Error(name, err) => {
            eprintln!("  Failed: {} - {}", name, err);
        }
    }
})?;

// Extract specific files
let files = vec![
    "Character\\Human\\Male\\HumanMale.m2",
    "Character\\Human\\Male\\HumanMaleSkin00.skin",
];

for file in files {
    if let Ok(data) = archive.read_file(file) {
        let output_path = Path::new("extracted").join(file);
        std::fs::create_dir_all(output_path.parent().unwrap())?;
        std::fs::write(output_path, data)?;
    }
}
```

## Loading Textures (BLP)

### Basic Texture Loading

```rust
use warcraft_rs::blp::{Blp, MipLevel};

// Load BLP texture
let blp = Blp::open("Textures/Minimap/MinimapMask.blp")?;

println!("Texture info:");
println!("  Size: {}x{}", blp.width(), blp.height());
println!("  Format: {:?}", blp.format());
println!("  Mipmap levels: {}", blp.mipmap_count());

// Get RGBA data for use with graphics APIs
let rgba_data = blp.to_rgba8(MipLevel(0))?;

// Save as PNG (requires image crate)
use image::{ImageBuffer, Rgba};
let image = ImageBuffer::<Rgba<u8>, _>::from_raw(
    blp.width(),
    blp.height(),
    rgba_data
).unwrap();
image.save("texture.png")?;
```

### Working with Mipmaps

```rust
// Iterate through mipmap levels
for level in 0..blp.mipmap_count() {
    let mip_data = blp.to_rgba8(MipLevel(level))?;
    let (width, height) = blp.mipmap_dimensions(MipLevel(level));

    println!("Mipmap {}: {}x{}", level, width, height);

    // Use mipmap data...
}

// Get specific mipmap for LOD
let lod_distance = 100.0;
let mip_level = (lod_distance / 50.0).log2().max(0.0) as u8;
let texture_data = blp.to_rgba8(MipLevel(mip_level.min(blp.mipmap_count() - 1)))?;
```

## Loading Models (M2)

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
