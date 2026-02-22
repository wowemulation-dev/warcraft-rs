# World of Warcraft Coordinate Systems

This document explains the coordinate system used by World of Warcraft and provides utilities for transforming coordinates to common target systems used in 3D applications and game engines.

## WoW Coordinate System

World of Warcraft uses a **right-handed coordinate system** with the following axis orientations:

- **X-axis**: **North** (positive X points north)
- **Y-axis**: **West** (positive Y points west)  
- **Z-axis**: **Up** (positive Z points upward, 0 = sea level)

```
       Z (Up)
       ^
       |
       |
Y <----+
(West)  \
         \
          v X (North)
```

This coordinate system is used consistently across all WoW file formats including:
- M2 models (vertices, bones, attachments)
- ADT terrain tiles 
- WMO world objects
- Camera positions and orientations

## Common Target Systems

### Blender (Right-handed)
- **Right** = (1, 0, 0) → X-axis
- **Forward** = (0, 1, 0) → Y-axis  
- **Up** = (0, 0, 1) → Z-axis

```
       Z (Up)
       ^
       |
       |
       +----> Y (Forward)  
      /
     /
    v X (Right)
```

### Unity (Left-handed)
- **Right** = (1, 0, 0) → X-axis
- **Up** = (0, 1, 0) → Y-axis
- **Forward** = (0, 0, 1) → Z-axis

```
    Y (Up)
    ^
    |
    |
    +----> X (Right)
   /
  /
 v Z (Forward)
```

### Unreal Engine (Left-handed)
- **Forward** = (1, 0, 0) → X-axis
- **Right** = (0, 1, 0) → Y-axis
- **Up** = (0, 0, 1) → Z-axis

## Transformation Formulas

### WoW → Blender

**Positions/Vertices:**
```
blender_position = (wow_y, -wow_x, wow_z)
```

**Rotations (Quaternions):**
```
blender_quaternion = (wow_w, wow_y, -wow_x, wow_z)
```

**Example with actual coordinates:**
```
WoW position: (100.0, 200.0, 50.0)    # 100 north, 200 west, 50 up
Blender position: (200.0, -100.0, 50.0)  # 200 forward, -100 right, 50 up
```

### WoW → Unity

**Positions/Vertices:**
```
unity_position = (-wow_y, wow_z, wow_x)
```

**Rotations (Quaternions):**
```
unity_quaternion = (wow_y, -wow_z, -wow_x, wow_w)
```

### WoW → Unreal Engine

**Positions/Vertices:**
```
unreal_position = (wow_x, -wow_y, wow_z)
```

**Rotations (Quaternions):**
```
unreal_quaternion = (-wow_x, wow_y, -wow_z, wow_w)
```

## Code Examples

### Basic Usage

```rust
use wow_m2::coordinate::{CoordinateSystem, transform_position, transform_quaternion};

// Load a model
let model = wow_m2::M2Model::load("character.m2")?;

// Transform vertices for Blender
for vertex in &model.vertices {
    let blender_pos = transform_position(vertex.position, CoordinateSystem::Blender);
    println!("WoW: {:?} → Blender: {:?}", vertex.position, blender_pos);
}

// Transform bone rotations for Unity
for bone in &model.bones {
    if let Some(rotation) = bone.rotation {
        let unity_rot = transform_quaternion(rotation, CoordinateSystem::Unity);
        println!("WoW: {:?} → Unity: {:?}", rotation, unity_rot);
    }
}
```

### Batch Transformation

```rust
use wow_m2::coordinate::{CoordinateTransformer, CoordinateSystem};

// Create a transformer for consistent conversions
let transformer = CoordinateTransformer::new(CoordinateSystem::Blender);

// Transform all model data at once
let transformed_model = transformer.transform_model(&model)?;

// Or transform specific data types
let blender_vertices = transformer.transform_positions(&model.vertices);
let blender_bones = transformer.transform_bones(&model.bones);
```

### Working with Animations

```rust
use wow_m2::coordinate::transform_animation_data;

// Load animation file
let anim = wow_m2::AnimFile::load("character0-0.anim")?;

// Transform animation keyframes for target system
let blender_anim = transform_animation_data(&anim, CoordinateSystem::Blender)?;
```

## Why Models Appear Wrong Without Transformation

When loading WoW models directly into 3D applications without coordinate transformation, you'll typically see:

### Blender Issues:
- **Model rotated 90° clockwise** (what should face north faces east)
- **Model appears "sideways"** relative to Blender's default orientation
- **Animations don't align** with Blender's bone system expectations

### Unity Issues:  
- **Model faces wrong direction** (north becomes forward, confusing navigation)
- **Physics and collision detection problems** due to axis mismatch
- **Camera controls feel inverted** when adapted from WoW coordinate expectations

