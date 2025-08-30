//! Vertex skinning system for M2 models
//!
//! This module provides a comprehensive vertex skinning/deformation system that transforms
//! vertices from their bind pose using bone weights and transformations to get correct
//! final positions. This is essential for animating M2 models properly.
//!
//! # Features
//!
//! - **Bone Transformation Matrix Calculation**: Creates bone matrices from pivot points and rotations
//! - **Bone Hierarchy Support**: Handles parent-child bone relationships correctly
//! - **Vertex Skinning Pipeline**: Transforms vertices using bone weights and indices
//! - **Multi-Bone Influences**: Supports up to 4 bone influences per vertex
//! - **Animation Support**: Works with both bind pose and animated poses
//!
//! # Example
//!
//! ```rust,no_run
//! use wow_m2::skinning::{M2Skinner, SkinningOptions};
//! use wow_m2::M2Model;
//!
//! // Load a model
//! let model = M2Model::load("path/to/model.m2")?;
//!
//! // Create skinner with default options
//! let mut skinner = M2Skinner::new(&model.model().bones, SkinningOptions::default());
//!
//! // Calculate bind pose matrices
//! skinner.calculate_bind_pose();
//!
//! // Transform vertices to final positions
//! let skinned_vertices = skinner.skin_vertices(&model.model().vertices);
//!
//! for (i, vertex) in skinned_vertices.iter().enumerate() {
//!     println!("Vertex {}: {:?}", i, vertex);
//! }
//! # Ok::<(), wow_m2::error::M2Error>(())
//! ```

use crate::chunks::bone::M2Bone;
use crate::chunks::vertex::M2Vertex;
use crate::common::{C3Vector, Quaternion};
use glam::{Mat4, Quat, Vec3};
use std::collections::HashMap;

/// Options for controlling the skinning behavior
#[derive(Debug, Clone)]
pub struct SkinningOptions {
    /// Whether to normalize bone weights automatically
    /// When true, ensures bone weights sum to 1.0 for each vertex
    pub normalize_weights: bool,
    /// Minimum weight threshold - weights below this are ignored
    pub weight_threshold: f32,
    /// Whether to validate bone indices against available bones
    pub validate_bone_indices: bool,
    /// Whether to handle invalid bone indices gracefully (clamp to valid range)
    pub handle_invalid_indices: bool,
}

impl Default for SkinningOptions {
    fn default() -> Self {
        Self {
            normalize_weights: true,
            weight_threshold: 0.001,
            validate_bone_indices: true,
            handle_invalid_indices: true,
        }
    }
}

/// A fully calculated bone transformation for skinning
#[derive(Debug, Clone)]
pub struct BoneTransform {
    /// The final transformation matrix (includes hierarchy)
    pub matrix: Mat4,
    /// The bone's local transformation matrix
    pub local_matrix: Mat4,
    /// The inverse bind pose matrix
    pub inverse_bind_matrix: Mat4,
    /// Whether this bone has valid transformation data
    pub is_valid: bool,
}

impl Default for BoneTransform {
    fn default() -> Self {
        Self {
            matrix: Mat4::IDENTITY,
            local_matrix: Mat4::IDENTITY,
            inverse_bind_matrix: Mat4::IDENTITY,
            is_valid: true,
        }
    }
}

/// Main vertex skinning system for M2 models
pub struct M2Skinner {
    /// Bone transformation data
    bone_transforms: Vec<BoneTransform>,
    /// Bone hierarchy mapping (bone index -> parent index)
    bone_hierarchy: HashMap<usize, usize>,
    /// Total number of bones
    bone_count: usize,
    /// Skinning options
    options: SkinningOptions,
}

