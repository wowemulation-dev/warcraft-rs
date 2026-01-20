//! Animation state machine for M2 models
//!
//! The AnimationManager tracks animation playback state and provides
//! interpolated values for bones, textures, colors, and other animated properties.

use super::interpolation::interpolate_with_blend;
use super::state::{AnimationState, LcgRng};
use super::types::{Lerp, Quat, ResolvedTrack, Vec3};

/// Animation sequence data (resolved from M2Sequence)
#[derive(Debug, Clone)]
pub struct AnimSequence {
    /// Animation ID (e.g., 0=Stand, 4=Walk, 5=Run)
    pub id: u16,
    /// Sub-animation ID for variations
    pub sub_id: u16,
    /// Duration in milliseconds
    pub duration: u32,
    /// Movement speed
    pub movement_speed: f32,
    /// Flags
    pub flags: u32,
    /// Probability weight for variation selection
    pub frequency: u16,
    /// Minimum repeat count
    pub replay_min: u32,
    /// Maximum repeat count
    pub replay_max: u32,
    /// Blend time for transitions (milliseconds)
    pub blend_time: u32,
    /// Index of next variation (-1 if none)
    pub variation_next: i16,
    /// Index of aliased animation
    pub alias_next: u16,
}

impl AnimSequence {
    /// Calculate number of repeats for this animation
    pub fn calculate_repeats(&self, rng: &mut LcgRng) -> i32 {
        if self.replay_max <= self.replay_min {
            return self.replay_min as i32;
        }
        let range = (self.replay_max - self.replay_min) as f32;
        self.replay_min as i32 + (range * rng.next_f32()) as i32
    }

    /// Check if this is an alias (references another animation)
    pub fn is_alias(&self) -> bool {
        (self.flags & 0x40) != 0 && (self.flags & 0x20) == 0
    }
}

/// Resolved bone animation data
#[derive(Debug, Clone)]
pub struct ResolvedBone {
    /// Bone ID
    pub bone_id: i32,
    /// Bone flags
    pub flags: u32,
    /// Parent bone index (-1 if root)
    pub parent_bone: i16,
    /// Translation animation track
    pub translation: ResolvedTrack<Vec3>,
    /// Rotation animation track (quaternion)
    pub rotation: ResolvedTrack<Quat>,
    /// Scale animation track
    pub scale: ResolvedTrack<Vec3>,
    /// Pivot point
    pub pivot: Vec3,
}

/// M2 Animation Manager
///
/// Manages animation state and provides interpolated values for animated properties.
/// Modeled after noclip.website's WowM2AnimationManager.
#[derive(Debug, Clone)]
pub struct AnimationManager {
    /// Global sequence durations (milliseconds)
    global_sequence_durations: Vec<u32>,
    /// Current time for each global sequence
    global_sequence_times: Vec<f64>,
    /// Animation sequences
    sequences: Vec<AnimSequence>,
    /// Resolved bone data with animation tracks
    bones: Vec<ResolvedBone>,
    /// Current animation state
    current_animation: AnimationState,
    /// Next animation state (for blending)
    next_animation: AnimationState,
    /// Blend factor between current and next (1.0 = current only)
    blend_factor: f32,
    /// Random number generator for variation selection
    rng: LcgRng,
}

impl AnimationManager {
    /// Create a new AnimationManager
    ///
    /// # Arguments
    /// * `global_sequence_durations` - Durations of global sequences
    /// * `sequences` - Animation sequence definitions
    /// * `bones` - Resolved bone animation data
    pub fn new(
        global_sequence_durations: Vec<u32>,
        sequences: Vec<AnimSequence>,
        bones: Vec<ResolvedBone>,
    ) -> Self {
        let global_sequence_times = vec![0.0; global_sequence_durations.len()];

        // Find "Stand" animation (ID 0) as default
        let stand_index = sequences.iter().position(|s| s.id == 0);

        let mut rng = LcgRng::default();
        let mut current_animation = AnimationState::new(stand_index);

        // Set initial repeat count
        if let Some(idx) = stand_index {
            current_animation.repeat_times = sequences[idx].calculate_repeats(&mut rng);
        }

        Self {
            global_sequence_durations,
            global_sequence_times,
            sequences,
            bones,
            current_animation,
            next_animation: AnimationState::none(),
            blend_factor: 1.0,
            rng,
        }
    }

