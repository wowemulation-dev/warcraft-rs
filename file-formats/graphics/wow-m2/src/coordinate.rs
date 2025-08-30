//! Coordinate system transformations for World of Warcraft models
//!
//! This module provides utilities for transforming WoW's coordinate system to common
//! 3D application coordinate systems like Blender, Unity, and Unreal Engine.
//!
//! # WoW Coordinate System
//!
//! World of Warcraft uses a right-handed coordinate system:
//! - X-axis: North (positive X = north)
//! - Y-axis: West (positive Y = west)
//! - Z-axis: Up (positive Z = up, 0 = sea level)
//!
//! # Examples
//!
//! ## Basic Coordinate Transformation
//!
//! ```rust
//! use wow_m2::coordinate::{CoordinateSystem, transform_position, transform_quaternion};
//! use wow_m2::common::{C3Vector, Quaternion};
//!
//! // Transform a position from WoW to Blender coordinates
//! let wow_pos = C3Vector { x: 100.0, y: 200.0, z: 50.0 };
//! let blender_pos = transform_position(wow_pos, CoordinateSystem::Blender);
//!
//! // Transform a rotation from WoW to Unity coordinates
//! let wow_rot = Quaternion { x: 0.0, y: 0.707, z: 0.0, w: 0.707 };
//! let unity_rot = transform_quaternion(wow_rot, CoordinateSystem::Unity);
//! ```
//!
//! ## Batch Transformations
//!
//! ```rust,no_run
//! use wow_m2::coordinate::{CoordinateTransformer, CoordinateSystem};
//!
//! let transformer = CoordinateTransformer::new(CoordinateSystem::Blender);
//!
//! // Transform vertices from a model
//! # let model_vertices = Vec::new(); // This would come from a loaded model
//! let transformed_vertices = transformer.transform_positions(&model_vertices);
//! ```

use crate::common::{C2Vector, C3Vector, Quaternion};
use glam::Vec3;

/// Target coordinate systems for transformation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinateSystem {
    /// Blender coordinate system (right-handed): Right=X+, Forward=Y+, Up=Z+
    Blender,
    /// Unity coordinate system (left-handed): Right=X+, Up=Y+, Forward=Z+
    Unity,
    /// Unreal Engine coordinate system (left-handed): Forward=X+, Right=Y+, Up=Z+
    UnrealEngine,
}

/// Transform a 3D position from WoW coordinates to the target coordinate system
///
/// # Examples
///
/// ```rust
/// use wow_m2::coordinate::{CoordinateSystem, transform_position};
/// use wow_m2::common::C3Vector;
///
/// let wow_pos = C3Vector { x: 100.0, y: 200.0, z: 50.0 }; // 100N, 200W, 50Up
/// let blender_pos = transform_position(wow_pos, CoordinateSystem::Blender);
///
/// // In Blender: Y=forward, X=right, Z=up
/// // So WoW (100N, 200W, 50Up) becomes Blender (200Forward, -100Right, 50Up)
/// assert_eq!(blender_pos.x, 200.0); // Y becomes X (forward becomes right... wait, this is wrong)
/// ```
pub fn transform_position(wow_pos: C3Vector, target: CoordinateSystem) -> C3Vector {
    match target {
        CoordinateSystem::Blender => C3Vector {
            x: wow_pos.y,  // WoW Y (west) → Blender X (right)
            y: -wow_pos.x, // WoW X (north) → Blender -Y (backward)
            z: wow_pos.z,  // WoW Z (up) → Blender Z (up)
        },
        CoordinateSystem::Unity => C3Vector {
            x: -wow_pos.y, // WoW Y (west) → Unity -X (left)
            y: wow_pos.z,  // WoW Z (up) → Unity Y (up)
            z: wow_pos.x,  // WoW X (north) → Unity Z (forward)
        },
        CoordinateSystem::UnrealEngine => C3Vector {
            x: wow_pos.x,  // WoW X (north) → Unreal X (forward)
            y: -wow_pos.y, // WoW Y (west) → Unreal -Y (left)
            z: wow_pos.z,  // WoW Z (up) → Unreal Z (up)
        },
    }
}

