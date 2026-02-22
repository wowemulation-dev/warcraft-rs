# 🏛️ WMO Rendering Guide

## Overview

WMO (World Map Objects) are large static structures in World of Warcraft such as
buildings, dungeons, and cities. Unlike smaller decorative objects (M2 models),
WMOs have complex interior/exterior structures, multiple groups, portals, and
advanced lighting. This guide covers loading, processing, and rendering WMO files
using `warcraft-rs`.

## Prerequisites

Before working with WMO files, ensure you have:

- Understanding of 3D graphics and scene graphs
- Knowledge of BSP trees and portal rendering
- `warcraft-rs` installed with the `wmo` feature enabled
- Graphics API experience (OpenGL/Vulkan/DirectX/WebGPU)
- Familiarity with occlusion culling techniques

## Understanding WMO Structure

### WMO Components

WMO files consist of:

- **Root file** (`.wmo`): Contains general information, materials, portals
- **Group files** (`_000.wmo`, `_001.wmo`, etc.): Individual building sections
- **Portal system**: Visibility determination between rooms
- **Doodad sets**: Furniture and decorative object placements
- **Lighting**: Pre-baked vertex lighting and light definitions

### Key Concepts

- **Groups**: Self-contained mesh sections (rooms, floors)
- **Portals**: Openings between groups for visibility culling
- **BSP Tree**: Binary space partitioning for collision detection
- **Batches**: Render batches with material assignments
- **MOCVs**: Vertex colors for pre-baked lighting
- **Liquid**: Water planes within buildings

## Step-by-Step Instructions

### 1. Loading WMO Files

```rust
use wow_wmo::{WmoRoot, WmoGroup, WmoParser, WmoGroupParser};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

struct LoadedWmo {
    root: WmoRoot,
    groups: Vec<WmoGroup>,
}

fn load_wmo(root_path: &str) -> Result<LoadedWmo, Box<dyn std::error::Error>> {
    // Load root WMO file
    let file = File::open(root_path)?;
    let mut reader = BufReader::new(file);
    let root = WmoParser::new().parse_root(&mut reader)?;

    println!("Groups: {}", root.groups.len());
    println!("Portals: {}", root.portals.len());
    println!("Materials: {}", root.materials.len());
    println!("Doodad Sets: {}", root.doodad_sets.len());

    // Load group files
    let mut groups = Vec::new();
    let base_path = root_path.trim_end_matches(".wmo");

    for i in 0..root.header.n_groups {
        let group_path = format!("{}_{:03}.wmo", base_path, i);
        if Path::new(&group_path).exists() {
            let group_file = File::open(&group_path)?;
            let mut group_reader = BufReader::new(group_file);
            let group = WmoGroupParser::new().parse_group(&mut group_reader, i)?;
            groups.push(group);
        }
    }

    Ok(LoadedWmo { root, groups })
}

// Load with LOD support
fn load_wmo_with_lod(root_path: &str) -> Result<Vec<LoadedWmo>, Box<dyn std::error::Error>> {
    let mut lods = Vec::new();

    // Try to load LOD versions
    for lod_level in 0..3 {
        let lod_path = if lod_level == 0 {
            root_path.to_string()
        } else {
            root_path.replace(".wmo", &format!("_lod{}.wmo", lod_level))
        };

        if Path::new(&lod_path).exists() {
            let wmo = load_wmo(&lod_path)?;
            lods.push(wmo);
        }
    }

    Ok(lods)
}
```

### 2. Processing WMO Groups

```rust
use wow_wmo::{WmoGroup, WmoGroupFlags, BoundingBox, WmoBatch};

#[derive(Debug, Clone)]
struct ProcessedGroup {
    vertices: Vec<GpuVertex>,
    indices: Vec<u32>,
    batches: Vec<WmoBatch>,
    bounding_box: BoundingBox,
    is_indoor: bool,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct GpuVertex {
    position: [f32; 3],
    normal: [f32; 3],
    texcoord: [f32; 2],
    vertex_color: [f32; 4],
}

fn process_wmo_group(group: &WmoGroup) -> ProcessedGroup {
    let mut vertices = Vec::with_capacity(group.vertices.len());

    // Process vertices
    for (i, vertex) in group.vertices.iter().enumerate() {
        let vertex_color = if let Some(ref colors) = group.vertex_colors {
            if i < colors.len() {
                let color = &colors[i];
                [
                    color.r as f32 / 255.0,
                    color.g as f32 / 255.0,
                    color.b as f32 / 255.0,
                    color.a as f32 / 255.0,
                ]
            } else {
                [1.0, 1.0, 1.0, 1.0]
            }
        } else {
            [1.0, 1.0, 1.0, 1.0]
        };

        let normal = if i < group.normals.len() {
            [group.normals[i].x, group.normals[i].y, group.normals[i].z]
        } else {
            [0.0, 0.0, 1.0]
        };

        let texcoord = if i < group.tex_coords.len() {
            [group.tex_coords[i].u, group.tex_coords[i].v]
        } else {
            [0.0, 0.0]
        };

        vertices.push(GpuVertex {
            position: [vertex.x, vertex.y, vertex.z],
            normal,
            texcoord,
            vertex_color,
        });
    }

    ProcessedGroup {
        vertices,
        indices: group.indices.iter().map(|&i| i as u32).collect(),
        batches: group.batches.clone(),
        bounding_box: group.header.bounding_box,
        is_indoor: group.header.flags.contains(WmoGroupFlags::INDOOR),
    }
}
```

