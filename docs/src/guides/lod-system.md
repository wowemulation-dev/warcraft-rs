# 📊 Level of Detail (LoD) System Guide

## Overview

The Level of Detail (LoD) system in World of Warcraft optimizes terrain rendering
by using different levels of detail based on viewing distance. This guide explains
how to implement terrain LoD using the warcraft-rs library, focusing on the
WDL/ADT system.

## Prerequisites

Before implementing LOD systems, ensure you have:

- Understanding of 3D graphics optimization techniques
- Knowledge of mesh simplification algorithms
- Familiarity with view-dependent rendering
- Experience with performance profiling
- Understanding of GPU bandwidth limitations

## Understanding LOD in WoW

### LOD Types

- **Geometric LOD**: Simplified mesh versions
- **Texture LOD**: Mipmapping and resolution reduction
- **Shader LOD**: Simplified shading models
- **Animation LOD**: Reduced bone counts
- **Terrain LOD**: Chunk simplification
- **Object LOD**: Billboard replacements

### LOD Metrics

- **Screen Space Error**: Pixel deviation tolerance
- **Distance-based**: Simple distance thresholds
- **View-dependent**: Considers viewing angle
- **Performance-based**: Dynamic adjustment
- **Memory-based**: Texture/mesh budget

## Step-by-Step Instructions

### 1. Core LOD System Architecture

```rust
use nalgebra::{Vector3, Point3};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LodSystem {
    configs: HashMap<LodCategory, LodConfig>,
    metrics: LodMetrics,
    performance_monitor: PerformanceMonitor,
    adaptive_settings: AdaptiveSettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LodCategory {
    Terrain,
    Character,
    Creature,
    Vegetation,
    Building,
    Prop,
    Effect,
}

#[derive(Debug, Clone)]
pub struct LodConfig {
    distances: Vec<f32>,
    screen_space_thresholds: Vec<f32>,
    quality_levels: Vec<QualityLevel>,
    transition_type: TransitionType,
}

#[derive(Debug, Clone)]
pub struct QualityLevel {
    vertex_reduction: f32,
    texture_scale: f32,
    shader_complexity: ShaderComplexity,
    animation_quality: AnimationQuality,
}

#[derive(Debug, Clone, Copy)]
pub enum TransitionType {
    Instant,
    Fade(f32),    // fade duration
    Dither,
    Morph(f32),   // morph duration
}

#[derive(Debug, Clone, Copy)]
pub enum ShaderComplexity {
    Ultra,    // Full PBR
    High,     // Simplified PBR
    Medium,   // Phong
    Low,      // Lambert
    Minimal,  // Flat shading
}

impl LodSystem {
    pub fn new() -> Self {
        let mut configs = HashMap::new();

        // Configure LOD for different object types
        configs.insert(LodCategory::Character, LodConfig {
            distances: vec![0.0, 30.0, 60.0, 120.0, 200.0],
            screen_space_thresholds: vec![1.0, 0.5, 0.25, 0.1, 0.05],
            quality_levels: vec![
                QualityLevel {
                    vertex_reduction: 1.0,
                    texture_scale: 1.0,
                    shader_complexity: ShaderComplexity::Ultra,
                    animation_quality: AnimationQuality::Full,
                },
                QualityLevel {
                    vertex_reduction: 0.7,
                    texture_scale: 1.0,
                    shader_complexity: ShaderComplexity::High,
                    animation_quality: AnimationQuality::Full,
                },
                QualityLevel {
                    vertex_reduction: 0.4,
                    texture_scale: 0.5,
                    shader_complexity: ShaderComplexity::Medium,
                    animation_quality: AnimationQuality::Reduced,
                },
                QualityLevel {
                    vertex_reduction: 0.2,
                    texture_scale: 0.25,
                    shader_complexity: ShaderComplexity::Low,
                    animation_quality: AnimationQuality::Minimal,
                },
                QualityLevel {
                    vertex_reduction: 0.1,
                    texture_scale: 0.125,
                    shader_complexity: ShaderComplexity::Minimal,
                    animation_quality: AnimationQuality::None,
                },
            ],
            transition_type: TransitionType::Fade(0.5),
        });

        configs.insert(LodCategory::Terrain, LodConfig {
            distances: vec![0.0, 100.0, 300.0, 600.0, 1000.0],
            screen_space_thresholds: vec![2.0, 1.0, 0.5, 0.25, 0.1],
            quality_levels: Self::create_terrain_quality_levels(),
            transition_type: TransitionType::Morph(1.0),
        });

        Self {
            configs,
            metrics: LodMetrics::new(),
            performance_monitor: PerformanceMonitor::new(),
            adaptive_settings: AdaptiveSettings::default(),
        }
    }

    pub fn select_lod(
        &self,
        object: &LodObject,
        camera: &Camera,
        viewport: &Viewport,
    ) -> LodSelection {
        let config = &self.configs[&object.category];

        // Calculate multiple metrics
        let distance = (object.center - camera.position()).magnitude();
        let screen_size = self.calculate_screen_size(object, camera, viewport);
        let importance = self.calculate_importance(object, camera);

        // Performance-based adjustment
        let performance_bias = self.performance_monitor.get_lod_bias();

        // Select LOD level
        let mut selected_level = 0;

        // Distance-based selection
        for (i, &threshold) in config.distances.iter().enumerate().skip(1) {
            if distance > threshold * (1.0 + performance_bias) {
                selected_level = i;
            } else {
                break;
            }
        }

        // Screen space override
        for (i, &threshold) in config.screen_space_thresholds.iter().enumerate() {
            if screen_size < threshold {
                selected_level = selected_level.max(i);
            }
        }

        // Importance override
        if importance > 0.8 {
            selected_level = selected_level.saturating_sub(1);
        }

        LodSelection {
            level: selected_level,
            quality: config.quality_levels[selected_level].clone(),
            transition: self.calculate_transition(object, selected_level),
        }
    }

    fn calculate_screen_size(
        &self,
        object: &LodObject,
        camera: &Camera,
        viewport: &Viewport,
    ) -> f32 {
        let distance = (object.center - camera.position()).magnitude();
        let angular_size = 2.0 * (object.radius / distance).atan();
        let screen_size = angular_size * viewport.height as f32 / camera.fov();

        screen_size
    }

    fn calculate_importance(&self, object: &LodObject, camera: &Camera) -> f32 {
        let mut importance = 0.5;

        // View direction importance
        let to_object = (object.center - camera.position()).normalize();
        let view_dot = camera.forward().dot(&to_object);
        importance += view_dot * 0.3;

        // Object-specific importance
        importance += object.base_importance * 0.2;

        importance.clamp(0.0, 1.0)
    }
}
```

