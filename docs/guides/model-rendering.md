# 🎨 Model Rendering Guide

## Overview

Rendering World of Warcraft models efficiently requires understanding of
GPU techniques, shader programming, and optimization strategies. This guide covers
advanced rendering techniques for M2 models and WMOs using `warcraft-rs`, including
materials, lighting, effects, and performance optimization.

## Prerequisites

Before implementing model rendering, ensure you have:

- Strong understanding of graphics APIs (wgpu, Vulkan, OpenGL)
- Knowledge of shader programming (WGSL, GLSL, HLSL)
- Understanding of rendering pipelines and GPU architecture
- Familiarity with PBR (Physically Based Rendering) concepts
- Experience with graphics debugging tools

## Understanding WoW Rendering

### Rendering Features

- **Multi-pass Rendering**: Opaque, transparent, particle passes
- **Material System**: Diffuse, specular, emissive, environment maps
- **Lighting**: Vertex lighting, dynamic lights, ambient lighting
- **Effects**: Glow, transparency, billboarding, UV animation
- **Shadows**: Shadow mapping, cascaded shadows
- **Post-processing**: Bloom, fog, color grading

### Rendering Challenges

- **Draw Call Optimization**: Thousands of objects per frame
- **Transparency Sorting**: Proper alpha blending order
- **Texture Management**: Efficient texture binding
- **State Changes**: Minimizing GPU state switches
- **Memory Bandwidth**: Vertex data optimization

## Step-by-Step Instructions

### 1. Setting Up the Rendering Pipeline

```rust
use wgpu::*;
use bytemuck::{Pod, Zeroable};

pub struct ModelRenderer {
    device: Device,
    queue: Queue,
    pipelines: RenderPipelineCache,
    bind_group_layouts: BindGroupLayouts,
    shader_cache: ShaderCache,
}

pub struct RenderPipelineCache {
    opaque: HashMap<PipelineKey, RenderPipeline>,
    transparent: HashMap<PipelineKey, RenderPipeline>,
    particle: RenderPipeline,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PipelineKey {
    vertex_layout: VertexLayoutType,
    material_flags: MaterialFlags,
    blend_mode: BlendMode,
    cull_mode: CullMode,
}

pub struct BindGroupLayouts {
    camera: BindGroupLayout,
    model: BindGroupLayout,
    material: BindGroupLayout,
    lighting: BindGroupLayout,
}

impl ModelRenderer {
    pub fn new(device: Device, queue: Queue) -> Self {
        let shader_cache = ShaderCache::new(&device);
        let bind_group_layouts = Self::create_bind_group_layouts(&device);
        let pipelines = Self::create_pipelines(&device, &shader_cache, &bind_group_layouts);

        Self {
            device,
            queue,
            pipelines,
            bind_group_layouts,
            shader_cache,
        }
    }

    fn create_bind_group_layouts(device: &Device) -> BindGroupLayouts {
        // Camera bind group layout
        let camera = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(NonZeroU64::new(256).unwrap()),
                    },
                    count: None,
                },
            ],
        });

        // Model bind group layout (per-instance data)
        let model = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Model Bind Group Layout"),
            entries: &[
                // Model transform
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(NonZeroU64::new(64).unwrap()),
                    },
                    count: None,
                },
                // Bone matrices
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(NonZeroU64::new(64 * 256).unwrap()),
                    },
                    count: None,
                },
            ],
        });

        // Material bind group layout
        let material = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Material Bind Group Layout"),
            entries: &[
                // Material properties
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(NonZeroU64::new(64).unwrap()),
                    },
                    count: None,
                },
                // Diffuse texture
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // Texture sampler
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Lighting bind group layout
        let lighting = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Lighting Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(NonZeroU64::new(32 * 128).unwrap()),
                    },
                    count: None,
                },
            ],
        });

        BindGroupLayouts {
            camera,
            model,
            material,
            lighting,
        }
    }
}
```

### 2. Material System Implementation

