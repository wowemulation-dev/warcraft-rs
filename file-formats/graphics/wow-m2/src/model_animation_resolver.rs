/// Animation data resolver for M2 models
/// This module provides functionality to extract actual animation keyframe data
/// from M2 bone tracks, including bind pose transformations.
use crate::{
    M2Model, Result,
    chunks::{
        bone::M2Bone,
        m2_track_resolver::{M2TrackQuatExt, M2TrackVec3Ext},
    },
    common::C3Vector,
};
use std::io::{Cursor, Read, Seek};

/// Resolved bone animation data with actual keyframe values
#[derive(Debug, Clone)]
pub struct ResolvedBoneAnimation {
    /// Bone ID
    pub bone_id: i32,
    /// Parent bone ID  
    pub parent_bone: i16,
    /// Bone pivot point
    pub pivot: C3Vector,
    /// Translation keyframes (timestamps, values)
    pub translation: Option<(Vec<u32>, Vec<C3Vector>)>,
    /// Rotation keyframes (timestamps, quaternions as compressed format)
    pub rotation: Option<(Vec<u32>, Vec<[i16; 4]>)>,
    /// Scale keyframes (timestamps, values)
    pub scale: Option<(Vec<u32>, Vec<C3Vector>)>,
    /// Bind pose translation (first keyframe or default)
    pub bind_pose_translation: C3Vector,
    /// Bind pose rotation (first keyframe or identity)
    pub bind_pose_rotation: [f32; 4],
    /// Bind pose scale (first keyframe or 1,1,1)
    pub bind_pose_scale: C3Vector,
}

impl ResolvedBoneAnimation {
    /// Extract bind pose from the first keyframe or use defaults
    pub fn from_bone<R: Read + Seek>(bone: &M2Bone, reader: &mut R) -> Result<Self> {
        // Try to resolve translation track
        let (translation, bind_pose_translation) = if bone.translation.has_data() {
            let (timestamps, values, _ranges) = bone.translation.resolve_data(reader)?;
            let bind_trans = if !values.is_empty() {
                values[0]
            } else {
                C3Vector {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }
            };
            (Some((timestamps, values)), bind_trans)
        } else {
            (
                None,
                C3Vector {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            )
        };

        // Try to resolve rotation track
        let (rotation, bind_pose_rotation) = if bone.rotation.has_data() {
            let (timestamps, values, _ranges) = bone.rotation.resolve_data(reader)?;

            // Convert M2CompQuat to simple i16 arrays for storage
            let quat_values: Vec<[i16; 4]> = values.iter().map(|q| [q.x, q.y, q.z, q.w]).collect();

            let bind_rot = if !values.is_empty() {
                // Convert compressed quaternion to float quaternion for bind pose
                let q = &values[0];
                let quat = [
                    q.x as f32 / 32767.0,
                    q.y as f32 / 32767.0,
                    q.z as f32 / 32767.0,
                    q.w as f32 / 32767.0,
                ];

                // Check if quaternion is all zeros (invalid) and use identity instead
                if quat == [0.0, 0.0, 0.0, 0.0] {
                    [0.0, 0.0, 0.0, 1.0] // Identity quaternion
                } else {
                    quat
                }
            } else {
                [0.0, 0.0, 0.0, 1.0] // Identity quaternion
            };

            (Some((timestamps, quat_values)), bind_rot)
        } else {
            (None, [0.0, 0.0, 0.0, 1.0])
        };

        // Try to resolve scale track
        let (scale, bind_pose_scale) = if bone.scale.has_data() {
            let (timestamps, values, _ranges) = bone.scale.resolve_data(reader)?;
            let bind_scale = if !values.is_empty() {
                values[0]
            } else {
                C3Vector {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                }
            };
            (Some((timestamps, values)), bind_scale)
        } else {
            (
                None,
                C3Vector {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                },
            )
        };

        Ok(ResolvedBoneAnimation {
            bone_id: bone.bone_id,
            parent_bone: bone.parent_bone,
            pivot: bone.pivot,
            translation,
            rotation,
            scale,
            bind_pose_translation,
            bind_pose_rotation,
            bind_pose_scale,
        })
    }

    /// Check if this bone has any animation data
    pub fn has_animation(&self) -> bool {
        self.translation.is_some() || self.rotation.is_some() || self.scale.is_some()
    }

    /// Get the number of translation keyframes
    pub fn translation_keyframe_count(&self) -> usize {
        self.translation.as_ref().map(|(t, _)| t.len()).unwrap_or(0)
    }

    /// Get the number of rotation keyframes
    pub fn rotation_keyframe_count(&self) -> usize {
        self.rotation.as_ref().map(|(t, _)| t.len()).unwrap_or(0)
    }

    /// Get the number of scale keyframes
    pub fn scale_keyframe_count(&self) -> usize {
        self.scale.as_ref().map(|(t, _)| t.len()).unwrap_or(0)
    }
}

/// Extension trait for M2Model to resolve animation data
pub trait M2ModelAnimationExt {
    /// Resolve all bone animation data from the model
    fn resolve_bone_animations(&self, data: &[u8]) -> Result<Vec<ResolvedBoneAnimation>>;