### 3. Implementing Portal Culling

```rust
use wow_wmo::{WmoPortal, WmoPortalReference, Vec3};
use std::collections::HashSet;

struct PortalSystem {
    portals: Vec<WmoPortal>,
    relations: Vec<WmoPortalReference>,
    group_visibility: Vec<bool>,
}

impl PortalSystem {
    fn new(root: &WmoRoot) -> Self {
        Self {
            portals: root.portals.clone(),
            relations: root.portal_references.clone(),
            group_visibility: vec![false; root.header.n_groups as usize],
        }
    }

    fn update_visibility(&mut self, camera_pos: Vec3, camera_group: usize) {
        // Reset visibility
        self.group_visibility.fill(false);

        // Current group is always visible
        self.group_visibility[camera_group] = true;

        // Flood fill through portals
        let mut to_check = vec![camera_group];
        let mut checked = HashSet::new();

        while let Some(current_group) = to_check.pop() {
            if !checked.insert(current_group) {
                continue;
            }

            // Find portals connected to this group
            for relation in &self.relations {
                if relation.group_index == current_group as u16 {
                    let portal = &self.portals[relation.portal_index as usize];

                    // Check if camera can see through portal
                    if self.is_portal_visible(portal, camera_pos) {
                        // Note: WmoPortalReference doesn't have target_group
                        // This would need to be determined from portal geometry
                        // For now, mark all connected groups as visible
                        // In a real implementation, you'd determine the other group
                    }
                }
            }
        }
    }

    fn is_portal_visible(&self, portal: &WmoPortal, camera_pos: Vec3) -> bool {
        // Simple visibility check - can be enhanced with frustum culling
        let portal_center = Vec3 {
            x: portal.vertices.iter().map(|v| v.x).sum::<f32>() / portal.vertices.len() as f32,
            y: portal.vertices.iter().map(|v| v.y).sum::<f32>() / portal.vertices.len() as f32,
            z: portal.vertices.iter().map(|v| v.z).sum::<f32>() / portal.vertices.len() as f32,
        };

        // Check if camera is on the positive side of portal plane
        let to_portal = Vec3 {
            x: portal_center.x - camera_pos.x,
            y: portal_center.y - camera_pos.y,
            z: portal_center.z - camera_pos.z,
        };

        let dot = to_portal.x * portal.normal.x +
                  to_portal.y * portal.normal.y +
                  to_portal.z * portal.normal.z;

        dot > 0.0
    }
}
```

### 4. Material and Texture Setup