### 2. Mesh LOD Generation

```rust
use std::sync::Arc;

pub struct MeshLodGenerator {
    simplifier: MeshSimplifier,
    quality_settings: QualitySettings,
}

pub struct MeshSimplifier {
    error_threshold: f32,
    preserve_boundaries: bool,
    preserve_uv_seams: bool,
}

pub struct LodMesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    error_metric: f32,
    bounding_sphere: BoundingSphere,
}

impl MeshLodGenerator {
    pub fn generate_lods(
        &self,
        base_mesh: &Mesh,
        lod_count: usize,
    ) -> Vec<LodMesh> {
        let mut lods = Vec::with_capacity(lod_count);

        // LOD 0 is the original mesh
        lods.push(LodMesh {
            vertices: base_mesh.vertices.clone(),
            indices: base_mesh.indices.clone(),
            error_metric: 0.0,
            bounding_sphere: base_mesh.bounding_sphere.clone(),
        });

        // Generate progressively simplified meshes
        let mut current_mesh = base_mesh.clone();

        for i in 1..lod_count {
            let target_ratio = self.calculate_reduction_ratio(i, lod_count);
            let target_vertices = (base_mesh.vertices.len() as f32 * target_ratio) as usize;

            let simplified = self.simplifier.simplify_mesh(
                &current_mesh,
                target_vertices,
            );

            let error_metric = self.calculate_error_metric(&base_mesh, &simplified);

            lods.push(LodMesh {
                vertices: simplified.vertices.clone(),
                indices: simplified.indices.clone(),
                error_metric,
                bounding_sphere: simplified.calculate_bounding_sphere(),
            });

            current_mesh = simplified;
        }

        lods
    }

    fn calculate_reduction_ratio(&self, lod_level: usize, total_levels: usize) -> f32 {
        // Exponential reduction
        let t = lod_level as f32 / (total_levels - 1) as f32;
        0.1_f32.powf(t)
    }
}

impl MeshSimplifier {
    pub fn simplify_mesh(
        &self,
        mesh: &Mesh,
        target_vertices: usize,
    ) -> Mesh {
        // Quadric error metric simplification
        let mut quadrics = self.compute_vertex_quadrics(mesh);
        let mut edge_heap = self.build_edge_heap(mesh, &quadrics);
        let mut vertex_map = (0..mesh.vertices.len()).collect::<Vec<_>>();
        let mut active_vertices = mesh.vertices.len();

        // Collapse edges until target reached
        while active_vertices > target_vertices && !edge_heap.is_empty() {
            let edge = edge_heap.pop().unwrap();

            if self.can_collapse_edge(&edge, mesh) {
                let new_vertex = self.calculate_optimal_position(&edge, &quadrics);
                self.collapse_edge(
                    &edge,
                    new_vertex,
                    &mut quadrics,
                    &mut vertex_map,
                    &mut edge_heap,
                );
                active_vertices -= 1;
            }
        }

        // Build simplified mesh
        self.build_simplified_mesh(mesh, &vertex_map)
    }

    fn compute_vertex_quadrics(&self, mesh: &Mesh) -> Vec<Quadric> {
        let mut quadrics = vec![Quadric::zero(); mesh.vertices.len()];

        // Accumulate face quadrics
        for face in mesh.indices.chunks(3) {
            let v0 = &mesh.vertices[face[0] as usize];
            let v1 = &mesh.vertices[face[1] as usize];
            let v2 = &mesh.vertices[face[2] as usize];

            let face_quadric = Quadric::from_triangle(
                &v0.position,
                &v1.position,
                &v2.position,
            );

            for &idx in face {
                quadrics[idx as usize] += face_quadric;
            }
        }

        // Add boundary preservation constraints
        if self.preserve_boundaries {
            self.add_boundary_constraints(&mut quadrics, mesh);
        }

        quadrics
    }
}

#[derive(Debug, Clone, Copy)]
struct Quadric {
    matrix: [[f64; 4]; 4],
}

impl Quadric {
    fn zero() -> Self {
        Self {
            matrix: [[0.0; 4]; 4],
        }
    }

    fn from_triangle(v0: &Vector3<f32>, v1: &Vector3<f32>, v2: &Vector3<f32>) -> Self {
        // Calculate plane equation
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let normal = edge1.cross(&edge2).normalize();
        let d = -normal.dot(v0);

        // Build quadric matrix
        let a = normal.x as f64;
        let b = normal.y as f64;
        let c = normal.z as f64;
        let d = d as f64;

        Self {
            matrix: [
                [a*a, a*b, a*c, a*d],
                [a*b, b*b, b*c, b*d],
                [a*c, b*c, c*c, c*d],
                [a*d, b*d, c*d, d*d],
            ],
        }
    }

    fn evaluate(&self, pos: &Vector3<f32>) -> f64 {
        let v = [pos.x as f64, pos.y as f64, pos.z as f64, 1.0];
        let mut result = 0.0;

        for i in 0..4 {
            for j in 0..4 {
                result += v[i] * self.matrix[i][j] * v[j];
            }
        }

        result
    }
}
```