    /// Create an empty AnimationManager (no animations)
    pub fn empty() -> Self {
        Self {
            global_sequence_durations: Vec::new(),
            global_sequence_times: Vec::new(),
            sequences: Vec::new(),
            bones: Vec::new(),
            current_animation: AnimationState::none(),
            next_animation: AnimationState::none(),
            blend_factor: 1.0,
            rng: LcgRng::default(),
        }
    }

    /// Update animation state by the given delta time (milliseconds)
    pub fn update(&mut self, delta_time_ms: f64) {
        // Update current animation time
        self.current_animation.animation_time += delta_time_ms;

        // Update global sequence times
        for (i, time) in self.global_sequence_times.iter_mut().enumerate() {
            *time += delta_time_ms;
            if self.global_sequence_durations[i] > 0 {
                *time %= self.global_sequence_durations[i] as f64;
            }
        }

        // Handle animation transitions
        self.update_animation_transitions();
    }

    /// Handle animation looping and transitions
    fn update_animation_transitions(&mut self) {
        let Some(current_idx) = self.current_animation.animation_index else {
            return;
        };

        if current_idx >= self.sequences.len() {
            return;
        }

        let main_variation = &self.sequences[self.current_animation.main_variation_index];

        // Select next animation if needed
        if self.next_animation.animation_index.is_none()
            && main_variation.variation_next > -1
            && self.current_animation.repeat_times <= 0
        {
            self.select_next_variation();
        } else if self.current_animation.repeat_times > 0 {
            // Setup repeat of current animation
            self.next_animation = self.current_animation.clone();
            self.next_animation.repeat_times -= 1;
        }

        // Calculate blend factor
        let current_seq = &self.sequences[current_idx];
        let time_left = current_seq.duration as f64 - self.current_animation.animation_time;

        if let Some(next_idx) = self.next_animation.animation_index {
            let next_seq = &self.sequences[next_idx];
            let blend_time = next_seq.blend_time as f64;

            if blend_time > 0.0 && time_left < blend_time {
                self.next_animation.animation_time =
                    (blend_time - time_left) % next_seq.duration as f64;
                self.blend_factor = (time_left / blend_time) as f32;
            } else {
                self.blend_factor = 1.0;
            }
        }

        // Handle animation completion
        if self.current_animation.animation_time >= current_seq.duration as f64 {
            self.current_animation.repeat_times -= 1;

            if let Some(next_idx) = self.next_animation.animation_index {
                // Resolve aliases
                let resolved_idx = self.resolve_alias(next_idx);
                self.next_animation.animation_index = Some(resolved_idx);

                // Swap to next animation
                self.current_animation = self.next_animation.clone();
                self.next_animation = AnimationState::none();
                self.blend_factor = 1.0;
            } else if current_seq.duration > 0 {
                // Loop current animation
                self.current_animation.animation_time %= current_seq.duration as f64;
            }
        }
    }

    /// Select next variation based on frequency weights
    fn select_next_variation(&mut self) {
        let main_idx = self.current_animation.main_variation_index;
        let probability = (self.rng.next_f32() * 0x7fff as f32) as u16;

        let mut calc_prob = 0u16;
        let mut next_index = main_idx;

        loop {
            let seq = &self.sequences[next_index];
            calc_prob = calc_prob.saturating_add(seq.frequency);

            if calc_prob >= probability || seq.variation_next < 0 {
                break;
            }

            let potential_next = seq.variation_next as usize;
            if potential_next >= self.sequences.len() {
                break;
            }

            // Skip current animation in probability calculation
            if Some(potential_next) != self.current_animation.animation_index {
                next_index = potential_next;
            } else if seq.variation_next >= 0 {
                next_index = seq.variation_next as usize;
            }
        }

        self.next_animation.animation_index = Some(next_index);
        self.next_animation.animation_time = 0.0;
        self.next_animation.main_variation_index = main_idx;
        self.next_animation.repeat_times =
            self.sequences[next_index].calculate_repeats(&mut self.rng);
    }

