# M2 Skin Format 🎨

M2 .skin files contain mesh data and level-of-detail (LOD) information for M2
models.

## Overview

- **Extension**: `.skin`
- **Purpose**: Store mesh topology and LOD variants
- **Naming**: `<ModelName>00.skin`, `<ModelName>01.skin`, etc.
- **LOD Levels**: Usually 0-3 (0 = highest detail)
- **Benefits**: Reduced GPU memory, better performance at distance

## Structure

### Skin File Header

```rust
struct M2SkinHeader {
    magic: [u8; 4],           // "SKIN"
    vertices: M2Array<u16>,   // Vertex indices into M2 vertex list
    indices: M2Array<u16>,    // Triangle indices (3 per face)
    bones: M2Array<M2SkinBone>, // Bone influences
    submeshes: M2Array<M2SkinSubmesh>, // Mesh sections
    texture_units: M2Array<M2SkinTextureUnit>, // Texture assignments
    bone_count_max: u32,      // Max bones per draw call
}

struct M2SkinSubmesh {
    mesh_part_id: u16,        // Mesh part identifier
    starting_vertex: u16,     // First vertex in vertex list
    vertex_count: u16,        // Number of vertices
    starting_triangle: u16,   // First triangle index
    triangle_count: u16,      // Number of triangle indices
    bone_count: u16,          // Number of bones
    bone_combo_index: u16,    // Index into bone combination
    bone_influences: u16,     // Max bone influences used
    center_bone_index: u16,   // Bone for bounding calculations
    center_position: Vec3,    // Bounding center
    sort_center: Vec3,        // Sort center for alpha
    sort_radius: f32,         // Sort radius
}
```

### Geoset Types

```rust
// Standard geoset IDs for character models
enum GeosetType {
    Skin = 0,
    Hair = 1,
    Facial1 = 2,
    Facial2 = 3,
    Facial3 = 4,
    Gloves = 5,
    Boots = 6,
    Tail = 7,
    Ears = 8,
    Wristbands = 9,
    Kneepads = 10,
    Chest = 11,
    Pants = 12,
    Tabard = 13,
    Trousers = 14,
    Tabard2 = 15,
    Cape = 16,
    Feet = 17,
    Eyeglow = 18,
    Belt = 19,
    Tail2 = 20,
    // ... more
}
```

## Usage Example

```rust
use warcraft_rs::m2::{M2Model, M2Skin, LodLevel};

// Load model and skin
let mut model = M2Model::open("Character/Human/Female/HumanFemale.m2")?;
let skin = M2Skin::open("Character/Human/Female/HumanFemale00.skin")?;
model.set_skin(skin);

// Get mesh data
let mesh_data = model.build_mesh_data()?;
println!("Vertices: {}", mesh_data.vertices.len());
println!("Indices: {}", mesh_data.indices.len());
println!("Submeshes: {}", mesh_data.submeshes.len());

// Render each submesh
for submesh in &mesh_data.submeshes {
    let material = model.get_material(submesh.material_id);
    renderer.set_material(material);

    renderer.draw_indexed(
        &mesh_data.vertices,
        &mesh_data.indices[submesh.index_start..submesh.index_end],
    );
}

// Load different LOD
let lod1_skin = M2Skin::open("Character/Human/Female/HumanFemale01.skin")?;
model.set_skin(lod1_skin);
```

## LOD Management

### Selecting Appropriate LOD

```rust
struct LodSelector {
    lod_distances: [f32; 4],  // Distance thresholds
}

impl LodSelector {
    fn select_lod(&self, view_distance: f32, importance: f32) -> u8 {
        let adjusted_distance = view_distance / importance;

        for (lod, threshold) in self.lod_distances.iter().enumerate() {
            if adjusted_distance < *threshold {
                return lod as u8;
            }
        }

        3 // Lowest detail
    }
}

// Usage
let selector = LodSelector {
    lod_distances: [30.0, 60.0, 120.0, 250.0],
};

let lod = selector.select_lod(player_distance, 1.0);
let skin_file = format!("{}{:02}.skin", model_name, lod);
```

### Dynamic LOD Loading

```rust
struct DynamicLodModel {
    base_model: M2Model,
    skins: [Option<M2Skin>; 4],
    current_lod: u8,
}

impl DynamicLodModel {
    fn update_lod(&mut self, new_lod: u8) -> Result<()> {
        if new_lod != self.current_lod {
            // Load skin if not cached
            if self.skins[new_lod as usize].is_none() {
                let skin_path = format!("{}{:02}.skin",
                    self.base_model.base_name(), new_lod);
                self.skins[new_lod as usize] = Some(M2Skin::open(&skin_path)?);
            }

            // Apply skin
            if let Some(skin) = &self.skins[new_lod as usize] {
                self.base_model.set_skin(skin.clone());
                self.current_lod = new_lod;
            }
        }
        Ok(())
    }
}
```