### 3. Terrain LOD System

```rust
pub struct TerrainLodSystem {
    chunk_lods: HashMap<ChunkId, TerrainLod>,
    height_map_cache: LruCache<ChunkId, HeightMap>,
    normal_cache: LruCache<ChunkId, NormalMap>,
}

pub struct TerrainLod {
    levels: Vec<TerrainLodLevel>,
    current_level: usize,
    blend_factor: f32,
}

pub struct TerrainLodLevel {
    vertex_grid_size: usize,
    height_data: Vec<f32>,
    normal_data: Vec<Vector3<f32>>,
    index_buffer: Arc<Buffer>,
    skirt_indices: Option<Vec<u32>>,
}

impl TerrainLodSystem {
    pub fn generate_terrain_lod(
        &mut self,
        chunk: &TerrainChunk,
        camera: &Camera,
    ) -> TerrainRenderData {
        let chunk_center = chunk.get_center();
        let distance = (chunk_center - camera.position()).magnitude();

        // Select LOD level based on distance
        let lod_level = self.select_terrain_lod_level(distance);

        // Get or generate LOD data
        let lod_data = self.chunk_lods
            .entry(chunk.id)
            .or_insert_with(|| self.generate_chunk_lods(chunk));

        // Handle LOD transition
        if lod_data.current_level != lod_level {
            lod_data.blend_factor = 0.0;
            lod_data.current_level = lod_level;
        } else {
            lod_data.blend_factor = (lod_data.blend_factor + 0.02).min(1.0);
        }

        // Generate render data
        self.create_terrain_render_data(chunk, lod_data, lod_level)
    }

    fn generate_chunk_lods(&self, chunk: &TerrainChunk) -> TerrainLod {
        let mut levels = Vec::new();

        // Generate different LOD levels
        for i in 0..5 {
            let grid_size = 33 >> i; // 33, 17, 9, 5, 3
            let level = self.generate_lod_level(chunk, grid_size);
            levels.push(level);
        }

        TerrainLod {
            levels,
            current_level: 0,
            blend_factor: 1.0,
        }
    }

    fn generate_lod_level(
        &self,
        chunk: &TerrainChunk,
        grid_size: usize,
    ) -> TerrainLodLevel {
        let step = 32 / (grid_size - 1);
        let mut height_data = Vec::with_capacity(grid_size * grid_size);
        let mut normal_data = Vec::with_capacity(grid_size * grid_size);

        // Sample height map at reduced resolution
        for y in 0..grid_size {
            for x in 0..grid_size {
                let src_x = x * step;
                let src_y = y * step;

                let height = chunk.sample_height(src_x, src_y);
                let normal = chunk.calculate_normal(src_x, src_y);

                height_data.push(height);
                normal_data.push(normal);
            }
        }

        // Generate index buffer with proper triangulation
        let (indices, skirt_indices) = self.generate_terrain_indices(grid_size);

        TerrainLodLevel {
            vertex_grid_size: grid_size,
            height_data,
            normal_data,
            index_buffer: Arc::new(create_index_buffer(&indices)),
            skirt_indices: Some(skirt_indices),
        }
    }

    fn generate_terrain_indices(
        &self,
        grid_size: usize,
    ) -> (Vec<u32>, Vec<u32>) {
        let mut indices = Vec::new();
        let mut skirt_indices = Vec::new();

        // Main terrain triangles
        for y in 0..grid_size - 1 {
            for x in 0..grid_size - 1 {
                let tl = (y * grid_size + x) as u32;
                let tr = tl + 1;
                let bl = tl + grid_size as u32;
                let br = bl + 1;

                // Two triangles per quad
                indices.extend_from_slice(&[tl, bl, br, tl, br, tr]);
            }
        }

        // Skirt triangles to hide gaps between LOD levels
        let skirt_start = (grid_size * grid_size) as u32;

        // Top edge
        for x in 0..grid_size - 1 {
            let edge = x as u32;
            let skirt = skirt_start + x as u32;
            skirt_indices.extend_from_slice(&[edge, edge + 1, skirt + 1, edge, skirt + 1, skirt]);
        }

        // Similar for other edges...

        (indices, skirt_indices)
    }
}
```