    /// Resolve animation alias chain
    fn resolve_alias(&self, index: usize) -> usize {
        let mut current = index;
        let mut iterations = 0;

        while iterations < 100 {
            // Safety limit
            if current >= self.sequences.len() {
                return index;
            }

            let seq = &self.sequences[current];
            if !seq.is_alias() {
                return current;
            }

            current = seq.alias_next as usize;
            iterations += 1;
        }

        index
    }

    /// Set the current animation by ID
    pub fn set_animation_id(&mut self, id: u16) {
        let index = self.sequences.iter().position(|s| s.id == id);

        if let Some(idx) = index {
            self.current_animation = AnimationState::new(Some(idx));
            self.current_animation.repeat_times =
                self.sequences[idx].calculate_repeats(&mut self.rng);
            self.next_animation = AnimationState::none();
            self.blend_factor = 1.0;
        }
    }

    /// Set the current animation by index
    pub fn set_animation_index(&mut self, index: usize) {
        if index < self.sequences.len() {
            self.current_animation = AnimationState::new(Some(index));
            self.current_animation.repeat_times =
                self.sequences[index].calculate_repeats(&mut self.rng);
            self.next_animation = AnimationState::none();
            self.blend_factor = 1.0;
        }
    }

    /// Get available animation IDs
    pub fn get_animation_ids(&self) -> Vec<u16> {
        self.sequences.iter().map(|s| s.id).collect()
    }

    /// Get current animation time (milliseconds)
    pub fn current_time(&self) -> f64 {
        self.current_animation.animation_time
    }

    /// Get current animation index
    pub fn current_animation_index(&self) -> Option<usize> {
        self.current_animation.animation_index
    }

    /// Get number of bones
    pub fn bone_count(&self) -> usize {
        self.bones.len()
    }

    /// Get number of sequences
    pub fn sequence_count(&self) -> usize {
        self.sequences.len()
    }

    /// Get interpolated value from a track with current animation blending
    pub fn get_current_value<T: Lerp + Clone + Default>(&self, track: &ResolvedTrack<T>) -> T {
        self.get_current_value_with_default(track, T::default())
    }

    /// Get interpolated value with a custom default
    pub fn get_current_value_with_default<T: Lerp + Clone>(
        &self,
        track: &ResolvedTrack<T>,
        default: T,
    ) -> T {
        let Some(current_idx) = self.current_animation.animation_index else {
            return default;
        };

        interpolate_with_blend(
            track,
            current_idx,
            self.current_animation.animation_time,
            self.next_animation.animation_index,
            self.next_animation.animation_time,
            self.blend_factor,
            &self.global_sequence_times,
            default,
        )
    }

    /// Get bone translation for the given bone index
    pub fn get_bone_translation(&self, bone_index: usize) -> Vec3 {
        if bone_index >= self.bones.len() {
            return Vec3::ZERO;
        }
        self.get_current_value_with_default(&self.bones[bone_index].translation, Vec3::ZERO)
    }

    /// Get bone rotation for the given bone index
    pub fn get_bone_rotation(&self, bone_index: usize) -> Quat {
        if bone_index >= self.bones.len() {
            return Quat::IDENTITY;
        }
        self.get_current_value_with_default(&self.bones[bone_index].rotation, Quat::IDENTITY)
    }

    /// Get bone scale for the given bone index
    pub fn get_bone_scale(&self, bone_index: usize) -> Vec3 {
        if bone_index >= self.bones.len() {
            return Vec3::ONE;
        }
        self.get_current_value_with_default(&self.bones[bone_index].scale, Vec3::ONE)
    }

    /// Get bone data (for computing transforms)
    pub fn bones(&self) -> &[ResolvedBone] {
        &self.bones
    }

    /// Get sequences
    pub fn sequences(&self) -> &[AnimSequence] {
        &self.sequences
    }

    /// Get global sequence times
    pub fn global_times(&self) -> &[f64] {
        &self.global_sequence_times
    }

    /// Get blend factor
    pub fn blend_factor(&self) -> f32 {
        self.blend_factor
    }
}

/// Builder for creating AnimationManager from M2Model data
pub struct AnimationManagerBuilder;

