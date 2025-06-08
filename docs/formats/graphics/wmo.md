# WMO Format 🏰

WMO (World Map Object) files contain large static structures like buildings,
dungeons, and caves in World of Warcraft.

## Overview

- **Extension**: `.wmo` (root file), `_###.wmo` (group files)
- **Purpose**: Large static world geometry with interior/exterior areas
- **Components**: Root file + multiple group files
- **Features**: Portals, lighting, multiple detail levels, collision
- **Use Cases**: Buildings, dungeons, caves, large structures

## Version History

- **Classic (1.12.1)**: Original WMO format
- **TBC (2.4.3)**: Added vertex colors
- **WotLK (3.3.5)**: Enhanced lighting system
- **Cataclysm (4.3.4)**: Added terrain blending
- **MoP (5.4.8)**: Improved material system

## Structure

### Root File (.wmo)

```rust
struct WMORoot {
    // Header chunks
    mver: MVERChunk,          // Version
    mohd: MOHDChunk,          // Header info
    motx: MOTXChunk,          // Texture filenames
    momt: MOMTChunk,          // Materials
    mogn: MOGNChunk,          // Group names
    mogi: MOGIChunk,          // Group information
    mosb: MOSBChunk,          // Skybox (optional)
    mopv: MOPVChunk,          // Portal vertices
    mopt: MOPTChunk,          // Portal information
    mopr: MOPRChunk,          // Portal references
    movv: MOVVChunk,          // Visible vertices
    movb: MOVBChunk,          // Visible blocks
    molt: MOLTChunk,          // Lights
    mods: MODSChunk,          // Doodad sets
    modn: MODNChunk,          // Doodad names (M2 paths)
    modd: MODDChunk,          // Doodad definitions
    mfog: MFOGChunk,          // Fog definitions
    mcvp: MCVPChunk,          // Convex volume planes (optional)
}

struct MOHDChunk {
    n_materials: u32,         // Number of materials
    n_groups: u32,            // Number of groups
    n_portals: u32,           // Number of portals
    n_lights: u32,            // Number of lights
    n_doodad_names: u32,      // Number of M2 names
    n_doodad_defs: u32,       // Number of M2 instances
    n_doodad_sets: u32,       // Number of doodad sets
    ambient_color: Rgba,      // Ambient light color
    wmo_id: u32,              // WMO ID (from WMOAreaTable.dbc)
    bounding_box: BoundingBox,
    flags: u16,               // Global WMO flags
    num_lods: u16,            // Number of LOD files
}
```

### Group Files (_###.wmo)

```rust
struct WMOGroup {
    // Group header
    mogp: MOGPChunk,          // Group header

    // Geometry data
    mopy: MOPYChunk,          // Material info for triangles
    movi: MOVIChunk,          // Vertex indices
    movt: MOVTChunk,          // Vertices
    monr: MONRChunk,          // Normals
    motv: MOTVChunk,          // Texture coordinates
    moba: MOBAChunk,          // Render batches
    mocv: MOCVChunk,          // Vertex colors (2.0+)
    molr: MOLRChunk,          // Light references
    modr: MODRChunk,          // Doodad references
    mobn: MOBNChunk,          // BSP nodes
    mobr: MOBRChunk,          // BSP faces
    mliq: MLIQChunk,          // Liquids
    mori: MORIChunk,          // Triangle strips (optional)
    morb: MORBChunk,          // Additional render batches
}

struct MOGPChunk {
    group_name: u32,          // Offset into MOGN
    descriptive_name: u32,    // Offset into MOGN
    flags: u32,               // Group flags
    bounding_box: BoundingBox,
    portal_start: u16,        // First portal index
    portal_count: u16,        // Number of portals
    transBatchCount: u16,
    intBatchCount: u16,
    extBatchCount: u16,
    padding: u16,
    fog_indices: [u8; 4],     // Up to 4 fog definitions
    group_liquid: u32,        // Liquid type
    unique_id: u32,           // WMO group ID
}
```

## Usage Example

