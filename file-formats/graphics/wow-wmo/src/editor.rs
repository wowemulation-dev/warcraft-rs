//! High-level WMO editing API
//!
//! This module provides a user-friendly interface for modifying WMO files,
//! including materials, groups, transformations, and doodad management.

use crate::converter::WmoConverter;
use crate::error::{Result, WmoError};
use crate::types::{BoundingBox, Vec3};
use crate::version::WmoVersion;
use crate::wmo_group_types::{WmoGroup, WmoGroupHeader};
use crate::wmo_types::{WmoDoodadDef, WmoDoodadSet, WmoGroupInfo, WmoMaterial, WmoRoot};
use crate::writer::WmoWriter;

// Use WmoGroupFlags from wmo_group_types since that's where WmoGroupHeader uses it
use crate::wmo_group_types::WmoGroupFlags;

/// WMO editor for modifying WMO files
pub struct WmoEditor {
    /// Root WMO data
    root: WmoRoot,

    /// Group WMO data
    groups: Vec<WmoGroup>,

    /// Modified flag for root
    root_modified: bool,

    /// Modified flags for groups
    group_modified: Vec<bool>,

    /// Original version
    original_version: WmoVersion,
}

impl WmoEditor {
    /// Create a new WMO editor from a root WMO file
    pub fn new(root: WmoRoot) -> Self {
        let original_version = root.version;
        let group_count = root.groups.len();

        Self {
            root,
            groups: Vec::with_capacity(group_count),
            root_modified: false,
            group_modified: vec![false; group_count],
            original_version,
        }
    }

    /// Add a group to the editor
    pub fn add_group(&mut self, group: WmoGroup) -> Result<()> {
        // Verify group index
        let group_index = group.header.group_index as usize;

        if group_index >= self.root.groups.len() {
            return Err(WmoError::InvalidReference {
                field: "group_index".to_string(),
                value: group_index as u32,
                max: self.root.groups.len() as u32 - 1,
            });
        }

        // Ensure groups vector has enough capacity
        if self.groups.len() <= group_index {
            self.groups.resize_with(group_index + 1, || WmoGroup {
                header: WmoGroupHeader {
                    flags: WmoGroupFlags::empty(),
                    bounding_box: BoundingBox {
                        min: Vec3 {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        max: Vec3 {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },
                    },
                    name_offset: 0,
                    group_index: 0,
                },
                materials: Vec::new(),
                vertices: Vec::new(),
                normals: Vec::new(),
                tex_coords: Vec::new(),
                batches: Vec::new(),
                indices: Vec::new(),
                vertex_colors: None,
                bsp_nodes: None,
                liquid: None,
                doodad_refs: None,
            });
        }

        // Store the group
        self.groups[group_index] = group;
        self.group_modified[group_index] = true;

        Ok(())
    }

    /// Get a reference to the root WMO
    pub fn root(&self) -> &WmoRoot {
        &self.root
    }

    /// Get a mutable reference to the root WMO
    pub fn root_mut(&mut self) -> &mut WmoRoot {
        self.root_modified = true;
        &mut self.root
    }

    /// Get a reference to a specific group
    pub fn group(&self, index: usize) -> Option<&WmoGroup> {
        self.groups.get(index)
    }

    /// Get a mutable reference to a specific group
    pub fn group_mut(&mut self, index: usize) -> Option<&mut WmoGroup> {
        if index < self.group_modified.len() {
            self.group_modified[index] = true;
        }
        self.groups.get_mut(index)
    }

    /// Get the number of groups
    pub fn group_count(&self) -> usize {
        self.root.groups.len()
    }

    /// Check if a specific group has been loaded
    pub fn is_group_loaded(&self, index: usize) -> bool {
        index < self.groups.len() && self.group(index).is_some()
    }

    /// Check if a specific group has been modified
    pub fn is_group_modified(&self, index: usize) -> bool {
        index < self.group_modified.len() && self.group_modified[index]
    }

    /// Check if the root has been modified
    pub fn is_root_modified(&self) -> bool {
        self.root_modified
    }

    /// Convert to a specific version
    pub fn convert_to_version(&mut self, target_version: WmoVersion) -> Result<()> {
        // Only convert if necessary
        if self.root.version == target_version {
            return Ok(());
        }

        // Convert root
        let converter = WmoConverter::new();
        converter.convert_root(&mut self.root, target_version)?;
        self.root_modified = true;

        // Convert all loaded groups
        for (i, group) in self.groups.iter_mut().enumerate() {
            converter.convert_group(group, target_version, self.original_version)?;
            if i < self.group_modified.len() {
                self.group_modified[i] = true;
            }
        }

        Ok(())
    }

    /// Get the original version of the WMO
    pub fn original_version(&self) -> WmoVersion {
        self.original_version
    }

    /// Get the current version of the WMO
    pub fn current_version(&self) -> WmoVersion {
        self.root.version
    }

    /// Save the root WMO to a writer
    pub fn save_root<W: std::io::Write + std::io::Seek>(&self, writer: &mut W) -> Result<()> {
        // Write the root WMO
        let writer_obj = WmoWriter::new();
        writer_obj.write_root(writer, &self.root, self.root.version)?;

        Ok(())
    }

    /// Save a specific group to a writer
    pub fn save_group<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        index: usize,
    ) -> Result<()> {
        // Check if group exists
        let group = self
            .group(index)
            .ok_or_else(|| WmoError::InvalidReference {
                field: "group_index".to_string(),
                value: index as u32,
                max: self.groups.len() as u32 - 1,
            })?;

        // Write the group
        let writer_obj = WmoWriter::new();
        writer_obj.write_group(writer, group, self.root.version)?;

        Ok(())
    }

