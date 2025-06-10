# 🌍 Rendering ADT Terrain

## Overview

ADT (Azeroth Data Terrain) files contain the terrain data for World of Warcraft's
seamless world. Each ADT file represents a 533.33x533.33 yard tile of the game
world, divided into 16x16 chunks. This guide covers how to load, process, and
render ADT terrain using `warcraft-rs`.

## Prerequisites

Before rendering ADT terrain, ensure you have:

- Understanding of 3D graphics programming (OpenGL/Vulkan/DirectX)
- Basic knowledge of terrain rendering techniques
- `warcraft-rs` installed with the `adt` and `wdt` features enabled
- A graphics rendering framework (wgpu, glow, etc.)
- Understanding of height maps and texture blending
- Knowledge of WDT files to determine which ADT tiles exist

## Understanding ADT Files

### ADT Structure

Each ADT file contains:

- **Height map data**: Vertex heights for terrain mesh
- **Texture information**: Up to 4 textures per chunk with alpha maps
- **Water data**: Lakes, rivers, and oceans
- **Shadow maps**: Pre-baked shadows
- **Vertex colors**: Lighting and shading information
- **Normal maps**: Surface normals for lighting
- **Holes**: Areas where terrain is not rendered
- **Doodad placement**: Positions for small objects
- **WMO placement**: Positions for buildings/large objects

### Coordinate System

- ADT tiles are arranged in a grid system
- Each map has 64x64 ADT tiles
- Coordinate format: `World_Map_TileX_TileY.adt`
- Example: `Azeroth_32_48.adt`

## Step-by-Step Instructions

### 1. Discovering ADT Tiles with WDT

Before loading ADT files, use the WDT file to determine which tiles exist:

```rust
use wow_wdt::{WdtReader, version::WowVersion};
use std::io::BufReader;
use std::fs::File;
use std::collections::HashSet;

fn discover_adt_tiles(map_name: &str, version: WowVersion) -> Result<HashSet<(usize, usize)>, Box<dyn std::error::Error>> {
    let wdt_path = format!("World/Maps/{}/{}.wdt", map_name, map_name);
    let file = File::open(wdt_path)?;
    let mut reader = WdtReader::new(BufReader::new(file), version);
    let wdt = reader.read()?;

    let mut existing_tiles = HashSet::new();

    // Skip WMO-only maps (dungeons, instances)
    if wdt.is_wmo_only() {
        println!("Map {} is WMO-only (no terrain tiles)", map_name);
        return Ok(existing_tiles);
    }

    // Find all tiles that have ADT data
    for y in 0..64 {
        for x in 0..64 {
            if let Some(tile_info) = wdt.get_tile(x, y) {
                if tile_info.has_adt {
                    existing_tiles.insert((x, y));
                    println!("Found ADT tile at [{}, {}] - Area ID: {}", x, y, tile_info.area_id);
                }
            }
        }
    }

    println!("Map {} has {} ADT tiles", map_name, existing_tiles.len());
    Ok(existing_tiles)
}

// Example usage
let existing_tiles = discover_adt_tiles("Azeroth", WowVersion::WotLK)?;
```

### 2. Loading ADT Files

```rust
use warcraft_rs::adt::{Adt, AdtFlags};
use warcraft_rs::common::{Vec3, ChunkId};

fn load_adt_file(filename: &str) -> Result<Adt, Box<dyn std::error::Error>> {
    // Load main terrain file
    let adt = Adt::from_file(filename)?;

    // Also load associated files
    let tex0_file = filename.replace(".adt", "_tex0.adt");
    let obj0_file = filename.replace(".adt", "_obj0.adt");
    let obj1_file = filename.replace(".adt", "_obj1.adt");

    // Load texture information
    let adt_tex = Adt::from_file(&tex0_file)?;

    // Load object placement
    let adt_obj0 = Adt::from_file(&obj0_file)?;
    let adt_obj1 = Adt::from_file(&obj1_file)?;

    Ok(adt)
}
```

### 2. Generating Terrain Mesh