### 4. Shader LOD System

```rust
pub struct ShaderLodSystem {
    shader_variants: HashMap<ShaderKey, ShaderProgram>,
    active_shaders: HashMap<MaterialId, ShaderKey>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ShaderKey {
    base_shader: ShaderType,
    complexity: ShaderComplexity,
    features: ShaderFeatures,
}

bitflags! {
    pub struct ShaderFeatures: u32 {
        const NORMAL_MAPPING = 0x01;
        const SPECULAR = 0x02;
        const SHADOWS = 0x04;
        const FOG = 0x08;
        const SKINNING = 0x10;
        const VERTEX_COLOR = 0x20;
        const TEXTURE_ANIMATION = 0x40;
        const ENVIRONMENT_MAP = 0x80;
    }
}

impl ShaderLodSystem {
    pub fn get_shader_for_lod(
        &self,
        material: &Material,
        lod_quality: &QualityLevel,
    ) -> &ShaderProgram {
        let features = self.determine_features(material, lod_quality);

        let key = ShaderKey {
            base_shader: material.shader_type,
            complexity: lod_quality.shader_complexity,
            features,
        };

        &self.shader_variants[&key]
    }

    fn determine_features(
        &self,
        material: &Material,
        quality: &QualityLevel,
    ) -> ShaderFeatures {
        let mut features = ShaderFeatures::empty();

        // Add features based on quality level
        match quality.shader_complexity {
            ShaderComplexity::Ultra => {
                features |= ShaderFeatures::NORMAL_MAPPING;
                features |= ShaderFeatures::SPECULAR;
                features |= ShaderFeatures::SHADOWS;
                features |= ShaderFeatures::FOG;
                features |= ShaderFeatures::ENVIRONMENT_MAP;
            }
            ShaderComplexity::High => {
                features |= ShaderFeatures::SPECULAR;
                features |= ShaderFeatures::SHADOWS;
                features |= ShaderFeatures::FOG;
            }
            ShaderComplexity::Medium => {
                features |= ShaderFeatures::FOG;
            }
            _ => {}
        }

        // Always include certain features
        if material.has_vertex_colors {
            features |= ShaderFeatures::VERTEX_COLOR;
        }

        features
    }

    pub fn generate_shader_variant(
        &mut self,
        key: &ShaderKey,
    ) -> ShaderProgram {
        let mut preprocessor = ShaderPreprocessor::new();

        // Set defines based on features
        if key.features.contains(ShaderFeatures::NORMAL_MAPPING) {
            preprocessor.define("USE_NORMAL_MAPPING", "1");
        }

        if key.features.contains(ShaderFeatures::SPECULAR) {
            preprocessor.define("USE_SPECULAR", "1");
        }

        if key.features.contains(ShaderFeatures::SHADOWS) {
            preprocessor.define("USE_SHADOWS", "1");
            preprocessor.define("SHADOW_CASCADE_COUNT", "4");
        }

        // Select shader template based on complexity
        let template = match key.complexity {
            ShaderComplexity::Ultra => include_str!("shaders/pbr.wgsl"),
            ShaderComplexity::High => include_str!("shaders/phong.wgsl"),
            ShaderComplexity::Medium => include_str!("shaders/lambert.wgsl"),
            ShaderComplexity::Low => include_str!("shaders/simple.wgsl"),
            ShaderComplexity::Minimal => include_str!("shaders/flat.wgsl"),
        };

        let processed = preprocessor.process(template);
        compile_shader(&processed)
    }
}
```

### 5. Animation LOD System