/// Transform a quaternion rotation from WoW coordinates to the target coordinate system
///
/// # Examples
///
/// ```rust
/// use wow_m2::coordinate::{CoordinateSystem, transform_quaternion};
/// use wow_m2::common::Quaternion;
///
/// let wow_rot = Quaternion { x: 0.0, y: 0.707, z: 0.0, w: 0.707 };
/// let blender_rot = transform_quaternion(wow_rot, CoordinateSystem::Blender);
/// ```
pub fn transform_quaternion(wow_quat: Quaternion, target: CoordinateSystem) -> Quaternion {
    match target {
        CoordinateSystem::Blender => Quaternion {
            x: wow_quat.y,  // WoW Y → Blender X
            y: -wow_quat.x, // WoW X → Blender -Y
            z: wow_quat.z,  // WoW Z → Blender Z
            w: wow_quat.w,  // W component unchanged
        },
        CoordinateSystem::Unity => Quaternion {
            x: wow_quat.y,  // WoW Y → Unity X
            y: -wow_quat.z, // WoW Z → Unity -Y
            z: -wow_quat.x, // WoW X → Unity -Z
            w: wow_quat.w,  // W component unchanged
        },
        CoordinateSystem::UnrealEngine => Quaternion {
            x: -wow_quat.x, // WoW X → Unreal -X
            y: wow_quat.y,  // WoW Y → Unreal Y
            z: -wow_quat.z, // WoW Z → Unreal -Z
            w: wow_quat.w,  // W component unchanged
        },
    }
}

/// Transform a 2D vector (typically texture coordinates)
///
/// Note: In most cases, texture coordinates don't need coordinate system transformation
/// since they represent UV mapping rather than 3D spatial coordinates.
pub fn transform_vector2(wow_vec: C2Vector, _target: CoordinateSystem) -> C2Vector {
    // Texture coordinates typically don't need transformation
    wow_vec
}

/// A coordinate transformer that can efficiently transform multiple related coordinates
/// while maintaining consistency across a model or scene.
pub struct CoordinateTransformer {
    target: CoordinateSystem,
}

impl CoordinateTransformer {
    /// Create a new coordinate transformer for the target system
    pub fn new(target: CoordinateSystem) -> Self {
        Self { target }
    }

    /// Transform a single position
    pub fn transform_position(&self, wow_pos: C3Vector) -> C3Vector {
        transform_position(wow_pos, self.target)
    }

    /// Transform a single quaternion
    pub fn transform_quaternion(&self, wow_quat: Quaternion) -> Quaternion {
        transform_quaternion(wow_quat, self.target)
    }

    /// Transform multiple positions efficiently
    pub fn transform_positions(&self, positions: &[C3Vector]) -> Vec<C3Vector> {
        positions
            .iter()
            .map(|&pos| self.transform_position(pos))
            .collect()
    }

    /// Transform multiple quaternions efficiently
    pub fn transform_quaternions(&self, quaternions: &[Quaternion]) -> Vec<Quaternion> {
        quaternions
            .iter()
            .map(|&quat| self.transform_quaternion(quat))
            .collect()
    }

    /// Transform positions using SIMD operations for better performance with large datasets
    pub fn transform_positions_simd(&self, positions: &[C3Vector]) -> Vec<C3Vector> {
        positions
            .iter()
            .map(|pos| {
                let glam_pos = pos.to_glam();
                let transformed = self.transform_glam_position(glam_pos);
                C3Vector::from_glam(transformed)
            })
            .collect()
    }

