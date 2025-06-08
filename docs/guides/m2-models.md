# 🎭 Loading M2 Models

## Overview

M2 (Model 2) files are World of Warcraft's primary 3D model format, used for
characters, creatures, items, and doodads. This guide covers loading, parsing, and
rendering M2 models using `warcraft-rs`, including handling associated files like
skins, animations, and physics data.

## Prerequisites

Before working with M2 models, ensure you have:

- Understanding of 3D model rendering (vertices, bones, animations)
- Basic knowledge of skeletal animation systems
- `warcraft-rs` installed with the `m2` feature enabled
- Graphics API knowledge (OpenGL/Vulkan/DirectX/WebGPU)
- Familiarity with texture mapping and shaders

## Understanding M2 Models

### M2 File Structure

M2 models consist of multiple files:

- **`.m2`**: Main model file containing geometry, bones, animations
- **`.skin`**: Mesh data and render batches
- **`.anim`**: External animation sequences
- **`.bone`**: Bone data (newer versions)
- **`.phys`**: Physics simulation data
- **`.skel`**: Shared skeleton data

### Key Components

- **Vertices**: Position, normal, texture coords, bone weights
- **Bones**: Hierarchical skeleton for animation
- **Animations**: Keyframe sequences for movement
- **Textures**: Material and texture references
- **Render Flags**: Blending modes, culling, transparency
- **Attachments**: Points for weapons, effects, etc.
- **Particles**: Particle emitter definitions
- **Ribbons**: Trail effects (capes, weapon trails)

## Step-by-Step Instructions

### 1. Loading M2 Model Files

```rust
use warcraft_rs::m2::{M2Model, M2Skin, M2Version};
use std::path::Path;

fn load_m2_model(model_path: &str) -> Result<(M2Model, Vec<M2Skin>), Box<dyn std::error::Error>> {
    // Load main M2 file
    let m2 = M2Model::from_file(model_path)?;

    println!("Loaded M2 model: {}", m2.name);
    println!("Version: {:?}", m2.version);
    println!("Vertices: {}", m2.vertex_count);
    println!("Bones: {}", m2.bones.len());
    println!("Animations: {}", m2.animations.len());

    // Load associated skin files
    let mut skins = Vec::new();
    for i in 0..m2.skin_profile_count {
        let skin_path = model_path.replace(".m2", &format!("{:02}.skin", i));
        if Path::new(&skin_path).exists() {
            let skin = M2Skin::from_file(&skin_path)?;
            skins.push(skin);
        }
    }

    Ok((m2, skins))
}

// For models with external animations
fn load_external_animations(m2: &M2Model, model_path: &str) -> Result<Vec<M2Animation>, Box<dyn std::error::Error>> {
    let mut animations = Vec::new();

    for anim_ref in &m2.animation_lookup {
        if anim_ref.is_external() {
            let anim_path = model_path.replace(".m2", &format!("{:04}.anim", anim_ref.animation_id));
            if Path::new(&anim_path).exists() {
                let anim = M2Animation::from_file(&anim_path)?;
                animations.push(anim);
            }
        }
    }

    Ok(animations)
}
```

### 2. Processing Model Vertices

```rust
use warcraft_rs::m2::{M2Vertex, M2Model};

#[derive(Debug, Clone)]
struct ProcessedVertex {
    position: [f32; 3],
    normal: [f32; 3],
    tex_coords: [[f32; 2]; 2],
    bone_indices: [u8; 4],
    bone_weights: [u8; 4],
}

fn process_model_vertices(m2: &M2Model) -> Vec<ProcessedVertex> {
    let mut processed = Vec::with_capacity(m2.vertices.len());

    for vertex in &m2.vertices {
        // Transform vertex position by global model bounds
        let position = transform_vertex_position(vertex.position, &m2.bounding_box);

        // Normalize the normal vector
        let normal = normalize_vec3(vertex.normal);

        processed.push(ProcessedVertex {
            position: [position.x, position.y, position.z],
            normal: [normal.x, normal.y, normal.z],
            tex_coords: [
                [vertex.tex_coords[0].x, vertex.tex_coords[0].y],
                [vertex.tex_coords[1].x, vertex.tex_coords[1].y],
            ],
            bone_indices: vertex.bone_indices,
            bone_weights: vertex.bone_weights,
        });
    }

    processed
}

fn normalize_bone_weights(weights: [u8; 4]) -> [f32; 4] {
    let sum: u32 = weights.iter().map(|&w| w as u32).sum();
    if sum == 0 {
        return [1.0, 0.0, 0.0, 0.0];
    }

    let factor = 1.0 / sum as f32;
    [
        weights[0] as f32 * factor,
        weights[1] as f32 * factor,
        weights[2] as f32 * factor,
        weights[3] as f32 * factor,
    ]
}
```