## Advanced Features

### Geoset Visibility

```rust
// Character customization through geoset control
struct CharacterCustomizer {
    model: M2Model,
    visible_geosets: HashSet<u16>,
}

impl CharacterCustomizer {
    fn set_armor_piece(&mut self, slot: ArmorSlot, item_id: u32) {
        // Hide conflicting geosets
        match slot {
            ArmorSlot::Chest => {
                self.hide_geoset(GeosetType::Chest);
                self.hide_geoset(GeosetType::Wristbands);
            }
            ArmorSlot::Legs => {
                self.hide_geoset(GeosetType::Pants);
                self.hide_geoset(GeosetType::Kneepads);
            }
            ArmorSlot::Boots => {
                self.hide_geoset(GeosetType::Boots);
                self.hide_geoset(GeosetType::Feet);
            }
            _ => {}
        }

        // Show item geosets
        let item_geosets = get_item_geosets(item_id);
        for geoset in item_geosets {
            self.show_geoset(geoset);
        }
    }

    fn render(&self) {
        for submesh in self.model.submeshes() {
            if self.visible_geosets.contains(&submesh.mesh_part_id) {
                renderer.draw_submesh(submesh);
            }
        }
    }
}
```

### Mesh Optimization

```rust
// Optimize skin data for GPU
fn optimize_skin(skin: &M2Skin) -> OptimizedSkin {
    let mut optimizer = MeshOptimizer::new();

    // Optimize vertex cache
    let optimized_indices = optimizer.optimize_vertex_cache(&skin.indices);

    // Remove duplicate vertices
    let (unique_vertices, index_map) = optimizer.remove_duplicates(&skin.vertices);

    // Generate vertex buffer regions for instancing
    let buffer_regions = optimizer.create_buffer_regions(&skin.submeshes);

    OptimizedSkin {
        vertices: unique_vertices,
        indices: optimized_indices,
        buffer_regions,
    }
}
```

### Bone Batching

```rust
// Minimize draw calls by batching submeshes with same bones
fn batch_submeshes(skin: &M2Skin) -> Vec<DrawBatch> {
    let mut batches: HashMap<Vec<u16>, DrawBatch> = HashMap::new();

    for submesh in &skin.submeshes {
        let bone_combo = skin.get_bone_combo(submesh.bone_combo_index);

        batches.entry(bone_combo.to_vec())
            .or_insert_with(|| DrawBatch::new(bone_combo))
            .add_submesh(submesh);
    }

    batches.into_values().collect()
}
```

## Common Patterns

### Multi-Resolution Rendering

```rust
struct MultiResRenderer {
    models: Vec<(M2Model, f32)>, // Model and distance
    lod_bias: f32,
}

impl MultiResRenderer {
    fn render(&mut self, camera: &Camera) {
        // Sort by distance
        self.models.sort_by(|a, b|
            b.1.partial_cmp(&a.1).unwrap()
        );

        for (model, distance) in &mut self.models {
            // Select LOD based on distance and settings
            let lod = self.calculate_lod(*distance);
            model.set_lod(lod);

            // Skip if too far
            if lod > 3 {
                continue;
            }

            // Render with appropriate detail
            model.render();
        }
    }
}
```

### Skin Streaming

```rust
struct SkinStreamer {
    cache: LruCache<String, M2Skin>,
    loading: HashSet<String>,
}

impl SkinStreamer {
    async fn get_skin(&mut self, path: &str) -> Result<&M2Skin> {
        if !self.cache.contains(path) && !self.loading.contains(path) {
            self.loading.insert(path.to_string());

            // Async load
            let skin = tokio::spawn(async move {
                M2Skin::open(path)
            });

            let loaded_skin = skin.await??;
            self.cache.put(path.to_string(), loaded_skin);
            self.loading.remove(path);
        }

        Ok(self.cache.get(path).unwrap())
    }
}
```

## Performance Considerations

- LOD 0 can have 10x more triangles than LOD 3
- Batch submeshes with same material/bones
- Use index buffers for efficient GPU usage
- Consider view frustum culling per submesh
- Preload next LOD during idle time

## Common Issues

### Missing LODs

- Not all models have all 4 LOD levels
- Fallback to nearest available LOD
- LOD 0 always exists

### Geoset IDs

- IDs not standardized across all models
- Character models follow convention
- Creature models may use custom IDs

### Bone Limits

- GPU has max bones per batch (usually 64-256)
- Split submeshes if exceeding limit
- Check `bone_count_max` field

## References

- [M2/.skin Format (wowdev.wiki)](https://wowdev.wiki/M2/.skin)
- [Geoset Types](https://wowdev.wiki/M2#Geosets)

## See Also

- [M2 Format](m2.md) - Main model format
- [LOD System Guide](../../guides/lod-system.md)
- [Character Customization Guide](../../guides/character-customization.md)
