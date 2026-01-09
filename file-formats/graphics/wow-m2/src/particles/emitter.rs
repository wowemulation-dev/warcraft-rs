//! Particle emitter runtime state

use super::emission::{
    EmissionParams, EmissionType, ParticleRng, create_planar, create_spherical, create_spline,
};
use super::particle::Particle;
use super::{PARTICLE_COORDINATE_FIX, TEXELS_PER_PARTICLE};
use crate::chunks::particle_emitter::{M2ParticleEmitter, M2ParticleEmitterType, M2ParticleFlags};

/// Current frame emitter parameters (from animation tracks)
#[derive(Debug, Clone, Default)]
pub struct EmitterParams {
    /// Whether the emitter is currently enabled
    pub enabled: bool,
    /// Gravity vector (typically [0, 0, -g])
    pub gravity: [f32; 3],
    /// Current emission speed
    pub emission_speed: f32,
    /// Speed variation factor
    pub speed_variation: f32,
    /// Vertical angle range (radians)
    pub vertical_range: f32,
    /// Horizontal angle range (radians)
    pub horizontal_range: f32,
    /// Current particle lifespan
    pub lifespan: f32,
    /// Current emission rate (particles per second)
    pub emission_rate: f32,
    /// Emission area length
    pub emission_area_length: f32,
    /// Emission area width
    pub emission_area_width: f32,
    /// Z source for directional emission
    pub z_source: f32,
}

/// Blending mode for particles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlendMode {
    /// Opaque rendering
    #[default]
    Opaque,
    /// Alpha blending (src * alpha + dst * (1-alpha))
    AlphaBlend,
    /// Additive blending (src + dst)
    Additive,
    /// Modulate blending (src * dst)
    Modulate,
    /// Alpha key (hard cutoff at threshold)
    AlphaKey,
}

impl BlendMode {
    /// Convert from M2 blending type byte
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => BlendMode::Opaque,
            1 => BlendMode::AlphaKey,
            2 => BlendMode::AlphaBlend,
            3 => BlendMode::Additive,
            4 => BlendMode::Modulate,
            _ => BlendMode::AlphaBlend,
        }
    }

    /// Get the alpha test threshold for this blend mode
    pub fn alpha_test(&self) -> f32 {
        match self {
            BlendMode::Opaque => -1.0,
            BlendMode::AlphaKey => 0.501_960_8,
            _ => 0.003_921_569,
        }
    }
}

/// Runtime particle emitter
#[derive(Debug, Clone)]
pub struct ParticleEmitter {
    /// Emission type (planar, spherical, spline)
    emission_type: EmissionType,
    /// Current particles
    particles: Vec<Particle>,
    /// Random number generator
    rng: ParticleRng,
    /// Current model matrix (bone transform * local offset)
    model_matrix: [f32; 16],
    /// Coordinate fix matrix
    coordinate_fix: [f32; 16],
    /// Wind vector
    wind: [f32; 3],
    /// Local position offset
    position: [f32; 3],
    /// Drag coefficient
    drag: f32,
    /// Fractional particles to emit (accumulated)
    particles_to_emit: f32,
    /// Lifespan variance
    lifespan_variance: f32,
    /// Maximum number of particles
    max_particles: usize,
    /// Bone index this emitter is attached to
    bone_index: u16,
    /// Texture index
    texture_index: u16,
    /// Blending mode
    blend_mode: BlendMode,
    /// Emitter flags
    flags: M2ParticleFlags,
    /// Current parameters from animation
    pub params: EmitterParams,
    /// Texture scale X (1 / rows)
    pub tex_scale_x: f32,
    /// Texture scale Y (1 / cols)
    pub tex_scale_y: f32,
    /// Texture column bit count for cell extraction
    tex_col_bits: u32,
    /// Texture column mask for cell extraction
    tex_col_mask: u32,
}