### 3. Building Render Meshes from Skins

```rust
use warcraft_rs::m2::{M2Skin, M2Model, RenderFlag};

struct ModelMesh {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    submeshes: Vec<Submesh>,
}

struct Submesh {
    index_start: u32,
    index_count: u32,
    material_id: u16,
    render_flags: RenderFlag,
}

fn build_render_mesh(m2: &M2Model, skin: &M2Skin, device: &Device) -> ModelMesh {
    // Reorder vertices according to skin
    let mut skin_vertices = Vec::with_capacity(skin.vertices.len());
    for &vertex_idx in &skin.vertices {
        skin_vertices.push(m2.vertices[vertex_idx as usize].clone());
    }

    // Process vertices
    let processed = process_model_vertices_from_slice(&skin_vertices);

    // Create vertex buffer
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("M2 Vertex Buffer"),
        contents: bytemuck::cast_slice(&processed),
        usage: BufferUsages::VERTEX,
    });

    // Build submeshes from skin
    let mut submeshes = Vec::new();
    let mut all_indices = Vec::new();

    for submesh in &skin.submeshes {
        let material = &m2.materials[submesh.material_id as usize];

        submeshes.push(Submesh {
            index_start: all_indices.len() as u32,
            index_count: submesh.index_count as u32,
            material_id: submesh.material_id,
            render_flags: material.render_flags,
        });

        // Add indices for this submesh
        for i in 0..submesh.index_count {
            all_indices.push(skin.indices[(submesh.index_start + i) as usize]);
        }
    }

    // Create index buffer
    let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("M2 Index Buffer"),
        contents: bytemuck::cast_slice(&all_indices),
        usage: BufferUsages::INDEX,
    });

    ModelMesh {
        vertex_buffer,
        index_buffer,
        submeshes,
    }
}
```

### 4. Setting Up Skeletal Animation

```rust
use warcraft_rs::m2::{M2Bone, M2Animation, AnimationBlock};
use nalgebra::{Matrix4, Vector3, Quaternion};

struct BoneTransform {
    translation: Vector3<f32>,
    rotation: Quaternion<f32>,
    scale: Vector3<f32>,
}

struct AnimationState {
    animation_id: u16,
    current_time: u32,
    looping: bool,
    bone_matrices: Vec<Matrix4<f32>>,
}

impl AnimationState {
    fn new(bone_count: usize) -> Self {
        Self {
            animation_id: 0,
            current_time: 0,
            looping: true,
            bone_matrices: vec![Matrix4::identity(); bone_count],
        }
    }

    fn update(&mut self, m2: &M2Model, animation: &M2Animation, delta_ms: u32) {
        // Update animation time
        self.current_time += delta_ms;
        if self.current_time >= animation.duration {
            if self.looping {
                self.current_time %= animation.duration;
            } else {
                self.current_time = animation.duration - 1;
            }
        }

        // Calculate bone transforms
        self.calculate_bone_matrices(m2, animation);
    }

    fn calculate_bone_matrices(&mut self, m2: &M2Model, animation: &M2Animation) {
        // First pass: calculate local transforms
        let mut local_transforms = Vec::with_capacity(m2.bones.len());

        for (bone_idx, bone) in m2.bones.iter().enumerate() {
            let transform = self.interpolate_bone_transform(bone, animation, self.current_time);
            local_transforms.push(transform);
        }

        // Second pass: calculate world transforms
        for (bone_idx, bone) in m2.bones.iter().enumerate() {
            let local_matrix = transform_to_matrix(&local_transforms[bone_idx]);

            if bone.parent_bone == -1 {
                // Root bone
                self.bone_matrices[bone_idx] = local_matrix;
            } else {
                // Child bone - multiply by parent transform
                let parent_matrix = self.bone_matrices[bone.parent_bone as usize];
                self.bone_matrices[bone_idx] = parent_matrix * local_matrix;
            }
        }
    }

    fn interpolate_bone_transform(&self, bone: &M2Bone, animation: &M2Animation, time: u32) -> BoneTransform {
        // Get animation tracks for this bone
        let translation = interpolate_vec3_track(&bone.translation, animation.sequence_id, time);
        let rotation = interpolate_quat_track(&bone.rotation, animation.sequence_id, time);
        let scale = interpolate_vec3_track(&bone.scale, animation.sequence_id, time);

        BoneTransform {
            translation,
            rotation,
            scale,
        }
    }
}

fn transform_to_matrix(transform: &BoneTransform) -> Matrix4<f32> {
    let translation = Matrix4::new_translation(&transform.translation);
    let rotation = transform.rotation.to_homogeneous();
    let scale = Matrix4::new_nonuniform_scaling(&transform.scale);

    translation * rotation * scale
}
```