```rust
use bitflags::bitflags;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct GpuMaterial {
    pub base_color: [f32; 4],
    pub emissive: [f32; 3],
    pub metallic: f32,
    pub roughness: f32,
    pub flags: u32,
    pub blend_mode: u32,
    pub _padding: u32,
}

bitflags! {
    pub struct MaterialFlags: u32 {
        const UNLIT = 0x01;
        const UNFOGGED = 0x02;
        const TWO_SIDED = 0x04;
        const DEPTH_TEST = 0x08;
        const DEPTH_WRITE = 0x10;
        const ALPHA_TEST = 0x20;
        const ADDITIVE = 0x40;
        const ENVIRONMENT_MAP = 0x80;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Opaque,
    AlphaBlend,
    Additive,
    Multiply,
    AlphaKey,
}

pub struct MaterialManager {
    materials: HashMap<u32, Material>,
    bind_groups: HashMap<u32, BindGroup>,
    default_material: Material,
}

#[derive(Debug, Clone)]
pub struct Material {
    pub gpu_data: GpuMaterial,
    pub textures: MaterialTextures,
    pub render_flags: RenderFlags,
}

#[derive(Debug, Clone)]
pub struct MaterialTextures {
    pub diffuse: Option<TextureId>,
    pub normal: Option<TextureId>,
    pub specular: Option<TextureId>,
    pub emissive: Option<TextureId>,
    pub environment: Option<TextureId>,
}

impl MaterialManager {
    pub fn create_material_from_m2(
        &mut self,
        m2_material: &M2Material,
        texture_manager: &TextureManager,
    ) -> u32 {
        let material = Material {
            gpu_data: GpuMaterial {
                base_color: [1.0, 1.0, 1.0, 1.0],
                emissive: [0.0, 0.0, 0.0],
                metallic: 0.0,
                roughness: 0.8,
                flags: m2_material.flags.bits(),
                blend_mode: m2_material.blend_mode as u32,
                _padding: 0,
            },
            textures: MaterialTextures {
                diffuse: texture_manager.get_texture(m2_material.texture_id),
                normal: None,
                specular: None,
                emissive: None,
                environment: None,
            },
            render_flags: self.determine_render_flags(m2_material),
        };

        let material_id = self.materials.len() as u32;
        self.materials.insert(material_id, material);

        // Create bind group
        self.create_material_bind_group(material_id);

        material_id
    }

    fn determine_render_flags(&self, m2_material: &M2Material) -> RenderFlags {
        let mut flags = RenderFlags::empty();

        if m2_material.flags.contains(MaterialFlags::UNLIT) {
            flags |= RenderFlags::DISABLE_LIGHTING;
        }

        if m2_material.flags.contains(MaterialFlags::TWO_SIDED) {
            flags |= RenderFlags::DISABLE_CULLING;
        }

        match m2_material.blend_mode {
            1 => flags |= RenderFlags::ALPHA_BLEND,
            2 => flags |= RenderFlags::ADDITIVE_BLEND,
            _ => {}
        }

        flags
    }
}
```

### 3. Advanced Shader System

