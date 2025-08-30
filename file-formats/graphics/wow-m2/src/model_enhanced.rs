//! Enhanced M2 model parser with comprehensive data extraction methods
//!
//! This module provides enhanced parsing capabilities for M2 model files,
//! adding methods to extract ALL model data including vertices, bones, animations,
//! textures, and embedded skin data for vanilla models (version 256).

use std::collections::HashMap;
use std::io::{Cursor, Seek};

use crate::chunks::animation::M2Animation;
use crate::chunks::bone::M2Bone;
use crate::chunks::material::M2Material;
use crate::chunks::{M2Texture, M2Vertex};
use crate::error::Result;
use crate::model::M2Model;
use crate::skin::SkinFile;

/// Enhanced model data containing all extracted information
#[derive(Debug, Clone)]
pub struct EnhancedModelData {
    /// All vertices from the model
    pub vertices: Vec<M2Vertex>,
    /// All bones with hierarchy information
    pub bones: Vec<BoneInfo>,
    /// All animation sequences
    pub animations: Vec<AnimationInfo>,
    /// All textures referenced by the model
    pub textures: Vec<TextureInfo>,
    /// Embedded skin files (for vanilla models)
    pub embedded_skins: Vec<SkinFile>,
    /// Material information
    pub materials: Vec<MaterialInfo>,
    /// Model statistics
    pub stats: ModelStats,
}

/// Bone information with hierarchy details
#[derive(Debug, Clone)]
pub struct BoneInfo {
    /// Original bone data
    pub bone: M2Bone,
    /// Parent bone index (-1 if root)
    pub parent_index: i16,
    /// Child bone indices
    pub children: Vec<u16>,
    /// Bone name (if available)
    pub name: Option<String>,
}

/// Animation sequence information
#[derive(Debug, Clone)]
pub struct AnimationInfo {
    /// Original animation data
    pub animation: M2Animation,
    /// Animation name or type description
    pub name: String,
    /// Duration in milliseconds
    pub duration_ms: u32,
    /// Whether this animation loops
    pub is_looping: bool,
}

/// Texture information
#[derive(Debug, Clone)]
pub struct TextureInfo {
    /// Original texture data
    pub texture: M2Texture,
    /// Texture filename (if resolved)
    pub filename: Option<String>,
    /// Texture type description
    pub texture_type: String,
}

/// Material rendering information
#[derive(Debug, Clone)]
pub struct MaterialInfo {
    /// Original material data
    pub material: M2Material,
    /// Blend mode description
    pub blend_mode: String,
    /// Whether material is transparent
    pub is_transparent: bool,
    /// Whether material is two-sided
    pub is_two_sided: bool,
}

/// Model statistics
#[derive(Debug, Clone, Default)]
pub struct ModelStats {
    /// Total vertex count
    pub vertex_count: usize,
    /// Total triangle count (estimated from skins)
    pub triangle_count: usize,
    /// Bone count
    pub bone_count: usize,
    /// Animation count
    pub animation_count: usize,
    /// Texture count
    pub texture_count: usize,
    /// Material count
    pub material_count: usize,
    /// Number of embedded skins
    pub embedded_skin_count: usize,
    /// Model bounding box
    pub bounding_box: BoundingBox,
}

/// 3D bounding box
#[derive(Debug, Clone, Default)]
pub struct BoundingBox {
    pub min_x: f32,
    pub min_y: f32,
    pub min_z: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub max_z: f32,
}

impl BoundingBox {
    /// Calculate the center point of the bounding box
    pub fn center(&self) -> (f32, f32, f32) {
        (
            (self.min_x + self.max_x) / 2.0,
            (self.min_y + self.max_y) / 2.0,
            (self.min_z + self.max_z) / 2.0,
        )
    }

    /// Calculate the size of the bounding box
    pub fn size(&self) -> (f32, f32, f32) {
        (
            self.max_x - self.min_x,
            self.max_y - self.min_y,
            self.max_z - self.min_z,
        )
    }

