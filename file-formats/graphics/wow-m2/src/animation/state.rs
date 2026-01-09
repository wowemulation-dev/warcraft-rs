//! Animation state tracking for M2 models

/// Represents the current state of an animation
#[derive(Debug, Clone)]
pub struct AnimationState {
    /// Index of the current animation sequence
    pub animation_index: Option<usize>,
    /// Number of times to repeat the current animation
    pub repeat_times: i32,
    /// Current time within the animation (milliseconds)
    pub animation_time: f64,
    /// Index of the main variation (for animation loops)
    pub main_variation_index: usize,
}

impl AnimationState {
    /// Create a new animation state
    pub fn new(animation_index: Option<usize>) -> Self {
        Self {
            animation_index,
            repeat_times: 0,
            animation_time: 0.0,
            main_variation_index: animation_index.unwrap_or(0),
        }
    }

    /// Create an empty/inactive animation state
    pub fn none() -> Self {
        Self {
            animation_index: None,
            repeat_times: 0,
            animation_time: 0.0,
            main_variation_index: 0,
        }
    }

    /// Check if this state has an active animation
    pub fn is_active(&self) -> bool {
        self.animation_index.is_some()
    }

    /// Reset the animation time to the beginning
    pub fn reset_time(&mut self) {
        self.animation_time = 0.0;
    }
}

impl Default for AnimationState {
    fn default() -> Self {
        Self::none()
    }
}

/// Simple linear congruential generator for deterministic randomness
/// Used for animation variation selection (matches noclip behavior)
#[derive(Debug, Clone)]
pub struct LcgRng {
    state: u32,
}

impl LcgRng {
    /// Create a new RNG with the given seed
    pub fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    /// Generate next random u16
    pub fn next_u16(&mut self) -> u16 {
        self.state = self.state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        self.state %= 1 << 31;
        self.state as u16
    }

    /// Generate next random f32 in [0.0, 1.0)
    pub fn next_f32(&mut self) -> f32 {
        self.next_u16() as f32 / u16::MAX as f32
    }
}

impl Default for LcgRng {
    fn default() -> Self {
        Self::new(1312) // Default seed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_state_new() {
        let state = AnimationState::new(Some(5));
        assert_eq!(state.animation_index, Some(5));
        assert_eq!(state.repeat_times, 0);
        assert_eq!(state.animation_time, 0.0);
        assert_eq!(state.main_variation_index, 5);
        assert!(state.is_active());
    }

    #[test]
    fn test_animation_state_none() {
        let state = AnimationState::none();
        assert_eq!(state.animation_index, None);
        assert!(!state.is_active());
    }

    #[test]
    fn test_lcg_rng_deterministic() {
        let mut rng1 = LcgRng::new(42);
        let mut rng2 = LcgRng::new(42);

        for _ in 0..10 {
            assert_eq!(rng1.next_u16(), rng2.next_u16());
        }
    }

    #[test]
    fn test_lcg_rng_range() {
        let mut rng = LcgRng::new(42);
        for _ in 0..100 {
            let f = rng.next_f32();
            assert!((0.0..1.0).contains(&f));
        }
    }
}
