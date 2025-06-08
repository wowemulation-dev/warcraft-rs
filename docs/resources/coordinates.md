# 🗺️ Coordinate Systems

A concise guide to understanding World of Warcraft's coordinate systems.

## Overview

WoW uses multiple coordinate systems that can be confusing. This guide clarifies
how they work and how to convert between them.

## Coordinate Systems

### 1. Map Grid Coordinates

The world is divided into a 64×64 grid of ADT tiles:

```text
    0   1   2  ...  63
  ┌───┬───┬───┬───┬───┐
0 │   │   │   │   │   │
  ├───┼───┼───┼───┼───┤
1 │   │   │   │   │   │
  ├───┼───┼───┼───┼───┤
2 │   │ADT│   │   │   │  Each cell = 1 ADT file
  ├───┼───┼───┼───┼───┤   Size: 533.33333 yards
  │   │   │   │   │   │
  └───┴───┴───┴───┴───┘
 63
```

- **Origin**: Top-left corner at (0,0)
- **Direction**: X increases right, Y increases down
- **Range**: 0-63 for both axes

### 2. ADT Local Coordinates

Within each ADT, terrain is divided into 16×16 chunks:

```text
    0   1   2  ...  15
  ┌───┬───┬───┬───┬───┐
0 │   │   │   │   │   │
  ├───┼───┼───┼───┼───┤
1 │   │   │   │   │   │
  ├───┼───┼───┼───┼───┤   Each chunk = 33.3333 yards
2 │   │CHK│   │   │   │   Total ADT = 533.3333 yards
  ├───┼───┼───┼───┼───┤
  │   │   │   │   │   │
  └───┴───┴───┴───┴───┘
 15
```

### 3. World Coordinates

The global coordinate system with origin at the center of the map:

```text
              +X (North)
               ↑
               │
    ←──────────┼──────────→ -Y (East)
    +Y (West)  │ (0,0,0)
               │
               ↓
              -X (South)
```

- **Origin**: Center of the world (32,32 in grid coordinates)
- **Unit**: 1 unit = 1 yard (in-game)
- **Range**: ±17066.66656 yards from center

### 4. Client Coordinates

The coordinate system used in-game (displayed on maps):

```text
         North (+X)
             ↑
             │
West ←───────┼───────→ East (-Y)
(+Y)         │
             │
             ↓
         South (-X)
```

**Note**: This matches the world coordinate system!

## Conversion Formulas

### Grid to World Coordinates

```rust
fn grid_to_world(grid_x: u32, grid_y: u32) -> (f32, f32) {
    const TILE_SIZE: f32 = 533.33333;
    const GRID_CENTER: f32 = 32.0;

    // Grid (0,0) is at the northwest corner
    // X points North (grid_y increases southward)
    // Y points West (grid_x increases eastward)
    let world_x = (GRID_CENTER - grid_y as f32) * TILE_SIZE;
    let world_y = (GRID_CENTER - grid_x as f32) * TILE_SIZE;

    (world_x, world_y)
}
```

### World to Grid Coordinates

```rust
fn world_to_grid(world_x: f32, world_y: f32) -> (u32, u32) {
    const TILE_SIZE: f32 = 533.33333;
    const GRID_CENTER: f32 = 32.0;

    // Inverse of grid_to_world
    let grid_y = (GRID_CENTER - world_x / TILE_SIZE) as u32;
    let grid_x = (GRID_CENTER - world_y / TILE_SIZE) as u32;

    (grid_x, grid_y)
}
```

### ADT Local to World