### 5. Loading and Applying Textures

```rust
use warcraft_rs::m2::{M2Texture, TextureType};
use warcraft_rs::blp::Blp;

struct ModelTextures {
    textures: Vec<TextureHandle>,
    texture_transforms: Vec<TextureTransform>,
}

#[derive(Clone)]
struct TextureTransform {
    enabled: bool,
    translation: AnimationBlock<Vector2<f32>>,
    rotation: AnimationBlock<Quaternion<f32>>,
    scale: AnimationBlock<Vector2<f32>>,
}

async fn load_model_textures(m2: &M2Model, mpq_archive: &Archive) -> Result<ModelTextures, Box<dyn std::error::Error>> {
    let mut textures = Vec::new();
    let mut texture_transforms = Vec::new();

    for texture in &m2.textures {
        // Load texture file
        let texture_data = match texture.texture_type {
            TextureType::Filename => {
                // Extract from MPQ or load from file
                mpq_archive.extract(&texture.filename)?
            }
            TextureType::Hardcoded => {
                // Handle hardcoded textures (skin, hair, etc.)
                load_hardcoded_texture(texture.hardcoded_id)?
            }
        };

        // Parse BLP texture
        let blp = Blp::from_bytes(&texture_data)?;
        let texture_handle = upload_texture_to_gpu(&blp).await?;
        textures.push(texture_handle);

        // Store texture animation data
        texture_transforms.push(TextureTransform {
            enabled: texture.flags.contains(TextureFlags::ANIMATED),
            translation: texture.translation.clone(),
            rotation: texture.rotation.clone(),
            scale: texture.scale.clone(),
        });
    }

    Ok(ModelTextures {
        textures,
        texture_transforms,
    })
}
```

### 6. Implementing Model Renderer

```rust
pub struct M2Renderer {
    device: Device,
    queue: Queue,
    pipeline: RenderPipeline,
    bone_buffer: Buffer,
    texture_bind_groups: Vec<BindGroup>,
}

impl M2Renderer {
    pub fn new(device: Device, queue: Queue) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("M2 Shader"),
            source: ShaderSource::Wgsl(include_str!("m2_shader.wgsl")),
        });

        let pipeline = create_m2_pipeline(&device, &shader);

        // Create bone matrix buffer (max 256 bones)
        let bone_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Bone Matrices"),
            size: 256 * 64, // 256 4x4 matrices
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            device,
            queue,
            pipeline,
            bone_buffer,
            texture_bind_groups: Vec::new(),
        }
    }

    pub fn render_model(
        &self,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        model: &LoadedM2Model,
        animation_state: &AnimationState,
        camera: &Camera,
    ) {
        // Update bone matrices
        self.queue.write_buffer(
            &self.bone_buffer,
            0,
            bytemuck::cast_slice(&animation_state.bone_matrices),
        );

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("M2 Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(/* depth attachment */),
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &camera.bind_group, &[]);
        render_pass.set_bind_group(1, &self.bone_bind_group, &[]);

        // Render each submesh
        for (mesh_idx, mesh) in model.meshes.iter().enumerate() {
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), IndexFormat::Uint16);

            for submesh in &mesh.submeshes {
                // Set texture for this submesh
                let texture_idx = model.texture_lookup[submesh.material_id as usize];
                render_pass.set_bind_group(2, &self.texture_bind_groups[texture_idx], &[]);

                // Apply render flags
                self.apply_render_flags(&mut render_pass, submesh.render_flags);

                // Draw
                render_pass.draw_indexed(
                    submesh.index_start..(submesh.index_start + submesh.index_count),
                    0,
                    0..1,
                );
            }
        }
    }

    fn apply_render_flags(&self, render_pass: &mut RenderPass, flags: RenderFlag) {
        // Handle blending modes, culling, etc.
        // This would typically be done through pipeline variants
    }
}
```

