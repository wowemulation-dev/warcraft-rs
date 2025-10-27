// adt_builder.rs - Create new ADT files from scratch

use crate::Adt;
use crate::chunk::*;
use crate::error::Result;
use crate::mcnk_subchunks::*;
use crate::mh2o::Mh2oChunk as AdvancedMh2oChunk;
use crate::mh2o::{
    Mh2oEntry, Mh2oHeader, Mh2oInstance, Mh2oRenderMask, WaterLevelData, WaterVertex,
    WaterVertexData,
};
use crate::version::AdtVersion;
// use std::collections::HashMap;

/// Parameters for WMO placement
#[allow(dead_code)]
pub struct WmoPlacementParams {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    pub flags: u16,
    pub doodad_set: u16,
    pub name_set: u16,
}

/// Builder for creating new ADT files
pub struct AdtBuilder {
    /// The ADT version to create
    version: AdtVersion,
    /// Map of textures to use
    textures: Vec<String>,
    /// Map of models to use
    models: Vec<String>,
    /// Map of WMO files to use
    wmos: Vec<String>,
    /// Heightmap data (16x16 chunks, each 9x9 grid = 145 points)
    heights: Vec<Vec<f32>>,
    /// Texture layer data for each chunk
    layers: Vec<Vec<ChunkTextureInfo>>,
    /// Doodad placements
    doodads: Vec<DoodadPlacement>,
    /// WMO placements
    wmo_placements: Vec<ModelPlacement>,
    /// Water data
    water_chunks: Vec<Option<WaterInfo>>,
    /// Flight boundaries (TBC+)
    flight_bounds: Option<([i16; 9], [i16; 9])>,
    /// Texture effects (Cataclysm+)
    texture_effects: Vec<u32>,
}

/// Texture information for a chunk
#[derive(Debug, Clone)]
pub struct ChunkTextureInfo {
    /// Index in the textures list
    pub texture_id: u32,
    /// Texture flags
    pub flags: u32,
    /// Alpha map data (if not base layer)
    pub alpha_map: Option<Vec<u8>>,
    /// Effect ID
    pub effect_id: u32,
}

/// Water information for a chunk
#[derive(Debug, Clone)]
pub struct WaterInfo {
    /// Liquid type
    pub liquid_type: u16,
    /// Minimum height
    pub min_height: f32,
    /// Maximum height
    pub max_height: f32,
    /// Water vertices (optional)
    pub vertices: Option<Vec<(f32, [u8; 2])>>,
    /// X resolution
    pub x_res: u8,
    /// Y resolution
    pub y_res: u8,
}

impl AdtBuilder {
    /// Create a new ADT builder for the specified version
    pub fn new(version: AdtVersion) -> Self {
        // Initialize with empty data
        let mut heights = Vec::with_capacity(256);
        let mut layers = Vec::with_capacity(256);
        let mut water_chunks = Vec::with_capacity(256);

        // Initialize 256 empty chunks
        for _ in 0..256 {
            // Default height map (9x9 grid = 81 points, all zero)
            heights.push(vec![0.0; 145]);

            // Default layer (just base layer)
            layers.push(vec![ChunkTextureInfo {
                texture_id: 0,
                flags: 0,
                alpha_map: None,
                effect_id: 0,
            }]);

            // No water by default
            water_chunks.push(None);
        }

        Self {
            version,
            textures: Vec::new(),
            models: Vec::new(),
            wmos: Vec::new(),
            heights,
            layers,
            doodads: Vec::new(),
            wmo_placements: Vec::new(),
            water_chunks,
            flight_bounds: None,
            texture_effects: Vec::new(),
        }
    }

    /// Add a texture to the ADT
    pub fn add_texture(&mut self, texture_path: &str) -> u32 {
        // Check if texture already exists
        for (i, existing) in self.textures.iter().enumerate() {
            if existing == texture_path {
                return i as u32;
            }
        }

        // Add new texture
        let id = self.textures.len() as u32;
        self.textures.push(texture_path.to_string());
        id
    }