```wgsl
// model_shader.wgsl

// Vertex shader structures
struct CameraUniforms {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    eye_pos: vec3<f32>,
    time: f32,
}

struct ModelUniforms {
    model: mat4x4<f32>,
    normal_matrix: mat3x3<f32>,
    color: vec4<f32>,
}

struct MaterialUniforms {
    base_color: vec4<f32>,
    emissive: vec3<f32>,
    metallic: f32,
    roughness: f32,
    flags: u32,
    blend_mode: u32,
    _padding: u32,
}

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(1) @binding(0) var<uniform> model: ModelUniforms;
@group(1) @binding(1) var<storage, read> bone_matrices: array<mat4x4<f32>>;
@group(2) @binding(0) var<uniform> material: MaterialUniforms;
@group(2) @binding(1) var diffuse_texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec4<f32>,
    @location(3) texcoord0: vec2<f32>,
    @location(4) texcoord1: vec2<f32>,
    @location(5) color: vec4<f32>,
    @location(6) bone_indices: vec4<u32>,
    @location(7) bone_weights: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec3<f32>,
    @location(3) bitangent: vec3<f32>,
    @location(4) texcoord0: vec2<f32>,
    @location(5) texcoord1: vec2<f32>,
    @location(6) vertex_color: vec4<f32>,
    @location(7) view_dir: vec3<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Skeletal animation
    var skinned_pos = vec4<f32>(0.0);
    var skinned_normal = vec3<f32>(0.0);
    var skinned_tangent = vec3<f32>(0.0);

    for (var i = 0u; i < 4u; i++) {
        let bone_idx = in.bone_indices[i];
        let weight = in.bone_weights[i];

        if (weight > 0.0) {
            let bone_matrix = bone_matrices[bone_idx];
            skinned_pos += bone_matrix * vec4<f32>(in.position, 1.0) * weight;
            skinned_normal += (bone_matrix * vec4<f32>(in.normal, 0.0)).xyz * weight;
            skinned_tangent += (bone_matrix * vec4<f32>(in.tangent.xyz, 0.0)).xyz * weight;
        }
    }

    // Transform to world space
    let world_pos = model.model * skinned_pos;
    out.world_pos = world_pos.xyz;
    out.clip_position = camera.view_proj * world_pos;

    // Transform normals
    out.normal = normalize(model.normal_matrix * skinned_normal);
    out.tangent = normalize(model.normal_matrix * skinned_tangent);
    out.bitangent = cross(out.normal, out.tangent) * in.tangent.w;

    // Pass through vertex attributes
    out.texcoord0 = in.texcoord0;
    out.texcoord1 = in.texcoord1;
    out.vertex_color = in.color * model.color;

    // Calculate view direction
    out.view_dir = normalize(camera.eye_pos - out.world_pos);

    return out;
}

// Fragment shader with PBR lighting
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample textures
    var base_color = textureSample(diffuse_texture, texture_sampler, in.texcoord0);
    base_color *= material.base_color * in.vertex_color;

    // Alpha test
    if ((material.flags & 0x20u) != 0u && base_color.a < 0.5) {
        discard;
    }

    // Check if unlit
    if ((material.flags & 0x01u) != 0u) {
        return vec4<f32>(base_color.rgb + material.emissive, base_color.a);
    }

    // Normal mapping (if available)
    var N = normalize(in.normal);

    // PBR calculations
    let V = normalize(in.view_dir);
    let NdotV = max(dot(N, V), 0.0);

    // Lighting accumulation
    var Lo = vec3<f32>(0.0);

    // Directional light (sun)
    let L = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let H = normalize(V + L);
    let NdotL = max(dot(N, L), 0.0);
    let NdotH = max(dot(N, H), 0.0);
    let VdotH = max(dot(V, H), 0.0);

    // BRDF
    let D = distribution_ggx(NdotH, material.roughness);
    let G = geometry_smith(NdotV, NdotL, material.roughness);
    let F = fresnel_schlick(VdotH, vec3<f32>(0.04));

    let numerator = D * G * F;
    let denominator = 4.0 * NdotV * NdotL + 0.001;
    let specular = numerator / denominator;

    let kS = F;
    let kD = (vec3<f32>(1.0) - kS) * (1.0 - material.metallic);

    let radiance = vec3<f32>(2.0);
    Lo += (kD * base_color.rgb / 3.14159265 + specular) * radiance * NdotL;

    // Ambient
    let ambient = vec3<f32>(0.3) * base_color.rgb;

    // Final color
    var color = ambient + Lo + material.emissive;

    // Apply fog (if enabled)
    if ((material.flags & 0x02u) == 0u) {
        let fog_distance = length(in.world_pos - camera.eye_pos);
        let fog_factor = exp(-fog_distance * 0.001);
        color = mix(vec3<f32>(0.5, 0.6, 0.7), color, fog_factor);
    }

    return vec4<f32>(color, base_color.a);
}

// PBR helper functions
fn distribution_ggx(NdotH: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH2 = NdotH * NdotH;

    let numerator = a2;
    let denominator = NdotH2 * (a2 - 1.0) + 1.0;
    let denominator2 = 3.14159265 * denominator * denominator;

    return numerator / denominator2;
}

fn geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;

    return NdotV / (NdotV * (1.0 - k) + k);
}

fn geometry_smith(NdotV: f32, NdotL: f32, roughness: f32) -> f32 {
    let ggx1 = geometry_schlick_ggx(NdotV, roughness);
    let ggx2 = geometry_schlick_ggx(NdotL, roughness);

    return ggx1 * ggx2;
}

fn fresnel_schlick(cosTheta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}
```

### 4. Batch Rendering System