impl ParticleEmitter {
    /// Create a new particle emitter from parsed M2 data
    pub fn new(m2_emitter: &M2ParticleEmitter) -> Self {
        let emission_type = match m2_emitter.emitter_type {
            M2ParticleEmitterType::Plane => EmissionType::Planar,
            M2ParticleEmitterType::Sphere => EmissionType::Spherical,
            M2ParticleEmitterType::Spline => EmissionType::Spline,
            _ => EmissionType::Point,
        };

        // Calculate max particles based on lifespan and emission rate
        let max_lifespan = m2_emitter.lifetime.max(m2_emitter.max_lifetime);
        let max_rate = m2_emitter.emission_rate.max(m2_emitter.max_emission_rate);
        let max_particles = ((max_lifespan * max_rate * 1.5) as usize).max(16);

        // Calculate texture scale from tile dimensions
        // Default to 1x1 if not specified
        let tex_cols = 1u32; // Would come from texture_dimensions_cols
        let tex_rows = 1u32; // Would come from texture_dimension_rows
        let tex_scale_x = 1.0 / tex_rows as f32;
        let tex_scale_y = 1.0 / tex_cols as f32;
        let tex_col_bits = (tex_cols as f32).log2().ceil() as u32;
        let tex_col_mask = (1 << tex_col_bits) - 1;

        Self {
            emission_type,
            particles: Vec::with_capacity(max_particles),
            rng: ParticleRng::new(42), // Fixed seed for reproducibility
            model_matrix: identity_matrix(),
            coordinate_fix: PARTICLE_COORDINATE_FIX,
            wind: [0.0, 0.0, 0.0],
            position: [
                m2_emitter.position.x,
                m2_emitter.position.y,
                m2_emitter.position.z,
            ],
            drag: 0.0, // Drag would come from parsed data
            particles_to_emit: 0.0,
            lifespan_variance: m2_emitter.max_lifetime - m2_emitter.min_lifetime,
            max_particles,
            bone_index: m2_emitter.bone_index,
            texture_index: m2_emitter.texture_index,
            blend_mode: BlendMode::from_u8(m2_emitter.blending_type),
            flags: m2_emitter.flags,
            params: EmitterParams {
                enabled: true,
                gravity: [0.0, 0.0, -m2_emitter.gravity],
                emission_speed: m2_emitter.emission_velocity,
                speed_variation: m2_emitter.speed_variation,
                vertical_range: m2_emitter.vertical_range,
                horizontal_range: m2_emitter.horizontal_range,
                lifespan: m2_emitter.lifetime,
                emission_rate: m2_emitter.emission_rate,
                emission_area_length: m2_emitter.emission_area_length,
                emission_area_width: m2_emitter.emission_area_width,
                z_source: 0.0,
            },
            tex_scale_x,
            tex_scale_y,
            tex_col_bits,
            tex_col_mask,
        }
    }

    /// Get the bone index this emitter is attached to
    pub fn bone_index(&self) -> u16 {
        self.bone_index
    }

    /// Get the texture index
    pub fn texture_index(&self) -> u16 {
        self.texture_index
    }

    /// Get the blend mode
    pub fn blend_mode(&self) -> BlendMode {
        self.blend_mode
    }

    /// Get the current number of particles
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    /// Get the maximum number of particles
    pub fn max_particles(&self) -> usize {
        self.max_particles
    }

    /// Check if translate_particle_with_bone flag is set
    fn translate_with_bone(&self) -> bool {
        !self.flags.contains(M2ParticleFlags::FOLLOW_EMITTER)
    }

    /// Check if particles should go up (spherical emission)
    fn particles_go_up(&self) -> bool {
        self.flags.contains(M2ParticleFlags::SPHERE_AS_SOURCE)
    }

