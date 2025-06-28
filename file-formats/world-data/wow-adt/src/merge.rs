// merge.rs - Merge multiple ADT files

use crate::Adt;
use crate::chunk::*;
use crate::error::{AdtError, Result};
use crate::mh2o::Mh2oChunk as AdvancedMh2oChunk;
use crate::mh2o::{Mh2oEntry, Mh2oHeader};
use crate::version::AdtVersion;
use std::collections::{HashMap, HashSet};

/// Options for merging ADT files
#[derive(Debug, Clone)]
pub struct MergeOptions {
    /// Which chunks to include from each ADT (by coordinates)
    pub chunk_selection: Vec<Vec<(u32, u32)>>,
    /// Whether to merge textures
    pub merge_textures: bool,
    /// Whether to merge models
    pub merge_models: bool,
    /// Whether to merge doodads
    pub merge_doodads: bool,
    /// Whether to merge WMOs
    pub merge_wmos: bool,
    /// Target version for the merged ADT
    pub target_version: Option<AdtVersion>,
    /// Whether to reindex texture IDs
    pub reindex_textures: bool,
    /// Whether to reindex model IDs
    pub reindex_models: bool,
}

impl Default for MergeOptions {
    fn default() -> Self {
        Self {
            chunk_selection: Vec::new(), // Default: include all chunks
            merge_textures: true,
            merge_models: true,
            merge_doodads: true,
            merge_wmos: true,
            target_version: None, // Use version of first ADT
            reindex_textures: true,
            reindex_models: true,
        }
    }
}

