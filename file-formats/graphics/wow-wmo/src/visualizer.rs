//! WMO visualization and 3D export functionality
//!
//! This module provides utilities for exporting WMO data to common 3D formats
//! like OBJ/MTL for use in 3D modeling applications.

use crate::types::{Color, Vec3};
use crate::wmo_group_types::*;
use crate::wmo_types::*;

/// A simple structure to export mesh data in a format suitable for 3D rendering
#[derive(Debug, Clone)]
pub struct WmoMesh {
    /// Vertex positions
    pub positions: Vec<[f32; 3]>,

    /// Vertex normals
    pub normals: Vec<[f32; 3]>,

    /// Vertex texture coordinates
    pub tex_coords: Vec<[f32; 2]>,

    /// Vertex colors
    pub colors: Vec<[u8; 4]>,

    /// Indices (triangles)
    pub indices: Vec<u32>,

    /// Submeshes grouped by material
    pub submeshes: Vec<WmoSubmesh>,
}

/// A submesh of a WMO mesh, containing a subset of the triangles with the same material
#[derive(Debug, Clone)]
pub struct WmoSubmesh {
    /// Material index
    pub material_index: u16,

    /// Start index in the indices array
    pub start_index: u32,

    /// Number of indices
    pub index_count: u32,

    /// Texture filename
    pub texture_filename: Option<String>,
}

/// Helper for visualizing WMO files
pub struct WmoVisualizer;

impl Default for WmoVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

impl WmoVisualizer {
    /// Create a new WMO visualizer
    pub fn new() -> Self {
        Self
    }