```rust
use warcraft_rs::adt::{Adt, TerrainChunk};

#[derive(Debug, Clone)]
struct TerrainVertex {
    position: [f32; 3],
    normal: [f32; 3],
    texcoord: [f32; 2],
    vertex_color: [f32; 4],
}

fn generate_terrain_mesh(adt: &Adt) -> Vec<TerrainVertex> {
    let mut vertices = Vec::new();

    // ADT has 16x16 chunks
    for chunk_y in 0..16 {
        for chunk_x in 0..16 {
            let chunk = &adt.chunks[chunk_y * 16 + chunk_x];
            vertices.extend(generate_chunk_vertices(chunk, chunk_x, chunk_y));
        }
    }

    vertices
}

fn generate_chunk_vertices(chunk: &TerrainChunk, chunk_x: usize, chunk_y: usize) -> Vec<TerrainVertex> {
    let mut vertices = Vec::new();

    // Each chunk has 9x9 vertices (including corners shared with neighbors)
    // But internally uses 17x17 for proper tessellation
    const VERTS_PER_SIDE: usize = 9;

    for y in 0..VERTS_PER_SIDE {
        for x in 0..VERTS_PER_SIDE {
            let idx = y * VERTS_PER_SIDE + x;

            // Calculate world position
            let world_x = (chunk_x as f32 * 33.333) + (x as f32 * 4.166);
            let world_z = (chunk_y as f32 * 33.333) + (y as f32 * 4.166);
            let world_y = chunk.height_map[idx];

            // Get vertex normal
            let normal = chunk.normals[idx];

            // Calculate texture coordinates
            let u = x as f32 / 8.0;
            let v = y as f32 / 8.0;

            // Get vertex color (pre-baked lighting)
            let color = chunk.vertex_colors[idx];

            vertices.push(TerrainVertex {
                position: [world_x, world_y, world_z],
                normal: [normal.x, normal.y, normal.z],
                texcoord: [u, v],
                vertex_color: [color.r, color.g, color.b, color.a],
            });
        }
    }

    vertices
}
```

### 3. Generating Index Buffer

```rust
fn generate_terrain_indices(adt: &Adt) -> Vec<u32> {
    let mut indices = Vec::new();

    for chunk_idx in 0..256 {
        let chunk = &adt.chunks[chunk_idx];
        let base_vertex = (chunk_idx * 81) as u32; // 9x9 vertices per chunk

        // Check for holes in terrain
        let holes = chunk.holes;

        // Generate triangles for each quad
        for y in 0..8 {
            for x in 0..8 {
                let quad_idx = y * 8 + x;

                // Skip if this quad is a hole
                if holes & (1 << quad_idx) != 0 {
                    continue;
                }

                // Calculate vertex indices
                let tl = base_vertex + (y * 9 + x) as u32;
                let tr = tl + 1;
                let bl = tl + 9;
                let br = bl + 1;

                // Create two triangles per quad
                // Triangle 1
                indices.push(tl);
                indices.push(bl);
                indices.push(br);

                // Triangle 2
                indices.push(tl);
                indices.push(br);
                indices.push(tr);
            }
        }
    }

    indices
}
```

### 4. Loading and Applying Textures

```rust
use warcraft_rs::adt::{TextureInfo, AlphaMap};
use warcraft_rs::blp::Blp;

struct TerrainTextures {
    diffuse_maps: Vec<TextureId>,
    alpha_maps: Vec<TextureId>,
}

fn load_terrain_textures(adt: &Adt, adt_tex: &Adt) -> Result<TerrainTextures, Box<dyn std::error::Error>> {
    let mut diffuse_maps = Vec::new();
    let mut alpha_maps = Vec::new();

    // Load texture filenames
    let texture_files = &adt_tex.texture_filenames;

    // Load each referenced texture
    for filename in texture_files {
        let blp = Blp::from_file(filename)?;
        let texture_id = upload_texture_to_gpu(&blp);
        diffuse_maps.push(texture_id);
    }

    // Generate alpha maps for texture blending
    for chunk in &adt.chunks {
        for layer in &chunk.texture_layers[1..] { // Skip first layer (base)
            let alpha_texture = create_alpha_texture(&layer.alpha_map);
            alpha_maps.push(alpha_texture);
        }
    }

    Ok(TerrainTextures {
        diffuse_maps,
        alpha_maps,
    })
}

fn create_alpha_texture(alpha_map: &AlphaMap) -> TextureId {
    // Alpha maps can be compressed or uncompressed
    let alpha_data = match alpha_map {
        AlphaMap::Uncompressed(data) => data.clone(),
        AlphaMap::Compressed(data) => decompress_alpha_map(data),
    };

    // Upload as single-channel texture
    upload_alpha_texture(&alpha_data, 64, 64)
}
```