```rust
use wow_wmo::{WmoMaterial, WmoMaterialFlags, WmoRoot};

// Mock types for rendering (would be defined by your graphics library)
type TextureId = u32;
type BlendMode = u32;
type CullMode = u32;

struct WmoMaterialSet {
    materials: Vec<GpuMaterial>,
    textures: Vec<TextureId>,
}

struct GpuMaterial {
    diffuse_texture: TextureId,
    blend_mode: BlendMode,
    cull_mode: CullMode,
    flags: WmoMaterialFlags,
    shader_id: u32,
}

// Mock texture manager for example
struct TextureManager;
impl TextureManager {
    fn load_texture(&mut self, path: &str) -> Result<TextureId, Box<dyn std::error::Error>> {
        // Mock implementation
        Ok(0)
    }
}

fn load_wmo_materials(
    root: &WmoRoot,
    texture_manager: &mut TextureManager,
) -> Result<WmoMaterialSet, Box<dyn std::error::Error>> {
    let mut materials = Vec::new();
    let mut textures = Vec::new();

    for (i, wmo_material) in root.materials.iter().enumerate() {
        // Load diffuse texture using texture index
        let texture_path = if wmo_material.texture1 as usize < root.textures.len() {
            &root.textures[wmo_material.texture1 as usize]
        } else {
            "default.blp" // fallback
        };

        let texture_id = texture_manager.load_texture(texture_path)?;
        textures.push(texture_id);

        // Determine blend mode
        let blend_mode = if wmo_material.flags.contains(WmoMaterialFlags::UNLIT) {
            0 // Opaque
        } else if wmo_material.blend_mode == 1 {
            1 // AlphaBlend
        } else {
            0 // Opaque
        };

        // Determine cull mode
        let cull_mode = if wmo_material.flags.contains(WmoMaterialFlags::TWO_SIDED) {
            0 // None
        } else {
            1 // Back
        };

        materials.push(GpuMaterial {
            diffuse_texture: texture_id,
            blend_mode,
            cull_mode,
            flags: wmo_material.flags,
            shader_id: wmo_material.shader,
        });
    }

    Ok(WmoMaterialSet { materials, textures })
}

// Mock shader variant enum
enum ShaderVariant {
    Unlit,
    Window,
    Diffuse,
    Specular,
    Standard,
}

// Shader selection based on material properties
fn select_shader_for_material(material: &GpuMaterial) -> ShaderVariant {
    if material.flags.contains(WmoMaterialFlags::UNLIT) {
        ShaderVariant::Unlit
    } else if material.flags.contains(WmoMaterialFlags::WINDOW_LIGHT) {
        ShaderVariant::Window
    } else if material.shader_id == 1 {
        ShaderVariant::Diffuse
    } else if material.shader_id == 2 {
        ShaderVariant::Specular
    } else {
        ShaderVariant::Standard
    }
}
```

### 5. Implementing Doodad Placement

```rust
use wow_wmo::{WmoDoodadSet, WmoDoodadDef};
use std::collections::HashMap;
use std::sync::Arc;

// Mock M2 model type
struct M2Model;

struct WmoDoodadManager {
    doodad_sets: Vec<WmoDoodadSet>,
    instances: Vec<WmoDoodadDef>,
    models: HashMap<String, Arc<M2Model>>,
}

// Mock types for example
type Matrix4<T> = [[T; 4]; 4];
type Vector3<T> = [T; 3];
type Vector4<T> = [T; 4];
type Quaternion<T> = [T; 4];

struct M2ModelManager;
impl M2ModelManager {
    fn load_model(&mut self, _path: &str) -> Result<M2Model, Box<dyn std::error::Error>> {
        Ok(M2Model)
    }
}

impl WmoDoodadManager {
    fn new(root: &WmoRoot) -> Self {
        Self {
            doodad_sets: root.doodad_sets.clone(),
            instances: root.doodad_defs.clone(),
            models: HashMap::new(),
        }
    }

    fn load_doodad_set(
        &mut self,
        set_index: usize,
        model_manager: &mut M2ModelManager,
        doodad_names: &[String], // Would come from parsing MODN chunk
    ) -> Result<Vec<PlacedDoodad>, Box<dyn std::error::Error>> {
        let set = &self.doodad_sets[set_index];
        let mut placed_doodads = Vec::new();

        for i in set.start_doodad..(set.start_doodad + set.n_doodads) {
            let instance = &self.instances[i as usize];

            // Get filename from name offset (simplified)
            let filename = if instance.name_offset as usize < doodad_names.len() {
                &doodad_names[instance.name_offset as usize]
            } else {
                "unknown.m2"
            };

            // Load model if not cached
            let model = if let Some(cached) = self.models.get(filename) {
                cached.clone()
            } else {
                let model = Arc::new(model_manager.load_model(filename)?);
                self.models.insert(filename.to_string(), model.clone());
                model
            };

            // Create transform matrix
            let transform = create_doodad_transform(
                [instance.position.x, instance.position.y, instance.position.z],
                instance.orientation,
                instance.scale,
            );

            placed_doodads.push(PlacedDoodad {
                model,
                transform,
                color: [instance.color.r as f32 / 255.0, instance.color.g as f32 / 255.0,
                        instance.color.b as f32 / 255.0, instance.color.a as f32 / 255.0],
            });
        }

        Ok(placed_doodads)
    }
}

fn create_doodad_transform(
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
    scale: f32,
) -> Matrix4<f32> {
    // Simplified transform creation - in a real implementation
    // you'd use a proper math library like nalgebra or glam
    let mut transform = [[0.0f32; 4]; 4];

    // Identity matrix with scale
    transform[0][0] = scale;
    transform[1][1] = scale;
    transform[2][2] = scale;
    transform[3][3] = 1.0;

    // Set translation
    transform[3][0] = position[0];
    transform[3][1] = position[1];
    transform[3][2] = position[2];

    // Note: rotation quaternion conversion omitted for brevity
    transform
}

struct PlacedDoodad {
    model: Arc<M2Model>,
    transform: Matrix4<f32>,
    color: Vector4<f32>,
}
```

