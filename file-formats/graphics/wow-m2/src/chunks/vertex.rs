use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Write};

use crate::common::{C2Vector, C3Vector};
use crate::error::Result;
use crate::version::M2Version;

/// Validation mode for vertex data parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationMode {
    /// Apply all fixes aggressively - forces bone weights on vertices with zero weights
    /// This is the legacy behavior that ensures all vertices are animated
    Strict,
    /// Only fix clearly corrupted data - preserves intentional zero weights for static geometry
    /// Fixes out-of-bounds bone indices but preserves valid zero-weight vertices
    Permissive,
    /// No automatic fixes - preserves all original data
    /// Use with caution as this may result in invalid rendering
    None,
}

impl Default for ValidationMode {
    /// Default to Permissive mode - balances corruption fixes with data preservation
    fn default() -> Self {
        Self::Permissive
    }
}

bitflags::bitflags! {
    /// Vertex flags as defined in the M2 format
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct M2VertexFlags: u8 {
        /// Transform using bone 0
        const TRANSFORM_BONE_0 = 0x01;
        /// Transform using bone 1
        const TRANSFORM_BONE_1 = 0x02;
        /// Transform using bone 2
        const TRANSFORM_BONE_2 = 0x04;
        /// Transform using bone 3
        const TRANSFORM_BONE_3 = 0x08;
        /// Normal compressed
        const NORMAL_COMPRESSED = 0x10;
        /// Unknown 0x20
        const UNKNOWN_0x20 = 0x20;
        /// Unknown 0x40
        const UNKNOWN_0x40 = 0x40;
        /// Unknown 0x80
        const UNKNOWN_0x80 = 0x80;
    }
}

/// Represents a vertex in an M2 model
#[derive(Debug, Clone)]
pub struct M2Vertex {
    /// Position of the vertex
    pub position: C3Vector,
    /// Bone weights (0-255)
    pub bone_weights: [u8; 4],
    /// Bone indices
    pub bone_indices: [u8; 4],
    /// Normal vector
    pub normal: C3Vector,
    /// Primary texture coordinates
    pub tex_coords: C2Vector,
    /// Secondary texture coordinates (added in Cataclysm)
    pub tex_coords2: Option<C2Vector>,
}

impl M2Vertex {
    /// Parse a vertex from a reader based on the M2 version
    /// Uses default validation mode (Permissive)
    pub fn parse<R: Read>(reader: &mut R, version: u32) -> Result<Self> {
        Self::parse_with_validation(reader, version, None, ValidationMode::default())
    }

    /// Parse a vertex from a reader with bone index validation
    ///
    /// # Arguments
    ///
    /// * `reader` - The reader to parse from
    /// * `version` - The M2 version
    /// * `bone_count` - Optional bone count for validation (if None, no validation is performed)
    /// * `validation_mode` - Controls how aggressive the validation fixes are
    ///
    /// # Returns
    ///
    /// Returns a parsed vertex with validated bone indices
    pub fn parse_with_validation<R: Read>(
        reader: &mut R,
        _version: u32,
        bone_count: Option<u32>,
        validation_mode: ValidationMode,
    ) -> Result<Self> {
        // Position
        let position = C3Vector::parse(reader)?;

        // Bone weights
        let bone_weights = [
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
        ];

        // Bone indices
        let bone_indices = [
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
        ];

        // Normal
        let normal = C3Vector::parse(reader)?;

        // Texture coordinates
        let tex_coords = C2Vector::parse(reader)?;

        // Secondary texture coordinates (present in ALL versions, contrary to previous belief)
        // WMVx reference shows unknown1/unknown2 floats exist in vanilla
        let tex_coords2 = Some(C2Vector::parse(reader)?);

        // CRITICAL FIX: Validate bone indices and weights against actual bone count
        let (validated_bone_indices, validated_bone_weights) = if let Some(max_bones) = bone_count {
            Self::validate_bone_data(bone_indices, bone_weights, max_bones, validation_mode)
        } else {
            (bone_indices, bone_weights)
        };

        Ok(Self {
            position,
            bone_weights: validated_bone_weights,
            bone_indices: validated_bone_indices,
            normal,
            tex_coords,
            tex_coords2,
        })
    }

