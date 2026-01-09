//! Bone hierarchy transform computation for M2 skeletal animation
//!
//! This module computes bone transformation matrices from interpolated
//! animation values, handling parent chain traversal, pivot points,
//! and billboard bone modes.

use super::types::{Quat, Vec3};

/// 4x4 transformation matrix (column-major, like OpenGL/WebGL)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4 {
    /// Matrix data in column-major order
    pub data: [f32; 16],
}

impl Mat4 {
    /// Identity matrix
    pub const IDENTITY: Self = Self {
        data: [
            1.0, 0.0, 0.0, 0.0, // Column 0
            0.0, 1.0, 0.0, 0.0, // Column 1
            0.0, 0.0, 1.0, 0.0, // Column 2
            0.0, 0.0, 0.0, 1.0, // Column 3
        ],
    };

    /// Create identity matrix
    pub fn identity() -> Self {
        Self::IDENTITY
    }

    /// Create translation matrix
    pub fn from_translation(v: Vec3) -> Self {
        Self {
            data: [
                1.0, 0.0, 0.0, 0.0, // Column 0
                0.0, 1.0, 0.0, 0.0, // Column 1
                0.0, 0.0, 1.0, 0.0, // Column 2
                v.x, v.y, v.z, 1.0, // Column 3
            ],
        }
    }

    /// Create scale matrix
    pub fn from_scale(v: Vec3) -> Self {
        Self {
            data: [
                v.x, 0.0, 0.0, 0.0, // Column 0
                0.0, v.y, 0.0, 0.0, // Column 1
                0.0, 0.0, v.z, 0.0, // Column 2
                0.0, 0.0, 0.0, 1.0, // Column 3
            ],
        }
    }

    /// Create rotation matrix from quaternion
    pub fn from_rotation(q: Quat) -> Self {
        let x = q.x;
        let y = q.y;
        let z = q.z;
        let w = q.w;

        let x2 = x + x;
        let y2 = y + y;
        let z2 = z + z;

        let xx = x * x2;
        let xy = x * y2;
        let xz = x * z2;
        let yy = y * y2;
        let yz = y * z2;
        let zz = z * z2;
        let wx = w * x2;
        let wy = w * y2;
        let wz = w * z2;

        Self {
            data: [
                1.0 - (yy + zz),
                xy + wz,
                xz - wy,
                0.0,
                xy - wz,
                1.0 - (xx + zz),
                yz + wx,
                0.0,
                xz + wy,
                yz - wx,
                1.0 - (xx + yy),
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
            ],
        }
    }

    /// Create transformation matrix from rotation, translation, and scale
    pub fn from_rotation_translation_scale(rotation: Quat, translation: Vec3, scale: Vec3) -> Self {
        let x = rotation.x;
        let y = rotation.y;
        let z = rotation.z;
        let w = rotation.w;

        let x2 = x + x;
        let y2 = y + y;
        let z2 = z + z;

        let xx = x * x2;
        let xy = x * y2;
        let xz = x * z2;
        let yy = y * y2;
        let yz = y * z2;
        let zz = z * z2;
        let wx = w * x2;
        let wy = w * y2;
        let wz = w * z2;

        let sx = scale.x;
        let sy = scale.y;
        let sz = scale.z;

        Self {
            data: [
                (1.0 - (yy + zz)) * sx,
                (xy + wz) * sx,
                (xz - wy) * sx,
                0.0,
                (xy - wz) * sy,
                (1.0 - (xx + zz)) * sy,
                (yz + wx) * sy,
                0.0,
                (xz + wy) * sz,
                (yz - wx) * sz,
                (1.0 - (xx + yy)) * sz,
                0.0,
                translation.x,
                translation.y,
                translation.z,
                1.0,
            ],
        }
    }