    // Material editing methods

    /// Add a new material
    pub fn add_material(&mut self, material: WmoMaterial) -> usize {
        self.root_modified = true;
        self.root.materials.push(material);
        self.root.header.n_materials += 1;
        self.root.materials.len() - 1
    }

    /// Remove a material
    pub fn remove_material(&mut self, index: usize) -> Result<WmoMaterial> {
        if index >= self.root.materials.len() {
            return Err(WmoError::InvalidReference {
                field: "material_index".to_string(),
                value: index as u32,
                max: self.root.materials.len() as u32 - 1,
            });
        }

        self.root_modified = true;
        let material = self.root.materials.remove(index);
        self.root.header.n_materials -= 1;

        // Now we need to update all references to this material in groups
        for (i, group) in self.groups.iter_mut().enumerate() {
            let mut modified = false;

            // Update materials list
            for mat_idx in &mut group.materials {
                match (*mat_idx as usize).cmp(&index) {
                    std::cmp::Ordering::Equal => {
                        // This material has been removed, use a default one instead
                        *mat_idx = 0;
                        modified = true;
                    }
                    std::cmp::Ordering::Greater => {
                        // This material has been shifted down by one
                        *mat_idx -= 1;
                        modified = true;
                    }
                    std::cmp::Ordering::Less => {}
                }
            }

            // Update batches
            for batch in &mut group.batches {
                match (batch.material_id as usize).cmp(&index) {
                    std::cmp::Ordering::Equal => {
                        // This material has been removed, use a default one instead
                        batch.material_id = 0;
                        modified = true;
                    }
                    std::cmp::Ordering::Greater => {
                        // This material has been shifted down by one
                        batch.material_id -= 1;
                        modified = true;
                    }
                    std::cmp::Ordering::Less => {}
                }
            }

            if modified && i < self.group_modified.len() {
                self.group_modified[i] = true;
            }
        }

        Ok(material)
    }

    /// Get a reference to a material
    pub fn material(&self, index: usize) -> Option<&WmoMaterial> {
        self.root.materials.get(index)
    }

    /// Get a mutable reference to a material
    pub fn material_mut(&mut self, index: usize) -> Option<&mut WmoMaterial> {
        self.root_modified = true;
        self.root.materials.get_mut(index)
    }

    // Texture editing methods

    /// Add a new texture
    pub fn add_texture(&mut self, texture: String) -> usize {
        self.root_modified = true;
        self.root.textures.push(texture);
        self.root.textures.len() - 1
    }

    /// Remove a texture
    pub fn remove_texture(&mut self, index: usize) -> Result<String> {
        if index >= self.root.textures.len() {
            return Err(WmoError::InvalidReference {
                field: "texture_index".to_string(),
                value: index as u32,
                max: self.root.textures.len() as u32 - 1,
            });
        }

        self.root_modified = true;
        let texture = self.root.textures.remove(index);

        // Now we need to update all references to this texture in materials
        for material in &mut self.root.materials {
            match (material.texture1 as usize).cmp(&index) {
                std::cmp::Ordering::Equal => {
                    // This texture has been removed, use a default one instead
                    material.texture1 = 0;
                }
                std::cmp::Ordering::Greater => {
                    // This texture has been shifted down by one
                    material.texture1 -= 1;
                }
                std::cmp::Ordering::Less => {}
            }

            match (material.texture2 as usize).cmp(&index) {
                std::cmp::Ordering::Equal => {
                    // This texture has been removed, use a default one instead
                    material.texture2 = 0;
                }
                std::cmp::Ordering::Greater => {
                    // This texture has been shifted down by one
                    material.texture2 -= 1;
                }
                std::cmp::Ordering::Less => {}
            }
        }

        Ok(texture)
    }