### Root Cause:
These issues occur because each system interprets the same numeric coordinate values according to its own axis conventions. A point at `(100, 0, 0)` means "100 units north" in WoW but "100 units right" in Blender.

## Implementation Details

### Axis Mapping Table

| WoW Axis | Blender | Unity | Unreal |
|----------|---------|-------|--------|
| +X (North) | -Y | +Z | +X |
| +Y (West) | +X | -X | -Y |
| +Z (Up) | +Z | +Y | +Z |

### Quaternion Component Mapping

Quaternion transformations require careful component remapping because rotations around different axes need to be preserved correctly:

| WoW Quat | Blender | Unity | Unreal |
|----------|---------|-------|--------|
| x | -y | -z | -x |
| y | x | x | y |
| z | z | -y | -z |
| w | w | w | w |

### Matrix Transformation Approach

For more complex transformations or when working with transformation matrices directly:

**WoW → Blender Transformation Matrix:**
```
[ 0  1  0  0]   [wow_x]   [blender_x]
[-1  0  0  0] × [wow_y] = [blender_y]
[ 0  0  1  0]   [wow_z]   [blender_z]
[ 0  0  0  1]   [ 1   ]   [   1    ]
```

## Performance Considerations

### Bulk Transformations
When transforming large numbers of coordinates (thousands of vertices), use SIMD operations:

```rust
// Efficient batch transformation using glam's SIMD support
let wow_positions: &[glam::Vec3] = &vertex_positions;
let blender_positions: Vec<glam::Vec3> = wow_positions
    .iter()
    .map(|pos| glam::Vec3::new(pos.y, -pos.x, pos.z))
    .collect();
```

### In-Place Transformations
For memory efficiency, prefer in-place transformations when possible:

```rust
// Transform coordinates in-place to avoid allocations
for position in vertex_positions.iter_mut() {
    let temp_x = position.x;
    position.x = position.y;
    position.y = -temp_x;
    // position.z unchanged for WoW → Blender
}
```

## Common Pitfalls and Solutions

### Pitfall 1: Forgetting Quaternion Sign Corrections
**Problem:** Rotations appear correct but are mirrored or inverted.
**Solution:** Always apply the correct sign changes for quaternion components as shown in the transformation formulas.

### Pitfall 2: Mixing Coordinate Systems
**Problem:** Some model parts use transformed coordinates while others don't.
**Solution:** Consistently transform ALL coordinate data - vertices, bones, attachments, cameras, etc.

### Pitfall 3: Animation Timing Issues
**Problem:** Animations play correctly but bones rotate in wrong directions.
**Solution:** Transform animation keyframe data using the same coordinate system as the rest of the model.

### Pitfall 4: Texture Coordinate Confusion
**Problem:** Textures appear flipped or rotated incorrectly.
**Solution:** Texture coordinates (UV mappings) typically don't need coordinate system transformation - only 3D spatial coordinates do.

## Validation and Testing

### Visual Verification Checklist
When implementing coordinate transformations:

- [ ] Model faces the expected "forward" direction in target application
- [ ] Up/down orientation matches target system (Z+ or Y+ depending on system)
- [ ] Left/right handedness is correct (no mirroring)
- [ ] Animations rotate bones in expected directions
- [ ] Attachment points (weapons, accessories) align properly
- [ ] Camera positions and orientations work as expected

### Test Cases
Use these known coordinate transformations to verify implementation:

```rust
// Test case 1: Cardinal directions
assert_eq!(
    transform_position((1.0, 0.0, 0.0), CoordinateSystem::Blender),
    (0.0, -1.0, 0.0)  // North becomes -Y (backward) in Blender
);

// Test case 2: Identity quaternion
assert_eq!(
    transform_quaternion((0.0, 0.0, 0.0, 1.0), CoordinateSystem::Blender),
    (0.0, 0.0, 0.0, 1.0)  // Identity should remain identity
);
```

## See Also

- [Map Grid and World Coordinates](resources/coordinates.md) - ADT tile grid, world coordinates, and grid-to-world conversion

## References

- [wowdev.wiki ADT Format](https://wowdev.wiki/ADT) - Original coordinate system documentation
- [wowdev.wiki M2 Format](https://wowdev.wiki/M2) - Model coordinate specifications
- [Blender Coordinate System](https://docs.blender.org/manual/en/latest/advanced/coordinate_systems.html)
- [Unity Coordinate System](https://docs.unity3d.com/Manual/StandardAssetsImportingMeshes.html)
- [Unreal Engine Coordinate System](https://docs.unrealengine.com/4.27/en-US/Basics/CoordinateSpace/)