## Code Examples

### Complete M2 Model Loader

```rust
use warcraft_rs::m2::*;
use std::collections::HashMap;

pub struct M2ModelManager {
    models: HashMap<String, LoadedM2Model>,
    device: Device,
    queue: Queue,
}

pub struct LoadedM2Model {
    m2: M2Model,
    skins: Vec<M2Skin>,
    meshes: Vec<ModelMesh>,
    textures: ModelTextures,
    animations: HashMap<u16, M2Animation>,
    current_animation: AnimationState,
}

impl M2ModelManager {
    pub fn new(device: Device, queue: Queue) -> Self {
        Self {
            models: HashMap::new(),
            device,
            queue,
        }
    }

    pub async fn load_model(&mut self, path: &str) -> Result<&LoadedM2Model, Box<dyn std::error::Error>> {
        if self.models.contains_key(path) {
            return Ok(&self.models[path]);
        }

        // Load M2 and skins
        let (m2, skins) = load_m2_model(path)?;

        // Build render meshes
        let mut meshes = Vec::new();
        for skin in &skins {
            let mesh = build_render_mesh(&m2, skin, &self.device);
            meshes.push(mesh);
        }

        // Load textures
        let textures = load_model_textures(&m2, &self.archive).await?;

        // Load animations
        let mut animations = HashMap::new();
        for (idx, anim_def) in m2.animations.iter().enumerate() {
            animations.insert(idx as u16, anim_def.clone());
        }

        // Load external animations if any
        let external_anims = load_external_animations(&m2, path)?;
        for anim in external_anims {
            animations.insert(anim.id, anim);
        }

        let loaded = LoadedM2Model {
            m2,
            skins,
            meshes,
            textures,
            animations,
            current_animation: AnimationState::new(m2.bones.len()),
        };

        self.models.insert(path.to_string(), loaded);
        Ok(&self.models[path])
    }

    pub fn update_animation(&mut self, path: &str, animation_id: u16, delta_ms: u32) {
        if let Some(model) = self.models.get_mut(path) {
            if let Some(animation) = model.animations.get(&animation_id) {
                model.current_animation.update(&model.m2, animation, delta_ms);
            }
        }
    }
}
```

### Shader for M2 Models

```wgsl
// m2_shader.wgsl

struct Camera {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    position: vec3<f32>,
    _padding: f32,
}

struct BoneMatrices {
    bones: array<mat4x4<f32>, 256>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> bones: BoneMatrices;

@group(2) @binding(0)
var diffuse_texture: texture_2d<f32>;
@group(2) @binding(1)
var diffuse_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) texcoord: vec2<f32>,
    @location(3) texcoord2: vec2<f32>,
    @location(4) bone_indices: vec4<u32>,
    @location(5) bone_weights: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) texcoord: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Apply bone transformations
    var skinned_position = vec4<f32>(0.0);
    var skinned_normal = vec3<f32>(0.0);

    for (var i = 0u; i < 4u; i++) {
        let bone_idx = input.bone_indices[i];
        let weight = input.bone_weights[i];

        if (weight > 0.0) {
            let bone_matrix = bones.bones[bone_idx];
            skinned_position += bone_matrix * vec4<f32>(input.position, 1.0) * weight;
            skinned_normal += (bone_matrix * vec4<f32>(input.normal, 0.0)).xyz * weight;
        }
    }

    out.world_position = skinned_position.xyz;
    out.normal = normalize(skinned_normal);
    out.texcoord = input.texcoord;
    out.clip_position = camera.view_proj * vec4<f32>(out.world_position, 1.0);

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample diffuse texture
    var color = textureSample(diffuse_texture, diffuse_sampler, input.texcoord);

    // Basic lighting
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let n_dot_l = max(dot(input.normal, light_dir), 0.0);
    let ambient = vec3<f32>(0.3, 0.3, 0.4);

    color.xyz = color.xyz * (ambient + n_dot_l * 0.7);

    return color;
}
```