### 6. Rendering Pipeline

```rust
// Mock GPU types for example (would be from wgpu/vulkan/etc)
struct Device;
struct Queue;
struct RenderPipeline;
struct Buffer;

pub struct WmoRenderer {
    group_buffers: Vec<GroupGpuData>,
    material_set: WmoMaterialSet,
    portal_system: PortalSystem,
}

struct GroupGpuData {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    batches: Vec<WmoBatch>,
}

impl WmoRenderer {
    pub fn new(wmo: &LoadedWmo) -> Result<Self, Box<dyn std::error::Error>> {
        // Process groups
        let mut group_buffers = Vec::new();
        for group in &wmo.groups {
            let processed = process_wmo_group(group);

            // In a real implementation, you'd create GPU buffers here
            group_buffers.push(GroupGpuData {
                vertex_buffer: Buffer, // Mock buffer
                index_buffer: Buffer,  // Mock buffer
                batches: processed.batches,
            });
        }

        // Load materials
        let mut texture_manager = TextureManager;
        let material_set = load_wmo_materials(&wmo.root, &mut texture_manager)?;

        // Initialize portal system
        let portal_system = PortalSystem::new(&wmo.root);

        Ok(Self {
            group_buffers,
            material_set,
            portal_system,
        })
    }

    pub fn render(
        &mut self,
        wmo: &LoadedWmo,
        camera_pos: Vec3,
    ) {
        // Update portal visibility
        let camera_group = self.find_camera_group(camera_pos, wmo);
        self.portal_system.update_visibility(camera_pos, camera_group);

        // Render visible groups
        println!("Rendering WMO with {} groups", wmo.groups.len());

        for (group_idx, group) in wmo.groups.iter().enumerate() {
            if !self.portal_system.group_visibility[group_idx] {
                continue;
            }

            println!("Rendering group {}", group_idx);
            // In a real renderer, you'd submit draw calls here
        }
    }

    fn render_group(
        &self,
        render_pass: &mut RenderPass,
        group_idx: usize,
        pipeline: &RenderPipeline,
        camera: &Camera,
    ) {
        let group_data = &self.group_buffers[group_idx];

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, &camera.bind_group, &[]);
        render_pass.set_vertex_buffer(0, group_data.vertex_buffer.slice(..));
        render_pass.set_index_buffer(group_data.index_buffer.slice(..), IndexFormat::Uint32);

        // Render each batch with its material
        for batch in &group_data.batches {
            let material = &self.material_set.materials[batch.material_id as usize];

            // Set material bind group
            render_pass.set_bind_group(1, &material.bind_group, &[]);

            // Apply render state based on material
            self.apply_material_state(render_pass, material);

            // Draw
            render_pass.draw_indexed(
                batch.start_index..(batch.start_index + batch.index_count),
                0,
                0..1,
            );
        }
    }

    fn find_camera_group(&self, camera_pos: Vec3, wmo: &LoadedWmo) -> usize {
        // Find which group contains the camera
        for (idx, group) in wmo.groups.iter().enumerate() {
            let bbox = &group.header.bounding_box;
            if camera_pos.x >= bbox.min.x && camera_pos.x <= bbox.max.x &&
               camera_pos.y >= bbox.min.y && camera_pos.y <= bbox.max.y &&
               camera_pos.z >= bbox.min.z && camera_pos.z <= bbox.max.z {
                return idx;
            }
        }

        // Default to first group
        0
    }
}
```

## Code Examples

### Complete WMO Scene Manager