### 5. Implementing Texture Blending Shader

```glsl
// Vertex Shader
#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 texcoord;
layout(location = 3) in vec4 vertex_color;

layout(set = 0, binding = 0) uniform Uniforms {
    mat4 view_proj;
    vec3 sun_direction;
    float time;
};

layout(location = 0) out vec3 world_pos;
layout(location = 1) out vec3 out_normal;
layout(location = 2) out vec2 out_texcoord;
layout(location = 3) out vec4 out_vertex_color;

void main() {
    world_pos = position;
    out_normal = normal;
    out_texcoord = texcoord;
    out_vertex_color = vertex_color;

    gl_Position = view_proj * vec4(position, 1.0);
}

// Fragment Shader
#version 450

layout(location = 0) in vec3 world_pos;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 texcoord;
layout(location = 3) in vec4 vertex_color;

layout(set = 1, binding = 0) uniform sampler2D tex0;
layout(set = 1, binding = 1) uniform sampler2D tex1;
layout(set = 1, binding = 2) uniform sampler2D tex2;
layout(set = 1, binding = 3) uniform sampler2D tex3;

layout(set = 1, binding = 4) uniform sampler2D alpha1;
layout(set = 1, binding = 5) uniform sampler2D alpha2;
layout(set = 1, binding = 6) uniform sampler2D alpha3;

layout(location = 0) out vec4 out_color;

void main() {
    // Sample base texture
    vec4 color = texture(tex0, texcoord * 8.0);

    // Blend additional layers
    float a1 = texture(alpha1, texcoord).r;
    color = mix(color, texture(tex1, texcoord * 8.0), a1);

    float a2 = texture(alpha2, texcoord).r;
    color = mix(color, texture(tex2, texcoord * 8.0), a2);

    float a3 = texture(alpha3, texcoord).r;
    color = mix(color, texture(tex3, texcoord * 8.0), a3);

    // Apply vertex color (pre-baked lighting)
    color.rgb *= vertex_color.rgb * 2.0;

    // Simple diffuse lighting
    float NdotL = max(dot(normalize(normal), normalize(vec3(0.5, 1.0, 0.3))), 0.0);
    color.rgb *= 0.5 + 0.5 * NdotL;

    out_color = color;
}
```

### 6. Handling Water

```rust
use warcraft_rs::adt::{WaterChunk, LiquidType};

struct WaterMesh {
    vertices: Vec<WaterVertex>,
    indices: Vec<u32>,
    liquid_type: LiquidType,
}

#[derive(Debug, Clone)]
struct WaterVertex {
    position: [f32; 3],
    texcoord: [f32; 2],
    depth: f32,
}

fn generate_water_mesh(adt: &Adt) -> Vec<WaterMesh> {
    let mut water_meshes = Vec::new();

    for chunk in &adt.chunks {
        if let Some(water) = &chunk.water {
            let mesh = generate_chunk_water(water, chunk);
            water_meshes.push(mesh);
        }
    }

    water_meshes
}

fn generate_chunk_water(water: &WaterChunk, chunk: &TerrainChunk) -> WaterMesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Water uses 9x9 grid like terrain
    for y in 0..9 {
        for x in 0..9 {
            let idx = y * 9 + x;

            // Get water height and depth
            let height = water.height_map[idx];
            let depth = water.depth_map[idx];

            // Calculate position
            let pos_x = chunk.position.x + (x as f32 * 4.166);
            let pos_z = chunk.position.z + (y as f32 * 4.166);

            vertices.push(WaterVertex {
                position: [pos_x, height, pos_z],
                texcoord: [x as f32 / 8.0, y as f32 / 8.0],
                depth,
            });
        }
    }

    // Generate indices (same pattern as terrain)
    for y in 0..8 {
        for x in 0..8 {
            let tl = (y * 9 + x) as u32;
            let tr = tl + 1;
            let bl = tl + 9;
            let br = bl + 1;

            indices.extend_from_slice(&[tl, bl, br, tl, br, tr]);
        }
    }

    WaterMesh {
        vertices,
        indices,
        liquid_type: water.liquid_type,
    }
}
```

