# 🎬 Animation System Guide

## Overview

World of Warcraft's animation system is sophisticated, supporting skeletal animations,
morph targets, texture animations, and complex blending. This guide covers
implementing a complete animation system using `warcraft-rs`, including bone
hierarchies, animation tracks, blending, and advanced features like animation
events and facial expressions.

## Prerequisites

Before implementing the animation system, ensure you have:

- Strong understanding of skeletal animation concepts
- Knowledge of quaternion math and matrix transformations
- `warcraft-rs` installed with animation support
- Familiarity with interpolation techniques
- Understanding of animation state machines

## Understanding WoW Animation System

### Animation Components

- **Bones**: Hierarchical skeleton structure
- **Animation Sequences**: Named animations with timing data
- **Keyframes**: Transform data at specific times
- **Tracks**: Separate channels for translation, rotation, scale
- **Interpolation**: Linear, hermite, or bezier curves
- **Global Sequences**: Looping animations (texture scrolling, etc.)
- **Animation Lookup**: Mapping animation IDs to sequences

### Animation Types

- **Character Animations**: Walk, run, attack, idle, etc.
- **Facial Animations**: Expressions and lip sync
- **Texture Animations**: UV scrolling and transformations
- **Particle Animations**: Emitter behavior over time
- **Camera Animations**: Cutscene camera movements

## Step-by-Step Instructions

### 1. Building the Animation System Core

```rust
use nalgebra::{Vector3, Quaternion, Matrix4, Unit};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AnimationSystem {
    skeletons: HashMap<String, Skeleton>,
    animations: HashMap<String, AnimationClip>,
    blend_trees: HashMap<String, BlendTree>,
    global_time: f32,
}

#[derive(Debug, Clone)]
pub struct Skeleton {
    bones: Vec<Bone>,
    bone_names: HashMap<String, usize>,
    rest_pose: Vec<Transform>,
}

#[derive(Debug, Clone)]
pub struct Bone {
    name: String,
    parent: Option<usize>,
    flags: BoneFlags,
    pivot: Vector3<f32>,
}

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    translation: Vector3<f32>,
    rotation: Unit<Quaternion<f32>>,
    scale: Vector3<f32>,
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            translation: Vector3::zeros(),
            rotation: Unit::new_unchecked(Quaternion::identity()),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn to_matrix(&self) -> Matrix4<f32> {
        let t = Matrix4::new_translation(&self.translation);
        let r = self.rotation.to_homogeneous();
        let s = Matrix4::new_nonuniform_scaling(&self.scale);
        t * r * s
    }

    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        Self {
            translation: self.translation.lerp(&other.translation, t),
            rotation: Unit::new_unchecked(self.rotation.slerp(&other.rotation, t)),
            scale: self.scale.lerp(&other.scale, t),
        }
    }
}
```

### 2. Animation Tracks and Keyframes

```rust
use warcraft_rs::m2::{AnimationBlock, InterpolationType};

#[derive(Debug, Clone)]
pub struct AnimationClip {
    name: String,
    duration: u32, // milliseconds
    loop_mode: LoopMode,
    bone_tracks: Vec<BoneTrack>,
    events: Vec<AnimationEvent>,
}

#[derive(Debug, Clone)]
pub struct BoneTrack {
    bone_index: usize,
    translation: Track<Vector3<f32>>,
    rotation: Track<Quaternion<f32>>,
    scale: Track<Vector3<f32>>,
}

#[derive(Debug, Clone)]
pub struct Track<T> {
    keyframes: Vec<Keyframe<T>>,
    interpolation: InterpolationType,
}

#[derive(Debug, Clone)]
pub struct Keyframe<T> {
    time: u32,
    value: T,
    in_tangent: Option<T>,
    out_tangent: Option<T>,
}

#[derive(Debug, Clone, Copy)]
pub enum LoopMode {
    Once,
    Loop,
    PingPong,
    ClampForever,
}

impl<T: Interpolatable> Track<T> {
    pub fn sample(&self, time: u32, loop_mode: LoopMode, duration: u32) -> T {
        if self.keyframes.is_empty() {
            return T::default();
        }

        // Handle looping
        let time = match loop_mode {
            LoopMode::Once => time.min(duration),
            LoopMode::Loop => time % duration,
            LoopMode::PingPong => {
                let cycle = time / duration;
                if cycle % 2 == 0 {
                    time % duration
                } else {
                    duration - (time % duration)
                }
            }
            LoopMode::ClampForever => time.min(duration),
        };

        // Find surrounding keyframes
        let (prev, next) = self.find_keyframes(time);

        if prev == next {
            return self.keyframes[prev].value.clone();
        }

        // Calculate interpolation factor
        let prev_key = &self.keyframes[prev];
        let next_key = &self.keyframes[next];
        let t = (time - prev_key.time) as f32 / (next_key.time - prev_key.time) as f32;

        // Interpolate based on type
        match self.interpolation {
            InterpolationType::None => prev_key.value.clone(),
            InterpolationType::Linear => {
                prev_key.value.lerp(&next_key.value, t)
            }
            InterpolationType::Hermite => {
                self.hermite_interpolate(prev, next, t)
            }
            InterpolationType::Bezier => {
                self.bezier_interpolate(prev, next, t)
            }
        }
    }

    fn find_keyframes(&self, time: u32) -> (usize, usize) {
        // Binary search for efficiency
        let pos = self.keyframes.binary_search_by_key(&time, |k| k.time);

        match pos {
            Ok(idx) => (idx, idx),
            Err(idx) => {
                if idx == 0 {
                    (0, 0)
                } else if idx >= self.keyframes.len() {
                    let last = self.keyframes.len() - 1;
                    (last, last)
                } else {
                    (idx - 1, idx)
                }
            }
        }
    }

    fn hermite_interpolate(&self, prev_idx: usize, next_idx: usize, t: f32) -> T {
        let p0 = &self.keyframes[prev_idx];
        let p1 = &self.keyframes[next_idx];

        let m0 = p0.out_tangent.as_ref().unwrap_or(&p0.value);
        let m1 = p1.in_tangent.as_ref().unwrap_or(&p1.value);

        // Hermite interpolation formula
        let t2 = t * t;
        let t3 = t2 * t;

        let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
        let h10 = t3 - 2.0 * t2 + t;
        let h01 = -2.0 * t3 + 3.0 * t2;
        let h11 = t3 - t2;

        p0.value.scale(h00)
            .add(&m0.scale(h10))
            .add(&p1.value.scale(h01))
            .add(&m1.scale(h11))
    }
}

trait Interpolatable: Clone + Default {
    fn lerp(&self, other: &Self, t: f32) -> Self;
    fn scale(&self, s: f32) -> Self;
    fn add(&self, other: &Self) -> Self;
}

impl Interpolatable for Vector3<f32> {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
    }

    fn scale(&self, s: f32) -> Self {
        self * s
    }

    fn add(&self, other: &Self) -> Self {
        self + other
    }
}

impl Interpolatable for Quaternion<f32> {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self.slerp(other, t)
    }

    fn scale(&self, s: f32) -> Self {
        self.powf(s)
    }

    fn add(&self, other: &Self) -> Self {
        self * other
    }
}
```