```rust
use warcraft_rs::wmo::*;
use std::sync::Arc;

pub struct WmoSceneManager {
    loaded_wmos: HashMap<String, Arc<LoadedWmo>>,
    instances: Vec<WmoInstance>,
    renderer: WmoRenderer,
    doodad_renderer: M2Renderer,
}

pub struct LoadedWmo {
    wmo: Wmo,
    gpu_data: WmoGpuData,
    doodad_sets: Vec<Vec<PlacedDoodad>>,
    collision_mesh: CollisionMesh,
}

pub struct WmoInstance {
    wmo: Arc<LoadedWmo>,
    transform: Matrix4<f32>,
    doodad_set: usize,
    tint_color: Vector4<f32>,
}

impl WmoSceneManager {
    pub fn new(device: Device, queue: Queue) -> Self {
        Self {
            loaded_wmos: HashMap::new(),
            instances: Vec::new(),
            renderer: WmoRenderer::new(device.clone(), queue.clone()),
            doodad_renderer: M2Renderer::new(device, queue),
        }
    }

    pub async fn load_wmo(&mut self, path: &str) -> Result<Arc<LoadedWmo>, Box<dyn std::error::Error>> {
        if let Some(cached) = self.loaded_wmos.get(path) {
            return Ok(cached.clone());
        }

        // Load WMO files
        let wmo = load_wmo(path)?;

        // Create GPU resources
        let gpu_data = self.renderer.create_gpu_data(&wmo)?;

        // Load doodad sets
        let mut doodad_sets = Vec::new();
        for i in 0..wmo.root.doodad_sets.len() {
            let doodads = load_doodad_set(&wmo, i)?;
            doodad_sets.push(doodads);
        }

        // Generate collision mesh
        let collision_mesh = generate_collision_mesh(&wmo)?;

        let loaded = Arc::new(LoadedWmo {
            wmo,
            gpu_data,
            doodad_sets,
            collision_mesh,
        });

        self.loaded_wmos.insert(path.to_string(), loaded.clone());
        Ok(loaded)
    }

    pub fn add_instance(&mut self, wmo: Arc<LoadedWmo>, transform: Matrix4<f32>, doodad_set: usize) {
        self.instances.push(WmoInstance {
            wmo,
            transform,
            doodad_set,
            tint_color: Vector4::new(1.0, 1.0, 1.0, 1.0),
        });
    }

    pub fn render_all(
        &mut self,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        camera: &Camera,
    ) {
        // Group instances by WMO for efficient rendering
        let mut instance_groups: HashMap<Arc<LoadedWmo>, Vec<&WmoInstance>> = HashMap::new();

        for instance in &self.instances {
            instance_groups
                .entry(instance.wmo.clone())
                .or_insert_with(Vec::new)
                .push(instance);
        }

        // Render each WMO type
        for (wmo, instances) in instance_groups {
            for instance in instances {
                // Set instance transform
                self.renderer.set_instance_transform(&instance.transform);

                // Render WMO
                self.renderer.render(encoder, view, camera, &wmo.wmo);

                // Render doodads
                if instance.doodad_set < wmo.doodad_sets.len() {
                    for doodad in &wmo.doodad_sets[instance.doodad_set] {
                        let world_transform = instance.transform * doodad.transform;
                        self.doodad_renderer.render_model(
                            encoder,
                            view,
                            &doodad.model,
                            &world_transform,
                            camera,
                        );
                    }
                }
            }
        }
    }
}
```

### WMO Collision Detection