## Best Practices

### 1. Model Instancing

```rust
struct M2Instance {
    transform: Matrix4<f32>,
    animation_state: AnimationState,
    tint_color: Vector4<f32>,
}

struct InstancedM2Renderer {
    instance_buffer: Buffer,
    max_instances: usize,
}

impl InstancedM2Renderer {
    pub fn render_instances(
        &self,
        encoder: &mut CommandEncoder,
        model: &LoadedM2Model,
        instances: &[M2Instance],
    ) {
        // Update instance buffer
        let instance_data: Vec<InstanceData> = instances
            .iter()
            .map(|inst| InstanceData {
                transform: inst.transform.into(),
                color: inst.tint_color.into(),
            })
            .collect();

        self.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instance_data),
        );

        // Render with instancing
        render_pass.draw_indexed(
            0..model.index_count,
            0,
            0..instances.len() as u32,
        );
    }
}
```

### 2. Animation Blending

```rust
pub struct AnimationBlender {
    blend_time: f32,
    source_animation: u16,
    target_animation: u16,
    blend_factor: f32,
}

impl AnimationBlender {
    pub fn blend_animations(
        &self,
        m2: &M2Model,
        source: &AnimationState,
        target: &AnimationState,
    ) -> Vec<Matrix4<f32>> {
        let mut blended_matrices = Vec::with_capacity(m2.bones.len());

        for i in 0..m2.bones.len() {
            let source_matrix = source.bone_matrices[i];
            let target_matrix = target.bone_matrices[i];

            // Decompose matrices
            let (source_trans, source_rot, source_scale) = decompose_matrix(source_matrix);
            let (target_trans, target_rot, target_scale) = decompose_matrix(target_matrix);

            // Interpolate components
            let trans = source_trans.lerp(&target_trans, self.blend_factor);
            let rot = source_rot.slerp(&target_rot, self.blend_factor);
            let scale = source_scale.lerp(&target_scale, self.blend_factor);

            // Reconstruct matrix
            let blended = Matrix4::from_translation(trans) *
                         Matrix4::from(rot) *
                         Matrix4::from_scale(scale);

            blended_matrices.push(blended);
        }

        blended_matrices
    }
}
```

### 3. LOD Support

```rust
pub struct M2LodSelector {
    screen_space_threshold: f32,
}

impl M2LodSelector {
    pub fn select_skin_lod(
        &self,
        model: &LoadedM2Model,
        camera: &Camera,
        model_position: Vector3<f32>,
    ) -> usize {
        // Calculate screen space size
        let distance = (camera.position - model_position).magnitude();
        let screen_size = model.m2.bounding_radius / distance;

        // Select appropriate skin LOD
        if screen_size > self.screen_space_threshold {
            0 // Highest detail
        } else if screen_size > self.screen_space_threshold * 0.5 {
            1.min(model.skins.len() - 1)
        } else {
            2.min(model.skins.len() - 1) // Lowest detail
        }
    }
}
```

## Common Issues and Solutions

### Issue: Incorrect Bone Weights

**Problem**: Model appears distorted during animation.

**Solution**:

```rust
fn validate_and_fix_bone_weights(vertex: &mut M2Vertex) {
    // Ensure weights sum to 255 (1.0 when normalized)
    let sum: u32 = vertex.bone_weights.iter().map(|&w| w as u32).sum();

    if sum == 0 {
        // No weights - bind to first bone
        vertex.bone_weights[0] = 255;
        vertex.bone_indices[0] = 0;
    } else if sum != 255 {
        // Normalize weights
        let factor = 255.0 / sum as f32;
        for weight in &mut vertex.bone_weights {
            *weight = (*weight as f32 * factor) as u8;
        }
    }
}
```