impl AnimationManagerBuilder {
    /// Create an AnimationManager from M2Model data
    ///
    /// This resolves all bone animation tracks from the raw M2 data and creates
    /// a fully functional animation manager ready for playback.
    ///
    /// # Arguments
    /// * `model` - The parsed M2 model
    /// * `data` - The raw M2 file bytes (needed to resolve animation data offsets)
    ///
    /// # Returns
    /// An AnimationManager ready for animation playback, or an error if resolution fails
    ///
    /// # Example
    /// ```rust,ignore
    /// use wow_m2::{M2Model, animation::AnimationManagerBuilder};
    ///
    /// let data = std::fs::read("model.m2")?;
    /// let model = M2Model::parse(&mut std::io::Cursor::new(&data))?;
    /// let manager = AnimationManagerBuilder::from_model(&model, &data)?;
    ///
    /// // Update animation each frame
    /// manager.update(delta_time_ms);
    /// let translation = manager.get_bone_translation(0);
    /// ```
    pub fn from_model(model: &crate::M2Model, data: &[u8]) -> crate::Result<AnimationManager> {
        use std::io::Cursor;

        // Extract global sequence durations
        let global_sequence_durations: Vec<u32> = model.global_sequences.to_vec();

        // Convert M2Animation to AnimSequence
        let sequences: Vec<AnimSequence> = model
            .animations
            .iter()
            .map(|seq| {
                // Duration calculation differs by version:
                // - Classic: end_timestamp - start_timestamp
                // - BC+: start_timestamp IS the duration
                let duration = seq
                    .end_timestamp
                    .map(|end| end.saturating_sub(seq.start_timestamp))
                    .unwrap_or(seq.start_timestamp);

                // Replay range (Classic only, defaults for BC+)
                let (replay_min, replay_max) = seq
                    .replay
                    .map(|r| (r.minimum as u32, r.maximum as u32))
                    .unwrap_or((0, 0));

                AnimSequence {
                    id: seq.animation_id,
                    sub_id: seq.sub_animation_id,
                    duration,
                    movement_speed: seq.movement_speed,
                    flags: seq.flags,
                    frequency: seq.frequency as u16,
                    replay_min,
                    replay_max,
                    blend_time: 150, // Default blend time (milliseconds)
                    variation_next: seq.next_animation.unwrap_or(-1),
                    alias_next: seq.aliasing.unwrap_or(0),
                }
            })
            .collect();

        let num_sequences = sequences.len();

        // Resolve bone animation data
        let mut cursor = Cursor::new(data);
        let mut bones = Vec::with_capacity(model.bones.len());

        for bone in &model.bones {
            // Resolve translation track
            let translation =
                Self::resolve_vec3_track(&bone.translation, &mut cursor, num_sequences)?;

            // Resolve rotation track (quaternion)
            let rotation = Self::resolve_quat_track(&bone.rotation, &mut cursor, num_sequences)?;

            // Resolve scale track
            let scale = Self::resolve_vec3_track(&bone.scale, &mut cursor, num_sequences)?;

            bones.push(ResolvedBone {
                bone_id: bone.bone_id,
                flags: bone.flags.bits(),
                parent_bone: bone.parent_bone,
                translation,
                rotation,
                scale,
                pivot: Vec3::new(bone.pivot.x, bone.pivot.y, bone.pivot.z),
            });
        }

        Ok(AnimationManager::new(
            global_sequence_durations,
            sequences,
            bones,
        ))
    }

