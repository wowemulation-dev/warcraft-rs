# World Data Formats

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
use wow_wdt::{WdtReader, version::WowVersion};
use std::fs::File;
use std::io::BufReader;

// Load map definition
let file = File::open("World/Maps/Azeroth/Azeroth.wdt")?;
let mut reader = WdtReader::new(BufReader::new(file), WowVersion::WotLK);
let wdt = reader.read()?;

// Check if tile exists
if let Some(tile_info) = wdt.get_tile(32, 48) {
    if tile_info.has_adt {
        // Load the terrain tile
        // let adt = Adt::open("World/Maps/Azeroth/Azeroth_32_48.adt")?;
        println!("ADT tile exists at [32, 48] - Area ID: {}", tile_info.area_id);
    }
}
```

### Streaming Large Worlds

```rust
use wow_wdt::{WdtReader, version::WowVersion, tile_to_world, world_to_tile};
use std::collections::HashSet;

// This example shows how you might implement world streaming
// (actual implementation would be in your game engine)

struct WorldStreamer {
    existing_tiles: HashSet<(usize, usize)>,
    loaded_tiles: HashSet<(usize, usize)>,
    view_distance: usize,
}

impl WorldStreamer {
    fn new(wdt_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(wdt_path)?;
        let mut reader = WdtReader::new(std::io::BufReader::new(file), WowVersion::WotLK);
        let wdt = reader.read()?;

        let mut existing_tiles = HashSet::new();
        for y in 0..64 {
            for x in 0..64 {
                if let Some(tile_info) = wdt.get_tile(x, y) {
                    if tile_info.has_adt {
                        existing_tiles.insert((x, y));
                    }
                }
            }
        }

        Ok(Self {
            existing_tiles,
            loaded_tiles: HashSet::new(),
            view_distance: 5,
        })
    }

    fn update(&mut self, player_x: f32, player_y: f32) {
        let (center_tile_x, center_tile_y) = world_to_tile(player_x, player_y);

        // Determine which tiles should be loaded
        let mut needed_tiles = HashSet::new();
        for dy in -(self.view_distance as i32)..=(self.view_distance as i32) {
            for dx in -(self.view_distance as i32)..=(self.view_distance as i32) {
                let tile_x = (center_tile_x as i32 + dx).max(0).min(63) as usize;
                let tile_y = (center_tile_y as i32 + dy).max(0).min(63) as usize;

                if self.existing_tiles.contains(&(tile_x, tile_y)) {
                    needed_tiles.insert((tile_x, tile_y));
                }
            }
        }

        // Load new tiles, unload distant ones
        for &tile in &needed_tiles {
            if !self.loaded_tiles.contains(&tile) {
                // Load ADT tile here
                println!("Loading tile {:?}", tile);
                self.loaded_tiles.insert(tile);
            }
        }

        self.loaded_tiles.retain(|tile| needed_tiles.contains(tile));
    }
}
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