### Issue: Texture Coordinates Out of Range

**Problem**: Textures appear stretched or tiled incorrectly.

**Solution**:

```rust
fn fix_texture_coordinates(texcoord: Vector2<f32>) -> Vector2<f32> {
    // M2 texture coordinates can exceed [0,1] range
    // Use wrapping for tiled textures
    Vector2::new(
        texcoord.x.fract(),
        texcoord.y.fract(),
    )
}
```

### Issue: Animation Playback Speed

**Problem**: Animations play too fast or too slow.

**Solution**:

```rust
impl AnimationState {
    fn update_with_playback_speed(&mut self, m2: &M2Model, animation: &M2Animation, delta_ms: u32, speed: f32) {
        // Apply playback speed modifier
        let adjusted_delta = (delta_ms as f32 * speed) as u32;

        self.current_time += adjusted_delta;

        // Handle animation flags
        if animation.flags.contains(AnimationFlags::LOOPED) {
            self.current_time %= animation.duration;
        } else if self.current_time >= animation.duration {
            self.current_time = animation.duration - 1;
            self.finished = true;
        }
    }
}
```

## Performance Tips

### 1. Batch Similar Models

```rust
pub struct M2Batcher {
    batches: HashMap<ModelId, ModelBatch>,
}

struct ModelBatch {
    instances: Vec<M2Instance>,
    instance_buffer: Buffer,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl M2Batcher {
    pub fn add_instance(&mut self, model_id: ModelId, instance: M2Instance) {
        self.batches
            .entry(model_id)
            .or_insert_with(|| ModelBatch::new())
            .instances
            .push(instance);
    }

    pub fn render_all(&self, encoder: &mut CommandEncoder, view: &TextureView) {
        for (model_id, batch) in &self.batches {
            // Render entire batch with single draw call
            self.render_batch(encoder, view, batch);
        }
    }
}
```

### 2. Async Model Loading

```rust
use tokio::task;

pub async fn load_models_async(paths: Vec<String>) -> Vec<Result<LoadedM2Model, Box<dyn std::error::Error>>> {
    let mut tasks = Vec::new();

    for path in paths {
        let task = task::spawn(async move {
            let (m2, skins) = load_m2_model(&path)?;
            let textures = load_model_textures(&m2).await?;

            Ok(LoadedM2Model {
                m2,
                skins,
                textures,
                // ... other fields
            })
        });
        tasks.push(task);
    }

    // Wait for all models to load
    let mut results = Vec::new();
    for task in tasks {
        results.push(task.await.unwrap());
    }

    results
}
```

### 3. Geometry Optimization

```rust
pub fn optimize_m2_geometry(skin: &M2Skin) -> OptimizedMesh {
    use meshopt::*;

    // Optimize vertex cache
    let optimized_indices = optimize_vertex_cache(&skin.indices, skin.vertices.len());

    // Remove duplicate vertices
    let (unique_vertices, remap) = generate_vertex_remap(&skin.vertices);
    let remapped_indices = remap_index_buffer(&optimized_indices, &remap);

    // Optimize for GPU vertex fetch
    let final_indices = optimize_vertex_fetch(
        &remapped_indices,
        &unique_vertices,
    );

    OptimizedMesh {
        vertices: unique_vertices,
        indices: final_indices,
    }
}
```

## Related Guides

- [📦 Working with MPQ Archives](./mpq-archives.md) - Extract M2 files from archives
- [🖼️ Texture Loading Guide](./texture-loading.md) - Load BLP textures for models
- [🎬 Animation System Guide](./animation-system.md) - Advanced animation techniques
- [🎨 Model Rendering Guide](./model-rendering.md) - Rendering optimization

## References

- [M2 Format Documentation](https://wowdev.wiki/M2) - Complete M2 format specification
- [WoW Model Viewer](https://github.com/Marlamin/WoWModelViewer) - Reference implementation
- [Skeletal Animation](https://learnopengl.com/Guest-Articles/2020/Skeletal-Animation) - Understanding skeletal animation
- [GPU Skinning](https://developer.nvidia.com/gpugems/gpugems/part-i-natural-effects/chapter-4-animation-gpu) - GPU-based skeletal animation