    /// Resolve a Vec3 animation track from M2 data
    fn resolve_vec3_track<R: std::io::Read + std::io::Seek>(
        track: &crate::chunks::m2_track::M2TrackVec3,
        reader: &mut R,
        num_sequences: usize,
    ) -> crate::Result<ResolvedTrack<Vec3>> {
        use crate::chunks::m2_track_resolver::M2TrackVec3Ext;

        if !track.has_data() {
            return Ok(ResolvedTrack::empty());
        }

        let (timestamps_flat, values_flat, ranges) = track.resolve_data(reader)?;

        // Convert C3Vector to Vec3
        let values_vec3: Vec<Vec3> = values_flat
            .into_iter()
            .map(|v| Vec3::new(v.x, v.y, v.z))
            .collect();

        // Convert global_sequence: 0xFFFF means no global sequence, map to -1
        let global_sequence = if track.base.global_sequence == 0xFFFF {
            -1i16
        } else {
            track.base.global_sequence as i16
        };

        // If using global sequence, put all data in one sequence slot
        if global_sequence >= 0 {
            return Ok(ResolvedTrack {
                interpolation_type: track.base.interpolation_type as u16,
                global_sequence,
                timestamps: vec![timestamps_flat],
                values: vec![values_vec3],
            });
        }

        // Split by animation sequence using ranges
        let (timestamps, values) = Self::split_by_ranges(
            timestamps_flat,
            values_vec3,
            ranges.as_deref(),
            num_sequences,
        );

        Ok(ResolvedTrack {
            interpolation_type: track.base.interpolation_type as u16,
            global_sequence,
            timestamps,
            values,
        })
    }

    /// Resolve a Quat animation track from M2 data
    fn resolve_quat_track<R: std::io::Read + std::io::Seek>(
        track: &crate::chunks::m2_track::M2TrackQuat,
        reader: &mut R,
        num_sequences: usize,
    ) -> crate::Result<ResolvedTrack<Quat>> {
        use crate::chunks::m2_track_resolver::M2TrackQuatExt;

        if !track.has_data() {
            return Ok(ResolvedTrack::empty());
        }

        let (timestamps_flat, values_flat, ranges) = track.resolve_data(reader)?;

        // Convert M2CompQuat to Quat
        let values_quat: Vec<Quat> = values_flat
            .into_iter()
            .map(|q| {
                let (x, y, z, w) = q.to_float_quaternion();
                Quat::new(x, y, z, w).normalize()
            })
            .collect();

        // Convert global_sequence
        let global_sequence = if track.base.global_sequence == 0xFFFF {
            -1i16
        } else {
            track.base.global_sequence as i16
        };

        // If using global sequence, put all data in one sequence slot
        if global_sequence >= 0 {
            return Ok(ResolvedTrack {
                interpolation_type: track.base.interpolation_type as u16,
                global_sequence,
                timestamps: vec![timestamps_flat],
                values: vec![values_quat],
            });
        }

        // Split by animation sequence using ranges
        let (timestamps, values) = Self::split_by_ranges(
            timestamps_flat,
            values_quat,
            ranges.as_deref(),
            num_sequences,
        );

        Ok(ResolvedTrack {
            interpolation_type: track.base.interpolation_type as u16,
            global_sequence,
            timestamps,
            values,
        })
    }