### 3. Animation State Machine

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct AnimationStateMachine {
    states: HashMap<String, AnimationState>,
    transitions: Vec<StateTransition>,
    current_state: String,
    parameters: HashMap<String, AnimationParameter>,
    transition_queue: VecDeque<TransitionInfo>,
}

#[derive(Debug, Clone)]
pub struct AnimationState {
    name: String,
    animation_clip: String,
    speed: f32,
    motion: Option<RootMotion>,
}

#[derive(Debug, Clone)]
pub struct StateTransition {
    from: String,
    to: String,
    duration: f32,
    conditions: Vec<TransitionCondition>,
}

#[derive(Debug, Clone)]
pub enum TransitionCondition {
    ParameterEquals(String, AnimationParameter),
    ParameterGreaterThan(String, f32),
    ParameterLessThan(String, f32),
    OnAnimationEnd,
}

#[derive(Debug, Clone)]
pub enum AnimationParameter {
    Float(f32),
    Int(i32),
    Bool(bool),
    Trigger(bool),
}

impl AnimationStateMachine {
    pub fn new(initial_state: String) -> Self {
        Self {
            states: HashMap::new(),
            transitions: Vec::new(),
            current_state: initial_state,
            parameters: HashMap::new(),
            transition_queue: VecDeque::new(),
        }
    }

    pub fn update(&mut self, delta_time: f32) -> Option<AnimationTransition> {
        // Check transition conditions
        self.check_transitions();

        // Process active transitions
        if let Some(mut transition) = self.transition_queue.front_mut() {
            transition.progress += delta_time / transition.duration;

            if transition.progress >= 1.0 {
                // Complete transition
                let completed = self.transition_queue.pop_front().unwrap();
                self.current_state = completed.to_state;

                return Some(AnimationTransition {
                    from: completed.from_state,
                    to: completed.to_state,
                    blend_factor: 1.0,
                });
            }

            return Some(AnimationTransition {
                from: transition.from_state.clone(),
                to: transition.to_state.clone(),
                blend_factor: transition.progress,
            });
        }

        None
    }

    fn check_transitions(&mut self) {
        for transition in &self.transitions {
            if transition.from != self.current_state {
                continue;
            }

            let mut all_conditions_met = true;

            for condition in &transition.conditions {
                if !self.evaluate_condition(condition) {
                    all_conditions_met = false;
                    break;
                }
            }

            if all_conditions_met {
                self.transition_queue.push_back(TransitionInfo {
                    from_state: transition.from.clone(),
                    to_state: transition.to.clone(),
                    duration: transition.duration,
                    progress: 0.0,
                });
                break;
            }
        }
    }