    /// Add a model to the ADT
    pub fn add_model(&mut self, model_path: &str) -> u32 {
        // Check if model already exists
        for (i, existing) in self.models.iter().enumerate() {
            if existing == model_path {
                return i as u32;
            }
        }

        // Add new model
        let id = self.models.len() as u32;
        self.models.push(model_path.to_string());
        id
    }

    /// Add a WMO to the ADT
    pub fn add_wmo(&mut self, wmo_path: &str) -> u32 {
        // Check if WMO already exists
        for (i, existing) in self.wmos.iter().enumerate() {
            if existing == wmo_path {
                return i as u32;
            }
        }

        // Add new WMO
        let id = self.wmos.len() as u32;
        self.wmos.push(wmo_path.to_string());
        id
    }

    /// Set heights for a specific chunk
    pub fn set_chunk_heights(
        &mut self,
        chunk_x: usize,
        chunk_y: usize,
        heights: &[f32],
    ) -> Result<()> {
        let chunk_idx = chunk_y * 16 + chunk_x;

        if chunk_idx >= self.heights.len() {
            return Err(crate::error::AdtError::ParseError(format!(
                "Invalid chunk indices: ({chunk_x}, {chunk_y})"
            )));
        }

        if heights.len() != 145 {
            return Err(crate::error::AdtError::ParseError(format!(
                "Invalid heights length: {}, expected 145",
                heights.len()
            )));
        }

        // Copy heights
        self.heights[chunk_idx] = heights.to_vec();

        Ok(())
    }

    /// Set a specific height value
    pub fn set_height(
        &mut self,
        chunk_x: usize,
        chunk_y: usize,
        vertex_x: usize,
        vertex_y: usize,
        height: f32,
    ) -> Result<()> {
        let chunk_idx = chunk_y * 16 + chunk_x;

        if chunk_idx >= self.heights.len() {
            return Err(crate::error::AdtError::ParseError(format!(
                "Invalid chunk indices: ({chunk_x}, {chunk_y})"
            )));
        }

        let vertex_idx = vertex_y * 9 + vertex_x;

        if vertex_idx >= 145 {
            return Err(crate::error::AdtError::ParseError(format!(
                "Invalid vertex indices: ({vertex_x}, {vertex_y})"
            )));
        }

        // Set height
        self.heights[chunk_idx][vertex_idx] = height;

        Ok(())
    }

    /// Add a texture layer to a chunk
    pub fn add_chunk_layer(
        &mut self,
        chunk_x: usize,
        chunk_y: usize,
        texture_id: u32,
        flags: u32,
        alpha_map: Option<Vec<u8>>,
        effect_id: u32,
    ) -> Result<()> {
        let chunk_idx = chunk_y * 16 + chunk_x;

        if chunk_idx >= self.layers.len() {
            return Err(crate::error::AdtError::ParseError(format!(
                "Invalid chunk indices: ({chunk_x}, {chunk_y})"
            )));
        }

        if texture_id >= self.textures.len() as u32 {
            return Err(crate::error::AdtError::ParseError(format!(
                "Invalid texture ID: {texture_id}"
            )));
        }

        // If this is not the base layer, alpha map is required
        let is_base_layer = self.layers[chunk_idx].is_empty();

        if !is_base_layer && alpha_map.is_none() {
            return Err(crate::error::AdtError::ParseError(
                "Alpha map is required for non-base layers".to_string(),
            ));
        }

        // Validate alpha map size
        if let Some(ref alpha) = alpha_map {
            let expected_size = if flags & (MclyFlags::UseBigAlpha as u32) != 0 {
                64 * 64 // 64x64 alpha map
            } else {
                32 * 32 // 32x32 alpha map
            };

            if alpha.len() != expected_size {
                return Err(crate::error::AdtError::ParseError(format!(
                    "Invalid alpha map size: {}, expected {}",
                    alpha.len(),
                    expected_size
                )));
            }
        }

        // Add layer
        self.layers[chunk_idx].push(ChunkTextureInfo {
            texture_id,
            flags,
            alpha_map,
            effect_id,
        });

        Ok(())
    }