    /// Multiply two matrices (self * other)
    pub fn mul(&self, other: &Self) -> Self {
        let a = &self.data;
        let b = &other.data;

        let a00 = a[0];
        let a01 = a[1];
        let a02 = a[2];
        let a03 = a[3];
        let a10 = a[4];
        let a11 = a[5];
        let a12 = a[6];
        let a13 = a[7];
        let a20 = a[8];
        let a21 = a[9];
        let a22 = a[10];
        let a23 = a[11];
        let a30 = a[12];
        let a31 = a[13];
        let a32 = a[14];
        let a33 = a[15];

        let b00 = b[0];
        let b01 = b[1];
        let b02 = b[2];
        let b03 = b[3];
        let b10 = b[4];
        let b11 = b[5];
        let b12 = b[6];
        let b13 = b[7];
        let b20 = b[8];
        let b21 = b[9];
        let b22 = b[10];
        let b23 = b[11];
        let b30 = b[12];
        let b31 = b[13];
        let b32 = b[14];
        let b33 = b[15];

        Self {
            data: [
                b00 * a00 + b01 * a10 + b02 * a20 + b03 * a30,
                b00 * a01 + b01 * a11 + b02 * a21 + b03 * a31,
                b00 * a02 + b01 * a12 + b02 * a22 + b03 * a32,
                b00 * a03 + b01 * a13 + b02 * a23 + b03 * a33,
                b10 * a00 + b11 * a10 + b12 * a20 + b13 * a30,
                b10 * a01 + b11 * a11 + b12 * a21 + b13 * a31,
                b10 * a02 + b11 * a12 + b12 * a22 + b13 * a32,
                b10 * a03 + b11 * a13 + b12 * a23 + b13 * a33,
                b20 * a00 + b21 * a10 + b22 * a20 + b23 * a30,
                b20 * a01 + b21 * a11 + b22 * a21 + b23 * a31,
                b20 * a02 + b21 * a12 + b22 * a22 + b23 * a32,
                b20 * a03 + b21 * a13 + b22 * a23 + b23 * a33,
                b30 * a00 + b31 * a10 + b32 * a20 + b33 * a30,
                b30 * a01 + b31 * a11 + b32 * a21 + b33 * a31,
                b30 * a02 + b31 * a12 + b32 * a22 + b33 * a32,
                b30 * a03 + b31 * a13 + b32 * a23 + b33 * a33,
            ],
        }
    }

    /// Transform a point by this matrix
    pub fn transform_point(&self, p: Vec3) -> Vec3 {
        let m = &self.data;
        Vec3 {
            x: m[0] * p.x + m[4] * p.y + m[8] * p.z + m[12],
            y: m[1] * p.x + m[5] * p.y + m[9] * p.z + m[13],
            z: m[2] * p.x + m[6] * p.y + m[10] * p.z + m[14],
        }
    }

    /// Transform a normal (direction) by this matrix (ignores translation)
    pub fn transform_normal(&self, n: Vec3) -> Vec3 {
        let m = &self.data;
        Vec3 {
            x: m[0] * n.x + m[4] * n.y + m[8] * n.z,
            y: m[1] * n.x + m[5] * n.y + m[9] * n.z,
            z: m[2] * n.x + m[6] * n.y + m[10] * n.z,
        }
    }

    /// Get matrix as flat array for GPU upload
    pub fn as_array(&self) -> &[f32; 16] {
        &self.data
    }

    /// Get matrix as 4x3 for GPU upload (strips last row, common for skinning)
    pub fn as_4x3(&self) -> [f32; 12] {
        [
            self.data[0],
            self.data[1],
            self.data[2],
            self.data[4],
            self.data[5],
            self.data[6],
            self.data[8],
            self.data[9],
            self.data[10],
            self.data[12],
            self.data[13],
            self.data[14],
        ]
    }
}

impl Default for Mat4 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl std::ops::Mul for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Self) -> Self::Output {
        Mat4::mul(&self, &rhs)
    }
}

/// Bone flags for transform computation
#[derive(Debug, Clone, Copy, Default)]
pub struct BoneFlags {
    /// Don't inherit parent translation
    pub ignore_parent_translate: bool,
    /// Don't inherit parent scale
    pub ignore_parent_scale: bool,
    /// Don't inherit parent rotation
    pub ignore_parent_rotation: bool,
    /// Spherical billboard (always faces camera)
    pub spherical_billboard: bool,
    /// Cylindrical billboard locked to X axis
    pub cylindrical_billboard_lock_x: bool,
    /// Cylindrical billboard locked to Y axis
    pub cylindrical_billboard_lock_y: bool,
    /// Cylindrical billboard locked to Z axis
    pub cylindrical_billboard_lock_z: bool,
}

impl BoneFlags {
    /// Parse bone flags from M2 bone flags value
    pub fn from_raw(flags: u32) -> Self {
        Self {
            ignore_parent_translate: (flags & 0x01) != 0,
            ignore_parent_scale: (flags & 0x02) != 0,
            ignore_parent_rotation: (flags & 0x04) != 0,
            spherical_billboard: (flags & 0x08) != 0,
            cylindrical_billboard_lock_x: (flags & 0x10) != 0,
            cylindrical_billboard_lock_y: (flags & 0x20) != 0,
            cylindrical_billboard_lock_z: (flags & 0x40) != 0,
        }
    }