```rust
use warcraft_rs::wmo::{WMO, WMOInstance, DoodadSet};

// Load WMO
let mut wmo = WMO::open("World/wmo/Azeroth/Buildings/Stormwind/Stormwind.wmo")?;

// Get information
println!("WMO: {}", wmo.name());
println!("Groups: {}", wmo.group_count());
println!("Materials: {}", wmo.material_count());
println!("Portals: {}", wmo.portal_count());

// Load all groups
wmo.load_all_groups()?;

// Render specific doodad set
wmo.set_doodad_set(DoodadSet::Normal)?;

// Iterate through groups
for (idx, group) in wmo.groups().enumerate() {
    println!("Group {}: {} vertices, {} indices",
        idx, group.vertex_count(), group.index_count());

    // Check if indoor/outdoor
    if group.is_indoor() {
        println!("  Indoor group");
    }

    // Render group
    for batch in group.render_batches() {
        let material = wmo.get_material(batch.material_id);
        renderer.set_material(material);
        renderer.draw_batch(batch);
    }
}

// Handle portals for culling
for portal in wmo.portals() {
    if portal.is_visible_from(camera_pos) {
        // Render groups visible through this portal
    }
}
```

## Rendering System

### Portal-Based Culling

```rust
struct WMORenderer {
    frustum: Frustum,
    visited_groups: HashSet<u16>,
}

impl WMORenderer {
    fn render_wmo(&mut self, wmo: &WMO, camera_group: u16) {
        self.visited_groups.clear();
        self.render_group_recursive(wmo, camera_group);
    }

    fn render_group_recursive(&mut self, wmo: &WMO, group_idx: u16) {
        if self.visited_groups.contains(&group_idx) {
            return;
        }
        self.visited_groups.insert(group_idx);

        let group = &wmo.groups[group_idx as usize];

        // Frustum culling
        if !self.frustum.intersects_box(&group.bounds) {
            return;
        }

        // Render this group
        self.render_group(group);

        // Check portals to other groups
        for portal_idx in group.portal_start..group.portal_start + group.portal_count {
            let portal = &wmo.portals[portal_idx as usize];

            if self.frustum.intersects_portal(portal) {
                // Render connected group
                let connected_group = portal.get_connected_group(group_idx);
                self.render_group_recursive(wmo, connected_group);
            }
        }
    }
}
```

### Material System

```rust
struct WMOMaterial {
    flags: u32,
    shader: u32,              // Shader ID
    blend_mode: u32,          // Blending mode
    texture_1: u32,           // Diffuse texture
    emissive_color: Rgba,     // Emissive color
    emissive_multiplier: f32, // Added in 2.0
    texture_2: u32,           // Env map or second texture
    diff_color: Rgba,         // Added in Cata
    terrain_type: u32,        // Footstep sounds
    texture_3: u32,           // Added in Cata
    color_3: Rgba,
    flags_3: u32,
    runtime_data: [u32; 4],   // Not saved
}

impl WMOMaterial {
    fn setup_render_state(&self, renderer: &mut Renderer) {
        // Set textures
        renderer.bind_texture(0, self.texture_1);
        if self.flags & 0x80 != 0 {
            renderer.bind_texture(1, self.texture_2);
        }

        // Set blend mode
        match self.blend_mode {
            0 => renderer.set_blend_mode(BlendMode::Opaque),
            1 => renderer.set_blend_mode(BlendMode::AlphaKey),
            2 => renderer.set_blend_mode(BlendMode::Alpha),
            3 => renderer.set_blend_mode(BlendMode::NoAlphaAdd),
            4 => renderer.set_blend_mode(BlendMode::Add),
            5 => renderer.set_blend_mode(BlendMode::Mod),
            6 => renderer.set_blend_mode(BlendMode::Mod2x),
            _ => {}
        }

        // Set shader
        renderer.set_shader(self.get_shader_program());
    }
}
```

## Advanced Features

### Liquid Rendering

```rust
struct WMOLiquid {
    x_verts: u32,
    y_verts: u32,
    x_tiles: u32,
    y_tiles: u32,
    corner: Vec3,
    liquid_type: u16,
    heights: Vec<f32>,
    flow: Vec<Vec2>,          // Flow direction per vertex
}

impl WMOLiquid {
    fn generate_mesh(&self) -> LiquidMesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Generate vertices
        for y in 0..self.y_verts {
            for x in 0..self.x_verts {
                let idx = y * self.x_verts + x;
                let height = self.heights[idx as usize];

                vertices.push(LiquidVertex {
                    position: Vec3::new(
                        self.corner.x + x as f32 * LIQUID_TILE_SIZE,
                        self.corner.y + y as f32 * LIQUID_TILE_SIZE,
                        height
                    ),
                    flow: self.flow[idx as usize],
                    depth: self.calculate_depth(x, y),
                });
            }
        }

        // Generate indices for tiles
        for y in 0..self.y_tiles {
            for x in 0..self.x_tiles {
                let idx = y * self.x_verts + x;

                // Two triangles per tile
                indices.extend_from_slice(&[
                    idx, idx + 1, idx + self.x_verts,
                    idx + 1, idx + self.x_verts + 1, idx + self.x_verts,
                ]);
            }
        }

        LiquidMesh { vertices, indices }
    }
}
```