## Code Examples

### Complete Terrain Renderer

```rust
use warcraft_rs::adt::{Adt, Map};
use wgpu::*;

pub struct TerrainRenderer {
    device: Device,
    queue: Queue,
    pipeline: RenderPipeline,
    terrain_meshes: Vec<TerrainMesh>,
    water_meshes: Vec<WaterMesh>,
    textures: TerrainTextures,
}

impl TerrainRenderer {
    pub fn new(device: Device, queue: Queue) -> Self {
        let pipeline = create_terrain_pipeline(&device);

        Self {
            device,
            queue,
            pipeline,
            terrain_meshes: Vec::new(),
            water_meshes: Vec::new(),
            textures: TerrainTextures::default(),
        }
    }

    pub fn load_map_area(&mut self, map: &Map, center_x: i32, center_y: i32, radius: i32) -> Result<(), Box<dyn std::error::Error>> {
        // Clear existing data
        self.terrain_meshes.clear();
        self.water_meshes.clear();

        // First, discover which tiles exist using WDT
        let existing_tiles = discover_adt_tiles(&map.internal_name, map.wow_version)?;

        // Load ADTs in radius around center
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let tile_x = center_x + dx;
                let tile_y = center_y + dy;

                // Check bounds
                if tile_x < 0 || tile_x >= 64 || tile_y < 0 || tile_y >= 64 {
                    continue;
                }

                // Only try to load tiles that actually exist
                if !existing_tiles.contains(&(tile_x as usize, tile_y as usize)) {
                    continue;
                }

                // Load ADT
                let filename = format!("World/Maps/{}/{}_{:02}_{:02}.adt",
                    map.internal_name, map.internal_name, tile_x, tile_y);

                if let Ok(adt) = Adt::from_file(&filename) {
                    let mesh = self.create_terrain_mesh(&adt)?;
                    self.terrain_meshes.push(mesh);

                    // Load water if present
                    if adt.has_water() {
                        let water_meshes = generate_water_mesh(&adt);
                        self.water_meshes.extend(water_meshes);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn render(&self, encoder: &mut CommandEncoder, view: &TextureView, camera: &Camera) {
        // Render terrain
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Terrain Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(/* ... */),
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &camera.bind_group, &[]);

            for mesh in &self.terrain_meshes {
                render_pass.set_bind_group(1, &mesh.texture_bind_group, &[]);
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
            }
        }

        // Render water in separate pass
        self.render_water(encoder, view, camera);
    }
}
```

### LOD System for Large Terrains