    /// Convert a WMO model into a mesh suitable for 3D rendering
    pub fn create_mesh(&self, root: &WmoRoot, groups: &[WmoGroup]) -> WmoMesh {
        let mut mesh = WmoMesh {
            positions: Vec::new(),
            normals: Vec::new(),
            tex_coords: Vec::new(),
            colors: Vec::new(),
            indices: Vec::new(),
            submeshes: Vec::new(),
        };

        let mut global_index_offset = 0;

        // Process each group
        for group in groups {
            // Calculate vertex offset for this group
            let vertex_offset = mesh.positions.len() as u32;

            // Add vertices
            for vertex in &group.vertices {
                mesh.positions.push([vertex.x, vertex.y, vertex.z]);
            }

            // Add normals
            if !group.normals.is_empty() {
                for normal in &group.normals {
                    mesh.normals.push([normal.x, normal.y, normal.z]);
                }
            } else {
                // If no normals, add default up normals
                for _ in 0..group.vertices.len() {
                    mesh.normals.push([0.0, 0.0, 1.0]);
                }
            }

            // Add texture coordinates
            if !group.tex_coords.is_empty() {
                for tex_coord in &group.tex_coords {
                    mesh.tex_coords.push([tex_coord.u, tex_coord.v]);
                }
            } else {
                // If no texture coordinates, add default ones
                for _ in 0..group.vertices.len() {
                    mesh.tex_coords.push([0.0, 0.0]);
                }
            }

            // Add colors
            if let Some(colors) = &group.vertex_colors {
                for color in colors {
                    mesh.colors.push([color.r, color.g, color.b, color.a]);
                }
            } else {
                // If no colors, add default white
                for _ in 0..group.vertices.len() {
                    mesh.colors.push([255, 255, 255, 255]);
                }
            }

            // Process batches
            for batch in &group.batches {
                let material_index = batch.material_id;
                let start_index = batch.start_index;
                let index_count = batch.count as u32;

                // Get texture filename if available
                let texture_filename = if material_index < root.materials.len() as u16 {
                    let material = &root.materials[material_index as usize];
                    if material.texture1 < root.textures.len() as u32 {
                        Some(root.textures[material.texture1 as usize].clone())
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Add indices (remapped to global index space)
                let submesh_start = mesh.indices.len() as u32;

                for i in 0..index_count {
                    let idx = global_index_offset + start_index + i;
                    if idx < group.indices.len() as u32 {
                        let vertex_index = group.indices[idx as usize] as u32;
                        mesh.indices.push(vertex_offset + vertex_index);
                    }
                }

                let submesh_count = mesh.indices.len() as u32 - submesh_start;

                // Add submesh
                mesh.submeshes.push(WmoSubmesh {
                    material_index,
                    start_index: submesh_start,
                    index_count: submesh_count,
                    texture_filename,
                });
            }

            global_index_offset += group.indices.len() as u32;
        }

        mesh
    }

    /// Extract doodad placement information for visualization
    pub fn extract_doodads(&self, root: &WmoRoot) -> Vec<WmoDoodadPlacement> {
        let mut placements = Vec::new();

        for (i, doodad) in root.doodad_defs.iter().enumerate() {
            // Find which set(s) this doodad belongs to
            let mut set_names = Vec::new();

            for set in &root.doodad_sets {
                let start = set.start_doodad as usize;
                let end = start + set.n_doodads as usize;

                if i >= start && i < end {
                    set_names.push(set.name.clone());
                }
            }

            // Get name reference
            // In a real implementation, you'd look up the name from doodad_names
            let name = format!("Doodad_{}", doodad.name_offset);

            placements.push(WmoDoodadPlacement {
                index: i,
                name,
                position: doodad.position,
                orientation: doodad.orientation,
                scale: doodad.scale,
                color: doodad.color,
                set_indices: set_names,
            });
        }

        placements
    }

    /// Generate a list of triangles for each group
    pub fn generate_triangles(&self, groups: &[WmoGroup]) -> Vec<Vec<WmoTriangle>> {
        let mut result = Vec::with_capacity(groups.len());

        for group in groups {
            let mut triangles = Vec::new();

            // Process each batch
            for batch in &group.batches {
                let material_id = batch.material_id;

                // Convert indices to triangles
                for i in 0..(batch.count / 3) {
                    let idx_base = (batch.start_index + i as u32 * 3) as usize;

                    if idx_base + 2 < group.indices.len() {
                        let idx1 = group.indices[idx_base] as usize;
                        let idx2 = group.indices[idx_base + 1] as usize;
                        let idx3 = group.indices[idx_base + 2] as usize;

                        if idx1 < group.vertices.len()
                            && idx2 < group.vertices.len()
                            && idx3 < group.vertices.len()
                        {
                            triangles.push(WmoTriangle {
                                vertices: [
                                    group.vertices[idx1],
                                    group.vertices[idx2],
                                    group.vertices[idx3],
                                ],
                                normals: if !group.normals.is_empty() {
                                    Some([
                                        group.normals.get(idx1).cloned().unwrap_or_default(),
                                        group.normals.get(idx2).cloned().unwrap_or_default(),
                                        group.normals.get(idx3).cloned().unwrap_or_default(),
                                    ])
                                } else {
                                    None
                                },
                                tex_coords: if !group.tex_coords.is_empty() {
                                    Some([
                                        group.tex_coords.get(idx1).cloned().unwrap_or_default(),
                                        group.tex_coords.get(idx2).cloned().unwrap_or_default(),
                                        group.tex_coords.get(idx3).cloned().unwrap_or_default(),
                                    ])
                                } else {
                                    None
                                },
                                colors: group.vertex_colors.as_ref().map(|colors| {
                                    [
                                        colors.get(idx1).cloned().unwrap_or_default(),
                                        colors.get(idx2).cloned().unwrap_or_default(),
                                        colors.get(idx3).cloned().unwrap_or_default(),
                                    ]
                                }),
                                material_id,
                            });
                        }
                    }
                }
            }

            result.push(triangles);
        }

        result
    }

    /// Export to OBJ format (simple)
    pub fn export_to_obj(&self, root: &WmoRoot, groups: &[WmoGroup]) -> String {
        let mut obj = String::new();

        // Write header
        obj.push_str("# WMO Model exported from wow_wmo\n");
        obj.push_str(&format!("# Version: {}\n", root.version.to_raw()));
        obj.push_str(&format!("# Groups: {}\n\n", groups.len()));

        let mut global_vertex_offset = 1; // OBJ indices start at 1
        let mut global_normal_offset = 1;
        let mut global_texcoord_offset = 1;

        // Process each group
        for (group_idx, group) in groups.iter().enumerate() {
            obj.push_str(&format!("g Group_{group_idx}\n"));

            // Write vertices
            for v in &group.vertices {
                obj.push_str(&format!("v {} {} {}\n", v.x, v.y, v.z));
            }

            // Write texture coordinates
            for t in &group.tex_coords {
                obj.push_str(&format!("vt {} {}\n", t.u, t.v));
            }

            // Write normals
            for n in &group.normals {
                obj.push_str(&format!("vn {} {} {}\n", n.x, n.y, n.z));
            }

            // Process batches
            for batch in &group.batches {
                let material_id = batch.material_id;
                obj.push_str(&format!("usemtl Material_{material_id}\n"));

                // Write faces
                for i in 0..(batch.count / 3) {
                    let idx_base = (batch.start_index + i as u32 * 3) as usize;

                    if idx_base + 2 < group.indices.len() {
                        let idx1 = group.indices[idx_base] as usize + global_vertex_offset;
                        let idx2 = group.indices[idx_base + 1] as usize + global_vertex_offset;
                        let idx3 = group.indices[idx_base + 2] as usize + global_vertex_offset;

                        let tex1 = idx1 - global_vertex_offset + global_texcoord_offset;
                        let tex2 = idx2 - global_vertex_offset + global_texcoord_offset;
                        let tex3 = idx3 - global_vertex_offset + global_texcoord_offset;

                        let norm1 = idx1 - global_vertex_offset + global_normal_offset;
                        let norm2 = idx2 - global_vertex_offset + global_normal_offset;
                        let norm3 = idx3 - global_vertex_offset + global_normal_offset;

                        if !group.tex_coords.is_empty() && !group.normals.is_empty() {
                            obj.push_str(&format!(
                                "f {idx1}/{tex1}/{norm1} {idx2}/{tex2}/{norm2} {idx3}/{tex3}/{norm3}\n"
                            ));
                        } else if !group.tex_coords.is_empty() {
                            obj.push_str(&format!(
                                "f {idx1}/{tex1} {idx2}/{tex2} {idx3}/{tex3}\n"
                            ));
                        } else if !group.normals.is_empty() {
                            obj.push_str(&format!(
                                "f {idx1}//{norm1} {idx2}//{norm2} {idx3}//{norm3}\n"
                            ));
                        } else {
                            obj.push_str(&format!("f {idx1} {idx2} {idx3}\n"));
                        }
                    }
                }
            }

            // Update global offsets
            global_vertex_offset += group.vertices.len();
            global_texcoord_offset += group.tex_coords.len();
            global_normal_offset += group.normals.len();

            obj.push('\n');
        }

        // Write material library reference
        obj.push_str("mtllib materials.mtl\n");

        obj
    }

    /// Export materials to MTL format
    pub fn export_to_mtl(&self, root: &WmoRoot) -> String {
        let mut mtl = String::new();

        // Write header
        mtl.push_str("# WMO Materials exported from wow_wmo\n");

        // Write each material
        for (i, material) in root.materials.iter().enumerate() {
            mtl.push_str(&format!("newmtl Material_{i}\n"));

            // Convert material properties to MTL format
            let diffuse = &material.diffuse_color;
            let ambient = &material.sidn_color;
            let emissive = &material.emissive_color;

            mtl.push_str(&format!(
                "Ka {} {} {}\n",
                ambient.r as f32 / 255.0,
                ambient.g as f32 / 255.0,
                ambient.b as f32 / 255.0
            ));

            mtl.push_str(&format!(
                "Kd {} {} {}\n",
                diffuse.r as f32 / 255.0,
                diffuse.g as f32 / 255.0,
                diffuse.b as f32 / 255.0
            ));

            mtl.push_str(&format!(
                "Ke {} {} {}\n",
                emissive.r as f32 / 255.0,
                emissive.g as f32 / 255.0,
                emissive.b as f32 / 255.0
            ));

            // Add texture if available
            if material.texture1 < root.textures.len() as u32 {
                let texture = &root.textures[material.texture1 as usize];
                mtl.push_str(&format!("map_Kd {texture}\n"));
            }

            // Add alpha if material has transparency
            if material.flags.contains(WmoMaterialFlags::TWO_SIDED) {
                mtl.push_str("d 0.5\n");
            } else {
                mtl.push_str("d 1.0\n");
            }

            mtl.push('\n');
        }

        mtl
    }
}

/// A visualization-friendly doodad placement
#[derive(Debug, Clone)]
pub struct WmoDoodadPlacement {
    /// Doodad index
    pub index: usize,

    /// Doodad name (typically M2 model path)
    pub name: String,

    /// Position
    pub position: Vec3,

    /// Orientation (quaternion)
    pub orientation: [f32; 4],

    /// Scale
    pub scale: f32,

    /// Color
    pub color: Color,

    /// Doodad set names that include this doodad
    pub set_indices: Vec<String>,
}

/// A single triangle from a WMO group
#[derive(Debug, Clone)]
pub struct WmoTriangle {
    /// Vertex positions
    pub vertices: [Vec3; 3],

    /// Vertex normals (if available)
    pub normals: Option<[Vec3; 3]>,

    /// Texture coordinates (if available)
    pub tex_coords: Option<[TexCoord; 3]>,

    /// Vertex colors (if available)
    pub colors: Option<[Color; 3]>,

    /// Material ID
    pub material_id: u16,
}