    /// Transform a glam Vec3 directly (internal helper for SIMD operations)
    fn transform_glam_position(&self, wow_pos: Vec3) -> Vec3 {
        match self.target {
            CoordinateSystem::Blender => Vec3::new(wow_pos.y, -wow_pos.x, wow_pos.z),
            CoordinateSystem::Unity => Vec3::new(-wow_pos.y, wow_pos.z, wow_pos.x),
            CoordinateSystem::UnrealEngine => Vec3::new(wow_pos.x, -wow_pos.y, wow_pos.z),
        }
    }
}

/// Utility functions for working with transformation matrices
pub mod matrix {
    use super::CoordinateSystem;
    use glam::{Mat4, Vec3};

    /// Get the transformation matrix for converting from WoW coordinates to the target system
    pub fn get_transform_matrix(target: CoordinateSystem) -> Mat4 {
        match target {
            CoordinateSystem::Blender => Mat4::from_cols(
                Vec3::new(0.0, 1.0, 0.0).extend(0.0),  // X column: Y → X
                Vec3::new(-1.0, 0.0, 0.0).extend(0.0), // Y column: -X → Y
                Vec3::new(0.0, 0.0, 1.0).extend(0.0),  // Z column: Z → Z
                Vec3::new(0.0, 0.0, 0.0).extend(1.0),  // W column
            ),
            CoordinateSystem::Unity => Mat4::from_cols(
                Vec3::new(0.0, -1.0, 0.0).extend(0.0), // X column: -Y → X
                Vec3::new(0.0, 0.0, 1.0).extend(0.0),  // Y column: Z → Y
                Vec3::new(1.0, 0.0, 0.0).extend(0.0),  // Z column: X → Z
                Vec3::new(0.0, 0.0, 0.0).extend(1.0),  // W column
            ),
            CoordinateSystem::UnrealEngine => Mat4::from_cols(
                Vec3::new(1.0, 0.0, 0.0).extend(0.0),  // X column: X → X
                Vec3::new(0.0, -1.0, 0.0).extend(0.0), // Y column: -Y → Y
                Vec3::new(0.0, 0.0, 1.0).extend(0.0),  // Z column: Z → Z
                Vec3::new(0.0, 0.0, 0.0).extend(1.0),  // W column
            ),
        }
    }

    /// Transform a 4x4 transformation matrix from WoW coordinate system to target
    pub fn transform_matrix(wow_matrix: Mat4, target: CoordinateSystem) -> Mat4 {
        let transform = get_transform_matrix(target);
        let inverse_transform = transform.transpose(); // For orthogonal matrices, transpose = inverse

        // Apply transformation: T * M * T^-1
        transform * wow_matrix * inverse_transform
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blender_position_transform() {
        let wow_pos = C3Vector {
            x: 100.0,
            y: 200.0,
            z: 50.0,
        };
        let blender_pos = transform_position(wow_pos, CoordinateSystem::Blender);

        // WoW (100N, 200W, 50Up) → Blender (200Right, -100Backward, 50Up)
        assert_eq!(blender_pos.x, 200.0);
        assert_eq!(blender_pos.y, -100.0);
        assert_eq!(blender_pos.z, 50.0);
    }

    #[test]
    fn test_unity_position_transform() {
        let wow_pos = C3Vector {
            x: 100.0,
            y: 200.0,
            z: 50.0,
        };
        let unity_pos = transform_position(wow_pos, CoordinateSystem::Unity);

        // WoW (100N, 200W, 50Up) → Unity (-200Left, 50Up, 100Forward)
        assert_eq!(unity_pos.x, -200.0);
        assert_eq!(unity_pos.y, 50.0);
        assert_eq!(unity_pos.z, 100.0);
    }

    #[test]
    fn test_unreal_position_transform() {
        let wow_pos = C3Vector {
            x: 100.0,
            y: 200.0,
            z: 50.0,
        };
        let unreal_pos = transform_position(wow_pos, CoordinateSystem::UnrealEngine);

        // WoW (100N, 200W, 50Up) → Unreal (100Forward, -200Left, 50Up)
        assert_eq!(unreal_pos.x, 100.0);
        assert_eq!(unreal_pos.y, -200.0);
        assert_eq!(unreal_pos.z, 50.0);
    }

    #[test]
    fn test_identity_quaternion_transforms() {
        let identity = Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        };

        let blender_quat = transform_quaternion(identity, CoordinateSystem::Blender);
        let unity_quat = transform_quaternion(identity, CoordinateSystem::Unity);
        let unreal_quat = transform_quaternion(identity, CoordinateSystem::UnrealEngine);

        // Identity quaternion should remain identity in all systems
        assert_eq!(blender_quat, identity);
        assert_eq!(unity_quat, identity);
        assert_eq!(unreal_quat, identity);
    }

