//! Individual particle representation

/// A single particle in the system
#[derive(Debug, Clone, Default)]
pub struct Particle {
    /// Current age in seconds
    pub age: f32,
    /// Total lifespan in seconds
    pub lifespan: f32,
    /// Current color (RGBA, 0.0-1.0)
    pub color: [f32; 4],
    /// Current scale (width, height)
    pub scale: [f32; 2],
    /// Texture coordinates for head cell
    pub tex_coord_head: [f32; 2],
    /// Texture coordinates for tail cell (for ribbon particles)
    pub tex_coord_tail: [f32; 2],
    /// World-space position
    pub position: [f32; 3],
    /// Velocity vector
    pub velocity: [f32; 3],
}

impl Particle {
    /// Create a new particle with initial position, velocity, and lifespan
    pub fn new(position: [f32; 3], velocity: [f32; 3], lifespan: f32) -> Self {
        Self {
            age: 0.0,
            lifespan,
            color: [1.0, 1.0, 1.0, 1.0],
            scale: [1.0, 1.0],
            tex_coord_head: [0.0, 0.0],
            tex_coord_tail: [0.0, 0.0],
            position,
            velocity,
        }
    }

    /// Check if the particle is still alive
    #[inline]
    pub fn is_alive(&self) -> bool {
        self.age < self.lifespan
    }

    /// Get the age as a percentage of lifespan (0.0 to 1.0)
    #[inline]
    pub fn age_percent(&self) -> f32 {
        if self.lifespan > 0.0 {
            (self.age / self.lifespan).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }

    /// Update particle physics
    ///
    /// # Arguments
    /// * `dt` - Delta time in seconds
    /// * `force` - Combined force vector (wind - gravity)
    /// * `drag` - Drag coefficient (0.0 = no drag, 1.0 = full drag)
    pub fn update_physics(&mut self, dt: f32, force: [f32; 3], drag: f32) {
        // Apply forces to velocity
        self.velocity[0] += force[0] * dt;
        self.velocity[1] += force[1] * dt;
        self.velocity[2] += force[2] * dt;

        // Apply drag
        if drag > 0.0 {
            let drag_factor = (1.0 - drag).powf(dt);
            self.velocity[0] *= drag_factor;
            self.velocity[1] *= drag_factor;
            self.velocity[2] *= drag_factor;
        }

        // Update position
        self.position[0] += self.velocity[0] * dt;
        self.position[1] += self.velocity[1] * dt;
        self.position[2] += self.velocity[2] * dt;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_new() {
        let p = Particle::new([1.0, 2.0, 3.0], [0.1, 0.2, 0.3], 5.0);
        assert_eq!(p.position, [1.0, 2.0, 3.0]);
        assert_eq!(p.velocity, [0.1, 0.2, 0.3]);
        assert_eq!(p.lifespan, 5.0);
        assert_eq!(p.age, 0.0);
        assert!(p.is_alive());
    }

    #[test]
    fn test_particle_age_percent() {
        let mut p = Particle::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 10.0);
        assert_eq!(p.age_percent(), 0.0);

        p.age = 5.0;
        assert_eq!(p.age_percent(), 0.5);

        p.age = 10.0;
        assert_eq!(p.age_percent(), 1.0);

        p.age = 15.0; // Past lifespan
        assert_eq!(p.age_percent(), 1.0); // Clamped
    }

    #[test]
    fn test_particle_is_alive() {
        let mut p = Particle::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 5.0);
        assert!(p.is_alive());

        p.age = 4.9;
        assert!(p.is_alive());

        p.age = 5.0;
        assert!(!p.is_alive());

        p.age = 6.0;
        assert!(!p.is_alive());
    }

    #[test]
    fn test_particle_physics() {
        let mut p = Particle::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 10.0);

        // Apply gravity (downward force)
        p.update_physics(1.0, [0.0, 0.0, -9.8], 0.0);

        // Velocity should be affected
        assert!((p.velocity[2] - (-9.8)).abs() < 0.001);

        // Position should be updated
        assert!((p.position[0] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_particle_drag() {
        let mut p = Particle::new([0.0, 0.0, 0.0], [10.0, 0.0, 0.0], 10.0);

        // Apply drag
        p.update_physics(1.0, [0.0, 0.0, 0.0], 0.5);

        // Velocity should be reduced
        assert!(p.velocity[0] < 10.0);
        assert!(p.velocity[0] > 0.0);
    }
}