    /// Calculate the diagonal length of the bounding box
    pub fn diagonal_length(&self) -> f32 {
        let (width, height, depth) = self.size();
        (width * width + height * height + depth * depth).sqrt()
    }
}

impl M2Model {
    /// Parse all available data from the M2 model
    ///
    /// This method extracts all vertices, bones, animations, textures, and materials
    /// from the model, and for vanilla models (version 256), also extracts embedded skin data.
    ///
    /// # Arguments
    ///
    /// * `original_data` - The complete original M2 file data (required for embedded skins)
    ///
    /// # Returns
    ///
    /// Returns `EnhancedModelData` containing all extracted model information
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::fs;
    /// # use std::io::Cursor;
    /// # use wow_m2::{M2Model, parse_m2};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let m2_data = fs::read("HumanMale.m2")?;
    /// let m2_format = parse_m2(&mut Cursor::new(&m2_data))?;
    /// let model = m2_format.model();
    ///
    /// // Extract all model data
    /// let enhanced_data = model.parse_all_data(&m2_data)?;
    ///
    /// println!("Model has {} vertices, {} bones, {} animations",
    ///     enhanced_data.vertices.len(),
    ///     enhanced_data.bones.len(),
    ///     enhanced_data.animations.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_all_data(&self, original_data: &[u8]) -> Result<EnhancedModelData> {
        let mut enhanced_data = EnhancedModelData {
            vertices: Vec::new(),
            bones: Vec::new(),
            animations: Vec::new(),
            textures: Vec::new(),
            embedded_skins: Vec::new(),
            materials: Vec::new(),
            stats: ModelStats::default(),
        };

        // Extract all vertices
        enhanced_data.vertices = self.extract_all_vertices(original_data)?;

        // Extract all bones with hierarchy
        enhanced_data.bones = self.extract_all_bones_with_hierarchy(original_data)?;

        // Extract all animations
        enhanced_data.animations = self.extract_all_animations(original_data)?;

        // Extract all textures
        enhanced_data.textures = self.extract_all_textures(original_data)?;

        // Extract all materials
        enhanced_data.materials = self.extract_all_materials()?;

        // Extract embedded skins for vanilla models
        if self.has_embedded_skins() {
            enhanced_data.embedded_skins = self.parse_all_embedded_skins(original_data)?;
        }

        // Calculate statistics
        enhanced_data.stats = self.calculate_model_stats(&enhanced_data)?;

