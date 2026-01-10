//! Keyframe interpolation for M2 animation tracks

use super::types::{Lerp, ResolvedTrack};

/// Find the index of the keyframe at or before the given time
///
/// Returns None if the track has no keyframes.
/// For interpolation, this returns the index of the earlier keyframe
/// in the bracketing pair (so we can interpolate between `[index]` and `[index+1]`).
pub fn find_timestamp_index(timestamps: &[u32], time: f64) -> Option<usize> {
    if timestamps.is_empty() {
        return None;
    }

    if timestamps.len() == 1 {
        return Some(0);
    }

    let last_index = timestamps.len() - 1;

    // If past or at the last timestamp, return last index
    if time >= timestamps[last_index] as f64 {
        return Some(last_index);
    }

    // Binary search for the bracketing keyframe
    // We want the largest index where timestamps[index] <= time
    let mut low = 0;
    let mut high = last_index;

    while low < high {
        let mid = (low + high).div_ceil(2);
        if timestamps[mid] as f64 <= time {
            low = mid;
        } else {
            high = mid - 1;
        }
    }

    Some(low)
}

/// Interpolate a track value at the given time
///
/// # Arguments
/// * `track` - The resolved track with keyframe data
/// * `animation_index` - Index of the animation sequence
/// * `time` - Current time in milliseconds
/// * `global_times` - Global sequence times (for tracks using global sequences)
/// * `default` - Default value if track has no data
pub fn interpolate_track<T: Lerp + Clone>(
    track: &ResolvedTrack<T>,
    animation_index: usize,
    time: f64,
    global_times: &[f64],
    default: T,
) -> T {
    // Handle global sequence
    let effective_time = if track.uses_global_sequence() {
        let gs_index = track.global_sequence as usize;
        if gs_index < global_times.len() {
            global_times[gs_index]
        } else {
            time
        }
    } else {
        time
    };

    // Get correct animation index (fall back to 0 if out of range)
    let anim_idx = if animation_index < track.timestamps.len() {
        animation_index
    } else if !track.timestamps.is_empty() {
        0
    } else {
        return default;
    };

    let timestamps = &track.timestamps[anim_idx];
    let values = &track.values[anim_idx];

    if timestamps.is_empty() || values.is_empty() {
        return default;
    }

    // Find keyframe index
    let Some(time_index) = find_timestamp_index(timestamps, effective_time) else {
        return values.first().cloned().unwrap_or(default);
    };

    // If at or past last keyframe, return last value
    if time_index >= timestamps.len() - 1 {
        return values.last().cloned().unwrap_or(default);
    }

    // Get bracketing keyframes
    let time1 = timestamps[time_index] as f64;
    let time2 = timestamps[time_index + 1] as f64;
    let value1 = &values[time_index];
    let value2 = &values[time_index + 1];

    // Handle interpolation based on type
    match track.interpolation_type {
        0 => {
            // None - step interpolation
            value1.clone()
        }
        1 => {
            // Linear interpolation
            let t = if time2 > time1 {
                ((effective_time - time1) / (time2 - time1)) as f32
            } else {
                0.0
            };
            value1.lerp(value2, t.clamp(0.0, 1.0))
        }
        2 | 3 => {
            // Bezier/Hermite - treat as linear for now
            // Full implementation would use tangent data
            let t = if time2 > time1 {
                ((effective_time - time1) / (time2 - time1)) as f32
            } else {
                0.0
            };
            value1.lerp(value2, t.clamp(0.0, 1.0))
        }
        _ => value1.clone(),
    }
}