    /// Add a doodad placement
    pub fn add_doodad(
        &mut self,
        model_id: u32,
        position: [f32; 3],
        rotation: [f32; 3],
        scale: f32,
        flags: u16,
    ) -> Result<()> {
        if model_id >= self.models.len() as u32 {
            return Err(crate::error::AdtError::ParseError(format!(
                "Invalid model ID: {model_id}"
            )));
        }

        // Add doodad
        self.doodads.push(DoodadPlacement {
            name_id: model_id,
            unique_id: self.doodads.len() as u32, // Auto-assign unique ID
            position,
            rotation,
            scale,
            flags,
        });

        Ok(())
    }

    /// Add a WMO placement
    #[allow(clippy::too_many_arguments)]
    pub fn add_wmo_placement(
        &mut self,
        wmo_id: u32,
        position: [f32; 3],
        rotation: [f32; 3],
        bounds_min: [f32; 3],
        bounds_max: [f32; 3],
        flags: u16,
        doodad_set: u16,
        name_set: u16,
    ) -> Result<()> {
        if wmo_id >= self.wmos.len() as u32 {
            return Err(crate::error::AdtError::ParseError(format!(
                "Invalid WMO ID: {wmo_id}"
            )));
        }

        // Add WMO
        self.wmo_placements.push(ModelPlacement {
            name_id: wmo_id,
            unique_id: self.wmo_placements.len() as u32, // Auto-assign unique ID
            position,
            rotation,
            bounds_min,
            bounds_max,
            flags,
            doodad_set,
            name_set,
            padding: 0,
        });

        Ok(())
    }

    /// Add water to a chunk
    #[allow(clippy::too_many_arguments)]
    pub fn add_water(
        &mut self,
        chunk_x: usize,
        chunk_y: usize,
        liquid_type: u16,
        min_height: f32,
        max_height: f32,
        vertices: Option<Vec<(f32, [u8; 2])>>,
        x_res: u8,
        y_res: u8,
    ) -> Result<()> {
        let chunk_idx = chunk_y * 16 + chunk_x;

        if chunk_idx >= self.water_chunks.len() {
            return Err(crate::error::AdtError::ParseError(format!(
                "Invalid chunk indices: ({chunk_x}, {chunk_y})"
            )));
        }

        // Validate vertices
        if let Some(ref verts) = vertices {
            let expected_size = x_res as usize * y_res as usize;

            if verts.len() != expected_size {
                return Err(crate::error::AdtError::ParseError(format!(
                    "Invalid vertices size: {}, expected {}",
                    verts.len(),
                    expected_size
                )));
            }
        }

        // Add water
        self.water_chunks[chunk_idx] = Some(WaterInfo {
            liquid_type,
            min_height,
            max_height,
            vertices,
            x_res,
            y_res,
        });

        Ok(())
    }

    /// Set flight boundaries (TBC+)
    /// Set flight boundary planes (TBC+)
    ///
    /// Each plane contains 9 int16 values defining the boundary.
    /// This is the proper 36-byte MFBO structure validated against TrinityCore.
    pub fn set_flight_bounds(&mut self, max_plane: [i16; 9], min_plane: [i16; 9]) -> Result<()> {
        if self.version < AdtVersion::TBC {
            return Err(crate::error::AdtError::ParseError(format!(
                "Flight boundaries not supported in version: {}",
                self.version
            )));
        }

        self.flight_bounds = Some((max_plane, min_plane));

        Ok(())
    }

    /// Add a texture effect (Cataclysm+)
    pub fn add_texture_effect(&mut self, effect_id: u32) -> Result<()> {
        if self.version < AdtVersion::Cataclysm {
            return Err(crate::error::AdtError::ParseError(format!(
                "Texture effects not supported in version: {}",
                self.version
            )));
        }

        self.texture_effects.push(effect_id);

        Ok(())
    }