    /// Split flat timestamp/value arrays by animation sequence ranges
    ///
    /// Pre-WotLK M2s store ranges as pairs of (start, end) indices.
    /// WotLK+ M2s don't have ranges and store one timestamp/value per sequence.
    fn split_by_ranges<T: Clone>(
        timestamps_flat: Vec<u32>,
        values_flat: Vec<T>,
        ranges: Option<&[u32]>,
        num_sequences: usize,
    ) -> (Vec<Vec<u32>>, Vec<Vec<T>>) {
        if let Some(ranges) = ranges {
            // Pre-WotLK: use ranges to split data
            let mut timestamps = Vec::with_capacity(num_sequences);
            let mut values = Vec::with_capacity(num_sequences);

            for i in 0..num_sequences {
                let range_idx = i * 2;
                if range_idx + 1 < ranges.len() {
                    let start = ranges[range_idx] as usize;
                    let end = ranges[range_idx + 1] as usize;

                    if start <= end && end <= timestamps_flat.len() && end <= values_flat.len() {
                        timestamps.push(timestamps_flat[start..end].to_vec());
                        values.push(values_flat[start..end].to_vec());
                    } else {
                        timestamps.push(Vec::new());
                        values.push(Vec::new());
                    }
                } else {
                    timestamps.push(Vec::new());
                    values.push(Vec::new());
                }
            }

            (timestamps, values)
        } else {
            // WotLK+: timestamps/values are stored per-sequence in order
            // This is a simplified view - in practice WotLK+ uses external .anim files
            // For embedded data, we assume it's for all sequences
            let mut timestamps = Vec::with_capacity(num_sequences);
            let mut values = Vec::with_capacity(num_sequences);

            // Put all data in first sequence, empty for rest
            if !timestamps_flat.is_empty() {
                timestamps.push(timestamps_flat);
                values.push(values_flat);
            }

            // Pad with empty vectors for remaining sequences
            while timestamps.len() < num_sequences {
                timestamps.push(Vec::new());
                values.push(Vec::new());
            }

            (timestamps, values)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_sequence(id: u16, duration: u32) -> AnimSequence {
        AnimSequence {
            id,
            sub_id: 0,
            duration,
            movement_speed: 0.0,
            flags: 0,
            frequency: 0x7fff,
            replay_min: 0,
            replay_max: 0,
            blend_time: 0,
            variation_next: -1,
            alias_next: 0,
        }
    }

    #[test]
    fn test_animation_manager_empty() {
        let manager = AnimationManager::empty();
        assert_eq!(manager.bone_count(), 0);
        assert_eq!(manager.sequence_count(), 0);
        assert_eq!(manager.current_animation_index(), None);
    }

    #[test]
    fn test_animation_manager_basic() {
        let sequences = vec![
            create_test_sequence(0, 1000), // Stand
            create_test_sequence(4, 500),  // Walk
        ];

        let manager = AnimationManager::new(vec![], sequences, vec![]);

        // Should start with Stand animation
        assert_eq!(manager.current_animation_index(), Some(0));
        assert_eq!(manager.sequence_count(), 2);
    }

    #[test]
    fn test_animation_update() {
        let sequences = vec![create_test_sequence(0, 1000)];
        let mut manager = AnimationManager::new(vec![], sequences, vec![]);

        // Advance time
        manager.update(500.0);
        assert!((manager.current_time() - 500.0).abs() < 0.001);

        // Advance past duration (should loop)
        manager.update(600.0);
        assert!(manager.current_time() < 1000.0);
    }

    #[test]
    fn test_set_animation() {
        let sequences = vec![create_test_sequence(0, 1000), create_test_sequence(4, 500)];
        let mut manager = AnimationManager::new(vec![], sequences, vec![]);

        manager.set_animation_id(4);
        assert_eq!(manager.current_animation_index(), Some(1));
        assert!((manager.current_time() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_global_sequences() {
        let sequences = vec![create_test_sequence(0, 1000)];
        let global_durations = vec![500, 1000];

        let mut manager = AnimationManager::new(global_durations, sequences, vec![]);

        // Update global times
        manager.update(250.0);

        let times = manager.global_times();
        assert!((times[0] - 250.0).abs() < 0.001);
        assert!((times[1] - 250.0).abs() < 0.001);

        // Check wrapping
        manager.update(300.0); // Total 550ms
        let times = manager.global_times();
        assert!((times[0] - 50.0).abs() < 0.001); // 550 % 500 = 50
        assert!((times[1] - 550.0).abs() < 0.001); // No wrap yet
    }

    #[test]
    fn test_bone_interpolation() {
        let bone = ResolvedBone {
            bone_id: 0,
            flags: 0,
            parent_bone: -1,
            translation: ResolvedTrack {
                interpolation_type: 1, // Linear
                global_sequence: -1,
                timestamps: vec![vec![0, 100]],
                values: vec![vec![Vec3::ZERO, Vec3::new(10.0, 0.0, 0.0)]],
            },
            rotation: ResolvedTrack::empty(),
            scale: ResolvedTrack::empty(),
            pivot: Vec3::ZERO,
        };

        let sequences = vec![create_test_sequence(0, 1000)];
        let mut manager = AnimationManager::new(vec![], sequences, vec![bone]);

        // At time 0
        let trans = manager.get_bone_translation(0);
        assert!((trans.x - 0.0).abs() < 0.001);

        // At time 50
        manager.update(50.0);
        let trans = manager.get_bone_translation(0);
        assert!((trans.x - 5.0).abs() < 0.001);

        // At time 100
        manager.update(50.0);
        let trans = manager.get_bone_translation(0);
        assert!((trans.x - 10.0).abs() < 0.001);
    }
}