    /// Validate and fix bone indices and weights to ensure data integrity
    ///
    /// This fixes the critical issues where vertices in vanilla/TBC models have:
    /// 1. Bone indices that exceed the actual number of bones in the model
    /// 2. Zero bone weights for all bones (making vertices unmovable)
    /// 3. Invalid weight distributions
    ///
    /// The behavior depends on the validation mode:
    /// - `Strict`: Applies all fixes aggressively, forces weights on zero-weight vertices
    /// - `Permissive`: Fixes out-of-bounds indices, preserves valid zero weights for static geometry
    /// - `None`: Preserves original data without any fixes
    ///
    /// # Arguments
    ///
    /// * `bone_indices` - The raw bone indices from the vertex data
    /// * `bone_weights` - The raw bone weights from the vertex data
    /// * `bone_count` - The actual number of bones available in the model
    /// * `validation_mode` - Controls how aggressive the validation fixes are
    ///
    /// # Returns
    ///
    /// Returns (validated_indices, validated_weights) with fixes applied based on validation mode
    fn validate_bone_data(
        mut bone_indices: [u8; 4],
        mut bone_weights: [u8; 4],
        bone_count: u32,
        validation_mode: ValidationMode,
    ) -> ([u8; 4], [u8; 4]) {
        // Skip all validation if mode is None
        if validation_mode == ValidationMode::None {
            return (bone_indices, bone_weights);
        }

        // Track if we had invalid indices before fixing them (for Permissive mode corruption detection)
        let had_invalid_indices = bone_indices.iter().any(|&idx| (idx as u32) >= bone_count);

        // Fix invalid bone indices (both Strict and Permissive modes)
        // CRITICAL BUG FIX: Don't cast bone_count to u8 - this causes wraparound for bone_count > 255!
        // Instead, cast bone_index to u32 for comparison
        for bone_index in &mut bone_indices {
            if (*bone_index as u32) >= bone_count {
                // Invalid bone index found - clamp to valid range
                // Using 0 as a safe fallback (root bone)
                *bone_index = 0;
            }
        }

        // Fix zero bone weights issue - behavior depends on validation mode
        let total_weight: u32 = bone_weights.iter().map(|&w| w as u32).sum();
        if total_weight == 0 {
            match validation_mode {
                ValidationMode::Strict => {
                    // Strict mode: Force bone weight assignment for zero-weight vertices
                    // This ensures all vertices are animated but may break static geometry
                    bone_weights[0] = 255;
                    bone_indices[0] = 0; // Ensure it's the root bone
                }
                ValidationMode::Permissive => {
                    // Permissive mode: Preserve zero weights for static geometry
                    // Only fix if this appears to be corruption (had invalid bone indices with zero weights)
                    if had_invalid_indices {
                        // This looks like corruption - fix it
                        bone_weights[0] = 255;
                        bone_indices[0] = 0;
                    }
                    // Otherwise, preserve the zero weights as they may be intentional for static geometry
                }
                ValidationMode::None => {
                    // Already handled above - no changes
                }
            }
        }

        (bone_indices, bone_weights)
    }

    /// Write a vertex to a writer based on the M2 version
    ///
    /// Vertex size is always 48 bytes for all versions:
    /// position (12) + bone_weights (4) + bone_indices (4) + normal (12) + tex_coords (8) + tex_coords2 (8)
    ///
    /// Note: Secondary texture coordinates exist in ALL M2 versions (verified against vanilla files).
    /// They may be unused/zero in pre-Cataclysm models but the storage space is still present.
    pub fn write<W: Write>(&self, writer: &mut W, _version: u32) -> Result<()> {
        // Position (12 bytes)
        self.position.write(writer)?;

        // Bone weights (4 bytes)
        for &weight in &self.bone_weights {
            writer.write_u8(weight)?;
        }

        // Bone indices (4 bytes)
        for &index in &self.bone_indices {
            writer.write_u8(index)?;
        }

        // Normal (12 bytes)
        self.normal.write(writer)?;

        // Primary texture coordinates (8 bytes)
        self.tex_coords.write(writer)?;

        // Secondary texture coordinates (8 bytes) - present in all versions
        if let Some(tex_coords2) = self.tex_coords2 {
            tex_coords2.write(writer)?;
        } else {
            // Write default values
            C2Vector { x: 0.0, y: 0.0 }.write(writer)?;
        }

        Ok(())
    }