```rust
fn adt_local_to_world(
    grid_x: u32,
    grid_y: u32,
    chunk_x: u32,
    chunk_y: u32,
    local_x: f32,
    local_y: f32
) -> (f32, f32, f32) {
    const TILE_SIZE: f32 = 533.33333;
    const CHUNK_SIZE: f32 = 33.33333;
    const GRID_CENTER: f32 = 32.0;

    // Convert to world coordinates
    let world_x = (GRID_CENTER - grid_y as f32) * TILE_SIZE
                  - chunk_x as f32 * CHUNK_SIZE
                  - local_x;

    let world_y = (GRID_CENTER - grid_x as f32) * TILE_SIZE
                  - chunk_y as f32 * CHUNK_SIZE
                  - local_y;

    (world_x, world_y, 0.0) // Z handled separately
}
```

## Quick Reference

| System | Origin | X Direction | Y Direction | Notes |
|--------|--------|-------------|-------------|-------|
| Grid | Top-left (0,0) | Right → | Down ↓ | File naming |
| World | Center (0,0,0) | North → | West → | Internal coords |
| Client | Center (0,0) | North → | West → | UI display |
| ADT Local | Top-left (0,0) | Right → | Down ↓ | Per-tile |

## Common Pitfalls

### 1. Coordinate System Consistency

World and client coordinates use the same system, so no conversion is needed:

```rust
// World and client coordinates are the same
fn world_to_client(world_x: f32, world_y: f32) -> (f32, f32) {
    (world_x, world_y)  // No conversion needed!
}
```

### 2. Grid Origin

Remember that grid (0,0) is NOT world (0,0):

```text
Grid (0,0) = World (17066.66656, 17066.66656)
Grid (32,32) = World (0, 0)
Grid (63,63) = World (-17066.66656, -17066.66656)
```

### 3. Chunk Boundaries

Chunks within ADT also use top-left origin:

```rust
// Get world position of chunk corner
fn chunk_corner_world_pos(
    grid_x: u32,
    grid_y: u32,
    chunk_x: u32,
    chunk_y: u32
) -> (f32, f32) {
    const TILE_SIZE: f32 = 533.33333;
    const CHUNK_SIZE: f32 = 33.33333;
    const HALF_GRID: f32 = 32.0;

    let world_x = (HALF_GRID - grid_y as f32) * TILE_SIZE
                  - chunk_x as f32 * CHUNK_SIZE;
    let world_y = (HALF_GRID - grid_x as f32) * TILE_SIZE
                  - chunk_y as f32 * CHUNK_SIZE;

    (world_x, world_y)
}
```

## Practical Examples

### Finding Player's ADT

```rust
fn get_player_adt(player_x: f32, player_y: f32) -> (u32, u32) {
    // Player coordinates are client coordinates
    let world_x = player_x;
    let world_y = -player_y;  // Flip Y!

    world_to_grid(world_x, world_y)
}
```

### Height Query

```rust
fn get_height_at_position(world: &World, x: f32, y: f32) -> Option<f32> {
    // Convert world to grid
    let (grid_x, grid_y) = world_to_grid(x, y);

    // Load ADT
    let adt = world.get_adt(grid_x, grid_y)?;

    // Convert to ADT-local coordinates
    let local_x = x - chunk_corner_world_pos(grid_x, grid_y, 0, 0).0;
    let local_y = y - chunk_corner_world_pos(grid_x, grid_y, 0, 0).1;

    // Query height
    adt.get_height(local_x, local_y)
}
```

## Visual Summary

```text
┌─────────────────────────────────────┐
│          WORLD MAP VIEW             │
│                                     │
│    Grid(0,0) ←───────→ Grid(63,0)   │
│         ↑                    ↑      │
│         │      North ↑       │      │
│         │            │       │      │
│         │   West ←───┼───→ East    │
│         │            │       │      │
│         │      South ↓       │      │
│         ↓                    ↓      │
│    Grid(0,63) ←───────→ Grid(63,63) │
│                                     │
│    World Origin = Grid(32,32)       │
└─────────────────────────────────────┘
```

## See Also

- [ADT Format](../formats/world-data/adt.md)
- [WDT Format](../formats/world-data/wdt.md)
- [Map IDs Reference](map-ids.md)
