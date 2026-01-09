//! Particle emission algorithms for different emitter types

use super::particle::Particle;

/// Emission type determining how particles are spawned
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmissionType {
    /// Particles spawn from a single point
    Point,
    /// Particles spawn within a 2D rectangular plane
    Planar,
    /// Particles spawn on a 3D sphere surface
    Spherical,
    /// Particles spawn along a spline curve
    Spline,
}

impl EmissionType {
    /// Convert from M2 emitter type byte
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => EmissionType::Planar,
            2 => EmissionType::Spherical,
            3 => EmissionType::Spline,
            _ => EmissionType::Point,
        }
    }
}

/// Parameters for particle emission
#[derive(Debug, Clone)]
pub struct EmissionParams {
    /// Emission area length (for planar/sphere radius)
    pub area_length: f32,
    /// Emission area width (for planar)
    pub area_width: f32,
    /// Emission speed
    pub speed: f32,
    /// Speed variation (random range)
    pub speed_variation: f32,
    /// Vertical emission angle range (radians)
    pub vertical_range: f32,
    /// Horizontal emission angle range (radians)
    pub horizontal_range: f32,
    /// Z source for directional emission
    pub z_source: f32,
    /// Particle lifespan
    pub lifespan: f32,
    /// Lifespan variance
    pub lifespan_variance: f32,
}

impl Default for EmissionParams {
    fn default() -> Self {
        Self {
            area_length: 1.0,
            area_width: 1.0,
            speed: 1.0,
            speed_variation: 0.0,
            vertical_range: 0.0,
            horizontal_range: 0.0,
            z_source: 0.0,
            lifespan: 1.0,
            lifespan_variance: 0.0,
        }
    }
}

/// Random number generator state (simple LCG)
#[derive(Debug, Clone)]
pub struct ParticleRng {
    state: u64,
}

impl ParticleRng {
    /// Create a new RNG with the given seed
    pub fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed as u64 },
        }
    }

    /// Generate a random f32 in range [0, 1)
    pub fn next_f32(&mut self) -> f32 {
        // LCG with parameters from Numerical Recipes
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        // Extract bits 33-62 (30 bits) for good randomness
        let bits = ((self.state >> 33) as u32) & 0x3FFF_FFFF;
        bits as f32 / (0x4000_0000 as f32)
    }

    /// Generate a random f32 in range [-a, a]
    pub fn random_range(&mut self, a: f32) -> f32 {
        if a == 0.0 {
            return 0.0;
        }
        (self.next_f32() * 2.0 - 1.0) * a
    }
}

/// Create a particle using planar emission
///
/// Particles spawn within a rectangular area and emit with velocity
/// based on polar/azimuth angles or toward a z_source point.
pub fn create_planar(params: &EmissionParams, rng: &mut ParticleRng) -> Particle {
    // Random position within the emission rectangle
    let position = [
        rng.random_range(1.0) * params.area_length * 0.5,
        rng.random_range(1.0) * params.area_width * 0.5,
        0.0,
    ];

    let velocity = if params.z_source.abs() < 0.001 {
        // Use polar/azimuth angles for velocity direction
        let polar = params.vertical_range * rng.random_range(1.0);
        let azimuth = params.horizontal_range * rng.random_range(1.0);

        let speed = emission_speed(params, rng);
        [
            azimuth.cos() * polar.sin() * speed,
            azimuth.sin() * polar.sin() * speed,
            polar.cos() * speed,
        ]
    } else {
        // Emit toward z_source point
        let mut vel = [position[0], position[1], position[2] - params.z_source];
        let mag = (vel[0] * vel[0] + vel[1] * vel[1] + vel[2] * vel[2]).sqrt();
        if mag > 0.0001 {
            let speed = emission_speed(params, rng);
            vel[0] = vel[0] / mag * speed;
            vel[1] = vel[1] / mag * speed;
            vel[2] = vel[2] / mag * speed;
        }
        vel
    };

    Particle::new(position, velocity, lifespan(params, rng))
}