/// Interpolate with blending between two animation states
///
/// # Arguments
/// * `track` - The resolved track
/// * `current_anim` - Current animation index
/// * `current_time` - Current animation time
/// * `next_anim` - Next animation index (for blending)
/// * `next_time` - Next animation time
/// * `blend_factor` - Blend factor (1.0 = current only, 0.0 = next only)
/// * `global_times` - Global sequence times
/// * `default` - Default value
#[allow(clippy::too_many_arguments)]
pub fn interpolate_with_blend<T: Lerp + Clone>(
    track: &ResolvedTrack<T>,
    current_anim: usize,
    current_time: f64,
    next_anim: Option<usize>,
    next_time: f64,
    blend_factor: f32,
    global_times: &[f64],
    default: T,
) -> T {
    let current_value = interpolate_track(
        track,
        current_anim,
        current_time,
        global_times,
        default.clone(),
    );

    // If blend factor is ~1.0 or no next animation, return current value
    if blend_factor >= 0.999 || next_anim.is_none() {
        return current_value;
    }

    let next_anim_idx = next_anim.unwrap();
    let next_value = interpolate_track(track, next_anim_idx, next_time, global_times, default);

    // Blend between current and next
    current_value.lerp(&next_value, 1.0 - blend_factor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::types::Vec3;

    #[test]
    fn test_find_timestamp_index_empty() {
        let timestamps: Vec<u32> = vec![];
        assert_eq!(find_timestamp_index(&timestamps, 0.0), None);
    }

    #[test]
    fn test_find_timestamp_index_single() {
        let timestamps = vec![100];
        assert_eq!(find_timestamp_index(&timestamps, 0.0), Some(0));
        assert_eq!(find_timestamp_index(&timestamps, 100.0), Some(0));
        assert_eq!(find_timestamp_index(&timestamps, 200.0), Some(0));
    }

    #[test]
    fn test_find_timestamp_index_multiple() {
        let timestamps = vec![0, 100, 200, 300];

        // Before first
        assert_eq!(find_timestamp_index(&timestamps, 0.0), Some(0));

        // Between keyframes
        assert_eq!(find_timestamp_index(&timestamps, 50.0), Some(0));
        assert_eq!(find_timestamp_index(&timestamps, 150.0), Some(1));
        assert_eq!(find_timestamp_index(&timestamps, 250.0), Some(2));

        // At keyframes
        assert_eq!(find_timestamp_index(&timestamps, 100.0), Some(1));
        assert_eq!(find_timestamp_index(&timestamps, 200.0), Some(2));

        // After last
        assert_eq!(find_timestamp_index(&timestamps, 400.0), Some(3));
    }

    #[test]
    fn test_interpolate_linear() {
        let track = ResolvedTrack {
            interpolation_type: 1, // Linear
            global_sequence: -1,
            timestamps: vec![vec![0, 100]],
            values: vec![vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 10.0, 10.0)]],
        };

        let global_times = vec![];

        // At start
        let v = interpolate_track(&track, 0, 0.0, &global_times, Vec3::ZERO);
        assert!((v.x - 0.0).abs() < 0.001);

        // At middle
        let v = interpolate_track(&track, 0, 50.0, &global_times, Vec3::ZERO);
        assert!((v.x - 5.0).abs() < 0.001);

        // At end
        let v = interpolate_track(&track, 0, 100.0, &global_times, Vec3::ZERO);
        assert!((v.x - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_interpolate_step() {
        let track = ResolvedTrack {
            interpolation_type: 0, // None (step)
            global_sequence: -1,
            timestamps: vec![vec![0, 100]],
            values: vec![vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 10.0, 10.0)]],
        };

        let global_times = vec![];

        // Should always return first value until we pass it
        let v = interpolate_track(&track, 0, 50.0, &global_times, Vec3::ZERO);
        assert!((v.x - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_interpolate_global_sequence() {
        let track = ResolvedTrack {
            interpolation_type: 1,
            global_sequence: 0, // Use global sequence 0
            timestamps: vec![vec![0, 100]],
            values: vec![vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 10.0, 10.0)]],
        };

        // Global sequence time at 50ms
        let global_times = vec![50.0];

        // Should use global time (50), not animation time (0)
        let v = interpolate_track(&track, 0, 0.0, &global_times, Vec3::ZERO);
        assert!((v.x - 5.0).abs() < 0.001);
    }
}