    /// Get bind pose for all bones (no animation, just rest position)
    fn get_bind_pose(&self, data: &[u8]) -> Result<Vec<ResolvedBoneAnimation>>;
}

impl M2ModelAnimationExt for M2Model {
    fn resolve_bone_animations(&self, data: &[u8]) -> Result<Vec<ResolvedBoneAnimation>> {
        let mut cursor = Cursor::new(data);
        let mut resolved = Vec::with_capacity(self.bones.len());

        for bone in &self.bones {
            resolved.push(ResolvedBoneAnimation::from_bone(bone, &mut cursor)?);
        }

        Ok(resolved)
    }

    fn get_bind_pose(&self, data: &[u8]) -> Result<Vec<ResolvedBoneAnimation>> {
        // Same as resolve_bone_animations but we already extract bind pose
        self.resolve_bone_animations(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunks::bone::M2BoneFlags;
    use crate::chunks::m2_track::{M2TrackQuat, M2TrackVec3};

    #[test]
    fn test_bind_pose_defaults() {
        let bone = M2Bone {
            bone_id: 0,
            flags: M2BoneFlags::TRANSFORMED,
            parent_bone: -1,
            submesh_id: 0,
            unknown: [0, 0],
            bone_name_crc: None,
            translation: M2TrackVec3 {
                base: crate::chunks::m2_track::M2TrackBase {
                    interpolation_type: crate::chunks::animation::M2InterpolationType::None,
                    global_sequence: 0xFFFF,
                },
                ranges: None,
                timestamps: crate::common::M2Array::new(0, 0),
                values: crate::common::M2Array::new(0, 0),
            },
            rotation: M2TrackQuat {
                base: crate::chunks::m2_track::M2TrackBase {
                    interpolation_type: crate::chunks::animation::M2InterpolationType::None,
                    global_sequence: 0xFFFF,
                },
                ranges: None,
                timestamps: crate::common::M2Array::new(0, 0),
                values: crate::common::M2Array::new(0, 0),
            },
            scale: M2TrackVec3 {
                base: crate::chunks::m2_track::M2TrackBase {
                    interpolation_type: crate::chunks::animation::M2InterpolationType::None,
                    global_sequence: 0xFFFF,
                },
                ranges: None,
                timestamps: crate::common::M2Array::new(0, 0),
                values: crate::common::M2Array::new(0, 0),
            },
            pivot: C3Vector {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
        };

        let data = vec![0u8; 1000];
        let mut cursor = Cursor::new(&data);

        let resolved = ResolvedBoneAnimation::from_bone(&bone, &mut cursor).unwrap();

        // Check defaults
        assert_eq!(
            resolved.bind_pose_translation,
            C3Vector {
                x: 0.0,
                y: 0.0,
                z: 0.0
            }
        );
        assert_eq!(resolved.bind_pose_rotation, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(
            resolved.bind_pose_scale,
            C3Vector {
                x: 1.0,
                y: 1.0,
                z: 1.0
            }
        );
        assert_eq!(
            resolved.pivot,
            C3Vector {
                x: 1.0,
                y: 2.0,
                z: 3.0
            }
        );
        assert!(!resolved.has_animation());
    }
}