    /// Generate normal vectors for the heightmap
    pub fn generate_normals(&self) -> Vec<Vec<[i8; 3]>> {
        let mut normals = Vec::with_capacity(self.heights.len());

        for heights in &self.heights {
            let mut chunk_normals = vec![[0i8; 3]; heights.len()];

            // Calculate normals for the heightmap grid
            // For each vertex in the 9x9 grid
            for y in 0..9 {
                for x in 0..9 {
                    let idx = y * 9 + x;
                    let height = heights[idx];

                    // Get heights of adjacent vertices
                    let h_left = if x > 0 {
                        heights[y * 9 + (x - 1)]
                    } else {
                        height
                    };
                    let h_right = if x < 8 {
                        heights[y * 9 + (x + 1)]
                    } else {
                        height
                    };
                    let h_up = if y > 0 {
                        heights[(y - 1) * 9 + x]
                    } else {
                        height
                    };
                    let h_down = if y < 8 {
                        heights[(y + 1) * 9 + x]
                    } else {
                        height
                    };

                    // Calculate derivatives
                    let dx = (h_right - h_left) * 0.5;
                    let dy = (h_down - h_up) * 0.5;

                    // Calculate normal vector
                    let nx = -dx;
                    let ny = -dy;
                    let nz = 1.0;

                    // Normalize
                    let length = (nx * nx + ny * ny + nz * nz).sqrt();
                    let nx = (nx / length * 127.0) as i8;
                    let ny = (ny / length * 127.0) as i8;
                    let nz = (nz / length * 127.0) as i8;

                    chunk_normals[idx] = [nx, ny, nz];
                }
            }

            // Additional normal calculations for the control vertices
            // (For simplicity, we'll just duplicate the nearest vertex normal)
            // Real implementation would do proper interpolation

            normals.push(chunk_normals);
        }

        normals
    }