```rust
pub struct AnimationLodSystem {
    bone_importance: HashMap<BoneId, f32>,
    lod_configs: Vec<AnimationLodConfig>,
}

#[derive(Debug, Clone)]
pub struct AnimationLodConfig {
    max_bones: usize,
    update_rate: f32,
    blend_quality: BlendQuality,
    ik_enabled: bool,
    procedural_enabled: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum AnimationQuality {
    Full,
    Reduced,
    Minimal,
    None,
}

impl AnimationLodSystem {
    pub fn optimize_animation(
        &self,
        skeleton: &Skeleton,
        animation: &Animation,
        quality: AnimationQuality,
    ) -> OptimizedAnimation {
        match quality {
            AnimationQuality::Full => {
                OptimizedAnimation {
                    active_bones: (0..skeleton.bones.len()).collect(),
                    update_rate: 60.0,
                    interpolation: InterpolationQuality::High,
                }
            }
            AnimationQuality::Reduced => {
                let active_bones = self.select_important_bones(skeleton, 30);
                OptimizedAnimation {
                    active_bones,
                    update_rate: 30.0,
                    interpolation: InterpolationQuality::Medium,
                }
            }
            AnimationQuality::Minimal => {
                let active_bones = self.select_important_bones(skeleton, 10);
                OptimizedAnimation {
                    active_bones,
                    update_rate: 15.0,
                    interpolation: InterpolationQuality::Low,
                }
            }
            AnimationQuality::None => {
                OptimizedAnimation {
                    active_bones: vec![0], // Root only
                    update_rate: 0.0,
                    interpolation: InterpolationQuality::None,
                }
            }
        }
    }

    fn select_important_bones(
        &self,
        skeleton: &Skeleton,
        max_bones: usize,
    ) -> Vec<usize> {
        // Sort bones by importance
        let mut bone_scores: Vec<(usize, f32)> = skeleton.bones
            .iter()
            .enumerate()
            .map(|(idx, bone)| {
                let base_importance = self.bone_importance
                    .get(&bone.id)
                    .copied()
                    .unwrap_or(0.5);

                // Factor in bone hierarchy depth
                let depth_factor = 1.0 / (bone.depth as f32 + 1.0);

                // Factor in number of vertices influenced
                let influence_factor = (bone.vertex_count as f32 / 1000.0).min(1.0);

                let score = base_importance * 0.5 +
                           depth_factor * 0.25 +
                           influence_factor * 0.25;

                (idx, score)
            })
            .collect();

        bone_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Always include root and important bones
        let mut selected = vec![0]; // Root

        for (bone_idx, _) in bone_scores.iter().take(max_bones - 1) {
            if *bone_idx != 0 {
                selected.push(*bone_idx);
            }
        }

        // Ensure parent bones are included
        self.ensure_bone_hierarchy(&mut selected, skeleton);

        selected
    }

    fn ensure_bone_hierarchy(
        &self,
        bones: &mut Vec<usize>,
        skeleton: &Skeleton,
    ) {
        let mut to_add = Vec::new();

        for &bone_idx in bones.iter() {
            let mut current = skeleton.bones[bone_idx].parent;

            while let Some(parent_idx) = current {
                if !bones.contains(&parent_idx) && !to_add.contains(&parent_idx) {
                    to_add.push(parent_idx);
                }
                current = skeleton.bones[parent_idx].parent;
            }
        }

        bones.extend(to_add);
        bones.sort();
        bones.dedup();
    }
}
```

### 6. Dynamic LOD Adjustment

```rust
pub struct DynamicLodController {
    target_frametime: f32,
    current_bias: f32,
    adjustment_rate: f32,
    history: VecDeque<FrameMetrics>,
}

#[derive(Debug, Clone)]
struct FrameMetrics {
    frame_time: f32,
    draw_calls: u32,
    triangles: u32,
    texture_memory: usize,
}

impl DynamicLodController {
    pub fn new(target_fps: f32) -> Self {
        Self {
            target_frametime: 1.0 / target_fps,
            current_bias: 0.0,
            adjustment_rate: 0.1,
            history: VecDeque::with_capacity(120),
        }
    }

    pub fn update(&mut self, metrics: FrameMetrics) {
        self.history.push_back(metrics);
        if self.history.len() > 120 {
            self.history.pop_front();
        }

        // Calculate average frame time
        let avg_frametime = self.history
            .iter()
            .map(|m| m.frame_time)
            .sum::<f32>() / self.history.len() as f32;

        // Adjust LOD bias based on performance
        if avg_frametime > self.target_frametime * 1.1 {
            // Performance too low, increase LOD bias
            self.current_bias = (self.current_bias + self.adjustment_rate).min(1.0);
        } else if avg_frametime < self.target_frametime * 0.9 {
            // Performance too high, decrease LOD bias
            self.current_bias = (self.current_bias - self.adjustment_rate).max(-0.5);
        }

        // Gradual adjustment to avoid sudden changes
        self.adjustment_rate = if (avg_frametime - self.target_frametime).abs() > 0.01 {
            0.1
        } else {
            0.02
        };
    }

    pub fn get_adjusted_distance(&self, base_distance: f32) -> f32 {
        base_distance * (1.0 + self.current_bias)
    }

    pub fn get_quality_multiplier(&self) -> f32 {
        1.0 - self.current_bias.max(0.0)
    }
}
```

## Code Examples

### Complete LOD Manager