```rust
use std::sync::Arc;

pub struct BatchRenderer {
    batches: HashMap<BatchKey, RenderBatch>,
    instance_data: Vec<InstanceData>,
    instance_buffer: Buffer,
    draw_commands: Vec<DrawCommand>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct BatchKey {
    mesh_id: u64,
    material_id: u32,
    pipeline_key: PipelineKey,
}

pub struct RenderBatch {
    instances: Vec<u32>, // indices into instance_data
    vertex_buffer: Arc<Buffer>,
    index_buffer: Arc<Buffer>,
    index_count: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct InstanceData {
    model_matrix: [[f32; 4]; 4],
    normal_matrix: [[f32; 3]; 3],
    color: [f32; 4],
    texture_transform: [f32; 4], // scale_x, scale_y, offset_x, offset_y
}

pub struct DrawCommand {
    batch_key: BatchKey,
    instance_start: u32,
    instance_count: u32,
}

impl BatchRenderer {
    pub fn prepare_frame(&mut self, models: &[ModelInstance]) {
        self.batches.clear();
        self.instance_data.clear();
        self.draw_commands.clear();

        // Group models by batch key
        for model in models {
            let batch_key = BatchKey {
                mesh_id: model.mesh.id(),
                material_id: model.material_id,
                pipeline_key: model.pipeline_key(),
            };

            let instance_idx = self.instance_data.len() as u32;
            self.instance_data.push(model.instance_data());

            self.batches
                .entry(batch_key)
                .or_insert_with(|| RenderBatch {
                    instances: Vec::new(),
                    vertex_buffer: model.mesh.vertex_buffer.clone(),
                    index_buffer: model.mesh.index_buffer.clone(),
                    index_count: model.mesh.index_count,
                })
                .instances
                .push(instance_idx);
        }

        // Generate draw commands
        for (batch_key, batch) in &self.batches {
            if !batch.instances.is_empty() {
                self.draw_commands.push(DrawCommand {
                    batch_key: batch_key.clone(),
                    instance_start: batch.instances[0],
                    instance_count: batch.instances.len() as u32,
                });
            }
        }

        // Sort draw commands for optimal rendering
        self.sort_draw_commands();

        // Update instance buffer
        self.update_instance_buffer();
    }

    fn sort_draw_commands(&mut self) {
        self.draw_commands.sort_by(|a, b| {
            // Sort by pipeline first (minimize state changes)
            match a.batch_key.pipeline_key.cmp(&b.batch_key.pipeline_key) {
                std::cmp::Ordering::Equal => {
                    // Then by material (minimize texture bindings)
                    match a.batch_key.material_id.cmp(&b.batch_key.material_id) {
                        std::cmp::Ordering::Equal => {
                            // Finally by mesh
                            a.batch_key.mesh_id.cmp(&b.batch_key.mesh_id)
                        }
                        other => other,
                    }
                }
                other => other,
            }
        });
    }

    pub fn render(
        &self,
        render_pass: &mut RenderPass,
        pipeline_cache: &RenderPipelineCache,
        material_manager: &MaterialManager,
    ) {
        let mut current_pipeline: Option<PipelineKey> = None;
        let mut current_material: Option<u32> = None;

        for command in &self.draw_commands {
            let batch = &self.batches[&command.batch_key];

            // Set pipeline if changed
            if current_pipeline != Some(command.batch_key.pipeline_key.clone()) {
                let pipeline = pipeline_cache.get(&command.batch_key.pipeline_key);
                render_pass.set_pipeline(pipeline);
                current_pipeline = Some(command.batch_key.pipeline_key.clone());
            }

            // Set material if changed
            if current_material != Some(command.batch_key.material_id) {
                let material_bind_group = material_manager.get_bind_group(command.batch_key.material_id);
                render_pass.set_bind_group(2, material_bind_group, &[]);
                current_material = Some(command.batch_key.material_id);
            }

            // Set mesh buffers
            render_pass.set_vertex_buffer(0, batch.vertex_buffer.slice(..));
            render_pass.set_index_buffer(batch.index_buffer.slice(..), IndexFormat::Uint32);

            // Draw instanced
            render_pass.draw_indexed(
                0..batch.index_count,
                0,
                command.instance_start..(command.instance_start + command.instance_count),
            );
        }
    }
}
```

### 5. Effect Rendering System

