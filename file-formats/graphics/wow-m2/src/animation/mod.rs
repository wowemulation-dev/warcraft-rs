//! M2 Animation System
//!
//! This module provides animation playback support for M2 models, including:
//! - Animation state machine for tracking current/next animations
//! - Keyframe interpolation (linear, step, hermite)
//! - Global sequence support
//! - Animation blending between states
//! - Bone hierarchy transform computation
//!
//! # Example
//!
//! ```rust,ignore
//! use wow_m2::animation::{
//!     AnimationManager, AnimSequence, ResolvedBone,
//!     BoneTransformComputer, Mat4,
//! };
//!
//! // Create manager from model data
//! let manager = AnimationManager::new(
//!     global_sequence_durations,
//!     sequences,
//!     bones,
//! );
//!
//! // Update animation state
//! manager.update(delta_time_ms);
//!
//! // Get interpolated bone values
//! let translation = manager.get_bone_translation(0);
//! let rotation = manager.get_bone_rotation(0);
//!
//! // Compute bone hierarchy transforms
//! let mut transform_computer = BoneTransformComputer::new(&pivots, &parents, &flags);
//! transform_computer.update(&translations, &rotations, &scales);
//! let gpu_data = transform_computer.get_gpu_data();
//! ```

mod bone_transform;
mod interpolation;
mod manager;
mod state;
mod types;

pub use bone_transform::{BoneFlags, BoneTransformComputer, ComputedBone, Mat4};
pub use interpolation::{find_timestamp_index, interpolate_track, interpolate_with_blend};
pub use manager::{AnimSequence, AnimationManager, ResolvedBone};
pub use state::{AnimationState, LcgRng};
pub use types::{Fixedi16, Lerp, Quat, ResolvedTrack, Vec3};