```rust
use warcraft_rs::adt::{Adt, LodLevel};

pub struct TerrainLodSystem {
    lod_levels: Vec<LodLevel>,
    view_distance: f32,
}

impl TerrainLodSystem {
    pub fn new(view_distance: f32) -> Self {
        Self {
            lod_levels: vec![
                LodLevel { distance: 100.0, skip: 1 },
                LodLevel { distance: 200.0, skip: 2 },
                LodLevel { distance: 400.0, skip: 4 },
                LodLevel { distance: 800.0, skip: 8 },
            ],
            view_distance,
        }
    }

    pub fn generate_lod_mesh(&self, adt: &Adt, camera_pos: Vec3) -> TerrainMesh {
        let adt_center = adt.get_center();
        let distance = (adt_center - camera_pos).length();

        // Determine LOD level
        let lod_skip = self.lod_levels
            .iter()
            .find(|lod| distance < lod.distance)
            .map(|lod| lod.skip)
            .unwrap_or(16);

        // Generate mesh with reduced vertices
        self.generate_mesh_with_skip(adt, lod_skip)
    }

    fn generate_mesh_with_skip(&self, adt: &Adt, skip: usize) -> TerrainMesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Sample vertices at reduced rate
        for chunk in &adt.chunks {
            for y in (0..9).step_by(skip) {
                for x in (0..9).step_by(skip) {
                    // Add vertex
                    vertices.push(create_vertex(chunk, x, y));
                }
            }
        }

        // Generate indices for reduced mesh
        // ... index generation with proper connectivity

        TerrainMesh { vertices, indices }
    }
}
```

## Best Practices

### 1. Chunk-Based Culling

```rust
pub struct FrustumCuller {
    view_frustum: Frustum,
}

impl FrustumCuller {
    pub fn cull_chunks(&self, adt: &Adt) -> Vec<usize> {
        let mut visible_chunks = Vec::new();

        for (idx, chunk) in adt.chunks.iter().enumerate() {
            let bounds = chunk.calculate_bounds();

            if self.view_frustum.intersects_aabb(&bounds) {
                visible_chunks.push(idx);
            }
        }

        visible_chunks
    }
}
```

### 2. Texture Streaming

```rust
pub struct TextureStreamer {
    cache: HashMap<String, TextureId>,
    max_cache_size: usize,
}

impl TextureStreamer {
    pub fn get_texture(&mut self, filename: &str) -> Result<TextureId, Box<dyn std::error::Error>> {
        // Check cache first
        if let Some(&tex_id) = self.cache.get(filename) {
            return Ok(tex_id);
        }

        // Load and cache
        let blp = Blp::from_file(filename)?;
        let tex_id = upload_texture(&blp);

        self.cache.insert(filename.to_string(), tex_id);
        self.enforce_cache_limit();

        Ok(tex_id)
    }
}
```

### 3. Height Map Queries

```rust
impl Adt {
    pub fn get_height_at_position(&self, x: f32, z: f32) -> Option<f32> {
        // Convert world coordinates to chunk coordinates
        let chunk_x = (x / 33.333) as usize;
        let chunk_z = (z / 33.333) as usize;

        if chunk_x >= 16 || chunk_z >= 16 {
            return None;
        }

        let chunk = &self.chunks[chunk_z * 16 + chunk_x];

        // Get position within chunk
        let local_x = x % 33.333;
        let local_z = z % 33.333;

        // Bilinear interpolation
        let fx = local_x / 4.166;
        let fz = local_z / 4.166;

        let x0 = fx.floor() as usize;
        let z0 = fz.floor() as usize;
        let x1 = (x0 + 1).min(8);
        let z1 = (z0 + 1).min(8);

        let fx = fx.fract();
        let fz = fz.fract();

        // Get four corner heights
        let h00 = chunk.height_map[z0 * 9 + x0];
        let h10 = chunk.height_map[z0 * 9 + x1];
        let h01 = chunk.height_map[z1 * 9 + x0];
        let h11 = chunk.height_map[z1 * 9 + x1];

        // Bilinear interpolation
        let h0 = h00 * (1.0 - fx) + h10 * fx;
        let h1 = h01 * (1.0 - fx) + h11 * fx;

        Some(h0 * (1.0 - fz) + h1 * fz)
    }
}
```

## Common Issues and Solutions

### Issue: Texture Seams

**Problem**: Visible seams between ADT tiles or chunks.

**Solution**:

```rust
// Ensure proper texture coordinate wrapping
fn fix_texture_seams(vertices: &mut [TerrainVertex]) {
    // Add small offset to texture coordinates at edges
    const SEAM_OFFSET: f32 = 0.5 / 512.0; // Half pixel for 512x512 texture

    for vertex in vertices {
        if vertex.texcoord[0] == 0.0 {
            vertex.texcoord[0] += SEAM_OFFSET;
        } else if vertex.texcoord[0] == 1.0 {
            vertex.texcoord[0] -= SEAM_OFFSET;
        }

        if vertex.texcoord[1] == 0.0 {
            vertex.texcoord[1] += SEAM_OFFSET;
        } else if vertex.texcoord[1] == 1.0 {
            vertex.texcoord[1] -= SEAM_OFFSET;
        }
    }
}
```

### Issue: Z-Fighting with Water

**Problem**: Flickering where water meets terrain.

**Solution**:

```rust
// Render water with slight offset
fn render_water_with_offset(water_height: f32) -> f32 {
    water_height + 0.01 // Small bias to prevent z-fighting
}

// Or use polygon offset in render state
let render_state = RenderState {
    polygon_offset: Some(PolygonOffset {
        factor: -1.0,
        units: -1.0,
    }),
    ..Default::default()
};
```

### Issue: Performance with Many ADTs

**Problem**: Frame rate drops when rendering large areas.

**Solution**:

```rust
pub struct AdtBatcher {
    batches: HashMap<TextureSetId, BatchedMesh>,
}

impl AdtBatcher {
    pub fn batch_adts(&mut self, adts: &[Adt]) {
        self.batches.clear();

        for adt in adts {
            for chunk in &adt.chunks {
                let texture_set = chunk.get_texture_set_id();
                let batch = self.batches.entry(texture_set).or_default();
                batch.add_chunk(chunk);
            }
        }
    }
}
```

## Performance Tips

### 1. GPU Instancing for Repeated Elements

```rust
// Instance doodads and small objects
pub struct DoodadRenderer {
    instance_buffer: Buffer,
    instances: Vec<DoodadInstance>,
}

#[repr(C)]
struct DoodadInstance {
    transform: [[f32; 4]; 4],
    color_variation: [f32; 4],
}
```

### 2. Texture Atlas for Terrain

```rust
pub struct TerrainTextureAtlas {
    atlas_texture: TextureId,
    texture_coords: HashMap<String, AtlasRegion>,
}

impl TerrainTextureAtlas {
    pub fn build_from_adts(adts: &[Adt]) -> Self {
        // Collect all unique textures
        let mut textures = HashSet::new();
        for adt in adts {
            textures.extend(adt.get_texture_filenames());
        }

        // Build atlas
        // ... atlas generation code
    }
}
```

### 3. Async ADT Loading

```rust
use tokio::task;

pub async fn load_adts_async(filenames: Vec<String>) -> Vec<Result<Adt, Box<dyn std::error::Error>>> {
    let mut tasks = Vec::new();

    for filename in filenames {
        let task = task::spawn_blocking(move || {
            Adt::from_file(&filename)
        });
        tasks.push(task);
    }

    let mut results = Vec::new();
    for task in tasks {
        results.push(task.await.unwrap());
    }

    results
}
```

## Related Guides

- [📦 Working with MPQ Archives](./mpq-archives.md) - Extract ADT files from game archives
- [🖼️ Texture Loading Guide](./texture-loading.md) - Load BLP textures for terrain
- [🏛️ WMO Rendering Guide](./wmo-rendering.md) - Render buildings placed on terrain
- [🎭 Loading M2 Models](./m2-models.md) - Load doodads and objects
- [📊 LOD System Guide](./lod-system.md) - Implement level-of-detail for terrain

## References

- [ADT Format Documentation](https://wowdev.wiki/ADT) - Complete ADT file format specification
- [WoW Coordinate System](https://wowdev.wiki/Coordinates) - Understanding WoW's coordinate system
- [Terrain Rendering Techniques](https://developer.nvidia.com/gpugems/gpugems2/part-i-geometric-complexity/chapter-2-terrain-rendering-using-gpu-based-geometry) - GPU-based terrain rendering
- [Texture Splatting](https://en.wikipedia.org/wiki/Texture_splatting) - Multi-texture terrain blending