    /// Update the emitter
    ///
    /// # Arguments
    /// * `dt_ms` - Delta time in milliseconds
    /// * `bone_transform` - 4x4 bone transform matrix (column-major)
    /// * `bone_post_billboard` - 4x4 post-billboard transform matrix (column-major)
    pub fn update(
        &mut self,
        dt_ms: f32,
        bone_transform: &[f32; 16],
        bone_post_billboard: &[f32; 16],
    ) {
        let dt = dt_ms / 1000.0;

        // Update model matrix: bone_post_billboard * bone_transform * local_offset
        self.update_model_matrix(bone_transform, bone_post_billboard);

        // Emit new particles if enabled
        if self.params.enabled {
            let emission_rate =
                self.params.emission_rate + self.rng.random_range(self.params.emission_rate * 0.1);
            self.particles_to_emit += emission_rate * dt;

            while self.particles_to_emit > 1.0 && self.particles.len() < self.max_particles {
                self.create_particle();
                self.particles_to_emit -= 1.0;
            }
        }

        // Calculate combined force
        let force = [
            self.wind[0] - self.params.gravity[0],
            self.wind[1] - self.params.gravity[1],
            self.wind[2] - self.params.gravity[2],
        ];

        // Update existing particles
        self.particles.retain_mut(|particle| {
            particle.age += dt;
            if !particle.is_alive() {
                return false;
            }

            // Update physics
            particle.update_physics(dt, force, self.drag);

            true
        });
    }

    /// Update parameters from animation state
    ///
    /// Call this before `update()` to sync animated parameters.
    pub fn update_params(&mut self, params: EmitterParams) {
        self.params = params;
    }

    /// Update the model matrix from bone transforms
    fn update_model_matrix(&mut self, bone_transform: &[f32; 16], bone_post_billboard: &[f32; 16]) {
        // Start with identity translated by local position
        let mut local = identity_matrix();
        local[12] = self.position[0];
        local[13] = self.position[1];
        local[14] = self.position[2];

        // Multiply: bone_post_billboard * bone_transform * local
        let temp = mat4_multiply(bone_transform, &local);
        let combined = mat4_multiply(bone_post_billboard, &temp);

        // Apply coordinate fix
        self.model_matrix = mat4_multiply(&combined, &self.coordinate_fix);
    }

    /// Create a new particle based on emission type
    fn create_particle(&mut self) {
        let emission_params = EmissionParams {
            area_length: self.params.emission_area_length,
            area_width: self.params.emission_area_width,
            speed: self.params.emission_speed,
            speed_variation: self.params.speed_variation,
            vertical_range: self.params.vertical_range,
            horizontal_range: self.params.horizontal_range,
            z_source: self.params.z_source,
            lifespan: self.params.lifespan,
            lifespan_variance: self.lifespan_variance,
        };

        // Get flags before mutable borrow
        let particles_go_up = self.particles_go_up();

        let mut particle = match self.emission_type {
            EmissionType::Planar => create_planar(&emission_params, &mut self.rng),
            EmissionType::Spherical => {
                create_spherical(&emission_params, &mut self.rng, particles_go_up)
            }
            EmissionType::Spline => {
                // For spline, we'd need spline points - use origin for now
                create_spline(&emission_params, &mut self.rng, [0.0, 0.0, 0.0])
            }
            EmissionType::Point => {
                // Point emission - particles start at origin with random velocity
                let speed = self.params.emission_speed
                    * (1.0 + self.rng.random_range(self.params.speed_variation));
                Particle::new(
                    [0.0, 0.0, 0.0],
                    [0.0, 0.0, speed],
                    self.params.lifespan + self.rng.random_range(self.lifespan_variance),
                )
            }
        };

        // Transform particle by model matrix if not following emitter
        if !self.translate_with_bone() {
            particle.position = transform_point(&particle.position, &self.model_matrix);
            particle.velocity = transform_vector(&particle.velocity, &self.model_matrix);
        }

        self.particles.push(particle);
    }

    /// Fill texture data for GPU upload
    ///
    /// Returns a Vec of f32 values in the format:
    /// - Texel 0: position.xyz + 0.0
    /// - Texel 1: color.rgba
    /// - Texel 2: scale.xy + 0.0 + 0.0
    /// - Texel 3: tex_coord.xy + 0.0 + 0.0
    pub fn fill_texture_data(&self) -> Vec<f32> {
        let mut data = vec![0.0; self.max_particles * TEXELS_PER_PARTICLE * 4];

        for (i, particle) in self.particles.iter().enumerate() {
            let base = i * TEXELS_PER_PARTICLE * 4;

            // Get position (optionally transformed)
            let pos = if self.translate_with_bone() {
                transform_point(&particle.position, &self.model_matrix)
            } else {
                particle.position
            };

            // Texel 0: position
            data[base] = pos[0];
            data[base + 1] = pos[1];
            data[base + 2] = pos[2];
            data[base + 3] = 0.0;

            // Texel 1: color
            data[base + 4] = particle.color[0];
            data[base + 5] = particle.color[1];
            data[base + 6] = particle.color[2];
            data[base + 7] = particle.color[3];

            // Texel 2: scale
            data[base + 8] = particle.scale[0];
            data[base + 9] = particle.scale[1];
            data[base + 10] = 0.0;
            data[base + 11] = 0.0;

            // Texel 3: tex coord
            data[base + 12] = particle.tex_coord_head[0];
            data[base + 13] = particle.tex_coord_head[1];
            data[base + 14] = 0.0;
            data[base + 15] = 0.0;
        }

        data
    }