```rust
use ncollide3d::shape::TriMesh;
use ncollide3d::query::{Ray, RayCast};

pub struct WmoCollisionSystem {
    meshes: HashMap<String, CollisionMesh>,
}

pub struct CollisionMesh {
    trimesh: TriMesh<f32>,
    groups: Vec<GroupCollisionData>,
}

struct GroupCollisionData {
    trimesh: TriMesh<f32>,
    is_indoor: bool,
    material_flags: Vec<MaterialFlags>,
}

impl WmoCollisionSystem {
    pub fn build_collision_mesh(wmo: &Wmo) -> CollisionMesh {
        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        let mut groups = Vec::new();

        for (group_idx, group) in wmo.groups.iter().enumerate() {
            let vertex_offset = all_vertices.len();

            // Collect collision vertices
            let mut group_vertices = Vec::new();
            let mut group_indices = Vec::new();

            for vertex in &group.vertices {
                let point = Point3::new(vertex.position.x, vertex.position.y, vertex.position.z);
                all_vertices.push(point);
                group_vertices.push(point);
            }

            // Collect collision triangles
            for batch in &group.batches {
                let material = &wmo.root.materials[batch.material_id as usize];

                // Skip non-collidable materials
                if material.flags.contains(MaterialFlags::NO_COLLISION) {
                    continue;
                }

                for i in (batch.start_index..(batch.start_index + batch.index_count)).step_by(3) {
                    let i0 = group.indices[i as usize] as usize;
                    let i1 = group.indices[(i + 1) as usize] as usize;
                    let i2 = group.indices[(i + 2) as usize] as usize;

                    all_indices.push([
                        vertex_offset + i0,
                        vertex_offset + i1,
                        vertex_offset + i2,
                    ]);

                    group_indices.push([i0, i1, i2]);
                }
            }

            // Create group collision mesh
            let group_mesh = TriMesh::new(
                group_vertices,
                group_indices,
                None,
            );

            groups.push(GroupCollisionData {
                trimesh: group_mesh,
                is_indoor: group.flags.contains(GroupFlags::INDOOR),
                material_flags: Vec::new(), // Populate with actual material flags
            });
        }

        // Create overall collision mesh
        let trimesh = TriMesh::new(all_vertices, all_indices, None);

        CollisionMesh { trimesh, groups }
    }

    pub fn raycast(
        &self,
        wmo_instance: &WmoInstance,
        ray: &Ray<f32>,
    ) -> Option<RaycastHit> {
        let mesh = &wmo_instance.wmo.collision_mesh;

        // Transform ray to WMO local space
        let inv_transform = wmo_instance.transform.try_inverse()?;
        let local_ray = Ray::new(
            inv_transform.transform_point(&ray.origin),
            inv_transform.transform_vector(&ray.dir),
        );

        // Perform raycast
        if let Some(toi) = mesh.trimesh.toi_with_ray(&Isometry3::identity(), &local_ray, f32::MAX, true) {
            // Transform hit back to world space
            let local_point = local_ray.point_at(toi);
            let world_point = wmo_instance.transform.transform_point(&local_point);

            Some(RaycastHit {
                point: world_point,
                distance: toi,
                normal: Vector3::y(), // Calculate actual normal
                material_flags: MaterialFlags::empty(),
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct RaycastHit {
    pub point: Point3<f32>,
    pub distance: f32,
    pub normal: Vector3<f32>,
    pub material_flags: MaterialFlags,
}
```

### WMO Shader Implementation

```wgsl
// wmo_shader.wgsl

struct Camera {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    position: vec3<f32>,
    time: f32,
}

struct Instance {
    transform: mat4x4<f32>,
    tint_color: vec4<f32>,
}

struct Material {
    flags: u32,
    blend_mode: u32,
    shader_id: u32,
    _padding: u32,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> instance: Instance;

@group(2) @binding(0)
var<uniform> material: Material;

@group(2) @binding(1)
var diffuse_texture: texture_2d<f32>;

@group(2) @binding(2)
var texture_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) texcoord: vec2<f32>,
    @location(3) vertex_color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) texcoord: vec2<f32>,
    @location(3) vertex_color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Transform position
    let world_pos = instance.transform * vec4<f32>(input.position, 1.0);
    out.world_position = world_pos.xyz;
    out.clip_position = camera.view_proj * world_pos;

    // Transform normal
    let normal_matrix = mat3x3<f32>(
        instance.transform[0].xyz,
        instance.transform[1].xyz,
        instance.transform[2].xyz,
    );
    out.normal = normalize(normal_matrix * input.normal);

    out.texcoord = input.texcoord;
    out.vertex_color = input.vertex_color * instance.tint_color;

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture
    var color = textureSample(diffuse_texture, texture_sampler, input.texcoord);

    // Apply vertex color (pre-baked lighting)
    color = color * input.vertex_color * 2.0;

    // Check material flags
    let is_unlit = (material.flags & 0x1u) != 0u;
    let is_window = (material.flags & 0x8u) != 0u;

    if (!is_unlit) {
        // Simple ambient + directional light
        let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
        let n_dot_l = max(dot(input.normal, light_dir), 0.0);
        let ambient = vec3<f32>(0.4, 0.4, 0.5);

        color.xyz = color.xyz * (ambient + n_dot_l * 0.6);
    }

    // Window material effect
    if (is_window) {
        // Add some transparency and reflectivity
        color.w = color.w * 0.8;
    }

    return color;
}
```

## Best Practices

### 1. LOD Selection