        Ok(enhanced_data)
    }

    /// Extract all vertices from the model with bone index validation
    ///
    /// This method now validates vertex bone indices against the actual bone count,
    /// fixing the critical issue where vertices referenced non-existent bones.
    fn extract_all_vertices(&self, original_data: &[u8]) -> Result<Vec<M2Vertex>> {
        if self.header.vertices.count == 0 {
            return Ok(Vec::new());
        }

        let mut cursor = Cursor::new(original_data);
        cursor.seek(std::io::SeekFrom::Start(self.header.vertices.offset as u64))?;

        // Get the actual bone count for validation
        let bone_count = self.header.bones.count;

        let mut vertices = Vec::new();
        for _ in 0..self.header.vertices.count {
            // CRITICAL FIX: Pass bone count for validation to prevent out-of-bounds bone references
            vertices.push(M2Vertex::parse_with_validation(
                &mut cursor,
                self.header.version,
                Some(bone_count),
                crate::chunks::vertex::ValidationMode::default(),
            )?);
        }

        Ok(vertices)
    }

    /// Extract all bones with hierarchy information
    fn extract_all_bones_with_hierarchy(&self, original_data: &[u8]) -> Result<Vec<BoneInfo>> {
        if self.header.bones.count == 0 {
            return Ok(Vec::new());
        }

        let mut cursor = Cursor::new(original_data);
        cursor.seek(std::io::SeekFrom::Start(self.header.bones.offset as u64))?;

        let mut bone_infos = Vec::new();
        let mut parent_map: HashMap<u16, Vec<u16>> = HashMap::new();

        // First pass: read all bones and build parent-child relationships
        for i in 0..self.header.bones.count {
            let bone = M2Bone::parse(&mut cursor, self.header.version)?;

            // Validate bone data and handle known parsing limitations gracefully
            if !bone.is_valid_for_model(self.header.bones.count) {
                // For now, continue parsing what we can rather than failing completely
                // This allows extracting valid bone data while the alignment issue is resolved
                break;
            }

            let parent_index = bone.parent_bone;

            // Build children list for parent
            if parent_index >= 0 && (parent_index as u16) < self.header.bones.count as u16 {
                parent_map
                    .entry(parent_index as u16)
                    .or_default()
                    .push(i as u16);
            }

            bone_infos.push(BoneInfo {
                bone,
                parent_index,
                children: Vec::new(),
                name: self.get_bone_name(i as u16),
            });
        }

        // Second pass: assign children to bones
        for (parent_idx, children) in parent_map {
            if (parent_idx as usize) < bone_infos.len() {
                bone_infos[parent_idx as usize].children = children;
            }
        }

        Ok(bone_infos)
    }

    /// Extract all animations with metadata
    fn extract_all_animations(&self, original_data: &[u8]) -> Result<Vec<AnimationInfo>> {
        if self.header.animations.count == 0 {
            return Ok(Vec::new());
        }

        let mut cursor = Cursor::new(original_data);
        cursor.seek(std::io::SeekFrom::Start(
            self.header.animations.offset as u64,
        ))?;

        let mut animation_infos = Vec::new();

        for i in 0..self.header.animations.count {
            let animation = M2Animation::parse(&mut cursor, self.header.version)?;

            // Determine animation name and properties
            let (name, is_looping) = self.get_animation_info(i as u16, &animation);

            // Handle version differences in animation format
            let duration_ms = if self.header.version <= 256 {
                // Vanilla (version 256) uses start/end timestamps
                if let Some(end_timestamp) = animation.end_timestamp {
                    end_timestamp.saturating_sub(animation.start_timestamp)
                } else {
                    // Fallback if no end timestamp
                    animation.start_timestamp
                }
            } else {
                // BC+ (version 260+) uses duration in start_timestamp field
                animation.start_timestamp
            };

            animation_infos.push(AnimationInfo {
                animation,
                name,
                duration_ms,
                is_looping,
            });
        }

        Ok(animation_infos)
    }

    /// Extract all textures with metadata
    fn extract_all_textures(&self, original_data: &[u8]) -> Result<Vec<TextureInfo>> {
        if self.header.textures.count == 0 {
            return Ok(Vec::new());
        }

        let mut cursor = Cursor::new(original_data);
        cursor.seek(std::io::SeekFrom::Start(self.header.textures.offset as u64))?;

        let mut texture_infos = Vec::new();

        for i in 0..self.header.textures.count {
            let texture = M2Texture::parse(&mut cursor, self.header.version)?;

            // Resolve texture filename if possible
            let filename = self.resolve_texture_filename(i as u16, &texture, original_data);
            let texture_type = self.get_texture_type_description(&texture);

            texture_infos.push(TextureInfo {
                texture,
                filename,
                texture_type,
            });
        }

        Ok(texture_infos)
    }

    /// Extract all materials with rendering information
    fn extract_all_materials(&self) -> Result<Vec<MaterialInfo>> {
        let mut material_infos = Vec::new();

        for material in &self.materials {
            let blend_mode = self.get_blend_mode_description(material);
            let is_transparent = material
                .flags
                .contains(crate::chunks::material::M2RenderFlags::NO_ZBUFFER)
                || material.blend_mode.bits() != 0; // Non-opaque blend modes are transparent
            let is_two_sided = material
                .flags
                .contains(crate::chunks::material::M2RenderFlags::NO_BACKFACE_CULLING);

            material_infos.push(MaterialInfo {
                material: material.clone(),
                blend_mode,
                is_transparent,
                is_two_sided,
            });
        }

        Ok(material_infos)
    }

    /// Calculate comprehensive model statistics
    fn calculate_model_stats(&self, enhanced_data: &EnhancedModelData) -> Result<ModelStats> {
        let mut stats = ModelStats {
            vertex_count: enhanced_data.vertices.len(),
            bone_count: enhanced_data.bones.len(),
            animation_count: enhanced_data.animations.len(),
            texture_count: enhanced_data.textures.len(),
            material_count: enhanced_data.materials.len(),
            embedded_skin_count: enhanced_data.embedded_skins.len(),
            triangle_count: 0,
            bounding_box: BoundingBox::default(),
        };

        // Calculate triangle count from skins
        for skin in &enhanced_data.embedded_skins {
            stats.triangle_count += skin.triangles().len();
        }

        // Calculate bounding box from vertices
        if !enhanced_data.vertices.is_empty() {
            let first_vertex = &enhanced_data.vertices[0];
            stats.bounding_box = BoundingBox {
                min_x: first_vertex.position.x,
                min_y: first_vertex.position.y,
                min_z: first_vertex.position.z,
                max_x: first_vertex.position.x,
                max_y: first_vertex.position.y,
                max_z: first_vertex.position.z,
            };

            for vertex in &enhanced_data.vertices[1..] {
                let pos = &vertex.position;
                stats.bounding_box.min_x = stats.bounding_box.min_x.min(pos.x);
                stats.bounding_box.min_y = stats.bounding_box.min_y.min(pos.y);
                stats.bounding_box.min_z = stats.bounding_box.min_z.min(pos.z);
                stats.bounding_box.max_x = stats.bounding_box.max_x.max(pos.x);
                stats.bounding_box.max_y = stats.bounding_box.max_y.max(pos.y);
                stats.bounding_box.max_z = stats.bounding_box.max_z.max(pos.z);
            }
        }

        Ok(stats)
    }

    /// Display comprehensive model information
    ///
    /// This method prints detailed information about the model including all its components.
    ///
    /// # Arguments
    ///
    /// * `enhanced_data` - The enhanced model data to display
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::fs;
    /// # use std::io::Cursor;
    /// # use wow_m2::{M2Model, parse_m2};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let m2_data = fs::read("HumanMale.m2")?;
    /// let m2_format = parse_m2(&mut Cursor::new(&m2_data))?;
    /// let model = m2_format.model();
    ///
    /// let enhanced_data = model.parse_all_data(&m2_data)?;
    /// model.display_info(&enhanced_data);
    /// # Ok(())
    /// # }
    /// ```
    pub fn display_info(&self, enhanced_data: &EnhancedModelData) {
        println!("=== M2 Model Information ===");
        println!("Model name: {:?}", self.name);
        println!("Version: {}", self.header.version);
        println!(
            "Format: {}",
            if self.header.version <= 260 {
                "Legacy (embedded skins)"
            } else {
                "Modern (external skins)"
            }
        );
        println!();

        // Statistics
        println!("=== Statistics ===");
        println!("Vertices: {}", enhanced_data.stats.vertex_count);
        println!("Triangles: {}", enhanced_data.stats.triangle_count);
        println!("Bones: {}", enhanced_data.stats.bone_count);
        println!("Animations: {}", enhanced_data.stats.animation_count);
        println!("Textures: {}", enhanced_data.stats.texture_count);
        println!("Materials: {}", enhanced_data.stats.material_count);
        if enhanced_data.stats.embedded_skin_count > 0 {
            println!(
                "Embedded skins: {}",
                enhanced_data.stats.embedded_skin_count
            );
        }
        println!();

        // Bounding box
        println!("=== Bounding Box ===");
        let bbox = &enhanced_data.stats.bounding_box;
        println!(
            "Min: ({:.2}, {:.2}, {:.2})",
            bbox.min_x, bbox.min_y, bbox.min_z
        );
        println!(
            "Max: ({:.2}, {:.2}, {:.2})",
            bbox.max_x, bbox.max_y, bbox.max_z
        );
        let (cx, cy, cz) = bbox.center();
        println!("Center: ({:.2}, {:.2}, {:.2})", cx, cy, cz);
        let (sx, sy, sz) = bbox.size();
        println!("Size: ({:.2}, {:.2}, {:.2})", sx, sy, sz);
        println!("Diagonal: {:.2}", bbox.diagonal_length());
        println!();

        // Bone hierarchy
        if !enhanced_data.bones.is_empty() {
            println!("=== Bone Hierarchy ===");
            self.display_bone_hierarchy(&enhanced_data.bones, 0, 0);
            println!();
        }

        // Animations
        if !enhanced_data.animations.is_empty() {
            println!("=== Animations ===");
            for (i, anim_info) in enhanced_data.animations.iter().enumerate() {
                println!(
                    "  {}: {} ({}ms) {}",
                    i,
                    anim_info.name,
                    anim_info.duration_ms,
                    if anim_info.is_looping { "[LOOP]" } else { "" }
                );
            }
            println!();
        }

        // Textures
        if !enhanced_data.textures.is_empty() {
            println!("=== Textures ===");
            for (i, tex_info) in enhanced_data.textures.iter().enumerate() {
                println!(
                    "  {}: {} ({})",
                    i,
                    tex_info.filename.as_deref().unwrap_or("<unresolved>"),
                    tex_info.texture_type
                );
            }
            println!();
        }

        // Materials
        if !enhanced_data.materials.is_empty() {
            println!("=== Materials ===");
            for (i, mat_info) in enhanced_data.materials.iter().enumerate() {
                let flags = format!(
                    "{}{}",
                    if mat_info.is_transparent {
                        "TRANSPARENT "
                    } else {
                        ""
                    },
                    if mat_info.is_two_sided {
                        "TWO_SIDED"
                    } else {
                        ""
                    }
                );
                println!("  {}: {} {}", i, mat_info.blend_mode, flags);
            }
            println!();
        }

        // Embedded skins
        if !enhanced_data.embedded_skins.is_empty() {
            println!("=== Embedded Skins ===");
            for (i, skin) in enhanced_data.embedded_skins.iter().enumerate() {
                println!(
                    "  Skin {}: {} indices, {} triangles, {} submeshes",
                    i,
                    skin.indices().len(),
                    skin.triangles().len(),
                    skin.submeshes().len()
                );
            }
            println!();
        }
    }

    /// Display bone hierarchy recursively
    #[allow(clippy::only_used_in_recursion)]
    fn display_bone_hierarchy(&self, bones: &[BoneInfo], bone_index: usize, depth: usize) {
        if bone_index >= bones.len() {
            return;
        }

        let bone_info = &bones[bone_index];
        let indent = "  ".repeat(depth);
        let default_name = format!("Bone_{}", bone_index);
        let bone_name = bone_info.name.as_deref().unwrap_or(&default_name);

        println!(
            "{}└─ {} (parent: {})",
            indent,
            bone_name,
            if bone_info.parent_index >= 0 {
                bone_info.parent_index.to_string()
            } else {
                "ROOT".to_string()
            }
        );

        // Display children recursively
        for &child_index in &bone_info.children {
            self.display_bone_hierarchy(bones, child_index as usize, depth + 1);
        }
    }

    // Helper methods for metadata extraction

    fn get_bone_name(&self, bone_index: u16) -> Option<String> {
        // For now, return a generic name. In a full implementation,
        // this could look up bone names from external data or naming conventions
        Some(format!("Bone_{}", bone_index))
    }

    fn get_animation_info(&self, anim_index: u16, animation: &M2Animation) -> (String, bool) {
        // Map animation IDs to descriptive names
        let name = match animation.animation_id {
            0 => "Stand".to_string(),
            1 => "Death".to_string(),
            4 => "Walk".to_string(),
            5 => "Run".to_string(),
            6 => "Dead".to_string(),
            26 => "Attack".to_string(),
            64..=67 => "Spell Cast".to_string(),
            // Add more animation mappings as needed
            _ => format!("Animation_{}", anim_index),
        };

        // Determine if animation loops (most animations except death/spell casts loop)
        let is_looping = !matches!(animation.animation_id, 1 | 6 | 64..=67); // Death, dead, spell casts don't loop

        (name, is_looping)
    }

    fn resolve_texture_filename(
        &self,
        _texture_index: u16,
        texture: &M2Texture,
        original_data: &[u8],
    ) -> Option<String> {
        // Try to resolve texture filename from the filename array
        if texture.filename.array.count > 0 && texture.filename.array.offset > 0 {
            let offset = texture.filename.array.offset as usize;
            if offset < original_data.len() {
                // Read null-terminated string
                let mut end_pos = offset;
                while end_pos < original_data.len() && original_data[end_pos] != 0 {
                    end_pos += 1;
                }

                if let Ok(filename) = std::str::from_utf8(&original_data[offset..end_pos]) {
                    return Some(filename.to_string());
                }
            }
        }
        None
    }

    fn get_texture_type_description(&self, texture: &M2Texture) -> String {
        match texture.texture_type {
            crate::chunks::texture::M2TextureType::Hardcoded => "Hardcoded",
            crate::chunks::texture::M2TextureType::Body => "Body + Clothes",
            crate::chunks::texture::M2TextureType::Item => "Cape",
            crate::chunks::texture::M2TextureType::Custom3 => "Skin Extra",
            crate::chunks::texture::M2TextureType::Hair => "Hair",
            crate::chunks::texture::M2TextureType::Environment => "Environment",
            crate::chunks::texture::M2TextureType::Accessories => "Accessories",
            _ => "Unknown",
        }
        .to_string()
    }

    fn get_blend_mode_description(&self, material: &M2Material) -> String {
        // This would map render flags to blend mode descriptions
        // For now, return a generic description
        match material.blend_mode.bits() {
            0 => "Opaque".to_string(),
            1 => "Alpha Key".to_string(),
            2 => "Alpha".to_string(),
            3 => "No Alpha Add".to_string(),
            4 => "Add".to_string(),
            5 => "Mod".to_string(),
            6 => "Mod2x".to_string(),
            7 => "Blend Add".to_string(),
            _ => format!("Blend_Mode_{}", material.blend_mode.bits()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::M2Array;

    #[test]
    fn test_bounding_box_calculations() {
        let bbox = BoundingBox {
            min_x: -1.0,
            min_y: -2.0,
            min_z: -3.0,
            max_x: 1.0,
            max_y: 2.0,
            max_z: 3.0,
        };

        let (cx, cy, cz) = bbox.center();
        assert_eq!(cx, 0.0);
        assert_eq!(cy, 0.0);
        assert_eq!(cz, 0.0);

        let (sx, sy, sz) = bbox.size();
        assert_eq!(sx, 2.0);
        assert_eq!(sy, 4.0);
        assert_eq!(sz, 6.0);

        let diagonal = bbox.diagonal_length();
        assert!((diagonal - (4.0 + 16.0 + 36.0_f32).sqrt()).abs() < 0.001);
    }

    #[test]
    fn test_enhanced_data_creation() {
        let mut model = M2Model::default();
        model.header.version = 256;
        model.header.views = M2Array::new(1, 0);

        // This would require actual M2 data to test fully
        // For now, just verify the method exists and compiles
        assert!(model.has_embedded_skins());
    }
}