    /// Build the ADT file
    pub fn build(self) -> Result<Adt> {
        // Generate normals
        let normals = self.generate_normals();

        // Create MVER chunk
        let mver = MverChunk {
            version: self.version.to_mver_value(),
        };

        // Create MTEX chunk
        let mtex = if !self.textures.is_empty() {
            Some(MtexChunk {
                filenames: self.textures,
            })
        } else {
            None
        };

        // Create MMDX chunk
        let mmdx = if !self.models.is_empty() {
            Some(MmdxChunk {
                filenames: self.models,
            })
        } else {
            None
        };

        // Create MMID chunk
        let mmid = if let Some(ref mmdx) = mmdx {
            // Create offsets into MMDX chunk
            let mut offsets = Vec::with_capacity(mmdx.filenames.len());
            let mut current_offset = 0;

            for filename in &mmdx.filenames {
                offsets.push(current_offset);
                current_offset += filename.len() as u32 + 1; // +1 for null terminator
            }

            Some(MmidChunk { offsets })
        } else {
            None
        };

        // Create MWMO chunk
        let mwmo = if !self.wmos.is_empty() {
            Some(MwmoChunk {
                filenames: self.wmos,
            })
        } else {
            None
        };

        // Create MWID chunk
        let mwid = if let Some(ref mwmo) = mwmo {
            // Create offsets into MWMO chunk
            let mut offsets = Vec::with_capacity(mwmo.filenames.len());
            let mut current_offset = 0;

            for filename in &mwmo.filenames {
                offsets.push(current_offset);
                current_offset += filename.len() as u32 + 1; // +1 for null terminator
            }

            Some(MwidChunk { offsets })
        } else {
            None
        };

        // Create MDDF chunk
        let mddf = if !self.doodads.is_empty() {
            Some(MddfChunk {
                doodads: self.doodads,
            })
        } else {
            None
        };

        // Create MODF chunk
        let modf = if !self.wmo_placements.is_empty() {
            Some(ModfChunk {
                models: self.wmo_placements,
            })
        } else {
            None
        };

        // Create MCNK chunks
        let mut mcnk_chunks = Vec::with_capacity(256);

        for chunk_idx in 0..256 {
            let chunk_y = chunk_idx / 16;
            let chunk_x = chunk_idx % 16;

            // Get height map
            let height_map = if chunk_idx < self.heights.len() {
                self.heights[chunk_idx].clone()
            } else {
                vec![0.0; 145]
            };

            // Get normals
            let chunk_normals = if chunk_idx < normals.len() {
                normals[chunk_idx]
                    .iter()
                    .map(|&[x, y, z]| [x as u8, y as u8, z as u8])
                    .collect()
            } else {
                vec![[0, 0, 127]; 145]
            };

            // Get texture layers
            let texture_layers = if chunk_idx < self.layers.len() {
                // Convert to McnkTextureLayer format
                self.layers[chunk_idx]
                    .iter()
                    .enumerate()
                    .map(|(i, layer)| {
                        McnkTextureLayer {
                            texture_id: layer.texture_id,
                            flags: layer.flags,
                            alpha_map_offset: if i == 0 { 0 } else { i as u32 * 64 * 64 }, // Placeholder
                            effect_id: layer.effect_id,
                        }
                    })
                    .collect()
            } else {
                vec![McnkTextureLayer {
                    texture_id: 0,
                    flags: 0,
                    alpha_map_offset: 0,
                    effect_id: 0,
                }]
            };

            // Get alpha maps
            let alpha_maps = if chunk_idx < self.layers.len() {
                // Skip base layer, extract alpha maps from other layers
                self.layers[chunk_idx]
                    .iter()
                    .skip(1)
                    .flat_map(|layer| {
                        if let Some(alpha_map) = layer.alpha_map.clone() {
                            alpha_map.into_iter()
                        } else {
                            Vec::new().into_iter()
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            };

            // Create MCNK chunk
            let mcnk = McnkChunk {
                flags: 0,
                ix: chunk_x as u32,
                iy: chunk_y as u32,
                n_layers: texture_layers.len() as u32,
                n_doodad_refs: 0,
                mcvt_offset: 0, // Will be set during writing
                mcnr_offset: 0, // Will be set during writing
                mcly_offset: 0, // Will be set during writing
                mcrf_offset: 0, // Will be set during writing
                mcal_offset: 0, // Will be set during writing
                mcal_size: 0,   // Will be set during writing
                mcsh_offset: 0, // Will be set during writing
                mcsh_size: 0,   // Will be set during writing
                area_id: 0,
                n_map_obj_refs: 0,
                holes: 0,
                s1: 0,
                s2: 0,
                d1: 0,
                d2: 0,
                d3: 0,
                pred_tex: 0,
                n_effect_doodad: 0,
                mcse_offset: 0, // Will be set during writing
                n_sound_emitters: 0,
                liquid_offset: 0, // Will be set during writing
                liquid_size: 0,   // Will be set during writing
                position: [
                    chunk_x as f32 * 533.333_3,
                    chunk_y as f32 * 533.333_3,
                    0.0, // Will be determined from heightmap
                ],
                mccv_offset: 0, // Will be set during writing
                mclv_offset: 0, // Will be set during writing
                texture_id: 0,
                props: 0,
                effect_id: 0,
                height_map,
                normals: chunk_normals,
                texture_layers,
                doodad_refs: Vec::new(),
                map_obj_refs: Vec::new(),
                alpha_maps,
                mclq: None,                // No liquid data in builder
                vertex_colors: Vec::new(), // No vertex colors in builder
            };

            mcnk_chunks.push(mcnk);
        }

        // Create MFBO chunk (TBC+)
        let mfbo = if let Some((max_plane, min_plane)) = self.flight_bounds {
            Some(MfboChunk {
                max: max_plane,
                min: min_plane,
            })
        } else if self.version >= AdtVersion::TBC {
            // Default flight bounds planes
            Some(MfboChunk {
                max: [0; 9],
                min: [0; 9],
            })
        } else {
            None
        };

        // Create MH2O chunk (WotLK+)
        let _mh2o = if self.version >= AdtVersion::WotLK {
            // Create water data
            let mut chunks = Vec::with_capacity(256);

            for chunk_idx in 0..256 {
                let water_info = if chunk_idx < self.water_chunks.len() {
                    self.water_chunks[chunk_idx].clone()
                } else {
                    None
                };

                if let Some(info) = water_info {
                    // Create water instance
                    let instance = Mh2oInstance {
                        liquid_type: info.liquid_type,
                        liquid_object: 0,
                        x_offset: 0,
                        y_offset: 0,
                        width: 8,  // Full chunk coverage (8 cells = 9 vertices)
                        height: 8, // Full chunk coverage (8 cells = 9 vertices)
                        level_data: if info.vertices.is_some() {
                            WaterLevelData::Variable {
                                min_height: info.min_height,
                                max_height: info.max_height,
                                offset_height_map: 0, // Will be set during writing
                                heights: None,        // Will be generated during writing
                            }
                        } else {
                            WaterLevelData::Uniform {
                                min_height: info.min_height,
                                max_height: info.max_height,
                            }
                        },
                        vertex_data: if let Some(vertices) = info.vertices {
                            Some(WaterVertexData {
                                offset_vertex_data: 0, // Will be set during writing
                                x_vertices: info.x_res,
                                y_vertices: info.y_res,
                                vertices: Some(
                                    vertices
                                        .iter()
                                        .map(|(depth, flow)| WaterVertex {
                                            depth: *depth,
                                            flow: *flow,
                                        })
                                        .collect(),
                                ),
                            })
                        } else {
                            None
                        },
                        attributes: Vec::new(),
                    };

                    // Create render mask (all cells have water)
                    let render_mask = Some(Mh2oRenderMask { mask: [0xFF; 8] });

                    chunks.push(Mh2oEntry {
                        header: Mh2oHeader {
                            offset_instances: 0, // Will be set during writing
                            layer_count: 1,
                            offset_attributes: 0, // Will be set during writing
                        },
                        instances: vec![instance],
                        attributes: None,
                        render_mask,
                    });
                } else {
                    // No water
                    chunks.push(Mh2oEntry {
                        header: Mh2oHeader {
                            offset_instances: 0,
                            layer_count: 0,
                            offset_attributes: 0,
                        },
                        instances: Vec::new(),
                        attributes: None,
                        render_mask: None,
                    });
                }
            }

            Some(AdvancedMh2oChunk { chunks })
        } else {
            None
        };

        // Create MTFX chunk (Cataclysm+)
        let mtfx = if self.version >= AdtVersion::Cataclysm && !self.texture_effects.is_empty() {
            Some(MtfxChunk {
                effects: self
                    .texture_effects
                    .iter()
                    .map(|&id| TextureEffect {
                        use_cubemap: (id & 0x1) != 0,
                        texture_scale: ((id >> 4) & 0xF) as u8,
                        raw_flags: id,
                    })
                    .collect(),
            })
        } else {
            None
        };

        // Create MHDR chunk
        let mhdr = Some(MhdrChunk {
            flags: 0,
            mcin_offset: 0, // Will be set during writing
            mtex_offset: 0, // Will be set during writing
            mmdx_offset: 0, // Will be set during writing
            mmid_offset: 0, // Will be set during writing
            mwmo_offset: 0, // Will be set during writing
            mwid_offset: 0, // Will be set during writing
            mddf_offset: 0, // Will be set during writing
            modf_offset: 0, // Will be set during writing
            mfbo_offset: if self.version >= AdtVersion::TBC {
                Some(0)
            } else {
                None
            }, // Will be set during writing
            mh2o_offset: if self.version >= AdtVersion::WotLK {
                Some(0)
            } else {
                None
            }, // Will be set during writing
            mtfx_offset: if self.version >= AdtVersion::Cataclysm {
                Some(0)
            } else {
                None
            }, // Will be set during writing
        });

        // Create MCIN chunk
        let mcin = Some(McinChunk {
            entries: vec![
                McnkEntry {
                    offset: 0, // Will be set during writing
                    size: 0,   // Will be set during writing
                    flags: 0,
                    layer_count: 0,
                };
                256
            ],
        });

        // Create the ADT
        let adt = Adt {
            version: self.version,
            mver,
            mhdr,
            mcnk_chunks,
            mcin,
            mtex,
            mmdx,
            mmid,
            mwmo,
            mwid,
            mddf,
            modf,
            mfbo,
            mh2o: None, // TODO: Convert from mh20::Mh2oChunk to chunk::Mh2oChunk
            mtfx,
            mamp: None, // MAMP not supported in builder yet
            mtxp: None, // MTXP not supported in builder yet
        };

        Ok(adt)
    }
}

/// Helper function to create a flat terrain
pub fn create_flat_terrain(version: AdtVersion, base_height: f32) -> Result<Adt> {
    let mut builder = AdtBuilder::new(version);

    // Set all heights to the base height
    for chunk_y in 0..16 {
        for chunk_x in 0..16 {
            let heights = vec![base_height; 145];
            builder.set_chunk_heights(chunk_x, chunk_y, &heights)?;
        }
    }

    // Add a default texture
    let texture_id = builder.add_texture("textures/terrain/generic/grass_01.blp");

    // Set base layer for all chunks
    for chunk_y in 0..16 {
        for chunk_x in 0..16 {
            builder.add_chunk_layer(chunk_x, chunk_y, texture_id, 0, None, 0)?;
        }
    }

    // Build the ADT
    builder.build()
}

/// Helper function to create a terrain from a heightmap image
#[cfg(feature = "image")]
#[allow(dead_code)]
pub fn create_terrain_from_heightmap<P: AsRef<std::path::Path>>(
    version: AdtVersion,
    heightmap_path: P,
    min_height: f32,
    max_height: f32,
) -> Result<Adt> {
    use image::{GenericImageView, Pixel, open};

    // Open the heightmap image
    let img = open(heightmap_path).map_err(|e| {
        crate::error::AdtError::ParseError(format!("Failed to open heightmap image: {e}"))
    })?;

    // Resize to 145x145 if needed
    let img = if img.width() != 145 || img.height() != 145 {
        img.resize_exact(145, 145, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };

    // Create builder
    let mut builder = AdtBuilder::new(version);

    // Add a default texture
    let texture_id = builder.add_texture("textures/terrain/generic/grass_01.blp");

    // Process heightmap
    for chunk_y in 0..16 {
        for chunk_x in 0..16 {
            // Extract height values for this chunk
            let mut heights = Vec::with_capacity(145);

            // Get 9x9 grid starting at the chunk's corner
            for local_y in 0..9 {
                for local_x in 0..9 {
                    let img_x = chunk_x * 9 + local_x;
                    let img_y = chunk_y * 9 + local_y;

                    // Get heightmap value (0-255 for grayscale)
                    let pixel = img.get_pixel(img_x as u32, img_y as u32);
                    let height_value = pixel.to_luma().0[0] as f32 / 255.0;

                    // Map to height range
                    let height = min_height + height_value * (max_height - min_height);
                    heights.push(height);
                }
            }

            // Add inner control points (for a 9x9 grid with 8x8 inner points = 64 additional points)
            // For simplicity, we'll use interpolated values
            // Real implementation would need proper interpolation

            // Set heights for this chunk
            builder.set_chunk_heights(chunk_x, chunk_y, &heights)?;

            // Add base texture layer
            builder.add_chunk_layer(chunk_x, chunk_y, texture_id, 0, None, 0)?;
        }
    }

    // Build the ADT
    builder.build()
}