### BSP Tree Navigation

```rust
struct BSPTree {
    nodes: Vec<BSPNode>,
    faces: Vec<BSPFace>,
}

impl BSPTree {
    fn find_leaf(&self, point: Vec3) -> Option<u16> {
        let mut node_idx = 0i16;

        while node_idx >= 0 {
            let node = &self.nodes[node_idx as usize];
            let plane = Plane::from_coefficients(node.plane_coeffs);

            if plane.distance_to_point(point) >= 0.0 {
                node_idx = node.positive_child;
            } else {
                node_idx = node.negative_child;
            }
        }

        // Negative indices indicate leaves
        Some((-(node_idx + 1)) as u16)
    }

    fn get_colliding_faces(&self, ray: &Ray) -> Vec<CollisionFace> {
        let mut faces = Vec::new();
        self.traverse_ray(0, ray, &mut faces);
        faces
    }
}
```

### Lighting System

```rust
struct WMOLight {
    light_type: u8,           // 0=small, 1=large, 2=spot
    use_attenuation: bool,
    use_diffuse_color: bool,
    use_ambient_color: bool,
    diffuse_color: Rgba,
    ambient_color: Rgba,
    position: Vec3,
    diffuse_intensity: f32,
    ambient_intensity: f32,
    attenuation_start: f32,
    attenuation_end: f32,
}

struct WMOLightingSystem {
    lights: Vec<WMOLight>,
    light_grid: SpatialGrid<u16>,
}

impl WMOLightingSystem {
    fn calculate_lighting(&self, pos: Vec3, normal: Vec3) -> Rgba {
        let mut color = Rgba::BLACK;

        // Get nearby lights from spatial grid
        let nearby_lights = self.light_grid.query_sphere(pos, MAX_LIGHT_RANGE);

        for light_idx in nearby_lights {
            let light = &self.lights[light_idx as usize];
            let contribution = light.calculate_contribution(pos, normal);
            color = color.add(contribution);
        }

        color.clamp()
    }
}
```

## Common Patterns

### WMO Instancing

```rust
struct WMOInstance {
    wmo: Arc<WMO>,
    transform: Matrix4,
    doodad_set: u16,
    lighting_override: Option<Rgba>,
}

struct WMOInstanceRenderer {
    instances: HashMap<u32, Vec<WMOInstance>>,
}

impl WMOInstanceRenderer {
    fn render_all(&mut self, view: &ViewParams) {
        // Group instances by WMO
        for (wmo_id, instances) in &self.instances {
            let wmo = &instances[0].wmo;

            // Bind WMO resources once
            self.bind_wmo_resources(wmo);

            // Render all instances
            for instance in instances {
                self.set_instance_transform(&instance.transform);
                self.set_doodad_set(instance.doodad_set);
                self.render_wmo_instance(wmo, instance);
            }
        }
    }
}
```

### LOD Management

```rust
fn select_wmo_lod(wmo: &WMO, view_distance: f32) -> u16 {
    let lod_distances = [50.0, 150.0, 300.0, 600.0];

    for (lod, &distance) in lod_distances.iter().enumerate() {
        if view_distance < distance {
            return lod.min(wmo.num_lods - 1) as u16;
        }
    }

    wmo.num_lods - 1
}
```

## Performance Considerations

- Use portal culling for interior spaces
- Implement frustum culling per group
- Batch render calls by material
- Use BSP tree for efficient collision
- Cache vertex buffers per group
- Consider distance-based LOD

## Common Issues

### Missing Groups

- Check all group files exist (_000.wmo,_001.wmo, etc.)
- Group count from root file
- Handle missing groups gracefully

### Portal Connections

- Portals must reference valid groups
- Check portal winding order
- Validate portal connectivity

### Material References

- Texture IDs must be valid
- Check texture file existence
- Handle missing textures

## References

- [WMO Format (wowdev.wiki)](https://wowdev.wiki/WMO)
- [WMO Rendering](https://wowdev.wiki/WMO/Rendering)

## See Also

- [BLP Format](blp.md) - Texture format used by WMO
- [M2 Format](m2.md) - Doodads placed in WMO
- [ADT Format](../world-data/adt.md) - Terrain that WMOs sit on
- [WMO Rendering Guide](../../guides/wmo-rendering.md)
