# M2 Phys Format 🌊

M2 .phys files contain physics simulation data for cloth, hair, and other
dynamic elements in M2 models.

## Overview

- **Extension**: `.phys`
- **Purpose**: Define physics constraints and properties for dynamic bones
- **Introduced**: Cataclysm (4.0.0)
- **Use Cases**: Cloaks, hair, tabards, loose armor pieces
- **Engine**: Based on simplified Havok cloth simulation

## Structure

### Physics File Header

```rust
struct M2PhysHeader {
    version: u32,              // Version (usually 0)
    chunks: Vec<PhysChunk>,    // Variable chunks
}

enum PhysChunk {
    PHYS(PhysicsData),        // Main physics data
    BODY(PhysicsBody),        // Rigid body definitions
    BDY2(PhysicsBodyV2),      // Version 2 body data
    SHAP(PhysicsShapes),      // Collision shapes
    JOIN(PhysicsJoints),      // Joint constraints
    WELJ(WeldJoints),         // Welded joints
    SHP2(PhysicsShapesV2),   // Version 2 shapes
    PHYV(PhysicsVersion),     // Physics version info
}
```

### Physics Bone Data

```rust
struct PhysicsBone {
    bone_index: u16,          // Index in M2 bone array
    flags: u16,               // Physics flags
    mass: f32,                // Bone mass
    wind_resistance: f32,     // Wind interaction strength
    damping: f32,             // Motion damping
    max_distance: f32,        // Max distance from rest position
    stiffness: f32,           // Spring stiffness
    thickness: f32,           // Collision thickness
    gravity_scale: f32,       // Gravity multiplier
}

struct PhysicsConstraint {
    bone_a: u16,              // First bone index
    bone_b: u16,              // Second bone index
    distance: f32,            // Rest distance
    stretch_resistance: f32,  // Stretch stiffness
    compress_resistance: f32, // Compression stiffness
}
```

## Usage Example

```rust
use wow_m2::M2Model;

// Load model
let format = M2Model::load("Character/Human/Female/HumanFemale.m2")?;
let model = format.model();

// Physics data (.phys files) contains bone simulation parameters.
// The wow-m2 crate parses the M2 model and its bone hierarchy.
// Physics simulation (cloth, hair) would be implemented in your
// rendering engine using the bone data and .phys file constraints.
//
// The .phys file is a separate binary file with the same base name:
//   Character/Human/Female/HumanFemale.phys
//
// It contains rigid body definitions, collision shapes, and joint
// constraints that drive the physics simulation for dynamic bones.
```

## Physics Systems

### Cloth Simulation

```rust
struct ClothSimulator {
    particles: Vec<ClothParticle>,
    constraints: Vec<ClothConstraint>,
    collision_shapes: Vec<CollisionShape>,
}

impl ClothSimulator {
    fn simulate_step(&mut self, dt: f32) {
        // Apply forces
        for particle in &mut self.particles {
            if !particle.is_fixed {
                // Gravity
                particle.velocity += Vec3::new(0.0, -9.81, 0.0) * dt;

                // Wind
                let wind_force = self.calculate_wind_force(&particle);
                particle.velocity += wind_force * dt;

                // Damping
                particle.velocity *= 0.99;
            }
        }

        // Update positions
        for particle in &mut self.particles {
            particle.predicted_pos = particle.position + particle.velocity * dt;
        }

        // Solve constraints
        for _ in 0..4 { // Multiple iterations for stability
            self.solve_distance_constraints();
            self.solve_collision_constraints();
        }

        // Update velocities and positions
        for particle in &mut self.particles {
            particle.velocity = (particle.predicted_pos - particle.position) / dt;
            particle.position = particle.predicted_pos;
        }
    }
}
```

### Hair Physics

```rust
struct HairStrand {
    segments: Vec<HairSegment>,
    root_bone: u16,
    stiffness: f32,
    damping: f32,
}

impl HairStrand {
    fn update(&mut self, head_transform: &Matrix4, dt: f32) {
        // Fix root to head
        self.segments[0].position = head_transform * self.segments[0].rest_position;

        // Simulate each segment
        for i in 1..self.segments.len() {
            let parent = &self.segments[i-1];
            let segment = &mut self.segments[i];

            // Spring force to maintain length
            let to_parent = parent.position - segment.position;
            let current_length = to_parent.length();
            let rest_length = segment.rest_length;

            let spring_force = self.stiffness * (current_length - rest_length)
                * to_parent.normalize();

            // Apply forces
            segment.velocity += spring_force * dt;
            segment.velocity *= self.damping;
            segment.position += segment.velocity * dt;

            // Length constraint
            let dir = (segment.position - parent.position).normalize();
            segment.position = parent.position + dir * rest_length;
        }
    }
}
```

## Advanced Features

### Collision Detection