    /// Extract texture coordinates from cell index
    #[allow(dead_code)]
    fn extract_tex_coords(&self, cell: u16) -> [f32; 2] {
        let x_int = cell as u32 & self.tex_col_mask;
        let y_int = cell as u32 >> self.tex_col_bits;
        [
            x_int as f32 * self.tex_scale_x,
            y_int as f32 * self.tex_scale_y,
        ]
    }
}

// Matrix helper functions

/// Create a 4x4 identity matrix (column-major)
fn identity_matrix() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

/// Multiply two 4x4 matrices (column-major): result = a * b
fn mat4_multiply(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    let mut result = [0.0f32; 16];

    for col in 0..4 {
        for row in 0..4 {
            let mut sum = 0.0;
            for k in 0..4 {
                sum += a[k * 4 + row] * b[col * 4 + k];
            }
            result[col * 4 + row] = sum;
        }
    }

    result
}

/// Transform a point by a 4x4 matrix (with w=1)
fn transform_point(p: &[f32; 3], m: &[f32; 16]) -> [f32; 3] {
    let x = m[0] * p[0] + m[4] * p[1] + m[8] * p[2] + m[12];
    let y = m[1] * p[0] + m[5] * p[1] + m[9] * p[2] + m[13];
    let z = m[2] * p[0] + m[6] * p[1] + m[10] * p[2] + m[14];
    [x, y, z]
}

/// Transform a vector by a 4x4 matrix (with w=0, no translation)
fn transform_vector(v: &[f32; 3], m: &[f32; 16]) -> [f32; 3] {
    let x = m[0] * v[0] + m[4] * v[1] + m[8] * v[2];
    let y = m[1] * v[0] + m[5] * v[1] + m[9] * v[2];
    let z = m[2] * v[0] + m[6] * v[1] + m[10] * v[2];
    [x, y, z]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blend_mode_from_u8() {
        assert_eq!(BlendMode::from_u8(0), BlendMode::Opaque);
        assert_eq!(BlendMode::from_u8(1), BlendMode::AlphaKey);
        assert_eq!(BlendMode::from_u8(2), BlendMode::AlphaBlend);
        assert_eq!(BlendMode::from_u8(3), BlendMode::Additive);
        assert_eq!(BlendMode::from_u8(4), BlendMode::Modulate);
    }

    #[test]
    fn test_identity_matrix() {
        let m = identity_matrix();
        assert_eq!(m[0], 1.0);
        assert_eq!(m[5], 1.0);
        assert_eq!(m[10], 1.0);
        assert_eq!(m[15], 1.0);
    }

    #[test]
    fn test_transform_point() {
        let m = identity_matrix();
        let p = [1.0, 2.0, 3.0];
        let result = transform_point(&p, &m);
        assert_eq!(result, p);
    }

    #[test]
    fn test_transform_point_translation() {
        let mut m = identity_matrix();
        m[12] = 10.0; // Translate X
        m[13] = 20.0; // Translate Y
        m[14] = 30.0; // Translate Z

        let p = [1.0, 2.0, 3.0];
        let result = transform_point(&p, &m);
        assert_eq!(result, [11.0, 22.0, 33.0]);
    }

    #[test]
    fn test_mat4_multiply_identity() {
        let a = identity_matrix();
        let b = identity_matrix();
        let result = mat4_multiply(&a, &b);
        assert_eq!(result, identity_matrix());
    }
}