    /// Check if this bone uses any billboard mode
    pub fn is_billboard(&self) -> bool {
        self.spherical_billboard
            || self.cylindrical_billboard_lock_x
            || self.cylindrical_billboard_lock_y
            || self.cylindrical_billboard_lock_z
    }
}

/// Computed bone data for rendering
#[derive(Debug, Clone)]
pub struct ComputedBone {
    /// Pre-billboard transform (position in world space)
    pub transform: Mat4,
    /// Post-billboard transform (includes rotation/scale)
    pub post_billboard_transform: Mat4,
    /// Whether this bone uses spherical billboard
    pub is_spherical_billboard: bool,
}

impl Default for ComputedBone {
    fn default() -> Self {
        Self {
            transform: Mat4::IDENTITY,
            post_billboard_transform: Mat4::IDENTITY,
            is_spherical_billboard: false,
        }
    }
}

/// Bone transform computer for hierarchical bone matrices
///
/// Handles the transformation chain from local bone space to model space,
/// accounting for parent bones, pivot points, and billboard modes.
#[allow(dead_code)]
pub struct BoneTransformComputer {
    /// Computed bone transforms
    bones: Vec<ComputedBone>,
    /// Pivot matrices (translation by pivot point)
    pivots: Vec<Mat4>,
    /// Anti-pivot matrices (translation by -pivot point)
    anti_pivots: Vec<Mat4>,
    /// Parent bone indices (-1 for root bones)
    parents: Vec<i16>,
    /// Bone flags
    flags: Vec<BoneFlags>,
    /// Scratch matrix for computation
    scratch: Mat4,
}

impl BoneTransformComputer {
    /// Create a new bone transform computer
    ///
    /// # Arguments
    /// * `pivots` - Pivot points for each bone
    /// * `parents` - Parent bone index for each bone (-1 for root)
    /// * `flags` - Bone flags for each bone
    pub fn new(pivot_points: &[Vec3], parents: &[i16], raw_flags: &[u32]) -> Self {
        let count = pivot_points.len();

        let mut pivots = Vec::with_capacity(count);
        let mut anti_pivots = Vec::with_capacity(count);
        let mut flags = Vec::with_capacity(count);
        let mut bones = Vec::with_capacity(count);

        // Pre-compute pivot matrices and propagate billboard flags
        let mut is_billboard = vec![false; count];

        for i in 0..count {
            let pivot = pivot_points[i];
            pivots.push(Mat4::from_translation(pivot));
            anti_pivots.push(Mat4::from_translation(Vec3::new(
                -pivot.x, -pivot.y, -pivot.z,
            )));

            let bone_flags = BoneFlags::from_raw(raw_flags[i]);
            is_billboard[i] = bone_flags.spherical_billboard;
            flags.push(bone_flags);
            bones.push(ComputedBone::default());
        }

        // Propagate spherical billboard flag from parents (children inherit)
        for i in 0..count {
            let parent_idx = parents[i];
            if parent_idx >= 0 && (parent_idx as usize) < count && is_billboard[parent_idx as usize]
            {
                is_billboard[i] = true;
            }
        }

        // Set final billboard flags
        for i in 0..count {
            bones[i].is_spherical_billboard = is_billboard[i];
        }

        Self {
            bones,
            pivots,
            anti_pivots,
            parents: parents.to_vec(),
            flags,
            scratch: Mat4::IDENTITY,
        }
    }

    /// Create an empty computer (no bones)
    pub fn empty() -> Self {
        Self {
            bones: Vec::new(),
            pivots: Vec::new(),
            anti_pivots: Vec::new(),
            parents: Vec::new(),
            flags: Vec::new(),
            scratch: Mat4::IDENTITY,
        }
    }

    /// Get number of bones
    pub fn bone_count(&self) -> usize {
        self.bones.len()
    }