```rust
pub struct EffectRenderer {
    glow_pipeline: RenderPipeline,
    particle_pipeline: RenderPipeline,
    ribbon_pipeline: RenderPipeline,
    screen_effect_pipeline: RenderPipeline,
}

pub struct GlowEffect {
    intensity: f32,
    color: Vector3<f32>,
    size: f32,
}

pub struct ParticleSystem {
    emitters: Vec<ParticleEmitter>,
    particles: Vec<Particle>,
    vertex_buffer: Buffer,
    instance_buffer: Buffer,
}

impl EffectRenderer {
    pub fn render_glow_effects(
        &self,
        encoder: &mut CommandEncoder,
        models: &[ModelWithGlow],
        glow_texture: &TextureView,
        depth_texture: &TextureView,
    ) {
        // First pass: Render glowing parts to separate texture
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Glow Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: glow_texture,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: depth_texture,
                    depth_ops: Some(Operations {
                        load: LoadOp::Load,
                        store: false,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.glow_pipeline);

            for model in models {
                if model.has_glow() {
                    self.render_model_glow(&mut render_pass, model);
                }
            }
        }

        // Second pass: Blur glow texture
        self.blur_texture(encoder, glow_texture);

        // Third pass: Composite with main scene
        self.composite_glow(encoder, glow_texture);
    }

    pub fn render_particles(
        &self,
        render_pass: &mut RenderPass,
        particle_system: &ParticleSystem,
        camera: &Camera,
    ) {
        render_pass.set_pipeline(&self.particle_pipeline);

        // Update particle vertices (billboarding)
        let vertices = self.generate_particle_vertices(particle_system, camera);
        particle_system.vertex_buffer.write(&vertices);

        // Sort particles by distance for proper blending
        let sorted_indices = self.sort_particles_by_distance(particle_system, camera);

        render_pass.set_vertex_buffer(0, particle_system.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, particle_system.instance_buffer.slice(..));

        for emitter in &particle_system.emitters {
            render_pass.set_bind_group(2, &emitter.material_bind_group, &[]);

            let particle_range = emitter.particle_range();
            render_pass.draw(
                0..4, // Quad vertices
                particle_range.start as u32..particle_range.end as u32,
            );
        }
    }

    fn generate_particle_vertices(
        &self,
        system: &ParticleSystem,
        camera: &Camera,
    ) -> Vec<ParticleVertex> {
        let mut vertices = Vec::new();
        let right = camera.right();
        let up = camera.up();

        for particle in &system.particles {
            let size = particle.size * 0.5;

            // Billboard corners
            let corners = [
                particle.position - right * size - up * size,
                particle.position + right * size - up * size,
                particle.position + right * size + up * size,
                particle.position - right * size + up * size,
            ];

            for (i, corner) in corners.iter().enumerate() {
                vertices.push(ParticleVertex {
                    position: corner.into(),
                    texcoord: match i {
                        0 => [0.0, 1.0],
                        1 => [1.0, 1.0],
                        2 => [1.0, 0.0],
                        3 => [0.0, 0.0],
                        _ => unreachable!(),
                    },
                    color: particle.color.into(),
                });
            }
        }

        vertices
    }
}
```

### 6. Shadow Mapping

```rust
pub struct ShadowRenderer {
    shadow_maps: Vec<ShadowMap>,
    shadow_pipeline: RenderPipeline,
    cascade_data: CascadeData,
}

pub struct ShadowMap {
    texture: Texture,
    view: TextureView,
    size: u32,
    view_proj: Matrix4<f32>,
}

pub struct CascadeData {
    splits: Vec<f32>,
    matrices: Vec<Matrix4<f32>>,
}

impl ShadowRenderer {
    pub fn render_shadows(
        &mut self,
        encoder: &mut CommandEncoder,
        models: &[ModelInstance],
        light_direction: Vector3<f32>,
        camera: &Camera,
    ) {
        // Calculate cascade splits
        self.update_cascades(camera, light_direction);

        // Render each cascade
        for (cascade_idx, shadow_map) in self.shadow_maps.iter_mut().enumerate() {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some(&format!("Shadow Pass Cascade {}", cascade_idx)),
                color_attachments: &[],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &shadow_map.view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.shadow_pipeline);

            // Set cascade view-projection matrix
            let cascade_uniform = CascadeUniform {
                view_proj: shadow_map.view_proj.into(),
            };
            render_pass.set_bind_group(0, &cascade_uniform.bind_group, &[]);

            // Render models
            for model in models {
                if self.is_in_cascade_frustum(model, cascade_idx) {
                    self.render_model_depth_only(&mut render_pass, model);
                }
            }
        }
    }

    fn update_cascades(&mut self, camera: &Camera, light_dir: Vector3<f32>) {
        let view_matrix = camera.view_matrix();
        let proj_matrix = camera.projection_matrix();
        let inv_view_proj = (proj_matrix * view_matrix).try_inverse().unwrap();

        for (i, split_distance) in self.cascade_data.splits.iter().enumerate() {
            let near = if i == 0 {
                camera.near()
            } else {
                self.cascade_data.splits[i - 1]
            };
            let far = *split_distance;

            // Calculate cascade frustum corners
            let frustum_corners = self.calculate_frustum_corners(near, far, &inv_view_proj);

            // Calculate light view matrix
            let center = frustum_corners.iter().sum::<Vector3<f32>>() / 8.0;
            let light_view = Matrix4::look_at_rh(
                &Point3::from(center + light_dir * 100.0),
                &Point3::from(center),
                &Vector3::y(),
            );

            // Calculate light projection matrix
            let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
            let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);

            for corner in &frustum_corners {
                let view_space = light_view.transform_point(&Point3::from(*corner));
                min = min.zip_map(&view_space.coords, f32::min);
                max = max.zip_map(&view_space.coords, f32::max);
            }

            // Snap to texel grid to reduce shimmer
            let texel_size = (max.x - min.x) / self.shadow_maps[i].size as f32;
            min.x = (min.x / texel_size).floor() * texel_size;
            max.x = (max.x / texel_size).ceil() * texel_size;
            min.y = (min.y / texel_size).floor() * texel_size;
            max.y = (max.y / texel_size).ceil() * texel_size;

            let light_proj = Matrix4::new_orthographic(
                min.x, max.x,
                min.y, max.y,
                min.z - 50.0, max.z + 50.0,
            );

            self.shadow_maps[i].view_proj = light_proj * light_view;
            self.cascade_data.matrices[i] = self.shadow_maps[i].view_proj;
        }
    }
}
```