/// Merge multiple ADT files into one
pub fn merge_adts(adts: &[Adt], options: &MergeOptions) -> Result<Adt> {
    if adts.is_empty() {
        return Err(AdtError::ParseError("No ADTs to merge".to_string()));
    }

    // Determine target version
    let target_version = options.target_version.unwrap_or(adts[0].version());

    // Texture and model mappings
    let mut texture_map = HashMap::new();
    let mut model_map = HashMap::new();
    let mut wmo_map = HashMap::new();

    // Merged data
    let mut textures = Vec::new();
    let mut models = Vec::new();
    let mut wmos = Vec::new();
    let mut doodads = Vec::new();
    let mut wmo_placements = Vec::new();
    let mut mcnk_chunks = Vec::new();

    // Track which MCNK coordinates we've already included
    let mut included_coords = HashSet::new();

    // Process each ADT
    for (adt_idx, adt) in adts.iter().enumerate() {
        // Get the chunk selections for this ADT
        let empty_selection = Vec::new();
        let selected_chunks = if adt_idx < options.chunk_selection.len() {
            &options.chunk_selection[adt_idx]
        } else {
            &empty_selection // Empty = include all
        };

        // Merge textures
        if options.merge_textures {
            if let Some(ref mtex) = adt.mtex {
                for (i, texture) in mtex.filenames.iter().enumerate() {
                    // Check if this texture exists in the merged list
                    let merged_id = if options.reindex_textures {
                        // Find or add the texture
                        if let Some(&id) = texture_map.get(texture) {
                            id
                        } else {
                            let id = textures.len();
                            textures.push(texture.clone());
                            texture_map.insert(texture.clone(), id);
                            id
                        }
                    } else {
                        // Use the original ID (may cause conflicts)
                        let id = i;

                        // Ensure the textures vector is large enough
                        while textures.len() <= id {
                            textures.push(String::new());
                        }

                        // Update the texture
                        textures[id] = texture.clone();
                        id
                    };

                    // Store the mapping for this ADT's texture ID
                    texture_map.insert(format!("adt{adt_idx}:{i}"), merged_id);
                }
            }
        }

        // Merge models
        if options.merge_models {
            if let Some(ref mmdx) = adt.mmdx {
                for (i, model) in mmdx.filenames.iter().enumerate() {
                    // Check if this model exists in the merged list
                    let merged_id = if options.reindex_models {
                        // Find or add the model
                        if let Some(&id) = model_map.get(model) {
                            id
                        } else {
                            let id = models.len();
                            models.push(model.clone());
                            model_map.insert(model.clone(), id);
                            id
                        }
                    } else {
                        // Use the original ID (may cause conflicts)
                        let id = i;

                        // Ensure the models vector is large enough
                        while models.len() <= id {
                            models.push(String::new());
                        }

                        // Update the model
                        models[id] = model.clone();
                        id
                    };

                    // Store the mapping for this ADT's model ID
                    model_map.insert(format!("adt{adt_idx}:{i}"), merged_id);
                }
            }
        }

        // Merge WMOs
        if options.merge_wmos {
            if let Some(ref mwmo) = adt.mwmo {
                for (i, wmo) in mwmo.filenames.iter().enumerate() {
                    // Check if this WMO exists in the merged list
                    let merged_id = if options.reindex_models {
                        // Find or add the WMO
                        if let Some(&id) = wmo_map.get(wmo) {
                            id
                        } else {
                            let id = wmos.len();
                            wmos.push(wmo.clone());
                            wmo_map.insert(wmo.clone(), id);
                            id
                        }
                    } else {
                        // Use the original ID (may cause conflicts)
                        let id = i;

                        // Ensure the WMOs vector is large enough
                        while wmos.len() <= id {
                            wmos.push(String::new());
                        }

                        // Update the WMO
                        wmos[id] = wmo.clone();
                        id
                    };

                    // Store the mapping for this ADT's WMO ID
                    wmo_map.insert(format!("adt{adt_idx}:{i}"), merged_id);
                }
            }
        }

        // Merge doodads
        if options.merge_doodads && options.merge_models {
            if let Some(ref mddf) = adt.mddf {
                for doodad in &mddf.doodads {
                    // Look up the remapped model ID
                    let orig_id = doodad.name_id;

                    // Find the model path (if available)
                    let model_path = if let Some(ref mmdx) = adt.mmdx {
                        if (orig_id as usize) < mmdx.filenames.len() {
                            Some(&mmdx.filenames[orig_id as usize])
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Get the new model ID
                    let new_id = if let Some(path) = model_path {
                        // Look up by path
                        if let Some(&id) = model_map.get(path) {
                            id as u32
                        } else {
                            // Keep original ID if no mapping found
                            orig_id
                        }
                    } else {
                        // Look up by ADT index and original ID
                        let key = format!("adt{adt_idx}:{orig_id}");

                        if let Some(&id) = model_map.get(&key) {
                            id as u32
                        } else {
                            // Keep original ID if no mapping found
                            orig_id
                        }
                    };

                    // Create a new doodad with remapped ID
                    let mut new_doodad = doodad.clone();
                    new_doodad.name_id = new_id;
                    new_doodad.unique_id = doodads.len() as u32; // Assign new unique ID

                    doodads.push(new_doodad);
                }
            }
        }

        // Merge WMO placements
        if options.merge_wmos {
            if let Some(ref modf) = adt.modf {
                for wmo in &modf.models {
                    // Look up the remapped WMO ID
                    let orig_id = wmo.name_id;

                    // Find the WMO path (if available)
                    let wmo_path = if let Some(ref mwmo) = adt.mwmo {
                        if (orig_id as usize) < mwmo.filenames.len() {
                            Some(&mwmo.filenames[orig_id as usize])
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Get the new WMO ID
                    let new_id = if let Some(path) = wmo_path {
                        // Look up by path
                        if let Some(&id) = wmo_map.get(path) {
                            id as u32
                        } else {
                            // Keep original ID if no mapping found
                            orig_id
                        }
                    } else {
                        // Look up by ADT index and original ID
                        let key = format!("adt{adt_idx}:{orig_id}");

                        if let Some(&id) = wmo_map.get(&key) {
                            id as u32
                        } else {
                            // Keep original ID if no mapping found
                            orig_id
                        }
                    };

                    // Create a new WMO placement with remapped ID
                    let mut new_wmo = wmo.clone();
                    new_wmo.name_id = new_id;
                    new_wmo.unique_id = wmo_placements.len() as u32; // Assign new unique ID

                    wmo_placements.push(new_wmo);
                }
            }
        }

        // Merge MCNK chunks
        for mcnk in &adt.mcnk_chunks {
            // Check if we should include this chunk
            let include = if selected_chunks.is_empty() {
                // No selection = include all
                true
            } else {
                // Check if this chunk's coordinates are in the selection
                selected_chunks.contains(&(mcnk.ix, mcnk.iy))
            };

            if include {
                // Check if we already have a chunk at these coordinates
                let coords = (mcnk.ix, mcnk.iy);

                if included_coords.contains(&coords) {
                    // Skip this chunk (already included)
                    continue;
                }

                // Create new chunk with remapped texture IDs
                let mut new_mcnk = mcnk.clone();

                // Remap texture layer IDs
                if options.merge_textures && options.reindex_textures {
                    for layer in &mut new_mcnk.texture_layers {
                        // Look up the remapped texture ID
                        let orig_id = layer.texture_id;
                        let key = format!("adt{adt_idx}:{orig_id}");

                        if let Some(&id) = texture_map.get(&key) {
                            layer.texture_id = id as u32;
                        }
                    }
                }

                // Add the chunk
                mcnk_chunks.push(new_mcnk);
                included_coords.insert(coords);
            }
        }
    }

    // Create merged MTEX
    let mtex = if !textures.is_empty() {
        Some(MtexChunk {
            filenames: textures,
        })
    } else {
        None
    };

    // Create merged MMDX
    let mmdx = if !models.is_empty() {
        Some(MmdxChunk { filenames: models })
    } else {
        None
    };

    // Create merged MMID
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

    // Create merged MWMO
    let mwmo = if !wmos.is_empty() {
        Some(MwmoChunk { filenames: wmos })
    } else {
        None
    };

    // Create merged MWID
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

    // Create merged MDDF
    let mddf = if !doodads.is_empty() {
        Some(MddfChunk { doodads })
    } else {
        None
    };

    // Create merged MODF
    let modf = if !wmo_placements.is_empty() {
        Some(ModfChunk {
            models: wmo_placements,
        })
    } else {
        None
    };

    // Create merged MCIN
    let mcin = if !mcnk_chunks.is_empty() {
        Some(McinChunk {
            entries: vec![
                McnkEntry {
                    offset: 0, // Will be set during writing
                    size: 0,   // Will be set during writing
                    flags: 0,
                    layer_count: 0,
                };
                256
            ],
        })
    } else {
        None
    };

    // Create version-specific chunks

    // Merged MFBO (TBC+)
    let mfbo = if target_version >= AdtVersion::TBC {
        // Use the first ADT's MFBO if available
        let mut found_mfbo = None;

        for adt in adts {
            if let Some(ref mfbo) = adt.mfbo {
                found_mfbo = Some(mfbo.clone());
                break;
            }
        }

        // Default if none found
        found_mfbo.or({
            Some(MfboChunk {
                max: [0, 0],
                min: [0, 0],
                additional_data: Vec::new(),
            })
        })
    } else {
        None
    };

    // Merged MH2O (WotLK+)
    let mh2o = if target_version >= AdtVersion::WotLK {
        // Create empty water data (will be filled during writing)
        let chunks = vec![
            Mh2oEntry {
                header: Mh2oHeader {
                    offset_instances: 0,
                    layer_count: 0,
                    offset_render_mask: 0,
                },
                instances: Vec::new(),
                render_mask: None,
            };
            256
        ];

        Some(AdvancedMh2oChunk { chunks })
    } else {
        None
    };

    // Merged MTFX (Cataclysm+)
    let mtfx = if target_version >= AdtVersion::Cataclysm {
        // Use the first ADT's MTFX if available
        let mut found_mtfx = None;

        for adt in adts {
            if let Some(ref mtfx) = adt.mtfx {
                found_mtfx = Some(mtfx.clone());
                break;
            }
        }

        // Default if none found
        found_mtfx.or_else(|| {
            Some(MtfxChunk {
                effects: Vec::new(),
            })
        })
    } else {
        None
    };

    // Create the merged ADT
    let merged_adt = Adt {
        version: target_version,
        mver: MverChunk {
            version: target_version.to_mver_value(),
        },
        mhdr: Some(MhdrChunk {
            flags: 0,
            mcin_offset: 0, // Will be set during writing
            mtex_offset: 0, // Will be set during writing
            mmdx_offset: 0, // Will be set during writing
            mmid_offset: 0, // Will be set during writing
            mwmo_offset: 0, // Will be set during writing
            mwid_offset: 0, // Will be set during writing
            mddf_offset: 0, // Will be set during writing
            modf_offset: 0, // Will be set during writing
            mfbo_offset: if target_version >= AdtVersion::TBC {
                Some(0)
            } else {
                None
            }, // Will be set during writing
            mh2o_offset: if target_version >= AdtVersion::WotLK {
                Some(0)
            } else {
                None
            }, // Will be set during writing
            mtfx_offset: if target_version >= AdtVersion::Cataclysm {
                Some(0)
            } else {
                None
            }, // Will be set during writing
        }),
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
        mh2o,
        mtfx,
    };

    Ok(merged_adt)
}

/// Merge a specific chunk from one ADT into another
pub fn merge_chunk(
    target: &mut Adt,
    source: &Adt,
    source_coords: (u32, u32),
    target_coords: (u32, u32),
) -> Result<()> {
    // Find the source chunk
    let mut source_chunk = None;

    for chunk in &source.mcnk_chunks {
        if chunk.ix == source_coords.0 && chunk.iy == source_coords.1 {
            source_chunk = Some(chunk.clone());
            break;
        }
    }

    let source_chunk = source_chunk.ok_or_else(|| {
        AdtError::ParseError(format!(
            "Source chunk not found at coordinates ({}, {})",
            source_coords.0, source_coords.1
        ))
    })?;

    // Find or create the target chunk
    let mut target_idx = None;

    for (i, chunk) in target.mcnk_chunks.iter().enumerate() {
        if chunk.ix == target_coords.0 && chunk.iy == target_coords.1 {
            target_idx = Some(i);
            break;
        }
    }

    // Create a modified copy of the source chunk
    let mut new_chunk = source_chunk.clone();
    new_chunk.ix = target_coords.0;
    new_chunk.iy = target_coords.1;

    // Update position based on coordinates
    new_chunk.position[0] = target_coords.0 as f32 * 533.333_3;
    new_chunk.position[1] = target_coords.1 as f32 * 533.333_3;

    // Replace or add the chunk
    if let Some(idx) = target_idx {
        target.mcnk_chunks[idx] = new_chunk;
    } else {
        target.mcnk_chunks.push(new_chunk);
    }

    Ok(())
}

/// Extract a portion of an ADT into a new ADT
pub fn extract_portion(
    source: &Adt,
    start_x: u32,
    start_y: u32,
    width: u32,
    height: u32,
) -> Result<Adt> {
    // Validate dimensions
    if start_x + width > 16 || start_y + height > 16 {
        return Err(AdtError::ParseError(format!(
            "Invalid extraction region: ({}, {}) -> ({}, {})",
            start_x,
            start_y,
            start_x + width,
            start_y + height
        )));
    }

    // Create selection for the specified portion
    let mut selection = Vec::new();

    for y in start_y..(start_y + height) {
        for x in start_x..(start_x + width) {
            selection.push((x, y));
        }
    }

    // Create merge options
    let options = MergeOptions {
        chunk_selection: vec![selection],
        merge_textures: true,
        merge_models: true,
        merge_doodads: true,
        merge_wmos: true,
        target_version: Some(source.version()),
        reindex_textures: true,
        reindex_models: true,
    };

    // Merge (extract) the specified portion
    merge_adts(&[source.clone()], &options)
}