    /// Update all bone transforms from interpolated animation values
    ///
    /// # Arguments
    /// * `translations` - Interpolated translation for each bone
    /// * `rotations` - Interpolated rotation for each bone
    /// * `scales` - Interpolated scale for each bone
    pub fn update(&mut self, translations: &[Vec3], rotations: &[Quat], scales: &[Vec3]) {
        let count = self.bones.len();
        if count == 0 {
            return;
        }

        for i in 0..count {
            let translation = translations.get(i).copied().unwrap_or(Vec3::ZERO);
            let rotation = rotations.get(i).copied().unwrap_or(Quat::IDENTITY);
            let scale = scales.get(i).copied().unwrap_or(Vec3::ONE);

            // Build local transform: rotation * translation * scale
            self.scratch = Mat4::from_rotation_translation_scale(rotation, translation, scale);

            // Apply pivot: pivot * localTransform
            let local_bone_transform = self.pivots[i].mul(&self.scratch);

            let parent_idx = self.parents[i];
            let is_billboard = self.bones[i].is_spherical_billboard;

            if parent_idx >= 0 && (parent_idx as usize) < count {
                let parent_idx = parent_idx as usize;
                // Clone parent transforms to avoid borrow issues
                let parent_transform = self.bones[parent_idx].transform;
                let parent_post_billboard = self.bones[parent_idx].post_billboard_transform;

                if is_billboard {
                    // Billboard bones: transform is parent * antiPivot
                    // postBillboard accumulates the local transform
                    self.bones[i].transform = parent_transform.mul(&self.anti_pivots[i]);
                    self.bones[i].post_billboard_transform =
                        parent_post_billboard.mul(&local_bone_transform);
                } else {
                    // Regular bones: apply antiPivot to local transform
                    let final_local = local_bone_transform.mul(&self.anti_pivots[i]);
                    self.bones[i].post_billboard_transform =
                        parent_post_billboard.mul(&final_local);
                    // For non-billboard, transform is same as post_billboard
                    self.bones[i].transform = self.bones[i].post_billboard_transform;
                }
            } else {
                // Root bone (no parent)
                if is_billboard {
                    self.bones[i].transform = self.anti_pivots[i];
                    self.bones[i].post_billboard_transform = local_bone_transform;
                } else {
                    let final_local = local_bone_transform.mul(&self.anti_pivots[i]);
                    self.bones[i].post_billboard_transform = final_local;
                    self.bones[i].transform = final_local;
                }
            }
        }
    }

    /// Get computed bone data
    pub fn bones(&self) -> &[ComputedBone] {
        &self.bones
    }

    /// Get transform for a specific bone
    pub fn get_transform(&self, bone_index: usize) -> Mat4 {
        self.bones
            .get(bone_index)
            .map(|b| b.transform)
            .unwrap_or(Mat4::IDENTITY)
    }

    /// Get post-billboard transform for a specific bone
    pub fn get_post_billboard_transform(&self, bone_index: usize) -> Mat4 {
        self.bones
            .get(bone_index)
            .map(|b| b.post_billboard_transform)
            .unwrap_or(Mat4::IDENTITY)
    }

    /// Get combined transform for skinning (uses post_billboard for most bones)
    ///
    /// For regular bones, this returns post_billboard_transform.
    /// For billboard bones, the shader combines transform and post_billboard
    /// with the camera rotation.
    pub fn get_skinning_transform(&self, bone_index: usize) -> Mat4 {
        self.bones
            .get(bone_index)
            .map(|b| {
                if b.is_spherical_billboard {
                    // Billboard bones need special handling in shader
                    // Return identity here; shader will use both matrices
                    b.post_billboard_transform
                } else {
                    b.post_billboard_transform
                }
            })
            .unwrap_or(Mat4::IDENTITY)
    }

    /// Check if a bone uses spherical billboard
    pub fn is_spherical_billboard(&self, bone_index: usize) -> bool {
        self.bones
            .get(bone_index)
            .is_some_and(|b| b.is_spherical_billboard)
    }