## Code Examples

### Complete Render Frame

```rust
pub struct RenderFrame {
    models: Vec<ModelInstance>,
    transparent_models: Vec<ModelInstance>,
    particles: Vec<ParticleSystem>,
    lights: Vec<Light>,
    shadow_casters: Vec<ModelInstance>,
}

impl ModelRenderer {
    pub fn render_frame(
        &mut self,
        encoder: &mut CommandEncoder,
        frame: &RenderFrame,
        camera: &Camera,
        output_view: &TextureView,
    ) {
        // Update per-frame uniforms
        self.update_camera_uniforms(camera);
        self.update_lighting_uniforms(&frame.lights);

        // Shadow pass
        if !frame.shadow_casters.is_empty() {
            self.shadow_renderer.render_shadows(
                encoder,
                &frame.shadow_casters,
                self.sun_direction,
                camera,
            );
        }

        // Depth pre-pass for opaque objects
        self.render_depth_prepass(encoder, &frame.models);

        // Main render pass
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(self.depth_attachment()),
            });

            // Render opaque models
            render_pass.push_debug_group("Opaque Models");
            self.batch_renderer.prepare_frame(&frame.models);
            self.batch_renderer.render(
                &mut render_pass,
                &self.pipelines,
                &self.material_manager,
            );
            render_pass.pop_debug_group();

            // Render transparent models (sorted back-to-front)
            render_pass.push_debug_group("Transparent Models");
            let sorted_transparent = self.sort_transparent_models(&frame.transparent_models, camera);
            for model in sorted_transparent {
                self.render_transparent_model(&mut render_pass, model);
            }
            render_pass.pop_debug_group();

            // Render particles
            render_pass.push_debug_group("Particles");
            for particle_system in &frame.particles {
                self.effect_renderer.render_particles(
                    &mut render_pass,
                    particle_system,
                    camera,
                );
            }
            render_pass.pop_debug_group();
        }

        // Post-processing
        self.render_post_processing(encoder, output_view);
    }

    fn sort_transparent_models<'a>(
        &self,
        models: &'a [ModelInstance],
        camera: &Camera,
    ) -> Vec<&'a ModelInstance> {
        let mut sorted: Vec<_> = models.iter().collect();
        let camera_pos = camera.position();

        sorted.sort_by(|a, b| {
            let dist_a = (a.position() - camera_pos).magnitude_squared();
            let dist_b = (b.position() - camera_pos).magnitude_squared();
            dist_b.partial_cmp(&dist_a).unwrap()
        });

        sorted
    }
}
```

### GPU-Driven Rendering