```rust
enum CollisionShape {
    Sphere { center: Vec3, radius: f32 },
    Capsule { start: Vec3, end: Vec3, radius: f32 },
    Box { min: Vec3, max: Vec3 },
}

fn resolve_collision(particle: &mut ClothParticle, shape: &CollisionShape) {
    match shape {
        CollisionShape::Sphere { center, radius } => {
            let to_particle = particle.position - center;
            let distance = to_particle.length();

            if distance < *radius {
                // Push particle outside sphere
                particle.position = center + to_particle.normalize() * radius;
            }
        }
        CollisionShape::Capsule { start, end, radius } => {
            // Find closest point on line segment
            let closest = closest_point_on_segment(&particle.position, start, end);
            let to_particle = particle.position - closest;
            let distance = to_particle.length();

            if distance < *radius {
                particle.position = closest + to_particle.normalize() * radius;
            }
        }
        _ => {}
    }
}
```

### Wind Interaction

```rust
struct WindSystem {
    base_direction: Vec3,
    turbulence_scale: f32,
    gust_frequency: f32,
    time: f32,
}

impl WindSystem {
    fn get_wind_at(&self, position: Vec3) -> Vec3 {
        // Base wind
        let mut wind = self.base_direction;

        // Add turbulence
        let turb_x = noise_3d(position * self.turbulence_scale + self.time);
        let turb_y = noise_3d(position * self.turbulence_scale + self.time + 100.0);
        let turb_z = noise_3d(position * self.turbulence_scale + self.time + 200.0);

        wind += Vec3::new(turb_x, turb_y, turb_z) * 2.0;

        // Add gusts
        let gust_strength = (self.time * self.gust_frequency).sin().max(0.0);
        wind *= 1.0 + gust_strength * 3.0;

        wind
    }
}
```

### Performance Optimization

```rust
struct OptimizedPhysics {
    lod_distances: [f32; 3],
    simulation_rates: [u32; 3], // Simulation steps per frame
}

impl OptimizedPhysics {
    fn update(&mut self, models: &mut [PhysicsModel], camera: &Camera) {
        for model in models {
            let distance = (model.position - camera.position).length();

            // Determine LOD
            let lod = if distance < self.lod_distances[0] {
                0 // Full simulation
            } else if distance < self.lod_distances[1] {
                1 // Reduced simulation
            } else if distance < self.lod_distances[2] {
                2 // Minimal simulation
            } else {
                continue; // Skip physics
            };

            // Simulate at appropriate rate
            let steps = self.simulation_rates[lod];
            for _ in 0..steps {
                model.physics.step(0.016 / steps as f32);
            }
        }
    }
}
```

## Common Patterns

### Physics Asset Pipeline

```rust
struct PhysicsAssetLoader {
    cache: HashMap<String, Arc<M2Physics>>,
}

impl PhysicsAssetLoader {
    fn load_with_fallback(&mut self, model_path: &str) -> Option<Arc<M2Physics>> {
        // Try exact match
        let phys_path = model_path.replace(".m2", ".phys");
        if let Ok(physics) = load_phys_file(&phys_path) {
            return Some(Arc::new(physics));
        }

        // Try shared physics (e.g., all human females share cape physics)
        let model_type = extract_model_type(model_path);
        let shared_path = format!("Physics/Shared/{}.phys", model_type);
        if let Ok(physics) = M2Physics::open(&shared_path) {
            return Some(Arc::new(physics));
        }

        None // No physics data
    }
}
```

### Dynamic Quality Settings

```rust
struct PhysicsQualitySettings {
    enable_cloth: bool,
    enable_hair: bool,
    max_simulated_models: u32,
    collision_iterations: u32,
}

impl PhysicsQualitySettings {
    fn apply(&self, simulator: &mut PhysicsSimulator) {
        simulator.set_enabled(PhysicsType::Cloth, self.enable_cloth);
        simulator.set_enabled(PhysicsType::Hair, self.enable_hair);
        simulator.set_iterations(self.collision_iterations);
    }

    fn from_preset(preset: QualityPreset) -> Self {
        match preset {
            QualityPreset::Low => Self {
                enable_cloth: false,
                enable_hair: false,
                max_simulated_models: 5,
                collision_iterations: 1,
            },
            QualityPreset::High => Self {
                enable_cloth: true,
                enable_hair: true,
                max_simulated_models: 50,
                collision_iterations: 4,
            },
            _ => Self::default(),
        }
    }
}
```

## Performance Tips

- Use LOD system for physics simulation
- Disable physics for off-screen models
- Reduce iterations for distant objects
- Cache physics data - don't reload per instance
- Use spatial partitioning for collision detection

## Common Issues

### Stability Problems

- Too large time steps cause explosions
- Use fixed timestep with interpolation
- Multiple constraint iterations improve stability

### Version Compatibility

- .phys introduced in Cataclysm (4.0.0)
- Format evolved through expansions
- Not all models have physics data

### Performance Impact

- Physics can be CPU intensive
- Limit number of simulated models
- Consider GPU physics for crowds

## References

- [M2/.phys Format (wowdev.wiki)](https://wowdev.wiki/M2/.phys)
- [WoW Physics System](https://wowdev.wiki/Physics)

## See Also

- [M2 Format](m2.md) - Main model format