    fn evaluate_condition(&self, condition: &TransitionCondition) -> bool {
        match condition {
            TransitionCondition::ParameterEquals(name, expected) => {
                self.parameters.get(name) == Some(expected)
            }
            TransitionCondition::ParameterGreaterThan(name, threshold) => {
                if let Some(AnimationParameter::Float(value)) = self.parameters.get(name) {
                    value > threshold
                } else {
                    false
                }
            }
            TransitionCondition::ParameterLessThan(name, threshold) => {
                if let Some(AnimationParameter::Float(value)) = self.parameters.get(name) {
                    value < threshold
                } else {
                    false
                }
            }
            TransitionCondition::OnAnimationEnd => {
                // Check if current animation has ended
                false // Implement based on animation playback state
            }
        }
    }
}

#[derive(Debug, Clone)]
struct TransitionInfo {
    from_state: String,
    to_state: String,
    duration: f32,
    progress: f32,
}

#[derive(Debug, Clone)]
pub struct AnimationTransition {
    pub from: String,
    pub to: String,
    pub blend_factor: f32,
}
```

### 4. Animation Blending

```rust
#[derive(Debug, Clone)]
pub struct AnimationBlender {
    blend_mode: BlendMode,
    layers: Vec<AnimationLayer>,
}

#[derive(Debug, Clone)]
pub struct AnimationLayer {
    animation: String,
    weight: f32,
    mask: Option<BoneMask>,
    blend_mode: LayerBlendMode,
}

#[derive(Debug, Clone)]
pub struct BoneMask {
    bones: HashSet<usize>,
    include_descendants: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum BlendMode {
    Override,
    Additive,
    Blend,
}

#[derive(Debug, Clone, Copy)]
pub enum LayerBlendMode {
    Override,
    Additive,
    Multiply,
}

impl AnimationBlender {
    pub fn blend_animations(
        &self,
        animations: &HashMap<String, AnimationClip>,
        skeleton: &Skeleton,
        time: u32,
    ) -> Vec<Transform> {
        let bone_count = skeleton.bones.len();
        let mut final_transforms = vec![Transform::identity(); bone_count];
        let mut bone_weights = vec![0.0; bone_count];

        // Process each layer
        for layer in &self.layers {
            if layer.weight <= 0.0 {
                continue;
            }

            let animation = match animations.get(&layer.animation) {
                Some(anim) => anim,
                None => continue,
            };

            // Sample animation
            let layer_transforms = self.sample_animation(animation, skeleton, time);

            // Apply layer blending
            for bone_idx in 0..bone_count {
                // Check bone mask
                if let Some(mask) = &layer.mask {
                    if !self.is_bone_in_mask(bone_idx, mask, skeleton) {
                        continue;
                    }
                }

                let weight = layer.weight;

                match layer.blend_mode {
                    LayerBlendMode::Override => {
                        if bone_weights[bone_idx] < 1.0 {
                            let remaining = 1.0 - bone_weights[bone_idx];
                            let actual_weight = weight.min(remaining);

                            final_transforms[bone_idx] = final_transforms[bone_idx]
                                .interpolate(&layer_transforms[bone_idx], actual_weight);

                            bone_weights[bone_idx] += actual_weight;
                        }
                    }
                    LayerBlendMode::Additive => {
                        // Add to existing transform
                        let additive = layer_transforms[bone_idx];
                        final_transforms[bone_idx].translation += additive.translation * weight;

                        // Blend rotation additively
                        let added_rot = Quaternion::identity().slerp(&additive.rotation, weight);
                        final_transforms[bone_idx].rotation =
                            Unit::new_normalize(final_transforms[bone_idx].rotation.as_ref() * added_rot);
                    }
                    LayerBlendMode::Multiply => {
                        // Multiply transforms
                        final_transforms[bone_idx].scale.component_mul_assign(
                            &layer_transforms[bone_idx].scale.lerp(&Vector3::new(1.0, 1.0, 1.0), 1.0 - weight)
                        );
                    }
                }
            }
        }

        final_transforms
    }

    fn sample_animation(
        &self,
        animation: &AnimationClip,
        skeleton: &Skeleton,
        time: u32,
    ) -> Vec<Transform> {
        let mut transforms = skeleton.rest_pose.clone();

        for track in &animation.bone_tracks {
            let bone_idx = track.bone_index;

            transforms[bone_idx] = Transform {
                translation: track.translation.sample(time, animation.loop_mode, animation.duration),
                rotation: Unit::new_normalize(
                    track.rotation.sample(time, animation.loop_mode, animation.duration)
                ),
                scale: track.scale.sample(time, animation.loop_mode, animation.duration),
            };
        }

        transforms
    }