```rust
pub struct LodManager {
    lod_system: LodSystem,
    mesh_lods: HashMap<MeshId, Vec<LodMesh>>,
    terrain_lod: TerrainLodSystem,
    shader_lod: ShaderLodSystem,
    animation_lod: AnimationLodSystem,
    dynamic_controller: DynamicLodController,
    transition_manager: TransitionManager,
}

impl LodManager {
    pub fn new(config: LodConfig) -> Self {
        Self {
            lod_system: LodSystem::new(),
            mesh_lods: HashMap::new(),
            terrain_lod: TerrainLodSystem::new(),
            shader_lod: ShaderLodSystem::new(),
            animation_lod: AnimationLodSystem::new(),
            dynamic_controller: DynamicLodController::new(config.target_fps),
            transition_manager: TransitionManager::new(),
        }
    }

    pub fn prepare_frame(
        &mut self,
        scene: &Scene,
        camera: &Camera,
        viewport: &Viewport,
        frame_metrics: FrameMetrics,
    ) -> LodFrame {
        // Update dynamic LOD adjustment
        self.dynamic_controller.update(frame_metrics);

        let mut lod_frame = LodFrame::new();

        // Process each object in the scene
        for object in &scene.objects {
            let lod_object = LodObject {
                id: object.id,
                category: object.get_lod_category(),
                center: object.get_center(),
                radius: object.get_radius(),
                base_importance: object.importance,
            };

            let selection = self.lod_system.select_lod(
                &lod_object,
                camera,
                viewport,
            );

            // Get appropriate mesh LOD
            if let Some(mesh_lods) = self.mesh_lods.get(&object.mesh_id) {
                let mesh_lod = &mesh_lods[selection.level.min(mesh_lods.len() - 1)];

                // Handle LOD transition
                let transition_state = self.transition_manager.update_transition(
                    object.id,
                    selection.level,
                    selection.transition,
                );

                lod_frame.add_object(LodRenderObject {
                    object_id: object.id,
                    mesh: mesh_lod.clone(),
                    shader_key: self.shader_lod.get_shader_key(&object.material, &selection.quality),
                    animation_quality: selection.quality.animation_quality,
                    transition_state,
                });
            }
        }

        // Process terrain
        for chunk in &scene.terrain_chunks {
            let terrain_data = self.terrain_lod.generate_terrain_lod(
                chunk,
                camera,
            );
            lod_frame.add_terrain(terrain_data);
        }

        lod_frame
    }

    pub fn pregenerate_lods(&mut self, meshes: &[Mesh]) {
        let generator = MeshLodGenerator::new();

        for mesh in meshes {
            let lods = generator.generate_lods(mesh, 5);
            self.mesh_lods.insert(mesh.id, lods);
        }
    }
}

pub struct LodFrame {
    render_objects: Vec<LodRenderObject>,
    terrain_chunks: Vec<TerrainRenderData>,
    statistics: LodStatistics,
}

pub struct LodRenderObject {
    object_id: ObjectId,
    mesh: LodMesh,
    shader_key: ShaderKey,
    animation_quality: AnimationQuality,
    transition_state: TransitionState,
}
```

### LOD Transition Effects

```rust
pub struct TransitionManager {
    transitions: HashMap<ObjectId, TransitionState>,
    fade_renderer: FadeTransitionRenderer,
    morph_renderer: MorphTransitionRenderer,
}

#[derive(Debug, Clone)]
pub struct TransitionState {
    from_level: usize,
    to_level: usize,
    progress: f32,
    transition_type: TransitionType,
}

impl TransitionManager {
    pub fn update_transition(
        &mut self,
        object_id: ObjectId,
        new_level: usize,
        transition_type: TransitionType,
    ) -> TransitionState {
        let state = self.transitions.entry(object_id).or_insert_with(|| {
            TransitionState {
                from_level: new_level,
                to_level: new_level,
                progress: 1.0,
                transition_type,
            }
        });

        if state.to_level != new_level {
            // Start new transition
            state.from_level = state.to_level;
            state.to_level = new_level;
            state.progress = 0.0;
            state.transition_type = transition_type;
        } else if state.progress < 1.0 {
            // Update ongoing transition
            match state.transition_type {
                TransitionType::Instant => state.progress = 1.0,
                TransitionType::Fade(duration) => {
                    state.progress = (state.progress + 0.016 / duration).min(1.0);
                }
                TransitionType::Morph(duration) => {
                    state.progress = (state.progress + 0.016 / duration).min(1.0);
                }
                TransitionType::Dither => {
                    state.progress = (state.progress + 0.1).min(1.0);
                }
            }
        }

        state.clone()
    }
}

// Shader for fade transition
fn fade_transition_shader() -> &'static str {
    r#"
    struct TransitionUniforms {
        fade_factor: f32,
        _padding: vec3<f32>,
    }

    @group(3) @binding(0)
    var<uniform> transition: TransitionUniforms;

    @fragment
    fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
        var color = standard_shading(in);

        // Apply fade
        color.a *= transition.fade_factor;

        // Alpha test for dithering
        if (color.a < 0.01) {
            discard;
        }

        return color;
    }
    "#
}

// Shader for morph transition
fn morph_transition_shader() -> &'static str {
    r#"
    struct MorphUniforms {
        morph_factor: f32,
        _padding: vec3<f32>,
    }

    @group(3) @binding(0)
    var<uniform> morph: MorphUniforms;

    @group(3) @binding(1)
    var<storage, read> target_positions: array<vec3<f32>>;

    @vertex
    fn vs_main(in: VertexInput) -> VertexOutput {
        // Morph between LOD levels
        let morphed_position = mix(
            in.position,
            target_positions[in.vertex_index],
            morph.morph_factor
        );

        var out: VertexOutput;
        out.position = transform_position(morphed_position);
        // ... rest of vertex shader

        return out;
    }
    "#
}
```