```rust
pub struct WmoLodSelector {
    distance_thresholds: Vec<f32>,
}

impl WmoLodSelector {
    pub fn new() -> Self {
        Self {
            distance_thresholds: vec![100.0, 300.0, 600.0],
        }
    }

    pub fn select_lod(
        &self,
        wmo_lods: &[Wmo],
        camera_pos: Point3<f32>,
        wmo_center: Point3<f32>,
    ) -> usize {
        let distance = (camera_pos - wmo_center).norm();

        for (i, threshold) in self.distance_thresholds.iter().enumerate() {
            if distance < *threshold && i < wmo_lods.len() {
                return i;
            }
        }

        // Return lowest detail LOD
        wmo_lods.len() - 1
    }

    pub fn select_group_detail(
        &self,
        group: &WmoGroup,
        camera_pos: Point3<f32>,
    ) -> RenderDetail {
        let distance = group.bounding_box.distance_to_point(&camera_pos);

        if distance < 50.0 {
            RenderDetail::Full
        } else if distance < 200.0 {
            RenderDetail::Simplified
        } else {
            RenderDetail::BoundingBox
        }
    }
}

enum RenderDetail {
    Full,
    Simplified,
    BoundingBox,
}
```

### 2. Batching and Instancing

```rust
pub struct WmoBatcher {
    instance_data: HashMap<String, Vec<InstanceData>>,
    instance_buffers: HashMap<String, Buffer>,
}

impl WmoBatcher {
    pub fn add_instance(&mut self, wmo_path: &str, transform: Matrix4<f32>, color: Vector4<f32>) {
        self.instance_data
            .entry(wmo_path.to_string())
            .or_insert_with(Vec::new)
            .push(InstanceData {
                transform: transform.into(),
                color: color.into(),
            });
    }

    pub fn update_buffers(&mut self, device: &Device, queue: &Queue) {
        for (path, instances) in &self.instance_data {
            let buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("WMO Instance Buffer"),
                contents: bytemuck::cast_slice(instances),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });

            self.instance_buffers.insert(path.clone(), buffer);
        }
    }

    pub fn render_batched(
        &self,
        render_pass: &mut RenderPass,
        wmo_path: &str,
        base_vertices: u32,
    ) {
        if let Some(instance_buffer) = self.instance_buffers.get(wmo_path) {
            let instance_count = self.instance_data[wmo_path].len() as u32;

            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
            render_pass.draw(0..base_vertices, 0..instance_count);
        }
    }
}
```

### 3. Occlusion Culling

```rust
pub struct OcclusionCuller {
    query_pool: QuerySet,
    visibility_buffer: Buffer,
}

impl OcclusionCuller {
    pub fn test_group_visibility(
        &mut self,
        encoder: &mut CommandEncoder,
        group_bounds: &[BoundingBox],
        camera: &Camera,
    ) -> Vec<bool> {
        // Render bounding boxes with occlusion queries
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Occlusion Test Pass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(/* depth only */),
        });

        for (i, bounds) in group_bounds.iter().enumerate() {
            render_pass.begin_occlusion_query(i as u32);
            self.render_bounding_box(&mut render_pass, bounds, camera);
            render_pass.end_occlusion_query();
        }

        drop(render_pass);

        // Read back results
        self.read_visibility_results(encoder, group_bounds.len())
    }

    fn render_bounding_box(
        &self,
        render_pass: &mut RenderPass,
        bounds: &BoundingBox,
        camera: &Camera,
    ) {
        // Render conservative bounding box
        // Implementation details...
    }
}
```

## Common Issues and Solutions

### Issue: Z-Fighting Between Groups

**Problem**: Flickering at group boundaries.

**Solution**:

```rust
// Add small offset between groups
fn adjust_group_boundaries(groups: &mut [WmoGroup]) {
    const EPSILON: f32 = 0.001;

    for (i, group) in groups.iter_mut().enumerate() {
        // Slightly shrink each group's geometry
        for vertex in &mut group.vertices {
            let to_center = group.bounding_box.center() - vertex.position;
            vertex.position += to_center.normalize() * EPSILON;
        }
    }
}
```

### Issue: Portal Visibility Errors

**Problem**: Groups appearing/disappearing incorrectly.

**Solution**:

```rust
// Enhanced portal visibility with margin
fn is_portal_visible_conservative(
    portal: &Portal,
    camera_pos: Point3<f32>,
    camera_frustum: &Frustum,
) -> bool {
    // Add margin to portal bounds
    let expanded_bounds = portal.bounding_box.expanded(1.0);

    // Check frustum intersection
    if !camera_frustum.intersects_aabb(&expanded_bounds) {
        return false;
    }

    // Check if camera can see through portal
    let portal_normal = calculate_portal_normal(&portal.vertices);
    let to_portal = portal.center() - camera_pos;

    // Add angle threshold for conservative culling
    let dot = to_portal.normalize().dot(&portal_normal);
    dot > -0.1 // Slightly visible from behind
}
```