    fn is_bone_in_mask(&self, bone_idx: usize, mask: &BoneMask, skeleton: &Skeleton) -> bool {
        if mask.bones.contains(&bone_idx) {
            return true;
        }

        if mask.include_descendants {
            // Check if any ancestor is in the mask
            let mut current = bone_idx;
            while let Some(parent) = skeleton.bones[current].parent {
                if mask.bones.contains(&parent) {
                    return true;
                }
                current = parent;
            }
        }

        false
    }
}
```

### 5. Procedural Animation System

```rust
#[derive(Debug, Clone)]
pub struct ProceduralAnimator {
    ik_chains: Vec<IKChain>,
    physics_bones: Vec<PhysicsBone>,
    look_at_constraints: Vec<LookAtConstraint>,
}

#[derive(Debug, Clone)]
pub struct IKChain {
    end_effector: usize,
    chain_length: usize,
    target: Vector3<f32>,
    pole_target: Option<Vector3<f32>>,
    iterations: usize,
    tolerance: f32,
}

#[derive(Debug, Clone)]
pub struct PhysicsBone {
    bone_index: usize,
    mass: f32,
    damping: f32,
    stiffness: f32,
    gravity_scale: f32,
    velocity: Vector3<f32>,
    constraints: Vec<PhysicsConstraint>,
}

#[derive(Debug, Clone)]
pub struct LookAtConstraint {
    bone_index: usize,
    target: Vector3<f32>,
    up_vector: Vector3<f32>,
    weight: f32,
    limits: Option<RotationLimits>,
}

impl ProceduralAnimator {
    pub fn apply_procedural_animation(
        &mut self,
        transforms: &mut [Transform],
        skeleton: &Skeleton,
        world_matrices: &[Matrix4<f32>],
        delta_time: f32,
    ) {
        // Apply IK chains
        for chain in &self.ik_chains {
            self.solve_ik_chain(chain, transforms, skeleton, world_matrices);
        }

        // Apply physics simulation
        for physics_bone in &mut self.physics_bones {
            self.simulate_physics_bone(physics_bone, transforms, world_matrices, delta_time);
        }

        // Apply look-at constraints
        for constraint in &self.look_at_constraints {
            self.apply_look_at(constraint, transforms, world_matrices);
        }
    }

    fn solve_ik_chain(
        &self,
        chain: &IKChain,
        transforms: &mut [Transform],
        skeleton: &Skeleton,
        world_matrices: &[Matrix4<f32>],
    ) {
        // FABRIK (Forward And Backward Reaching Inverse Kinematics)
        let mut bone_indices = Vec::new();
        let mut current = chain.end_effector;

        // Build chain
        for _ in 0..chain.chain_length {
            bone_indices.push(current);
            if let Some(parent) = skeleton.bones[current].parent {
                current = parent;
            } else {
                break;
            }
        }

        bone_indices.reverse();

        // Store original positions
        let mut positions: Vec<Vector3<f32>> = bone_indices
            .iter()
            .map(|&idx| world_matrices[idx].transform_point(&Point3::origin()).coords)
            .collect();

        let base_pos = positions[0];

        // FABRIK iterations
        for _ in 0..chain.iterations {
            // Forward reaching
            positions[positions.len() - 1] = chain.target;

            for i in (0..positions.len() - 1).rev() {
                let direction = (positions[i] - positions[i + 1]).normalize();
                let bone_length = self.calculate_bone_length(&bone_indices, i, skeleton);
                positions[i] = positions[i + 1] + direction * bone_length;
            }

            // Backward reaching
            positions[0] = base_pos;

            for i in 0..positions.len() - 1 {
                let direction = (positions[i + 1] - positions[i]).normalize();
                let bone_length = self.calculate_bone_length(&bone_indices, i, skeleton);
                positions[i + 1] = positions[i] + direction * bone_length;
            }

            // Check tolerance
            let error = (positions[positions.len() - 1] - chain.target).magnitude();
            if error < chain.tolerance {
                break;
            }
        }

        // Apply rotations to achieve positions
        for i in 0..bone_indices.len() - 1 {
            let bone_idx = bone_indices[i];
            let child_idx = bone_indices[i + 1];

            // Calculate required rotation
            let current_dir = (world_matrices[child_idx].transform_point(&Point3::origin()) -
                             world_matrices[bone_idx].transform_point(&Point3::origin())).normalize();
            let target_dir = (positions[i + 1] - positions[i]).normalize();

            let rotation = Quaternion::rotation_between(&current_dir, &target_dir)
                .unwrap_or(Quaternion::identity());

            // Apply rotation in local space
            transforms[bone_idx].rotation = Unit::new_normalize(
                transforms[bone_idx].rotation.as_ref() * rotation
            );
        }
    }

    fn calculate_bone_length(&self, chain: &[usize], index: usize, skeleton: &Skeleton) -> f32 {
        if index >= chain.len() - 1 {
            return 0.0;
        }

        let bone = &skeleton.bones[chain[index]];
        let child = &skeleton.bones[chain[index + 1]];

        (child.pivot - bone.pivot).magnitude()
    }
}
```

### 6. Animation Events and Callbacks

```rust
#[derive(Debug, Clone)]
pub struct AnimationEvent {
    time: u32,
    event_type: AnimationEventType,
    parameters: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum AnimationEventType {
    Sound(String),
    Particle(String),
    Footstep(FootType),
    WeaponSwing,
    Custom(String),
}

#[derive(Debug, Clone, Copy)]
pub enum FootType {
    Left,
    Right,
}

pub struct AnimationEventHandler {
    handlers: HashMap<String, Box<dyn Fn(&AnimationEvent) + Send + Sync>>,
    queued_events: VecDeque<QueuedEvent>,
}

#[derive(Debug, Clone)]
struct QueuedEvent {
    event: AnimationEvent,
    fire_time: f32,
}

impl AnimationEventHandler {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            queued_events: VecDeque::new(),
        }
    }