## Best Practices

### 1. LOD Grouping

```rust
pub struct LodGroup {
    objects: Vec<ObjectId>,
    combined_bounds: BoundingBox,
    representative: ObjectId,
}

impl LodGroup {
    pub fn create_groups(objects: &[SceneObject], max_group_size: f32) -> Vec<LodGroup> {
        let mut groups = Vec::new();
        let mut processed = HashSet::new();

        for object in objects {
            if processed.contains(&object.id) {
                continue;
            }

            let mut group = LodGroup {
                objects: vec![object.id],
                combined_bounds: object.bounds.clone(),
                representative: object.id,
            };

            // Find nearby objects to group
            for other in objects {
                if processed.contains(&other.id) || other.id == object.id {
                    continue;
                }

                let distance = (object.position - other.position).magnitude();
                if distance < max_group_size {
                    group.objects.push(other.id);
                    group.combined_bounds.expand(&other.bounds);
                    processed.insert(other.id);
                }
            }

            processed.insert(object.id);
            groups.push(group);
        }

        groups
    }
}
```

### 2. LOD Prediction

```rust
pub struct LodPredictor {
    movement_history: HashMap<ObjectId, VecDeque<Vector3<f32>>>,
    prediction_frames: usize,
}

impl LodPredictor {
    pub fn predict_lod(
        &mut self,
        object: &SceneObject,
        camera: &Camera,
        prediction_time: f32,
    ) -> usize {
        // Track object movement
        let history = self.movement_history
            .entry(object.id)
            .or_insert_with(|| VecDeque::with_capacity(10));

        history.push_back(object.position);
        if history.len() > 10 {
            history.pop_front();
        }

        // Predict future position
        let velocity = self.calculate_velocity(history);
        let future_position = object.position + velocity * prediction_time;

        // Calculate LOD for predicted position
        let future_distance = (future_position - camera.position()).magnitude();
        self.distance_to_lod(future_distance)
    }

    fn calculate_velocity(&self, history: &VecDeque<Vector3<f32>>) -> Vector3<f32> {
        if history.len() < 2 {
            return Vector3::zeros();
        }

        let recent = history.back().unwrap();
        let previous = history.get(history.len() - 2).unwrap();

        (recent - previous) * 60.0 // Assuming 60 FPS
    }
}
```

### 3. Memory-Aware LOD

```rust
pub struct MemoryAwareLod {
    memory_budget: usize,
    current_usage: AtomicUsize,
    lod_memory_costs: HashMap<(MeshId, usize), usize>,
}

impl MemoryAwareLod {
    pub fn adjust_lod_for_memory(
        &self,
        base_lod: usize,
        mesh_id: MeshId,
    ) -> usize {
        let current = self.current_usage.load(Ordering::Relaxed);

        if current > self.memory_budget {
            // Force higher LOD (lower quality) to save memory
            (base_lod + 1).min(4)
        } else if current < self.memory_budget / 2 {
            // Allow lower LOD (higher quality) if memory available
            base_lod.saturating_sub(1)
        } else {
            base_lod
        }
    }

    pub fn track_lod_memory(&self, mesh_id: MeshId, lod_level: usize, size: usize) {
        self.lod_memory_costs.insert((mesh_id, lod_level), size);
    }
}
```

## Common Issues and Solutions

### Issue: LOD Popping

**Problem**: Visible transitions between LOD levels.

**Solution**:

```rust
pub fn smooth_lod_transition(
    current_lod: f32,
    target_lod: f32,
    delta_time: f32,
) -> f32 {
    // Use smooth step function
    let transition_speed = 2.0;
    let diff = target_lod - current_lod;

    if diff.abs() < 0.01 {
        target_lod
    } else {
        current_lod + diff * (transition_speed * delta_time).min(1.0)
    }
}

// In shader: blend between LOD levels
fn blend_lod_levels(
    lod1_output: vec4<f32>,
    lod2_output: vec4<f32>,
    blend_factor: f32,
) -> vec4<f32> {
    // Smooth interpolation
    let t = smoothstep(0.0, 1.0, blend_factor);
    return mix(lod1_output, lod2_output, t);
}
```

### Issue: Terrain Cracks

**Problem**: Gaps between different terrain LOD levels.

**Solution**:

```rust
pub fn generate_terrain_skirts(
    chunk: &TerrainChunk,
    lod_level: usize,
) -> Vec<SkirtVertex> {
    let mut skirt_vertices = Vec::new();
    let skirt_depth = 10.0; // Units below terrain

    // Generate skirt vertices for each edge
    for edge in &[Edge::North, Edge::South, Edge::East, Edge::West] {
        let edge_vertices = chunk.get_edge_vertices(*edge, lod_level);

        for vertex in edge_vertices {
            // Original vertex
            skirt_vertices.push(vertex.clone());

            // Skirt vertex (pushed down)
            let mut skirt_vertex = vertex.clone();
            skirt_vertex.position.y -= skirt_depth;
            skirt_vertices.push(skirt_vertex);
        }
    }

    skirt_vertices
}
```