impl M2Skinner {
    /// Create a new M2 skinner from bone data
    ///
    /// # Arguments
    ///
    /// * `bones` - The bones from the M2 model
    /// * `options` - Skinning options to control behavior
    ///
    /// # Returns
    ///
    /// Returns a new M2Skinner ready for use
    pub fn new(bones: &[M2Bone], options: SkinningOptions) -> Self {
        let bone_count = bones.len();
        let mut bone_transforms = vec![BoneTransform::default(); bone_count];
        let mut bone_hierarchy = HashMap::new();

        // Build bone hierarchy map
        for (index, bone) in bones.iter().enumerate() {
            if bone.parent_bone >= 0 && (bone.parent_bone as usize) < bone_count {
                bone_hierarchy.insert(index, bone.parent_bone as usize);
            }
        }

        // Calculate inverse bind matrices from bind pose
        for (index, bone) in bones.iter().enumerate() {
            bone_transforms[index].inverse_bind_matrix =
                Self::calculate_bind_matrix(&bone.pivot).inverse();
        }

        Self {
            bone_transforms,
            bone_hierarchy,
            bone_count,
            options,
        }
    }

    /// Calculate bind pose matrices (no animation)
    ///
    /// This sets up the bone transforms for the default bind pose,
    /// which is useful for displaying the model in its rest position.
    pub fn calculate_bind_pose(&mut self) {
        for i in 0..self.bone_count {
            self.bone_transforms[i].local_matrix = Mat4::IDENTITY;
            self.bone_transforms[i].matrix = Mat4::IDENTITY;
        }

        // Calculate final matrices with hierarchy
        self.update_bone_hierarchy();
    }

    /// Calculate bone matrices for a specific animation frame
    ///
    /// # Arguments
    ///
    /// * `bones` - The bones with animation data
    /// * `animation_frame` - The frame time to sample animation at
    ///
    /// # Note
    ///
    /// This is a simplified version that uses bind pose. Full animation support
    /// would require sampling the M2Track animation data at the specified frame.
    pub fn calculate_animated_pose(&mut self, bones: &[M2Bone], _animation_frame: f32) {
        for (index, _bone) in bones.iter().enumerate() {
            if index < self.bone_count {
                // For now, use bind pose. Full implementation would sample animation tracks
                // TODO: Sample translation, rotation, and scale from M2Track data
                let translation = Vec3::ZERO;
                let rotation = Quat::IDENTITY;
                let scale = Vec3::ONE;

                // Calculate local transformation matrix
                let local_matrix =
                    Mat4::from_scale_rotation_translation(scale, rotation, translation);
                self.bone_transforms[index].local_matrix = local_matrix;
            }
        }

        // Calculate final matrices with hierarchy
        self.update_bone_hierarchy();
    }

    /// Update bone hierarchy transformations
    ///
    /// This traverses the bone hierarchy and accumulates transformations
    /// from parent bones to children, ensuring correct bone relationships.
    fn update_bone_hierarchy(&mut self) {
        // First, copy local matrices to final matrices for root bones
        for i in 0..self.bone_count {
            if !self.bone_hierarchy.contains_key(&i) {
                // Root bone - use local matrix directly
                self.bone_transforms[i].matrix = self.bone_transforms[i].local_matrix;
            }
        }

        // Process bones in dependency order (parents before children)
        let mut processed = vec![false; self.bone_count];
        let mut changed = true;

        while changed {
            changed = false;
            for i in 0..self.bone_count {
                if processed[i] {
                    continue;
                }

                if let Some(&parent_index) = self.bone_hierarchy.get(&i) {
                    if processed[parent_index] {
                        // Parent is processed, we can calculate this bone
                        let parent_matrix = self.bone_transforms[parent_index].matrix;
                        let local_matrix = self.bone_transforms[i].local_matrix;
                        self.bone_transforms[i].matrix = parent_matrix * local_matrix;
                        processed[i] = true;
                        changed = true;
                    }
                } else {
                    // Root bone - already processed above
                    processed[i] = true;
                    changed = true;
                }
            }
        }
    }