    /// Get a reference to a texture
    pub fn texture(&self, index: usize) -> Option<&String> {
        self.root.textures.get(index)
    }

    /// Get a mutable reference to a texture
    pub fn texture_mut(&mut self, index: usize) -> Option<&mut String> {
        self.root_modified = true;
        self.root.textures.get_mut(index)
    }

    // Group editing methods

    /// Create a new group
    pub fn create_group(&mut self, name: String) -> usize {
        self.root_modified = true;

        // Create a new group info entry
        let group_info = WmoGroupInfo {
            flags: WmoGroupFlags::empty(),
            bounding_box: BoundingBox {
                min: Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                max: Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            name,
        };

        // Add to root
        self.root.groups.push(group_info);
        self.root.header.n_groups += 1;

        // Create placeholder group data
        let group_index = self.root.groups.len() - 1;
        let header = WmoGroupHeader {
            flags: WmoGroupFlags::empty(),
            bounding_box: BoundingBox {
                min: Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                max: Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            name_offset: 0, // Will be calculated when saving
            group_index: group_index as u32,
        };

        let group = WmoGroup {
            header,
            materials: Vec::new(),
            vertices: Vec::new(),
            normals: Vec::new(),
            tex_coords: Vec::new(),
            batches: Vec::new(),
            indices: Vec::new(),
            vertex_colors: None,
            bsp_nodes: None,
            liquid: None,
            doodad_refs: None,
        };

        // Add to groups
        if self.groups.len() <= group_index {
            self.groups.resize_with(group_index + 1, || WmoGroup {
                header: WmoGroupHeader {
                    flags: WmoGroupFlags::empty(),
                    bounding_box: BoundingBox {
                        min: Vec3 {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        max: Vec3 {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },
                    },
                    name_offset: 0,
                    group_index: 0,
                },
                materials: Vec::new(),
                vertices: Vec::new(),
                normals: Vec::new(),
                tex_coords: Vec::new(),
                batches: Vec::new(),
                indices: Vec::new(),
                vertex_colors: None,
                bsp_nodes: None,
                liquid: None,
                doodad_refs: None,
            });
        }

        self.groups.push(group);
        self.group_modified.push(true);

        group_index
    }

    /// Remove a group
    pub fn remove_group(&mut self, index: usize) -> Result<WmoGroupInfo> {
        if index >= self.root.groups.len() {
            return Err(WmoError::InvalidReference {
                field: "group_index".to_string(),
                value: index as u32,
                max: self.root.groups.len() as u32 - 1,
            });
        }

        self.root_modified = true;
        let group_info = self.root.groups.remove(index);
        self.root.header.n_groups -= 1;

        // Update group indices
        for (i, _group) in self.root.groups.iter_mut().enumerate() {
            if i >= index {
                // This group has been shifted down by one
                let group_idx = i as u32;

                if let Some(loaded_group) = self.groups.get_mut(i) {
                    loaded_group.header.group_index = group_idx;
                    if i < self.group_modified.len() {
                        self.group_modified[i] = true;
                    }
                }
            }
        }

        // Remove group data if loaded
        if index < self.groups.len() {
            self.groups.remove(index);
        }

        if index < self.group_modified.len() {
            self.group_modified.remove(index);
        }

        // Update portal references
        for portal_ref in &mut self.root.portal_references {
            match (portal_ref.group_index as usize).cmp(&index) {
                std::cmp::Ordering::Equal => {
                    // This portal reference now points to a non-existent group
                    // Setting to 0 is probably the safest option
                    portal_ref.group_index = 0;
                }
                std::cmp::Ordering::Greater => {
                    // This group has been shifted down by one
                    portal_ref.group_index -= 1;
                }
                std::cmp::Ordering::Less => {}
            }
        }

        Ok(group_info)
    }

    // Vertex manipulation methods

    /// Add a vertex to a group
    pub fn add_vertex(&mut self, group_index: usize, vertex: Vec3) -> Result<usize> {
        // Validate group index
        if group_index >= self.groups.len() {
            return Err(WmoError::InvalidReference {
                field: "group_index".to_string(),
                value: group_index as u32,
                max: self.groups.len() as u32 - 1,
            });
        }

        // Work with the group
        let vertex_index = {
            let group = &mut self.groups[group_index];

            // Add vertex
            group.vertices.push(vertex);

            // Update bounding box
            let min = &mut group.header.bounding_box.min;
            let max = &mut group.header.bounding_box.max;

            min.x = min.x.min(vertex.x);
            min.y = min.y.min(vertex.y);
            min.z = min.z.min(vertex.z);

            max.x = max.x.max(vertex.x);
            max.y = max.y.max(vertex.y);
            max.z = max.z.max(vertex.z);

            group.vertices.len() - 1
        };

        // Also update root group info
        if let Some(group_info) = self.root.groups.get_mut(group_index) {
            let info_min = &mut group_info.bounding_box.min;
            let info_max = &mut group_info.bounding_box.max;

            info_min.x = info_min.x.min(vertex.x);
            info_min.y = info_min.y.min(vertex.y);
            info_min.z = info_min.z.min(vertex.z);

            info_max.x = info_max.x.max(vertex.x);
            info_max.y = info_max.y.max(vertex.y);
            info_max.z = info_max.z.max(vertex.z);

            self.root_modified = true;
        }

        Ok(vertex_index)
    }

    /// Remove a vertex from a group
    pub fn remove_vertex(&mut self, group_index: usize, vertex_index: usize) -> Result<Vec3> {
        // Validate group index
        if group_index >= self.groups.len() {
            return Err(WmoError::InvalidReference {
                field: "group_index".to_string(),
                value: group_index as u32,
                max: self.groups.len() as u32 - 1,
            });
        }

        let group = &mut self.groups[group_index];

        if vertex_index >= group.vertices.len() {
            return Err(WmoError::InvalidReference {
                field: "vertex_index".to_string(),
                value: vertex_index as u32,
                max: group.vertices.len() as u32 - 1,
            });
        }

        // Remove vertex
        let vertex = group.vertices.remove(vertex_index);

        // Remove corresponding normal if present
        if vertex_index < group.normals.len() {
            group.normals.remove(vertex_index);
        }

        // Remove corresponding texture coordinate if present
        if vertex_index < group.tex_coords.len() {
            group.tex_coords.remove(vertex_index);
        }

        // Remove corresponding vertex color if present
        if let Some(colors) = &mut group.vertex_colors
            && vertex_index < colors.len()
        {
            colors.remove(vertex_index);
        }

        // Update indices
        for idx in &mut group.indices {
            match (*idx as usize).cmp(&vertex_index) {
                std::cmp::Ordering::Equal => {
                    // This index now points to a non-existent vertex
                    // Setting to 0 is probably the safest option
                    *idx = 0;
                }
                std::cmp::Ordering::Greater => {
                    // This index has been shifted down by one
                    *idx -= 1;
                }
                std::cmp::Ordering::Less => {}
            }
        }

        // Recalculate bounding box
        self.recalculate_group_bounding_box(group_index)?;

        Ok(vertex)
    }

    /// Recalculate the bounding box for a group
    pub fn recalculate_group_bounding_box(&mut self, group_index: usize) -> Result<()> {
        // Validate group index
        if group_index >= self.groups.len() {
            return Err(WmoError::InvalidReference {
                field: "group_index".to_string(),
                value: group_index as u32,
                max: self.groups.len() as u32 - 1,
            });
        }

        // Calculate the new bounding box
        let new_bounding_box = {
            let group = &mut self.groups[group_index];

            if group.vertices.is_empty() {
                // No vertices, use a default bounding box
                BoundingBox {
                    min: Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    max: Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                }
            } else {
                // Calculate from vertices
                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;
                let mut min_z = f32::MAX;
                let mut max_x = f32::MIN;
                let mut max_y = f32::MIN;
                let mut max_z = f32::MIN;

                for vertex in &group.vertices {
                    min_x = min_x.min(vertex.x);
                    min_y = min_y.min(vertex.y);
                    min_z = min_z.min(vertex.z);

                    max_x = max_x.max(vertex.x);
                    max_y = max_y.max(vertex.y);
                    max_z = max_z.max(vertex.z);
                }

                BoundingBox {
                    min: Vec3 {
                        x: min_x,
                        y: min_y,
                        z: min_z,
                    },
                    max: Vec3 {
                        x: max_x,
                        y: max_y,
                        z: max_z,
                    },
                }
            }
        };

        // Update the group's bounding box
        self.groups[group_index].header.bounding_box = new_bounding_box;

        // Also update root group info
        if let Some(group_info) = self.root.groups.get_mut(group_index) {
            group_info.bounding_box = new_bounding_box;
            self.root_modified = true;
        }

        Ok(())
    }

    /// Recalculate the global bounding box
    pub fn recalculate_global_bounding_box(&mut self) -> Result<()> {
        if self.root.groups.is_empty() {
            // No groups, use a default bounding box
            self.root.bounding_box = BoundingBox {
                min: Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                max: Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            };
        } else {
            // Calculate from group bounding boxes
            let mut min_x = f32::MAX;
            let mut min_y = f32::MAX;
            let mut min_z = f32::MAX;
            let mut max_x = f32::MIN;
            let mut max_y = f32::MIN;
            let mut max_z = f32::MIN;

            for group_info in &self.root.groups {
                min_x = min_x.min(group_info.bounding_box.min.x);
                min_y = min_y.min(group_info.bounding_box.min.y);
                min_z = min_z.min(group_info.bounding_box.min.z);

                max_x = max_x.max(group_info.bounding_box.max.x);
                max_y = max_y.max(group_info.bounding_box.max.y);
                max_z = max_z.max(group_info.bounding_box.max.z);
            }

            self.root.bounding_box = BoundingBox {
                min: Vec3 {
                    x: min_x,
                    y: min_y,
                    z: min_z,
                },
                max: Vec3 {
                    x: max_x,
                    y: max_y,
                    z: max_z,
                },
            };
        }

        self.root_modified = true;
        Ok(())
    }

    // Doodad manipulation methods

    /// Add a doodad definition
    pub fn add_doodad(&mut self, doodad: WmoDoodadDef) -> usize {
        self.root_modified = true;
        self.root.doodad_defs.push(doodad);
        self.root.header.n_doodad_defs += 1;
        self.root.header.n_doodad_names += 1; // Assuming name is also added

        self.root.doodad_defs.len() - 1
    }

    /// Remove a doodad definition
    pub fn remove_doodad(&mut self, index: usize) -> Result<WmoDoodadDef> {
        if index >= self.root.doodad_defs.len() {
            return Err(WmoError::InvalidReference {
                field: "doodad_index".to_string(),
                value: index as u32,
                max: self.root.doodad_defs.len() as u32 - 1,
            });
        }

        self.root_modified = true;
        let doodad = self.root.doodad_defs.remove(index);
        self.root.header.n_doodad_defs -= 1;

        // Update doodad references in sets
        for set in &mut self.root.doodad_sets {
            if set.start_doodad as usize <= index
                && index < (set.start_doodad + set.n_doodads) as usize
            {
                // This doodad was part of the set
                set.n_doodads -= 1;
            }

            if set.start_doodad as usize > index {
                // Doodads after the removed one have shifted down
                set.start_doodad -= 1;
            }
        }

        // Update doodad references in groups
        for (i, group) in self.groups.iter_mut().enumerate() {
            if let Some(refs) = &mut group.doodad_refs {
                let mut modified = false;

                for doodad_ref in refs.iter_mut() {
                    match (*doodad_ref as usize).cmp(&index) {
                        std::cmp::Ordering::Equal => {
                            // This reference now points to a non-existent doodad
                            // Remove this reference
                            *doodad_ref = 0;
                            modified = true;
                        }
                        std::cmp::Ordering::Greater => {
                            // This reference has been shifted down by one
                            *doodad_ref -= 1;
                            modified = true;
                        }
                        std::cmp::Ordering::Less => {
                            // No change needed
                        }
                    }
                }

                if modified && i < self.group_modified.len() {
                    self.group_modified[i] = true;
                }
            }
        }

        Ok(doodad)
    }

    /// Doodad set manipulation methods
    /// Add a doodad set
    pub fn add_doodad_set(&mut self, set: WmoDoodadSet) -> usize {
        self.root_modified = true;
        self.root.doodad_sets.push(set);
        self.root.header.n_doodad_sets += 1;

        self.root.doodad_sets.len() - 1
    }

    /// Remove a doodad set
    pub fn remove_doodad_set(&mut self, index: usize) -> Result<WmoDoodadSet> {
        if index >= self.root.doodad_sets.len() {
            return Err(WmoError::InvalidReference {
                field: "doodad_set_index".to_string(),
                value: index as u32,
                max: self.root.doodad_sets.len() as u32 - 1,
            });
        }

        self.root_modified = true;
        let set = self.root.doodad_sets.remove(index);
        self.root.header.n_doodad_sets -= 1;

        Ok(set)
    }
}