    /// Convert this vertex to a different version
    pub fn convert(&self, _target_version: M2Version) -> Self {
        // All versions now have the same structure, so no conversion needed
        self.clone()
    }

    /// Get the effective bone count used by this vertex
    pub fn effective_bone_count(&self) -> u32 {
        let mut count = 0;

        for i in 0..4 {
            if self.bone_weights[i] > 0 {
                count += 1;
            }
        }

        count
    }

    /// Calculate the size of this vertex in bytes for a specific version
    pub fn size_in_bytes(_version: M2Version) -> usize {
        // ALL versions have the same vertex structure size (48 bytes)
        // Position (3 floats) + bone weights (4 bytes) + bone indices (4 bytes) +
        // Normal (3 floats) + texture coords (2 floats) + secondary tex coords (2 floats)
        3 * 4 + 4 + 4 + 3 * 4 + 2 * 4 + 2 * 4 // = 48 bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_vertex_parse_classic() {
        let mut data = Vec::new();

        // Position
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&2.0f32.to_le_bytes());
        data.extend_from_slice(&3.0f32.to_le_bytes());

        // Bone weights
        data.push(255);
        data.push(128);
        data.push(64);
        data.push(0);

        // Bone indices
        data.push(0);
        data.push(1);
        data.push(2);
        data.push(3);