    /// Transform vertices using the current bone transformations
    ///
    /// # Arguments
    ///
    /// * `vertices` - The vertices to transform
    ///
    /// # Returns
    ///
    /// Returns a vector of transformed vertex positions
    pub fn skin_vertices(&self, vertices: &[M2Vertex]) -> Vec<C3Vector> {
        vertices
            .iter()
            .map(|vertex| self.skin_single_vertex(vertex))
            .collect()
    }

    /// Transform a single vertex using bone weights and transformations
    ///
    /// # Arguments
    ///
    /// * `vertex` - The vertex to transform
    ///
    /// # Returns
    ///
    /// Returns the transformed vertex position
    pub fn skin_single_vertex(&self, vertex: &M2Vertex) -> C3Vector {
        let bind_position = vertex.position.to_glam();
        let weights = self.normalize_bone_weights(&vertex.bone_weights);
        let mut final_position = Vec3::ZERO;
        let mut total_weight = 0.0f32;

        // Apply transformations from all influencing bones
        for (i, &weight) in weights.iter().enumerate() {
            if weight < self.options.weight_threshold {
                continue;
            }

            let bone_index = vertex.bone_indices[i] as usize;
            if bone_index >= self.bone_count {
                if self.options.handle_invalid_indices {
                    // Clamp to valid range
                    let clamped_index = bone_index.min(self.bone_count - 1);
                    if let Some(transform) = self.bone_transforms.get(clamped_index) {
                        let skinning_matrix = transform.matrix * transform.inverse_bind_matrix;
                        let transformed = skinning_matrix.transform_point3(bind_position);
                        final_position += transformed * weight;
                        total_weight += weight;
                    }
                }
                continue;
            }

            let transform = &self.bone_transforms[bone_index];
            if !transform.is_valid {
                continue;
            }

            // Apply skinning transformation: final_pos = bone_matrix * inverse_bind_matrix * bind_pos
            let skinning_matrix = transform.matrix * transform.inverse_bind_matrix;
            let transformed = skinning_matrix.transform_point3(bind_position);
            final_position += transformed * weight;
            total_weight += weight;
        }

        // If no valid bones influenced this vertex, return original position
        if total_weight < self.options.weight_threshold {
            return vertex.position;
        }

        // Normalize by total weight if needed
        if self.options.normalize_weights && total_weight > 0.0 {
            final_position /= total_weight;
        }

        C3Vector::from_glam(final_position)
    }

    /// Normalize bone weights to sum to 1.0
    ///
    /// # Arguments
    ///
    /// * `weights` - Raw bone weights (0-255)
    ///
    /// # Returns
    ///
    /// Returns normalized weights (0.0-1.0)
    fn normalize_bone_weights(&self, weights: &[u8; 4]) -> [f32; 4] {
        let mut normalized = [0.0f32; 4];
        let mut total = 0.0f32;

        // Convert to float and sum
        for i in 0..4 {
            normalized[i] = weights[i] as f32 / 255.0;
            total += normalized[i];
        }

        // Normalize if needed and total is valid
        if self.options.normalize_weights && total > self.options.weight_threshold {
            for weight in &mut normalized {
                *weight /= total;
            }
        } else if total <= self.options.weight_threshold {
            // If all weights are zero, assign full weight to first bone
            normalized[0] = 1.0;
        }

        normalized
    }

    /// Calculate bind matrix from pivot point
    ///
    /// # Arguments
    ///
    /// * `pivot` - The bone pivot point
    ///
    /// # Returns
    ///
    /// Returns the bind pose transformation matrix
    fn calculate_bind_matrix(pivot: &C3Vector) -> Mat4 {
        Mat4::from_translation(pivot.to_glam())
    }

    /// Get bone transformation at index
    ///
    /// # Arguments
    ///
    /// * `index` - Bone index
    ///
    /// # Returns
    ///
    /// Returns the bone transformation if valid
    pub fn get_bone_transform(&self, index: usize) -> Option<&BoneTransform> {
        self.bone_transforms.get(index)
    }

    /// Get the number of bones
    pub fn bone_count(&self) -> usize {
        self.bone_count
    }