    pub fn register_handler<F>(&mut self, event_type: &str, handler: F)
    where
        F: Fn(&AnimationEvent) + Send + Sync + 'static,
    {
        self.handlers.insert(event_type.to_string(), Box::new(handler));
    }

    pub fn process_animation_events(
        &mut self,
        animation: &AnimationClip,
        prev_time: u32,
        current_time: u32,
        global_time: f32,
    ) {
        // Handle looping
        let duration = animation.duration;

        match animation.loop_mode {
            LoopMode::Once => {
                self.collect_events_in_range(animation, prev_time, current_time, global_time);
            }
            LoopMode::Loop => {
                if current_time < prev_time {
                    // Wrapped around
                    self.collect_events_in_range(animation, prev_time, duration, global_time);
                    self.collect_events_in_range(animation, 0, current_time, global_time);
                } else {
                    self.collect_events_in_range(animation, prev_time, current_time, global_time);
                }
            }
            _ => {
                // Handle other loop modes
            }
        }

        // Fire queued events
        self.fire_ready_events(global_time);
    }

    fn collect_events_in_range(
        &mut self,
        animation: &AnimationClip,
        start_time: u32,
        end_time: u32,
        global_time: f32,
    ) {
        for event in &animation.events {
            if event.time > start_time && event.time <= end_time {
                self.queued_events.push_back(QueuedEvent {
                    event: event.clone(),
                    fire_time: global_time,
                });
            }
        }
    }

    fn fire_ready_events(&mut self, current_time: f32) {
        while let Some(queued) = self.queued_events.front() {
            if queued.fire_time <= current_time {
                let event = self.queued_events.pop_front().unwrap();
                self.fire_event(&event.event);
            } else {
                break;
            }
        }
    }

    fn fire_event(&self, event: &AnimationEvent) {
        let type_name = match &event.event_type {
            AnimationEventType::Sound(_) => "sound",
            AnimationEventType::Particle(_) => "particle",
            AnimationEventType::Footstep(_) => "footstep",
            AnimationEventType::WeaponSwing => "weapon_swing",
            AnimationEventType::Custom(name) => name,
        };

        if let Some(handler) = self.handlers.get(type_name) {
            handler(event);
        }
    }
}
```

## Code Examples

### Complete Animation Player

```rust
use warcraft_rs::m2::M2Model;

pub struct AnimationPlayer {
    model: Arc<M2Model>,
    skeleton: Skeleton,
    animation_system: AnimationSystem,
    state_machine: AnimationStateMachine,
    blender: AnimationBlender,
    event_handler: AnimationEventHandler,
    current_pose: Vec<Transform>,
    world_matrices: Vec<Matrix4<f32>>,
    animation_time: HashMap<String, u32>,
}