    /// Get all bone matrices as flat array for GPU upload
    ///
    /// Each bone contributes 2 mat4x3 matrices (24 floats):
    /// - transform (for billboard positioning)
    /// - post_billboard_transform (for final skinning)
    pub fn get_gpu_data(&self) -> Vec<f32> {
        let mut data = Vec::with_capacity(self.bones.len() * 28); // 24 floats + 4 for flags

        for bone in &self.bones {
            // Pack billboard flag as first element
            data.push(if bone.is_spherical_billboard {
                1.0
            } else {
                0.0
            });
            // Padding for alignment
            data.push(0.0);
            data.push(0.0);
            data.push(0.0);

            // Transform matrix (4x3)
            data.extend_from_slice(&bone.transform.as_4x3());

            // Post-billboard transform matrix (4x3)
            data.extend_from_slice(&bone.post_billboard_transform.as_4x3());
        }

        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mat4_identity() {
        let m = Mat4::identity();
        assert_eq!(m.data[0], 1.0);
        assert_eq!(m.data[5], 1.0);
        assert_eq!(m.data[10], 1.0);
        assert_eq!(m.data[15], 1.0);
    }

    #[test]
    fn test_mat4_translation() {
        let m = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let p = m.transform_point(Vec3::ZERO);
        assert!((p.x - 1.0).abs() < 0.001);
        assert!((p.y - 2.0).abs() < 0.001);
        assert!((p.z - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_mat4_scale() {
        let m = Mat4::from_scale(Vec3::new(2.0, 3.0, 4.0));
        let p = m.transform_point(Vec3::new(1.0, 1.0, 1.0));
        assert!((p.x - 2.0).abs() < 0.001);
        assert!((p.y - 3.0).abs() < 0.001);
        assert!((p.z - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_mat4_multiply_identity() {
        let a = Mat4::identity();
        let b = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let c = a.mul(&b);
        assert_eq!(c.data, b.data);
    }

    #[test]
    fn test_bone_flags() {
        let flags = BoneFlags::from_raw(0x08); // spherical billboard
        assert!(flags.spherical_billboard);
        assert!(!flags.cylindrical_billboard_lock_x);

        let flags = BoneFlags::from_raw(0x10); // cylindrical X
        assert!(!flags.spherical_billboard);
        assert!(flags.cylindrical_billboard_lock_x);
    }

    #[test]
    fn test_bone_transform_computer_empty() {
        let computer = BoneTransformComputer::empty();
        assert_eq!(computer.bone_count(), 0);
    }

    #[test]
    fn test_bone_transform_single_bone() {
        let pivots = vec![Vec3::ZERO];
        let parents = vec![-1i16];
        let flags = vec![0u32];

        let mut computer = BoneTransformComputer::new(&pivots, &parents, &flags);

        // Update with identity transform
        let translations = vec![Vec3::ZERO];
        let rotations = vec![Quat::IDENTITY];
        let scales = vec![Vec3::ONE];

        computer.update(&translations, &rotations, &scales);

        let transform = computer.get_transform(0);
        // Should be identity
        assert!((transform.data[0] - 1.0).abs() < 0.001);
        assert!((transform.data[5] - 1.0).abs() < 0.001);
        assert!((transform.data[10] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_bone_transform_with_pivot() {
        let pivots = vec![Vec3::new(0.0, 0.0, 10.0)];
        let parents = vec![-1i16];
        let flags = vec![0u32];

        let mut computer = BoneTransformComputer::new(&pivots, &parents, &flags);

        let translations = vec![Vec3::ZERO];
        let rotations = vec![Quat::IDENTITY];
        let scales = vec![Vec3::ONE];

        computer.update(&translations, &rotations, &scales);

        let transform = computer.get_transform(0);
        // Pivot and anti-pivot cancel out for identity rotation
        let p = transform.transform_point(Vec3::ZERO);
        assert!(p.x.abs() < 0.001);
        assert!(p.y.abs() < 0.001);
        assert!(p.z.abs() < 0.001);
    }

    #[test]
    fn test_bone_transform_parent_chain() {
        // Two bones: root translates by (1,0,0), child translates by (0,1,0)
        let pivots = vec![Vec3::ZERO, Vec3::ZERO];
        let parents = vec![-1i16, 0]; // bone 1 is child of bone 0
        let flags = vec![0u32, 0u32];

        let mut computer = BoneTransformComputer::new(&pivots, &parents, &flags);

        let translations = vec![Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)];
        let rotations = vec![Quat::IDENTITY, Quat::IDENTITY];
        let scales = vec![Vec3::ONE, Vec3::ONE];

        computer.update(&translations, &rotations, &scales);

        // Root bone should translate to (1,0,0)
        let root_transform = computer.get_transform(0);
        let p = root_transform.transform_point(Vec3::ZERO);
        assert!((p.x - 1.0).abs() < 0.001);

        // Child bone should be at (1,1,0) = parent + local
        let child_transform = computer.get_transform(1);
        let p = child_transform.transform_point(Vec3::ZERO);
        assert!((p.x - 1.0).abs() < 0.001);
        assert!((p.y - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_billboard_inheritance() {
        // Root is billboard, child should inherit
        let pivots = vec![Vec3::ZERO, Vec3::ZERO];
        let parents = vec![-1i16, 0];
        let flags = vec![0x08, 0]; // root is spherical billboard

        let computer = BoneTransformComputer::new(&pivots, &parents, &flags);

        assert!(computer.is_spherical_billboard(0));
        assert!(computer.is_spherical_billboard(1)); // inherited
    }
}