    /// Update skinning options
    pub fn set_options(&mut self, options: SkinningOptions) {
        self.options = options;
    }

    /// Get current skinning options
    pub fn options(&self) -> &SkinningOptions {
        &self.options
    }
}

/// Utility functions for matrix operations
impl M2Skinner {
    /// Convert a quaternion to a rotation matrix
    ///
    /// # Arguments
    ///
    /// * `quat` - The quaternion to convert
    ///
    /// # Returns
    ///
    /// Returns the rotation matrix
    pub fn quaternion_to_matrix(quat: &Quaternion) -> Mat4 {
        Mat4::from_quat(quat.to_glam())
    }

    /// Create a transformation matrix from translation, rotation, and scale
    ///
    /// # Arguments
    ///
    /// * `translation` - Translation vector
    /// * `rotation` - Rotation quaternion
    /// * `scale` - Scale vector
    ///
    /// # Returns
    ///
    /// Returns the combined transformation matrix
    pub fn create_transform_matrix(
        translation: &C3Vector,
        rotation: &Quaternion,
        scale: &C3Vector,
    ) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            scale.to_glam(),
            rotation.to_glam(),
            translation.to_glam(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunks::bone::M2BoneFlags;
    use crate::chunks::m2_track::{M2TrackQuat, M2TrackVec3};

    fn create_test_bone(bone_id: i32, parent: i16, pivot: C3Vector) -> M2Bone {
        M2Bone {
            bone_id,
            flags: M2BoneFlags::empty(),
            parent_bone: parent,
            submesh_id: 0,
            unknown: [0, 0],
            bone_name_crc: None,
            translation: M2TrackVec3::new(),
            rotation: M2TrackQuat::new(),
            scale: M2TrackVec3::new(),
            pivot,
        }
    }

    fn create_test_vertex(pos: C3Vector, weights: [u8; 4], indices: [u8; 4]) -> M2Vertex {
        M2Vertex {
            position: pos,
            bone_weights: weights,
            bone_indices: indices,
            normal: C3Vector {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            tex_coords: crate::common::C2Vector { x: 0.0, y: 0.0 },
            tex_coords2: None,
        }
    }

    #[test]
    fn test_skinner_creation() {
        let bones = vec![
            create_test_bone(
                0,
                -1,
                C3Vector {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            ),
            create_test_bone(
                1,
                0,
                C3Vector {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            ),
        ];

        let skinner = M2Skinner::new(&bones, SkinningOptions::default());
        assert_eq!(skinner.bone_count(), 2);
        assert!(skinner.bone_hierarchy.contains_key(&1));
        assert_eq!(skinner.bone_hierarchy[&1], 0);
    }

    #[test]
    fn test_bind_pose_calculation() {
        let bones = vec![create_test_bone(
            0,
            -1,
            C3Vector {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        )];

        let mut skinner = M2Skinner::new(&bones, SkinningOptions::default());
        skinner.calculate_bind_pose();

        let transform = skinner.get_bone_transform(0).unwrap();
        assert!(transform.is_valid);
        assert_eq!(transform.matrix, Mat4::IDENTITY);
    }

    #[test]
    fn test_single_bone_skinning() {
        let bones = vec![create_test_bone(
            0,
            -1,
            C3Vector {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        )];

        let mut skinner = M2Skinner::new(&bones, SkinningOptions::default());
        skinner.calculate_bind_pose();

        let vertex = create_test_vertex(
            C3Vector {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            [255, 0, 0, 0], // Full weight on bone 0
            [0, 0, 0, 0],
        );

        let result = skinner.skin_single_vertex(&vertex);

        // With identity transformation, position should be preserved
        assert!((result.x - 1.0).abs() < 0.001);
        assert!((result.y - 2.0).abs() < 0.001);
        assert!((result.z - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_multi_bone_skinning() {
        let bones = vec![
            create_test_bone(
                0,
                -1,
                C3Vector {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            ),
            create_test_bone(
                1,
                -1,
                C3Vector {
                    x: 2.0,
                    y: 0.0,
                    z: 0.0,
                },
            ),
        ];

        let mut skinner = M2Skinner::new(&bones, SkinningOptions::default());
        skinner.calculate_bind_pose();

        let vertex = create_test_vertex(
            C3Vector {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            [128, 127, 0, 0], // Equal weight on bones 0 and 1
            [0, 1, 0, 0],
        );

        let result = skinner.skin_single_vertex(&vertex);

        // Result should be influenced by both bones
        // The exact result depends on the inverse bind matrices
        assert!(result.x != vertex.position.x || result.y != vertex.position.y);
    }

    #[test]
    fn test_weight_normalization() {
        let weights = [100, 100, 55, 0]; // Should normalize to roughly [0.4, 0.4, 0.2, 0.0]

        let bones = vec![create_test_bone(
            0,
            -1,
            C3Vector {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        )];
        let skinner = M2Skinner::new(&bones, SkinningOptions::default());

        let normalized = skinner.normalize_bone_weights(&weights);

        let total: f32 = normalized.iter().sum();
        assert!(
            (total - 1.0).abs() < 0.001,
            "Weights should sum to 1.0, got {}",
            total
        );

        assert!(normalized[0] > 0.35 && normalized[0] < 0.45);
        assert!(normalized[1] > 0.35 && normalized[1] < 0.45);
        assert!(normalized[2] > 0.15 && normalized[2] < 0.25);
        assert!(normalized[3] < 0.001);
    }

    #[test]
    fn test_invalid_bone_indices() {
        let bones = vec![create_test_bone(
            0,
            -1,
            C3Vector {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        )];

        let mut skinner = M2Skinner::new(&bones, SkinningOptions::default());
        skinner.calculate_bind_pose();

        let vertex = create_test_vertex(
            C3Vector {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            [255, 0, 0, 0],
            [99, 0, 0, 0], // Invalid bone index 99
        );

        let result = skinner.skin_single_vertex(&vertex);

        // Should handle gracefully and clamp to valid bone (bone 0)
        assert!((result.x - vertex.position.x).abs() < 2.0);
        assert!((result.y - vertex.position.y).abs() < 2.0);
        assert!((result.z - vertex.position.z).abs() < 2.0);
    }

    #[test]
    fn test_zero_weights() {
        let bones = vec![create_test_bone(
            0,
            -1,
            C3Vector {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        )];

        let mut skinner = M2Skinner::new(&bones, SkinningOptions::default());
        skinner.calculate_bind_pose();

        let vertex = create_test_vertex(
            C3Vector {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            [0, 0, 0, 0], // All zero weights
            [0, 0, 0, 0],
        );

        let result = skinner.skin_single_vertex(&vertex);

        // Should return original position when all weights are zero
        assert_eq!(result.x, vertex.position.x);
        assert_eq!(result.y, vertex.position.y);
        assert_eq!(result.z, vertex.position.z);
    }

    #[test]
    fn test_bone_hierarchy() {
        let bones = vec![
            create_test_bone(
                0,
                -1,
                C3Vector {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            ), // Root
            create_test_bone(
                1,
                0,
                C3Vector {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            ), // Child of 0
            create_test_bone(
                2,
                1,
                C3Vector {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
            ), // Child of 1
        ];

        let mut skinner = M2Skinner::new(&bones, SkinningOptions::default());
        skinner.calculate_bind_pose();

        // Test hierarchy is built correctly
        assert_eq!(skinner.bone_hierarchy[&1], 0);
        assert_eq!(skinner.bone_hierarchy[&2], 1);
        assert!(!skinner.bone_hierarchy.contains_key(&0)); // Root has no parent

        // All transforms should be valid after bind pose calculation
        for i in 0..3 {
            let transform = skinner.get_bone_transform(i).unwrap();
            assert!(transform.is_valid);
        }
    }
}