impl AnimationPlayer {
    pub fn new(model: Arc<M2Model>) -> Self {
        let skeleton = build_skeleton_from_m2(&model);
        let animations = load_animations_from_m2(&model);

        let mut animation_system = AnimationSystem::new();
        animation_system.skeletons.insert("main".to_string(), skeleton.clone());

        for (name, clip) in animations {
            animation_system.animations.insert(name, clip);
        }

        let bone_count = skeleton.bones.len();

        Self {
            model,
            skeleton,
            animation_system,
            state_machine: AnimationStateMachine::new("idle".to_string()),
            blender: AnimationBlender::default(),
            event_handler: AnimationEventHandler::new(),
            current_pose: vec![Transform::identity(); bone_count],
            world_matrices: vec![Matrix4::identity(); bone_count],
            animation_time: HashMap::new(),
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        // Update state machine
        if let Some(transition) = self.state_machine.update(delta_time) {
            // Handle state transition
            self.handle_transition(transition);
        }

        // Update animation times
        for (anim_name, time) in &mut self.animation_time {
            if let Some(animation) = self.animation_system.animations.get(anim_name) {
                let prev_time = *time;
                *time = (*time + (delta_time * 1000.0) as u32) % animation.duration;

                // Process animation events
                self.event_handler.process_animation_events(
                    animation,
                    prev_time,
                    *time,
                    self.animation_system.global_time,
                );
            }
        }

        // Blend animations
        self.current_pose = self.blender.blend_animations(
            &self.animation_system.animations,
            &self.skeleton,
            self.get_current_animation_time(),
        );

        // Calculate world matrices
        self.calculate_world_matrices();

        // Update global time
        self.animation_system.global_time += delta_time;
    }

    fn calculate_world_matrices(&mut self) {
        for (bone_idx, bone) in self.skeleton.bones.iter().enumerate() {
            let local_matrix = self.current_pose[bone_idx].to_matrix();

            self.world_matrices[bone_idx] = if let Some(parent_idx) = bone.parent {
                self.world_matrices[parent_idx] * local_matrix
            } else {
                local_matrix
            };
        }
    }

    pub fn play_animation(&mut self, name: &str, fade_in: f32) {
        self.blender.layers.clear();
        self.blender.layers.push(AnimationLayer {
            animation: name.to_string(),
            weight: 1.0,
            mask: None,
            blend_mode: LayerBlendMode::Override,
        });

        self.animation_time.insert(name.to_string(), 0);
    }

    pub fn add_animation_layer(&mut self, name: &str, weight: f32, mask: Option<BoneMask>) {
        self.blender.layers.push(AnimationLayer {
            animation: name.to_string(),
            weight,
            mask,
            blend_mode: LayerBlendMode::Override,
        });

        self.animation_time.entry(name.to_string()).or_insert(0);
    }

    pub fn get_bone_matrices(&self) -> &[Matrix4<f32>] {
        &self.world_matrices
    }

    pub fn set_animation_speed(&mut self, animation: &str, speed: f32) {
        if let Some(state) = self.state_machine.states.get_mut(animation) {
            state.speed = speed;
        }
    }
}

fn build_skeleton_from_m2(model: &M2Model) -> Skeleton {
    let mut bones = Vec::new();
    let mut bone_names = HashMap::new();

    for (idx, m2_bone) in model.bones.iter().enumerate() {
        bones.push(Bone {
            name: format!("bone_{}", idx),
            parent: if m2_bone.parent_bone >= 0 {
                Some(m2_bone.parent_bone as usize)
            } else {
                None
            },
            flags: BoneFlags::from_bits(m2_bone.flags).unwrap_or_default(),
            pivot: m2_bone.pivot,
        });

        bone_names.insert(format!("bone_{}", idx), idx);
    }

    // Build rest pose
    let rest_pose = bones.iter().map(|_| Transform::identity()).collect();

    Skeleton {
        bones,
        bone_names,
        rest_pose,
    }
}
```

### Facial Animation System

```rust
pub struct FacialAnimationSystem {
    blend_shapes: Vec<BlendShape>,
    emotion_presets: HashMap<String, EmotionPreset>,
    lip_sync_data: Option<LipSyncData>,
    current_emotion: String,
    emotion_blend: f32,
}

#[derive(Debug, Clone)]
pub struct BlendShape {
    name: String,
    vertices: Vec<u32>,
    deltas: Vec<Vector3<f32>>,
    current_weight: f32,
}

#[derive(Debug, Clone)]
pub struct EmotionPreset {
    name: String,
    blend_shape_weights: HashMap<String, f32>,
    duration: f32,
}

#[derive(Debug, Clone)]
pub struct LipSyncData {
    phonemes: Vec<Phoneme>,
    current_phoneme: usize,
}

#[derive(Debug, Clone)]
pub struct Phoneme {
    time: f32,
    duration: f32,
    blend_shapes: HashMap<String, f32>,
}

impl FacialAnimationSystem {
    pub fn apply_facial_animation(
        &mut self,
        vertices: &mut [Vertex],
        audio_time: f32,
        emotion: &str,
        intensity: f32,
    ) {
        // Apply emotion preset
        if emotion != self.current_emotion {
            self.transition_emotion(emotion);
        }

        // Update emotion blend
        self.update_emotion_blend(intensity);

        // Apply lip sync if available
        if let Some(lip_sync) = &mut self.lip_sync_data {
            self.apply_lip_sync(lip_sync, audio_time);
        }

        // Apply blend shapes to vertices
        self.apply_blend_shapes(vertices);
    }

    fn apply_blend_shapes(&self, vertices: &mut [Vertex]) {
        for shape in &self.blend_shapes {
            if shape.current_weight > 0.0 {
                for (vert_idx, delta) in shape.vertices.iter().zip(&shape.deltas) {
                    let vertex = &mut vertices[*vert_idx as usize];
                    vertex.position += delta * shape.current_weight;
                }
            }
        }
    }

    fn transition_emotion(&mut self, new_emotion: &str) {
        self.current_emotion = new_emotion.to_string();
        self.emotion_blend = 0.0;

        // Reset blend shape weights
        for shape in &mut self.blend_shapes {
            shape.current_weight = 0.0;
        }
    }

    fn update_emotion_blend(&mut self, target_intensity: f32) {
        self.emotion_blend = self.emotion_blend.lerp(&target_intensity, 0.1);

        if let Some(preset) = self.emotion_presets.get(&self.current_emotion) {
            for (shape_name, target_weight) in &preset.blend_shape_weights {
                if let Some(shape) = self.blend_shapes.iter_mut()
                    .find(|s| s.name == *shape_name) {
                    shape.current_weight = shape.current_weight.lerp(
                        &(target_weight * self.emotion_blend),
                        0.2
                    );
                }
            }
        }
    }
}
```

## Best Practices

### 1. Animation Compression

```rust
pub struct AnimationCompressor {
    position_threshold: f32,
    rotation_threshold: f32,
    scale_threshold: f32,
}

impl AnimationCompressor {
    pub fn compress_animation(&self, clip: &AnimationClip) -> CompressedAnimation {
        let mut compressed = CompressedAnimation {
            name: clip.name.clone(),
            duration: clip.duration,
            tracks: Vec::new(),
        };

        for track in &clip.bone_tracks {
            let compressed_track = CompressedTrack {
                bone_index: track.bone_index,
                position_keys: self.compress_vector_track(&track.translation),
                rotation_keys: self.compress_quaternion_track(&track.rotation),
                scale_keys: self.compress_vector_track(&track.scale),
            };

            compressed.tracks.push(compressed_track);
        }

        compressed
    }

