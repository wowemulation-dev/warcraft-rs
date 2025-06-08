# ADT Format

ADT (Azeroth Data Terrain) files contain terrain and object information for map tiles.

## Overview

- **Extension**: `.adt`
- **Purpose**: Terrain geometry, textures, and object placement
- **Grid Size**: 16x16 chunks per ADT
- **Related Files**:
  - `_tex0.adt` - Texture information
  - `_obj0.adt` - Object placement
  - `_lod.adt` - Level of detail

## Structure

### Main Chunks

| Chunk | Size | Description |
|-------|------|-------------|
| MVER | 4 | Version |
| MHDR | 64 | Header with offsets |
| MCIN | 256*16 | Chunk information |
| MTEX | Variable | Texture filenames |
| MMDX | Variable | M2 model filenames |
| MMID | Variable | M2 model IDs |
| MWMO | Variable | WMO filenames |
| MWID | Variable | WMO IDs |
| MDDF | Variable | M2 placement |
| MODF | Variable | WMO placement |
| MCNK | 8448*256 | Terrain chunks |

### Terrain Chunk (MCNK)

```rust
struct McnkChunk {
    flags: u32,
    index_x: u32,
    index_y: u32,
    layers: u32,
    doodad_refs: u32,
    height_offset: u32,
    normal_offset: u32,
    // ... more fields
    vertices: [[f32; 145]; 9],  // Height values
    normals: [[Vec3; 145]; 9],  // Normal vectors
}
```

## Usage Example

```rust
use warcraft_rs::adt::{Adt, TerrainChunk};

// Load ADT file
let adt = Adt::open("World/Maps/Azeroth/Azeroth_32_48.adt")?;

// Access terrain chunks
for chunk in &adt.chunks {
    println!("Chunk ({}, {})", chunk.x, chunk.y);

    // Get height at specific position
    let height = chunk.get_height(8.5, 8.5);

    // Get texture layers
    for layer in &chunk.layers {
        println!("  Texture: {}", layer.texture_id);
    }
}

// Export heightmap
let heightmap = adt.export_heightmap();
heightmap.save("terrain_height.png")?;

// Find all doodads (M2 models)
for doodad in &adt.doodads {
    println!("Model: {} at {:?}", doodad.filename, doodad.position);
}
```

## Coordinate System

- Each ADT represents a 533.33333... yard square
- Split into 16x16 chunks
- Each chunk is 33.3333... yards square
- Height values at 9x9 vertices per chunk + 8x8 inner vertices

## Advanced Features

### Liquid Rendering

```rust
if let Some(water) = &chunk.liquid {
    for x in 0..9 {
        for y in 0..9 {
            let depth = water.get_depth(x, y);
            let flow = water.get_flow_direction(x, y);
        }
    }
}
```

### Texture Blending

ADT supports up to 4 texture layers per chunk with alpha blending:

```rust
let blended_color = chunk.calculate_texture_blend(x, y);
```

## Common Patterns

### Streaming Large Worlds

```rust
use warcraft_rs::adt::AdtManager;

let manager = AdtManager::new("World/Maps/Azeroth")?;
manager.set_view_distance(3); // Load 3x3 ADTs around player

// Update based on player position
manager.update_position(player_x, player_y);

// Get loaded ADTs
for adt in manager.loaded_adts() {
    // Render terrain
}
```

## Performance Considerations

- ADT files can be large (several MB each)
- Consider implementing LOD for distant terrain
- Cache frequently accessed ADTs
- Use frustum culling for chunks

## See Also

- [Rendering ADT Terrain Guide](../../guides/adt-rendering.md)
- [WDT Format](wdt.md) - World tables that reference ADTs
- [WDL Format](wdl.md) - Low-detail world data