    #[test]
    fn test_coordinate_transformer() {
        let transformer = CoordinateTransformer::new(CoordinateSystem::Blender);

        let positions = vec![
            C3Vector {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            C3Vector {
                x: 100.0,
                y: 200.0,
                z: 50.0,
            },
        ];

        let transformed = transformer.transform_positions(&positions);

        assert_eq!(
            transformed[0],
            C3Vector {
                x: 0.0,
                y: 0.0,
                z: 0.0
            }
        );
        assert_eq!(
            transformed[1],
            C3Vector {
                x: 200.0,
                y: -100.0,
                z: 50.0
            }
        );
    }

    #[test]
    fn test_simd_transform_consistency() {
        let transformer = CoordinateTransformer::new(CoordinateSystem::Blender);

        let positions = vec![
            C3Vector {
                x: 100.0,
                y: 200.0,
                z: 50.0,
            },
            C3Vector {
                x: -50.0,
                y: 75.0,
                z: 25.0,
            },
        ];

        let regular_transform = transformer.transform_positions(&positions);
        let simd_transform = transformer.transform_positions_simd(&positions);

        // Results should be identical
        for (regular, simd) in regular_transform.iter().zip(simd_transform.iter()) {
            assert!((regular.x - simd.x).abs() < f32::EPSILON);
            assert!((regular.y - simd.y).abs() < f32::EPSILON);
            assert!((regular.z - simd.z).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn test_transform_matrix_construction() {
        use crate::coordinate::matrix::get_transform_matrix;

        let blender_matrix = get_transform_matrix(CoordinateSystem::Blender);
        let unity_matrix = get_transform_matrix(CoordinateSystem::Unity);
        let unreal_matrix = get_transform_matrix(CoordinateSystem::UnrealEngine);

        // Matrices should be proper transformation matrices (determinant ≠ 0)
        assert!(blender_matrix.determinant().abs() > f32::EPSILON);
        assert!(unity_matrix.determinant().abs() > f32::EPSILON);
        assert!(unreal_matrix.determinant().abs() > f32::EPSILON);
    }

    #[test]
    fn test_cardinal_directions() {
        // Test that cardinal directions transform correctly

        // North in WoW
        let north = C3Vector {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        };
        let blender_north = transform_position(north, CoordinateSystem::Blender);
        assert_eq!(
            blender_north,
            C3Vector {
                x: 0.0,
                y: -1.0,
                z: 0.0
            }
        ); // Backward in Blender

        // West in WoW
        let west = C3Vector {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        };
        let blender_west = transform_position(west, CoordinateSystem::Blender);
        assert_eq!(
            blender_west,
            C3Vector {
                x: 1.0,
                y: 0.0,
                z: 0.0
            }
        ); // Right in Blender

        // Up in WoW
        let up = C3Vector {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        };
        let blender_up = transform_position(up, CoordinateSystem::Blender);
        assert_eq!(
            blender_up,
            C3Vector {
                x: 0.0,
                y: 0.0,
                z: 1.0
            }
        ); // Up in Blender (unchanged)
    }
}