### Issue: Animation Jitter at Distance

**Problem**: Animations look jerky on distant objects.

**Solution**:

```rust
pub struct AnimationRateController {
    base_rates: HashMap<AnimationQuality, f32>,
}

impl AnimationRateController {
    pub fn get_update_rate(
        &self,
        quality: AnimationQuality,
        distance: f32,
        importance: f32,
    ) -> f32 {
        let base_rate = self.base_rates[&quality];

        // Interpolate update rate based on distance
        let distance_factor = 1.0 - (distance / 200.0).min(1.0).powf(2.0);

        // Boost rate for important objects
        let importance_boost = 1.0 + importance * 0.5;

        base_rate * distance_factor * importance_boost
    }

    pub fn should_update_frame(
        &self,
        last_update: f32,
        current_time: f32,
        update_rate: f32,
    ) -> bool {
        if update_rate <= 0.0 {
            return false;
        }

        let frame_interval = 1.0 / update_rate;
        current_time - last_update >= frame_interval
    }
}
```

## Performance Tips

### 1. Hierarchical LOD

```rust
pub struct HierarchicalLod {
    octree: Octree<LodNode>,
}

struct LodNode {
    objects: Vec<ObjectId>,
    combined_lod: usize,
    bounds: BoundingBox,
}

impl HierarchicalLod {
    pub fn update_hierarchical_lod(
        &mut self,
        camera: &Camera,
    ) {
        self.octree.traverse_mut(|node, depth| {
            let distance = node.bounds.distance_to_point(&camera.position());

            // Higher levels in octree can use lower detail
            let depth_bias = depth as f32 * 0.5;
            let adjusted_distance = distance + depth_bias * 50.0;

            node.combined_lod = self.distance_to_lod(adjusted_distance);

            // Stop traversing if entire node is beyond max LOD
            adjusted_distance < 1000.0
        });
    }
}
```

### 2. Predictive LOD Loading

```rust
pub struct PredictiveLodLoader {
    loading_queue: Arc<Mutex<BinaryHeap<LoadRequest>>>,
    loaded_lods: Arc<RwLock<HashMap<(MeshId, usize), Arc<LodMesh>>>>,
}

impl PredictiveLodLoader {
    pub fn predict_and_load(
        &self,
        camera_path: &CameraPath,
        scene_objects: &[SceneObject],
    ) {
        let future_positions = camera_path.sample_future_positions(5.0);

        for position in future_positions {
            for object in scene_objects {
                let distance = (object.position - position).magnitude();
                let predicted_lod = self.distance_to_lod(distance);

                // Queue LOD for loading if not already loaded
                let key = (object.mesh_id, predicted_lod);
                if !self.loaded_lods.read().unwrap().contains_key(&key) {
                    self.queue_lod_load(object.mesh_id, predicted_lod, distance);
                }
            }
        }
    }
}
```

### 3. GPU-Based LOD Selection

```rust
// Compute shader for LOD selection
const LOD_SELECTION_SHADER: &str = r#"
@group(0) @binding(0)
var<storage, read> objects: array<ObjectData>;

@group(0) @binding(1)
var<storage, write> lod_indices: array<u32>;

@group(0) @binding(2)
var<uniform> camera: CameraData;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    if (index >= arrayLength(&objects)) {
        return;
    }

    let object = objects[index];
    let distance = length(object.center - camera.position);

    // Simple distance-based LOD selection
    var lod = 0u;
    if (distance > 200.0) { lod = 4u; }
    else if (distance > 100.0) { lod = 3u; }
    else if (distance > 50.0) { lod = 2u; }
    else if (distance > 25.0) { lod = 1u; }

    // Screen-space size override
    let screen_size = object.radius / distance * camera.screen_height;
    if (screen_size < 10.0) { lod = max(lod, 3u); }

    lod_indices[index] = lod;
}
"#;
```

## Related Guides

- [🎨 Model Rendering Guide](./model-rendering.md) - Render models with LOD
- [🌍 Rendering ADT Terrain](./adt-rendering.md) - Terrain LOD implementation
- [🎬 Animation System Guide](./animation-system.md) - Animation LOD techniques
- [🏛️ WMO Rendering Guide](./wmo-rendering.md) - Building LOD strategies

## References

- [Level of Detail for 3D Graphics](https://www.cs.unc.edu/~luebke/papers/book.html) - LOD reference book
- [View-Dependent Rendering](https://graphics.stanford.edu/papers/levoy-vdr/) - Advanced LOD techniques
- [Nanite Virtualized Geometry](https://advances.realtimerendering.com/s2021/) - LOD approaches
- [Mesh Optimization](https://github.com/zeux/meshoptimizer) - Mesh simplification algorithms