```rust
pub struct GpuDrivenRenderer {
    indirect_buffer: Buffer,
    visibility_buffer: Buffer,
    draw_commands_buffer: Buffer,
    cull_shader: ComputePipeline,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct DrawIndirectCommand {
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
}

impl GpuDrivenRenderer {
    pub fn cull_and_render(
        &mut self,
        encoder: &mut CommandEncoder,
        models: &[ModelInstance],
        camera: &Camera,
    ) {
        // Upload model data
        let model_data: Vec<GpuModelData> = models
            .iter()
            .map(|m| GpuModelData {
                bounding_sphere: m.bounding_sphere(),
                lod_distances: m.lod_distances(),
                instance_data: m.instance_data(),
            })
            .collect();

        self.model_buffer.write(&model_data);

        // Frustum culling compute pass
        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("GPU Culling Pass"),
            });

            compute_pass.set_pipeline(&self.cull_shader);
            compute_pass.set_bind_group(0, &self.cull_bind_group, &[]);

            let dispatch_size = (models.len() as u32 + 63) / 64;
            compute_pass.dispatch_workgroups(dispatch_size, 1, 1);
        }

        // Render using indirect draw
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("GPU-Driven Render Pass"),
                // ... attachments
            });

            render_pass.set_pipeline(&self.render_pipeline);

            // Multi-draw indirect
            render_pass.multi_draw_indirect(
                &self.indirect_buffer,
                0,
                models.len() as u32,
            );
        }
    }
}
```

## Best Practices

### 1. Render State Management

```rust
pub struct RenderStateManager {
    current_state: RenderState,
    state_cache: HashMap<RenderStateKey, RenderPipeline>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct RenderState {
    blend_mode: BlendMode,
    cull_mode: CullMode,
    depth_test: bool,
    depth_write: bool,
    stencil_test: bool,
}

impl RenderStateManager {
    pub fn set_state(
        &mut self,
        render_pass: &mut RenderPass,
        new_state: &RenderState,
    ) {
        if self.current_state != *new_state {
            // Get or create pipeline for this state
            let pipeline = self.get_or_create_pipeline(new_state);
            render_pass.set_pipeline(&pipeline);
            self.current_state = new_state.clone();
        }
    }

    fn get_or_create_pipeline(&mut self, state: &RenderState) -> &RenderPipeline {
        let key = RenderStateKey::from(state);

        self.state_cache.entry(key).or_insert_with(|| {
            self.create_pipeline_for_state(state)
        })
    }
}
```

### 2. Texture Binding Optimization

```rust
pub struct TextureBindingOptimizer {
    texture_arrays: HashMap<TextureFormat, TextureArray>,
    bindless_heap: Option<BindlessTextureHeap>,
}

impl TextureBindingOptimizer {
    pub fn optimize_texture_bindings(&mut self, materials: &[Material]) {
        // Group textures by format and size
        let mut texture_groups: HashMap<(TextureFormat, u32, u32), Vec<TextureId>> = HashMap::new();

        for material in materials {
            if let Some(texture_id) = material.textures.diffuse {
                let info = self.get_texture_info(texture_id);
                texture_groups
                    .entry((info.format, info.width, info.height))
                    .or_insert_with(Vec::new)
                    .push(texture_id);
            }
        }

        // Create texture arrays for groups
        for ((format, width, height), textures) in texture_groups {
            if textures.len() > 4 {
                self.create_texture_array(format, width, height, &textures);
            }
        }
    }
}
```

### 3. Draw Call Merging

```rust
pub struct DrawCallMerger {
    merge_distance: f32,
    max_vertices_per_buffer: u32,
}

impl DrawCallMerger {
    pub fn merge_static_geometry(
        &self,
        static_models: &[StaticModel],
    ) -> Vec<MergedMesh> {
        let mut merged_meshes = Vec::new();
        let mut spatial_hash = SpatialHash::new(self.merge_distance);

        // Group models by material and spatial proximity
        for model in static_models {
            spatial_hash.insert(model.position, model);
        }

        // Merge nearby models with same material
        for cell in spatial_hash.cells() {
            let mut groups: HashMap<u32, Vec<&StaticModel>> = HashMap::new();

            for model in cell {
                groups
                    .entry(model.material_id)
                    .or_insert_with(Vec::new)
                    .push(model);
            }

            for (material_id, models) in groups {
                if let Some(merged) = self.merge_models(&models) {
                    merged_meshes.push(merged);
                }
            }
        }

        merged_meshes
    }
}
```

## Common Issues and Solutions

### Issue: Z-Fighting

**Problem**: Flickering between overlapping surfaces.

**Solution**:

```rust
// Use polygon offset for decals
let decal_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
    // ... other settings
    depth_stencil: Some(DepthStencilState {
        format: TextureFormat::Depth32Float,
        depth_write_enabled: true,
        depth_compare: CompareFunction::LessEqual,
        stencil: StencilState::default(),
        bias: DepthBiasState {
            constant: -1,
            slope_scale: -1.0,
            clamp: 0.0,
        },
    }),
});
```