### Issue: Incorrect Indoor/Outdoor Lighting

**Problem**: Indoor areas too bright or outdoor areas too dark.

**Solution**:

```rust
// Separate lighting for indoor/outdoor
struct LightingSettings {
    outdoor_ambient: Vector3<f32>,
    outdoor_diffuse: Vector3<f32>,
    indoor_ambient: Vector3<f32>,
    indoor_diffuse: Vector3<f32>,
}

fn apply_group_lighting(
    group: &WmoGroup,
    settings: &LightingSettings,
) -> GroupLighting {
    if group.flags.contains(GroupFlags::OUTDOOR) {
        GroupLighting {
            ambient: settings.outdoor_ambient,
            diffuse: settings.outdoor_diffuse,
            use_vertex_color: true,
        }
    } else {
        GroupLighting {
            ambient: settings.indoor_ambient,
            diffuse: settings.indoor_diffuse,
            use_vertex_color: true,
        }
    }
}
```

## Performance Tips

### 1. Hierarchical Culling

```rust
pub struct HierarchicalCuller {
    octree: Octree<usize>,
}

impl HierarchicalCuller {
    pub fn build(wmo: &Wmo) -> Self {
        let mut octree = Octree::new(wmo.root.bounding_box.clone());

        for (i, group) in wmo.groups.iter().enumerate() {
            octree.insert(i, &group.bounding_box);
        }

        Self { octree }
    }

    pub fn get_visible_groups(&self, frustum: &Frustum) -> Vec<usize> {
        self.octree.query_frustum(frustum)
    }
}
```

### 2. Texture Atlas for WMOs

```rust
pub struct WmoTextureAtlas {
    atlas: TextureAtlas,
    material_mappings: HashMap<u32, AtlasRegion>,
}

impl WmoTextureAtlas {
    pub fn build_for_wmo(
        wmo: &Wmo,
        texture_manager: &TextureManager,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut atlas = TextureAtlas::new(4096);
        let mut mappings = HashMap::new();

        // Collect unique textures
        let mut textures = HashSet::new();
        for material in &wmo.root.materials {
            textures.insert(&material.texture);
        }

        // Pack into atlas
        for (i, texture_path) in textures.iter().enumerate() {
            let texture = texture_manager.load_texture(texture_path)?;
            let region = atlas.add_texture(texture_path, &texture)?;
            mappings.insert(i as u32, region);
        }

        Ok(Self {
            atlas,
            material_mappings: mappings,
        })
    }
}
```

### 3. Async WMO Loading

```rust
pub async fn load_wmo_async(
    path: String,
) -> Result<Wmo, Box<dyn std::error::Error>> {
    // Load root file
    let root_data = tokio::fs::read(&path).await?;
    let root = tokio::task::spawn_blocking(move || {
        WmoRoot::from_bytes(&root_data)
    }).await??;

    // Load groups in parallel
    let base_path = path.trim_end_matches(".wmo");
    let mut group_futures = Vec::new();

    for i in 0..root.group_count {
        let group_path = format!("{}_{:03}.wmo", base_path, i);
        group_futures.push(tokio::fs::read(group_path));
    }

    let group_data = futures::future::join_all(group_futures).await;

    // Parse groups
    let mut groups = Vec::new();
    for data in group_data {
        if let Ok(data) = data {
            let group = WmoGroup::from_bytes(&data)?;
            groups.push(group);
        }
    }

    Ok(Wmo { root, groups })
}
```

## Related Guides

- [📦 Working with MPQ Archives](./mpq-archives.md) - Extract WMO files from archives
- [🖼️ Texture Loading Guide](./texture-loading.md) - Load BLP textures for WMOs
- [🎭 Loading M2 Models](./m2-models.md) - Load doodads placed in WMOs
- [🌍 Rendering ADT Terrain](./adt-rendering.md) - Integrate WMOs with terrain
- [📊 LOD System Guide](./lod-system.md) - Implement LOD for large structures

## References

- [WMO Format Documentation](https://wowdev.wiki/WMO) - Complete WMO format specification
- [Portal Rendering](https://en.wikipedia.org/wiki/Portal_rendering) - Understanding portal-based visibility
- [BSP Trees](https://en.wikipedia.org/wiki/Binary_space_partitioning) - Binary space partitioning for WMOs
- [WoW Model Viewer](https://github.com/Marlamin/WoWModelViewer) - Reference WMO implementation