    fn compress_vector_track(&self, track: &Track<Vector3<f32>>) -> Vec<CompressedKey<[f16; 3]>> {
        if track.keyframes.is_empty() {
            return Vec::new();
        }

        let mut compressed = vec![self.compress_vector_key(&track.keyframes[0])];
        let mut last_value = track.keyframes[0].value;

        for key in track.keyframes.iter().skip(1) {
            let delta = (key.value - last_value).magnitude();

            if delta > self.position_threshold {
                compressed.push(self.compress_vector_key(key));
                last_value = key.value;
            }
        }

        // Always include last key
        let last = track.keyframes.last().unwrap();
        compressed.push(self.compress_vector_key(last));

        compressed
    }

    fn compress_vector_key(&self, key: &Keyframe<Vector3<f32>>) -> CompressedKey<[f16; 3]> {
        CompressedKey {
            time: key.time as u16,
            value: [
                half::f16::from_f32(key.value.x),
                half::f16::from_f32(key.value.y),
                half::f16::from_f32(key.value.z),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompressedAnimation {
    name: String,
    duration: u32,
    tracks: Vec<CompressedTrack>,
}

#[derive(Debug, Clone)]
pub struct CompressedTrack {
    bone_index: usize,
    position_keys: Vec<CompressedKey<[f16; 3]>>,
    rotation_keys: Vec<CompressedKey<[i16; 4]>>,
    scale_keys: Vec<CompressedKey<[f16; 3]>>,
}

#[derive(Debug, Clone)]
pub struct CompressedKey<T> {
    time: u16,
    value: T,
}
```

### 2. Animation Caching

```rust
pub struct AnimationCache {
    cache: LruCache<AnimationCacheKey, Arc<Vec<Transform>>>,
    max_entries: usize,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct AnimationCacheKey {
    animation_name: String,
    time: u32,
    blend_weights: Vec<OrderedFloat<f32>>,
}

impl AnimationCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            cache: LruCache::new(max_entries),
            max_entries,
        }
    }

    pub fn get_or_compute<F>(
        &mut self,
        key: AnimationCacheKey,
        compute: F,
    ) -> Arc<Vec<Transform>>
    where
        F: FnOnce() -> Vec<Transform>,
    {
        if let Some(cached) = self.cache.get(&key) {
            return cached.clone();
        }

        let computed = Arc::new(compute());
        self.cache.put(key, computed.clone());
        computed
    }
}
```

### 3. Multi-threaded Animation

```rust
use rayon::prelude::*;

pub struct ParallelAnimationProcessor {
    thread_pool: ThreadPool,
}

impl ParallelAnimationProcessor {
    pub fn process_animation_batch(
        &self,
        animations: &[AnimationInstance],
    ) -> Vec<AnimationResult> {
        animations
            .par_iter()
            .map(|instance| self.process_single_animation(instance))
            .collect()
    }

    fn process_single_animation(&self, instance: &AnimationInstance) -> AnimationResult {
        // Process animation on thread pool
        let pose = instance.player.calculate_pose(instance.time);
        let matrices = self.calculate_matrices(&pose);

        AnimationResult {
            instance_id: instance.id,
            bone_matrices: matrices,
        }
    }
}
```

## Common Issues and Solutions

### Issue: Animation Jitter

**Problem**: Animations appear jittery or stuttering.

**Solution**:

```rust
pub struct AnimationSmoother {
    history: VecDeque<Vec<Transform>>,
    max_history: usize,
}

impl AnimationSmoother {
    pub fn smooth_animation(&mut self, current_pose: Vec<Transform>) -> Vec<Transform> {
        self.history.push_back(current_pose.clone());

        if self.history.len() > self.max_history {
            self.history.pop_front();
        }

        // Average recent poses
        let mut smoothed = current_pose;
        let history_weight = 0.3;

        for (i, pose) in self.history.iter().rev().enumerate().skip(1) {
            let weight = history_weight * (1.0 / (i as f32 + 1.0));

            for (bone_idx, transform) in pose.iter().enumerate() {
                smoothed[bone_idx] = smoothed[bone_idx].interpolate(transform, weight);
            }
        }

        smoothed
    }
}
```

### Issue: Bone Hierarchy Errors

**Problem**: Child bones not following parent transformations.

**Solution**:

```rust
fn validate_bone_hierarchy(skeleton: &Skeleton) -> Result<(), String> {
    let mut visited = vec![false; skeleton.bones.len()];

    for (idx, bone) in skeleton.bones.iter().enumerate() {
        if let Some(parent) = bone.parent {
            if parent >= skeleton.bones.len() {
                return Err(format!("Bone {} has invalid parent {}", idx, parent));
            }

            if parent == idx {
                return Err(format!("Bone {} is its own parent", idx));
            }

            // Check for cycles
            let mut current = parent;
            let mut chain = HashSet::new();
            chain.insert(idx);

            while let Some(next_parent) = skeleton.bones[current].parent {
                if chain.contains(&next_parent) {
                    return Err(format!("Cycle detected in bone hierarchy at {}", idx));
                }
                chain.insert(current);
                current = next_parent;
            }
        }

        visited[idx] = true;
    }

    Ok(())
}
```

### Issue: Animation Blending Artifacts

**Problem**: Unnatural poses when blending between animations.

**Solution**:

```rust
pub struct SmartBlender {
    sync_markers: HashMap<String, Vec<SyncMarker>>,
}

#[derive(Debug, Clone)]
struct SyncMarker {
    time: f32,
    phase: f32,
    marker_type: SyncMarkerType,
}

impl SmartBlender {
    pub fn blend_with_sync(
        &self,
        from_anim: &str,
        to_anim: &str,
        blend_factor: f32,
    ) -> f32 {
        // Find matching sync markers
        let from_markers = self.sync_markers.get(from_anim);
        let to_markers = self.sync_markers.get(to_anim);

        if let (Some(from), Some(to)) = (from_markers, to_markers) {
            // Align animations based on sync markers
            let from_phase = self.calculate_phase(from);
            let to_phase = self.calculate_phase(to);

            // Adjust time to match phases
            let phase_diff = to_phase - from_phase;
            let time_adjustment = phase_diff * blend_factor;

            return time_adjustment;
        }

        0.0
    }
}
```

## Performance Tips

### 1. GPU Skinning

```glsl
// Vertex shader for GPU skinning
#version 450

layout(set = 0, binding = 0) uniform BoneMatrices {
    mat4 bones[256];
};

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 texcoord;
layout(location = 3) in uvec4 bone_indices;
layout(location = 4) in vec4 bone_weights;

layout(location = 0) out vec3 world_normal;
layout(location = 1) out vec2 out_texcoord;

void main() {
    mat4 skin_matrix =
        bones[bone_indices.x] * bone_weights.x +
        bones[bone_indices.y] * bone_weights.y +
        bones[bone_indices.z] * bone_weights.z +
        bones[bone_indices.w] * bone_weights.w;

    vec4 world_pos = skin_matrix * vec4(position, 1.0);
    gl_Position = view_proj * world_pos;

    world_normal = normalize((skin_matrix * vec4(normal, 0.0)).xyz);
    out_texcoord = texcoord;
}
```

### 2. Animation LOD

```rust
pub struct AnimationLod {
    bone_importance: HashMap<usize, f32>,
    distance_thresholds: Vec<f32>,
}

impl AnimationLod {
    pub fn get_active_bones(&self, distance: f32) -> HashSet<usize> {
        let importance_threshold = if distance < self.distance_thresholds[0] {
            0.0 // All bones
        } else if distance < self.distance_thresholds[1] {
            0.3 // Important bones only
        } else {
            0.7 // Critical bones only
        };

        self.bone_importance
            .iter()
            .filter(|(_, importance)| **importance >= importance_threshold)
            .map(|(idx, _)| *idx)
            .collect()
    }
}
```

### 3. Animation Streaming

```rust
pub struct AnimationStreamer {
    loaded_clips: HashMap<String, Arc<AnimationClip>>,
    loading_queue: Arc<Mutex<VecDeque<String>>>,
    loader_thread: Option<JoinHandle<()>>,
}

impl AnimationStreamer {
    pub async fn get_animation(&self, name: &str) -> Option<Arc<AnimationClip>> {
        if let Some(clip) = self.loaded_clips.get(name) {
            return Some(clip.clone());
        }

        // Queue for loading
        self.loading_queue.lock().unwrap().push_back(name.to_string());

        // Wait for load with timeout
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(5) {
            if let Some(clip) = self.loaded_clips.get(name) {
                return Some(clip.clone());
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        None
    }
}
```

## Related Guides

- [🎭 Loading M2 Models](./m2-models.md) - Load models with animation data
- [🎨 Model Rendering Guide](./model-rendering.md) - Render animated models
- [📊 LOD System Guide](./lod-system.md) - Animation LOD implementation

## References

- [Skeletal Animation](https://learnopengl.com/Guest-Articles/2020/Skeletal-Animation) - Understanding skeletal animation
- [Animation Compression](https://www.gdcvault.com/play/1020583/Animation-Compression-Theory-and-Practice) - GDC talk on animation compression
- [FABRIK Algorithm](http://www.andreasaristidou.com/FABRIK.html) - IK solving algorithm
- [Animation State Machines](https://docs.unity3d.com/Manual/AnimatorControllers.html) - Unity's approach to animation state machines