### Issue: Transparency Sorting

**Problem**: Incorrect rendering order for transparent objects.

**Solution**:

```rust
pub struct TransparencyManager {
    oit_buffers: OitBuffers, // Order-Independent Transparency
}

impl TransparencyManager {
    pub fn render_transparent_oit(
        &mut self,
        render_pass: &mut RenderPass,
        transparent_objects: &[TransparentObject],
    ) {
        // Use per-pixel linked lists or weighted blended OIT
        render_pass.set_pipeline(&self.oit_accumulation_pipeline);

        for object in transparent_objects {
            // Render to OIT buffers without sorting
            self.render_to_oit_buffer(render_pass, object);
        }

        // Resolve OIT buffer to final image
        self.resolve_oit(render_pass);
    }
}
```

### Issue: Shader Compilation Stutter

**Problem**: Frame drops when new shaders compile.

**Solution**:

```rust
pub struct ShaderPrecompiler {
    compile_queue: Arc<Mutex<VecDeque<ShaderVariant>>>,
    worker_thread: Option<JoinHandle<()>>,
}

impl ShaderPrecompiler {
    pub fn precompile_variants(&mut self, materials: &[Material]) {
        // Collect all possible shader variants
        let mut variants = HashSet::new();

        for material in materials {
            variants.insert(ShaderVariant::from_material(material));
        }

        // Queue for background compilation
        let mut queue = self.compile_queue.lock().unwrap();
        for variant in variants {
            queue.push_back(variant);
        }
    }
}
```

## Performance Tips

### 1. GPU Instancing

```rust
pub fn optimize_instancing(models: &[ModelInstance]) -> Vec<InstancedDraw> {
    let mut instanced_draws: HashMap<MeshId, Vec<InstanceData>> = HashMap::new();

    for model in models {
        instanced_draws
            .entry(model.mesh_id)
            .or_insert_with(Vec::new)
            .push(model.instance_data());
    }

    instanced_draws
        .into_iter()
        .filter(|(_, instances)| instances.len() > 1)
        .map(|(mesh_id, instances)| InstancedDraw {
            mesh_id,
            instances,
        })
        .collect()
}
```

### 2. Occlusion Culling

```rust
pub struct HiZOcclusionCuller {
    hi_z_buffer: Texture,
    hi_z_levels: Vec<TextureView>,
}

impl HiZOcclusionCuller {
    pub fn test_visibility(
        &self,
        bounding_boxes: &[BoundingBox],
        camera: &Camera,
    ) -> Vec<bool> {
        // Use hierarchical Z-buffer for fast occlusion tests
        let mut visibility = vec![true; bounding_boxes.len()];

        for (i, bbox) in bounding_boxes.iter().enumerate() {
            let screen_rect = self.project_to_screen(bbox, camera);
            let mip_level = self.select_mip_level(screen_rect);

            visibility[i] = self.test_rect_visibility(screen_rect, mip_level);
        }

        visibility
    }
}
```

### 3. Mesh LOD Selection

```rust
pub struct LodSelector {
    screen_space_error: f32,
}

impl LodSelector {
    pub fn select_lod(
        &self,
        model: &Model,
        camera: &Camera,
        screen_height: f32,
    ) -> usize {
        let distance = (model.center - camera.position()).magnitude();
        let screen_size = (model.radius / distance) * screen_height;

        // Select LOD based on screen coverage
        for (i, lod) in model.lods.iter().enumerate() {
            if screen_size * lod.error_metric < self.screen_space_error {
                return i;
            }
        }

        model.lods.len() - 1
    }
}
```

## Related Guides

- [🎭 Loading M2 Models](./m2-models.md) - Load models for rendering
- [🏛️ WMO Rendering Guide](./wmo-rendering.md) - Render world objects
- [🖼️ Texture Loading Guide](./texture-loading.md) - Texture management
- [🎬 Animation System Guide](./animation-system.md) - Animate rendered models
- [📊 LOD System Guide](./lod-system.md) - Level-of-detail rendering

## References

- [Real-Time Rendering](https://www.realtimerendering.com/) - Rendering techniques reference
- [GPU Gems](https://developer.nvidia.com/gpugems) - Advanced GPU programming
- [Learn OpenGL](https://learnopengl.com/) - OpenGL techniques
- [GPU-Driven Rendering](https://advances.realtimerendering.com/s2015/aaltonenhaar_siggraph2015_combined_final_footer_220dpi.pdf) - GPU-driven techniques