/// Create a particle using spherical emission
///
/// Particles spawn on a sphere surface and emit radially outward
/// or toward a z_source point.
pub fn create_spherical(
    params: &EmissionParams,
    rng: &mut ParticleRng,
    particles_go_up: bool,
) -> Particle {
    // Random radius between area_length and area_width
    let emission_area = params.area_width - params.area_length;
    let radius = params.area_length + rng.next_f32() * emission_area;

    // Random point on sphere using polar coordinates
    let polar = rng.random_range(1.0) * params.vertical_range;
    let azimuth = rng.random_range(1.0) * params.horizontal_range;

    let emission_dir = [
        polar.cos() * azimuth.cos(),
        polar.cos() * azimuth.sin(),
        polar.sin(),
    ];

    let position = [
        emission_dir[0] * radius,
        emission_dir[1] * radius,
        emission_dir[2] * radius,
    ];

    let velocity = if params.z_source.abs() < 0.001 {
        let speed = emission_speed(params, rng);
        if particles_go_up {
            [0.0, 0.0, speed]
        } else {
            [
                emission_dir[0] * speed,
                emission_dir[1] * speed,
                emission_dir[2] * speed,
            ]
        }
    } else {
        // Emit toward z_source point
        let mut vel = [position[0], position[1], position[2] - params.z_source];
        let mag = (vel[0] * vel[0] + vel[1] * vel[1] + vel[2] * vel[2]).sqrt();
        if mag > 0.0001 {
            let speed = emission_speed(params, rng);
            vel[0] = vel[0] / mag * speed;
            vel[1] = vel[1] / mag * speed;
            vel[2] = vel[2] / mag * speed;
        }
        vel
    };

    Particle::new(position, velocity, lifespan(params, rng))
}

/// Create a particle using spline emission
///
/// Particles spawn along a bezier spline curve.
/// Note: Spline points must be provided separately.
pub fn create_spline(
    params: &EmissionParams,
    rng: &mut ParticleRng,
    spline_point: [f32; 3],
) -> Particle {
    let position = spline_point;

    // Simple upward velocity for spline particles
    let speed = emission_speed(params, rng);
    let velocity = [0.0, 0.0, speed];

    Particle::new(position, velocity, lifespan(params, rng))
}

/// Calculate emission speed with variation
#[inline]
fn emission_speed(params: &EmissionParams, rng: &mut ParticleRng) -> f32 {
    params.speed * (1.0 + rng.random_range(params.speed_variation))
}

/// Calculate lifespan with variance
#[inline]
fn lifespan(params: &EmissionParams, rng: &mut ParticleRng) -> f32 {
    params.lifespan + rng.random_range(params.lifespan_variance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emission_type_from_u8() {
        assert_eq!(EmissionType::from_u8(0), EmissionType::Point);
        assert_eq!(EmissionType::from_u8(1), EmissionType::Planar);
        assert_eq!(EmissionType::from_u8(2), EmissionType::Spherical);
        assert_eq!(EmissionType::from_u8(3), EmissionType::Spline);
        assert_eq!(EmissionType::from_u8(99), EmissionType::Point);
    }

    #[test]
    fn test_particle_rng() {
        let mut rng = ParticleRng::new(12345);
        let v1 = rng.next_f32();
        let v2 = rng.next_f32();

        assert!((0.0..1.0).contains(&v1));
        assert!((0.0..1.0).contains(&v2));
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_random_range() {
        let mut rng = ParticleRng::new(12345);

        for _ in 0..100 {
            let v = rng.random_range(5.0);
            assert!((-5.0..=5.0).contains(&v));
        }

        // Zero range should return zero
        assert_eq!(rng.random_range(0.0), 0.0);
    }

    #[test]
    fn test_create_planar() {
        let mut rng = ParticleRng::new(42);
        let params = EmissionParams {
            area_length: 2.0,
            area_width: 2.0,
            speed: 1.0,
            lifespan: 5.0,
            ..Default::default()
        };

        let p = create_planar(&params, &mut rng);

        // Position should be within emission area
        assert!(p.position[0].abs() <= 1.0);
        assert!(p.position[1].abs() <= 1.0);
        assert_eq!(p.position[2], 0.0);

        // Particle should be alive
        assert!(p.is_alive());
    }

    #[test]
    fn test_create_spherical() {
        let mut rng = ParticleRng::new(42);
        let params = EmissionParams {
            area_length: 1.0,
            area_width: 2.0,
            speed: 1.0,
            vertical_range: std::f32::consts::PI,
            horizontal_range: std::f32::consts::PI * 2.0,
            lifespan: 5.0,
            ..Default::default()
        };

        let p = create_spherical(&params, &mut rng, false);

        // Position should be on sphere
        let dist = (p.position[0].powi(2) + p.position[1].powi(2) + p.position[2].powi(2)).sqrt();
        assert!((0.9..=2.1).contains(&dist));

        // Particle should be alive
        assert!(p.is_alive());
    }
}
