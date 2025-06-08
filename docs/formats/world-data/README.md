# 🌍 World Data Formats

World data formats define terrain, maps, and the game world structure.

## Supported Formats

### [ADT Format](adt.md)

**Azeroth Data Terrain** - Individual map tiles containing terrain mesh, textures,
and object placement.

- 533.33 x 533.33 yard tiles
- Height maps and normal maps
- Texture layers with alpha blending
- Doodad and WMO placement

### [WDT Format](wdt.md)

**World Data Table** - Map definition files that specify which ADT tiles exist
and global map properties.

- References to ADT files
- Map flags and settings
- Global WMO (like Stormwind)
- Ocean level definition

### [WDL Format](wdl.md)

**World Data Low-resolution** - Low-detail terrain used for distant rendering
and the world map.

- 64x64 low-res heightmap
- Basic texture information
- Used for far-view rendering
- Minimap data source

## World Structure

```text
World/
└── Maps/
    └── Azeroth/                    # Map name
        ├── Azeroth.wdt             # Map definition
        ├── Azeroth.wdl             # Low-detail version
        ├── Azeroth_32_48.adt       # Terrain tile (x=32, y=48)
        ├── Azeroth_32_48_tex0.adt  # Texture info
        ├── Azeroth_32_48_obj0.adt  # Object placement
        └── Azeroth_32_48_lod.adt   # Level of detail
```

## Coordinate System

- **Global**: Maps divided into 64x64 ADT grid
- **ADT**: Each ADT has 16x16 chunks
- **Chunk**: Each chunk has 9x9 vertices
- **Units**: 1 yard = 0.5 world units

## Common Patterns

### Loading a Map Section

```rust
use warcraft_rs::{Wdt, Adt};

// Load map definition
let wdt = Wdt::open("World/Maps/Azeroth/Azeroth.wdt")?;

// Check if tile exists
if wdt.has_adt(32, 48) {
    // Load the terrain tile
    let adt = Adt::open("World/Maps/Azeroth/Azeroth_32_48.adt")?;
}
```

### Streaming Large Worlds

```rust
use warcraft_rs::world::WorldManager;

let mut world = WorldManager::new("World/Maps/Azeroth")?;
world.set_view_distance(5); // Load 5x5 ADT grid

// Update based on player position
world.update(player_x, player_y);
```

## Rendering Considerations

1. **Level of Detail**: Use WDL for distant terrain
2. **Frustum Culling**: Cull ADT chunks outside view
3. **Texture Streaming**: Load textures on demand
4. **Height Queries**: Efficient terrain collision

## See Also

- [Rendering ADT Terrain Guide](../../guides/adt-rendering.md)
- [Coordinate Systems](../../resources/coordinates.md)
- [Map IDs Reference](../../resources/map-ids.md)