        // Normal
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&std::f32::consts::FRAC_1_SQRT_2.to_le_bytes());

        // Texture coordinates
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());

        // Secondary texture coordinates (now required for ALL versions)
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.5f32.to_le_bytes());

        let mut cursor = Cursor::new(data);
        let vertex = M2Vertex::parse(&mut cursor, M2Version::Vanilla.to_header_version()).unwrap();

        assert_eq!(vertex.position.x, 1.0);
        assert_eq!(vertex.position.y, 2.0);
        assert_eq!(vertex.position.z, 3.0);

        assert_eq!(vertex.bone_weights, [255, 128, 64, 0]);
        assert_eq!(vertex.bone_indices, [0, 1, 2, 3]);

        assert_eq!(vertex.normal.x, 0.5);
        assert_eq!(vertex.normal.y, 0.5);
        assert!((vertex.normal.z - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.0001);

        assert_eq!(vertex.tex_coords.x, 0.0);
        assert_eq!(vertex.tex_coords.y, 1.0);

        // Secondary texture coordinates should now be present in Vanilla
        assert!(vertex.tex_coords2.is_some());
        let tex_coords2 = vertex.tex_coords2.unwrap();
        assert_eq!(tex_coords2.x, 0.5);
        assert_eq!(tex_coords2.y, 0.5);
    }

    #[test]
    fn test_vertex_parse_cataclysm() {
        let mut data = Vec::new();

        // Position
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&2.0f32.to_le_bytes());
        data.extend_from_slice(&3.0f32.to_le_bytes());

        // Bone weights
        data.push(255);
        data.push(128);
        data.push(64);
        data.push(0);

        // Bone indices
        data.push(0);
        data.push(1);
        data.push(2);
        data.push(3);

        // Normal
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&std::f32::consts::FRAC_1_SQRT_2.to_le_bytes());

        // Texture coordinates
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());

        // Secondary texture coordinates (added in Cataclysm)
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.5f32.to_le_bytes());

        let mut cursor = Cursor::new(data);
        let vertex =
            M2Vertex::parse(&mut cursor, M2Version::Cataclysm.to_header_version()).unwrap();

        assert_eq!(vertex.position.x, 1.0);
        assert_eq!(vertex.position.y, 2.0);
        assert_eq!(vertex.position.z, 3.0);

        assert_eq!(vertex.bone_weights, [255, 128, 64, 0]);
        assert_eq!(vertex.bone_indices, [0, 1, 2, 3]);

        assert_eq!(vertex.normal.x, 0.5);
        assert_eq!(vertex.normal.y, 0.5);
        assert!((vertex.normal.z - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.0001);

        assert_eq!(vertex.tex_coords.x, 0.0);
        assert_eq!(vertex.tex_coords.y, 1.0);

        assert!(vertex.tex_coords2.is_some());
        let tex_coords2 = vertex.tex_coords2.unwrap();
        assert_eq!(tex_coords2.x, 0.5);
        assert_eq!(tex_coords2.y, 0.5);
    }

    #[test]
    fn test_vertex_write() {
        let vertex = M2Vertex {
            position: C3Vector {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            bone_weights: [255, 128, 64, 0],
            bone_indices: [0, 1, 2, 3],
            normal: C3Vector {
                x: 0.5,
                y: 0.5,
                z: std::f32::consts::FRAC_1_SQRT_2,
            },
            tex_coords: C2Vector { x: 0.0, y: 1.0 },
            tex_coords2: Some(C2Vector { x: 0.5, y: 0.5 }),
        };

        // Test writing in Vanilla format
        let mut classic_data = Vec::new();
        vertex
            .write(&mut classic_data, M2Version::Vanilla.to_header_version())
            .unwrap();

        // All versions have 48-byte vertices (including secondary tex coords)
        assert_eq!(classic_data.len(), 48);

        // Test writing in Cataclysm format
        let mut cata_data = Vec::new();
        vertex
            .write(&mut cata_data, M2Version::Cataclysm.to_header_version())
            .unwrap();

        // Same size (48 bytes) for all versions
        assert_eq!(cata_data.len(), 48);
    }

    #[test]
    fn test_vertex_conversion() {
        // Create a Classic vertex
        let classic_vertex = M2Vertex {
            position: C3Vector {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            bone_weights: [255, 128, 64, 0],
            bone_indices: [0, 1, 2, 3],
            normal: C3Vector {
                x: 0.5,
                y: 0.5,
                z: std::f32::consts::FRAC_1_SQRT_2,
            },
            tex_coords: C2Vector { x: 0.0, y: 1.0 },
            tex_coords2: Some(C2Vector { x: 0.5, y: 0.5 }),
        };

        // Convert to Cataclysm (no change expected)
        let cata_vertex = classic_vertex.convert(M2Version::Cataclysm);

        // Should be identical (all versions have same structure now)
        assert!(cata_vertex.tex_coords2.is_some());
        let tex_coords2 = cata_vertex.tex_coords2.unwrap();
        assert_eq!(tex_coords2.x, 0.5);
        assert_eq!(tex_coords2.y, 0.5);

        // Convert back to Classic (no change expected)
        let classic_vertex2 = cata_vertex.convert(M2Version::Vanilla);

        // Should still have secondary texture coordinates
        assert!(classic_vertex2.tex_coords2.is_some());
    }

    #[test]
    fn test_bone_index_validation() {
        // Test the critical bug fix for out-of-bounds bone indices
        let mut data = Vec::new();

        // Position
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&2.0f32.to_le_bytes());
        data.extend_from_slice(&3.0f32.to_le_bytes());

        // Bone weights
        data.push(255);
        data.push(128);
        data.push(64);
        data.push(0);

        // PROBLEMATIC bone indices that exceed available bones
        // This simulates the vanilla/TBC issue where vertices reference bones 196, 141, etc.
        data.push(196); // Invalid - should be clamped to 0
        data.push(141); // Invalid - should be clamped to 0
        data.push(100); // Invalid (>=96) - should be clamped to 0  
        data.push(255); // Invalid - should be clamped to 0

        // Normal
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&std::f32::consts::FRAC_1_SQRT_2.to_le_bytes());

        // Texture coordinates
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());

        // Secondary texture coordinates (required for all versions)
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.5f32.to_le_bytes());

        let mut cursor = Cursor::new(data);

        // Parse with validation for a model with only 96 bones (vanilla scenario)
        let vertex = M2Vertex::parse_with_validation(
            &mut cursor,
            M2Version::Vanilla.to_header_version(),
            Some(96),
            ValidationMode::default(),
        )
        .unwrap();

        // All invalid bone indices should be clamped to 0
        assert_eq!(vertex.bone_indices, [0, 0, 0, 0]);

        // Other data should be preserved
        assert_eq!(vertex.position.x, 1.0);
        assert_eq!(vertex.bone_weights, [255, 128, 64, 0]);
    }

    #[test]
    fn test_bone_index_validation_valid_indices() {
        // Test that valid bone indices are preserved
        let mut data = Vec::new();

        // Position
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&2.0f32.to_le_bytes());
        data.extend_from_slice(&3.0f32.to_le_bytes());

        // Bone weights
        data.push(255);
        data.push(128);
        data.push(64);
        data.push(0);

        // Valid bone indices within range
        data.push(0);
        data.push(10);
        data.push(50);
        data.push(95);

        // Normal
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&std::f32::consts::FRAC_1_SQRT_2.to_le_bytes());

        // Texture coordinates
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());

        // Secondary texture coordinates (required for all versions)
        data.extend_from_slice(&0.25f32.to_le_bytes());
        data.extend_from_slice(&0.75f32.to_le_bytes());

        let mut cursor = Cursor::new(data);

        // Parse with validation for a model with 96 bones
        let vertex = M2Vertex::parse_with_validation(
            &mut cursor,
            M2Version::Vanilla.to_header_version(),
            Some(96),
            ValidationMode::default(),
        )
        .unwrap();

        // Valid bone indices should be preserved
        assert_eq!(vertex.bone_indices, [0, 10, 50, 95]);
        // Valid bone weights should be preserved
        assert_eq!(vertex.bone_weights, [255, 128, 64, 0]);
    }

    #[test]
    fn test_zero_bone_weights_fix() {
        // Test the fix for vertices with zero bone weights (17% of vertices in issue)
        let mut data = Vec::new();

        // Position
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&2.0f32.to_le_bytes());
        data.extend_from_slice(&3.0f32.to_le_bytes());

        // ALL ZERO bone weights (this is the issue)
        data.push(0);
        data.push(0);
        data.push(0);
        data.push(0);

        // Some bone indices (could be invalid too)
        data.push(50); // Invalid - will be clamped
        data.push(100); // Invalid - will be clamped
        data.push(200); // Invalid - will be clamped
        data.push(255); // Invalid - will be clamped

        // Normal
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());

        // Texture coordinates
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());

        let mut cursor = Cursor::new(data);

        // Parse with validation for a model with 34 bones (Rabbit.m2 scenario)
        let vertex = M2Vertex::parse_with_validation(
            &mut cursor,
            M2Version::Vanilla.to_header_version(),
            Some(34),
            ValidationMode::Strict, // Use Strict mode to test the old behavior
        )
        .unwrap();

        // Zero weights should be fixed - first bone should get full weight
        assert_eq!(vertex.bone_weights[0], 255);
        assert_eq!(vertex.bone_weights[1], 0);
        assert_eq!(vertex.bone_weights[2], 0);
        assert_eq!(vertex.bone_weights[3], 0);

        // First bone index should be set to 0 (root bone) for safety
        assert_eq!(vertex.bone_indices[0], 0);

        // Total weight should no longer be zero
        let total_weight: u32 = vertex.bone_weights.iter().map(|&w| w as u32).sum();
        assert_eq!(total_weight, 255);
        assert!(
            total_weight > 0,
            "Vertex should no longer have zero total weight"
        );
    }

    #[test]
    fn test_critical_bug_bone_count_wraparound() {
        // Test the critical bug where bone_count > 255 causes u8 wraparound
        // This was causing validation to fail for models with many bones
        let mut data = Vec::new();

        // Position
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&2.0f32.to_le_bytes());
        data.extend_from_slice(&3.0f32.to_le_bytes());

        // Bone weights
        data.push(255);
        data.push(0);
        data.push(0);
        data.push(0);

        // Bone indices - these should be VALID for a model with 300 bones
        data.push(100);
        data.push(200);
        data.push(250);
        data.push(0);

        // Normal
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());

        // Texture coordinates
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.5f32.to_le_bytes());

        // Secondary texture coordinates
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());

        let mut cursor = std::io::Cursor::new(data);

        // Test with 300 bones - before the fix, bone_count as u8 would wrap to 44
        // This would incorrectly mark bone indices 100, 200, 250 as invalid
        let vertex = M2Vertex::parse_with_validation(
            &mut cursor,
            M2Version::Vanilla.to_header_version(),
            Some(300), // Large bone count that causes u8 wraparound
            ValidationMode::default(),
        )
        .unwrap();

        // These bone indices should be PRESERVED, not clamped to 0
        assert_eq!(
            vertex.bone_indices,
            [100, 200, 250, 0],
            "Bone indices should be preserved for models with > 255 bones, got: {:?}",
            vertex.bone_indices
        );

        // Weights should be unchanged
        assert_eq!(vertex.bone_weights, [255, 0, 0, 0]);
    }

    #[test]
    fn test_bone_count_255_boundary() {
        // Test the boundary case at exactly 255 bones
        let mut data = Vec::new();

        // Position
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&2.0f32.to_le_bytes());
        data.extend_from_slice(&3.0f32.to_le_bytes());

        // Bone weights
        data.push(255);
        data.push(0);
        data.push(0);
        data.push(0);

        // Test edge cases: bone indices 254 (valid) and 255 (invalid)
        data.push(254); // Should be valid for 255 bones (0-254)
        data.push(255); // Should be INVALID for 255 bones
        data.push(0);
        data.push(0);

        // Normal
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());

        // Texture coordinates
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.5f32.to_le_bytes());

        // Secondary texture coordinates
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());

        let mut cursor = std::io::Cursor::new(data);

        let vertex = M2Vertex::parse_with_validation(
            &mut cursor,
            M2Version::Vanilla.to_header_version(),
            Some(255),
            ValidationMode::default(),
        )
        .unwrap();

        // Bone index 254 should be preserved, 255 should be clamped to 0
        assert_eq!(
            vertex.bone_indices,
            [254, 0, 0, 0],
            "Bone index 254 should be valid, 255 should be clamped to 0, got: {:?}",
            vertex.bone_indices
        );
    }

    #[test]
    fn test_comprehensive_validation_all_issues() {
        // Test that all three critical issues are fixed simultaneously:
        // 1. Invalid bone indices: [51, 196, 141, 62] when only 34 bones exist
        // 2. Zero bone weights: All weights are 0
        // 3. Both issues combined (realistic corruption scenario)

        let mut data = Vec::new();

        // Position
        data.extend_from_slice(&(-2.5f32).to_le_bytes());
        data.extend_from_slice(&1.8f32.to_le_bytes());
        data.extend_from_slice(&0.3f32.to_le_bytes());

        // ZERO bone weights (Issue #3: Missing bone weights)
        data.push(0);
        data.push(0);
        data.push(0);
        data.push(0);

        // INVALID bone indices from the exact issue report (Issue #1)
        data.push(51); // > 34 bones
        data.push(196); // > 34 bones
        data.push(141); // > 34 bones
        data.push(62); // > 34 bones

        // Normal
        data.extend_from_slice(&std::f32::consts::FRAC_1_SQRT_2.to_le_bytes());
        data.extend_from_slice(&std::f32::consts::FRAC_1_SQRT_2.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());

        // Texture coordinates
        data.extend_from_slice(&0.25f32.to_le_bytes());
        data.extend_from_slice(&0.75f32.to_le_bytes());
        data.extend_from_slice(&0.1f32.to_le_bytes());
        data.extend_from_slice(&0.9f32.to_le_bytes());

        let mut cursor = Cursor::new(data);

        // Parse with validation for Rabbit.m2 scenario (34 bones as reported)
        let vertex = M2Vertex::parse_with_validation(
            &mut cursor,
            M2Version::Vanilla.to_header_version(),
            Some(34),               // Exact bone count from issue report
            ValidationMode::Strict, // Use Strict mode to test comprehensive fixes
        )
        .unwrap();

        // Verify all fixes applied:

        // Fix #1: Invalid bone indices should be corrected
        assert!(
            vertex.bone_indices.iter().all(|&idx| (idx as u32) < 34),
            "All bone indices should be < 34, got: {:?}",
            vertex.bone_indices
        );

        // Fix #3: Zero bone weights should be corrected
        let total_weight: u32 = vertex.bone_weights.iter().map(|&w| w as u32).sum();
        assert!(
            total_weight > 0,
            "Total bone weight should not be zero, got: {}",
            total_weight
        );
        assert_eq!(
            vertex.bone_weights[0], 255,
            "First bone should get full weight when all were zero"
        );

        // Safety: First bone should be root (0) for the weight assignment
        assert_eq!(
            vertex.bone_indices[0], 0,
            "First bone should be root bone for safety"
        );

        // Other data should be preserved
        assert_eq!(vertex.position.x, -2.5);
        assert_eq!(vertex.position.y, 1.8);
        assert_eq!(vertex.position.z, 0.3);

        println!("✅ All critical vertex issues fixed:");
        println!(
            "   Original bone indices: [51, 196, 141, 62] -> {:?}",
            vertex.bone_indices
        );
        println!(
            "   Original bone weights: [0, 0, 0, 0] -> {:?}",
            vertex.bone_weights
        );
        println!("   Total weight: 0 -> {}", total_weight);
    }

    #[test]
    fn test_validation_mode_strict() {
        // Test ValidationMode::Strict - should fix all zero weights
        let mut data = Vec::new();

        // Position
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&2.0f32.to_le_bytes());
        data.extend_from_slice(&3.0f32.to_le_bytes());

        // Zero bone weights (static geometry)
        data.push(0);
        data.push(0);
        data.push(0);
        data.push(0);

        // Valid bone indices
        data.push(0);
        data.push(1);
        data.push(2);
        data.push(3);

        // Normal
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());

        // Texture coordinates
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());

        let mut cursor = Cursor::new(data);

        let vertex = M2Vertex::parse_with_validation(
            &mut cursor,
            M2Version::Vanilla.to_header_version(),
            Some(10),
            ValidationMode::Strict,
        )
        .unwrap();

        // Strict mode should force weight on first bone
        assert_eq!(vertex.bone_weights[0], 255);
        assert_eq!(vertex.bone_weights[1], 0);
        assert_eq!(vertex.bone_weights[2], 0);
        assert_eq!(vertex.bone_weights[3], 0);
        assert_eq!(vertex.bone_indices[0], 0); // Should be root bone
    }

    #[test]
    fn test_validation_mode_permissive() {
        // Test ValidationMode::Permissive - should preserve valid zero weights
        let mut data = Vec::new();

        // Position
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&2.0f32.to_le_bytes());
        data.extend_from_slice(&3.0f32.to_le_bytes());

        // Zero bone weights (static geometry)
        data.push(0);
        data.push(0);
        data.push(0);
        data.push(0);

        // Valid bone indices - all within range
        data.push(0);
        data.push(1);
        data.push(2);
        data.push(3);

        // Normal
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());

        // Texture coordinates
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());

        let mut cursor = Cursor::new(data);

        let vertex = M2Vertex::parse_with_validation(
            &mut cursor,
            M2Version::Vanilla.to_header_version(),
            Some(10),
            ValidationMode::Permissive,
        )
        .unwrap();

        // Permissive mode should preserve zero weights when indices are valid
        assert_eq!(vertex.bone_weights, [0, 0, 0, 0]);
        assert_eq!(vertex.bone_indices, [0, 1, 2, 3]);

        let total_weight: u32 = vertex.bone_weights.iter().map(|&w| w as u32).sum();
        assert_eq!(
            total_weight, 0,
            "Static geometry should preserve zero weights"
        );
    }

    #[test]
    fn test_validation_mode_permissive_with_corruption() {
        // Test ValidationMode::Permissive with corrupted data - should fix it
        let mut data = Vec::new();

        // Position
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&2.0f32.to_le_bytes());
        data.extend_from_slice(&3.0f32.to_le_bytes());

        // Zero bone weights (static geometry)
        data.push(0);
        data.push(0);
        data.push(0);
        data.push(0);

        // Invalid bone indices - suggests corruption
        data.push(200); // Invalid
        data.push(150); // Invalid
        data.push(100); // Invalid
        data.push(255); // Invalid

        // Normal
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());

        // Texture coordinates
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());

        let mut cursor = Cursor::new(data);

        let vertex = M2Vertex::parse_with_validation(
            &mut cursor,
            M2Version::Vanilla.to_header_version(),
            Some(10), // Only 10 bones, so all indices are invalid
            ValidationMode::Permissive,
        )
        .unwrap();

        // Permissive mode should detect this as corruption and fix it
        assert_eq!(
            vertex.bone_weights[0], 255,
            "Should fix zero weights when corruption detected"
        );
        assert_eq!(
            vertex.bone_indices[0], 0,
            "Should use root bone when fixing corruption"
        );

        let total_weight: u32 = vertex.bone_weights.iter().map(|&w| w as u32).sum();
        assert!(total_weight > 0, "Should fix corrupted zero weights");
    }

    #[test]
    fn test_validation_mode_none() {
        // Test ValidationMode::None - should preserve all original data
        let mut data = Vec::new();

        // Position
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&2.0f32.to_le_bytes());
        data.extend_from_slice(&3.0f32.to_le_bytes());

        // Zero bone weights
        data.push(0);
        data.push(0);
        data.push(0);
        data.push(0);

        // Invalid bone indices
        data.push(200);
        data.push(150);
        data.push(100);
        data.push(255);

        // Normal
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());

        // Texture coordinates
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());

        let mut cursor = Cursor::new(data);

        let vertex = M2Vertex::parse_with_validation(
            &mut cursor,
            M2Version::Vanilla.to_header_version(),
            Some(10),
            ValidationMode::None,
        )
        .unwrap();

        // None mode should preserve original data exactly as-is
        assert_eq!(
            vertex.bone_weights,
            [0, 0, 0, 0],
            "Should preserve original weights"
        );
        assert_eq!(
            vertex.bone_indices,
            [200, 150, 100, 255],
            "Should preserve original indices"
        );

        let total_weight: u32 = vertex.bone_weights.iter().map(|&w| w as u32).sum();
        assert_eq!(total_weight, 0, "Should preserve zero total weight");
    }

    #[test]
    fn test_validation_mode_comparison() {
        // Compare all three modes with the same corrupted data
        let create_test_data = || -> Vec<u8> {
            let mut data = Vec::new();

            // Position
            data.extend_from_slice(&1.0f32.to_le_bytes());
            data.extend_from_slice(&2.0f32.to_le_bytes());
            data.extend_from_slice(&3.0f32.to_le_bytes());

            // Zero bone weights
            data.push(0);
            data.push(0);
            data.push(0);
            data.push(0);

            // Mixed valid/invalid bone indices
            data.push(0); // Valid
            data.push(200); // Invalid
            data.push(5); // Valid
            data.push(150); // Invalid

            // Normal
            data.extend_from_slice(&0.0f32.to_le_bytes());
            data.extend_from_slice(&1.0f32.to_le_bytes());
            data.extend_from_slice(&0.0f32.to_le_bytes());

            // Texture coordinates
            data.extend_from_slice(&0.5f32.to_le_bytes());
            data.extend_from_slice(&0.5f32.to_le_bytes());
            data.extend_from_slice(&0.0f32.to_le_bytes());
            data.extend_from_slice(&1.0f32.to_le_bytes());

            data
        };

        let bone_count = 10;

        // Test Strict mode
        let mut cursor_strict = Cursor::new(create_test_data());
        let vertex_strict = M2Vertex::parse_with_validation(
            &mut cursor_strict,
            M2Version::Vanilla.to_header_version(),
            Some(bone_count),
            ValidationMode::Strict,
        )
        .unwrap();

        // Test Permissive mode
        let mut cursor_permissive = Cursor::new(create_test_data());
        let vertex_permissive = M2Vertex::parse_with_validation(
            &mut cursor_permissive,
            M2Version::Vanilla.to_header_version(),
            Some(bone_count),
            ValidationMode::Permissive,
        )
        .unwrap();

        // Test None mode
        let mut cursor_none = Cursor::new(create_test_data());
        let vertex_none = M2Vertex::parse_with_validation(
            &mut cursor_none,
            M2Version::Vanilla.to_header_version(),
            Some(bone_count),
            ValidationMode::None,
        )
        .unwrap();

        // Verify different behaviors
        println!(
            "Strict mode: indices={:?}, weights={:?}",
            vertex_strict.bone_indices, vertex_strict.bone_weights
        );
        println!(
            "Permissive mode: indices={:?}, weights={:?}",
            vertex_permissive.bone_indices, vertex_permissive.bone_weights
        );
        println!(
            "None mode: indices={:?}, weights={:?}",
            vertex_none.bone_indices, vertex_none.bone_weights
        );

        // Strict: should fix both indices and weights
        assert_eq!(
            vertex_strict.bone_weights[0], 255,
            "Strict should force weights"
        );
        assert_eq!(
            vertex_strict.bone_indices,
            [0, 0, 5, 0],
            "Strict should fix invalid indices, preserve valid ones"
        );

        // Permissive: should fix indices but also detect corruption and fix weights due to invalid indices
        assert_eq!(
            vertex_permissive.bone_weights[0], 255,
            "Permissive should fix weights when corruption detected"
        );
        assert_eq!(
            vertex_permissive.bone_indices,
            [0, 0, 5, 0],
            "Permissive should fix invalid indices"
        );

        // None: should preserve everything
        assert_eq!(
            vertex_none.bone_weights,
            [0, 0, 0, 0],
            "None should preserve weights"
        );
        assert_eq!(
            vertex_none.bone_indices,
            [0, 200, 5, 150],
            "None should preserve indices"
        );
    }
}
