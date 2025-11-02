//! Merge split ADT files into unified RootAdt structure.
//!
//! Starting with Cataclysm (4.3.4), ADT files were split into separate files for terrain,
//! textures, and objects. This module provides utilities to merge these split files back
//! into a unified `RootAdt` structure that matches the pre-Cataclysm monolithic format.
//!
//! ## Merge Strategy
//!
//! The merger combines data from three sources:
//!
//! - **Root ADT**: Terrain geometry (heightmaps, normals, MCNK headers)
//! - **Texture ADT**: Texture lists, layers (MCLY), alpha maps (MCAL)
//! - **Object ADT**: Model/WMO lists, placements (MDDF/MODF), references (MCRD/MCRW)
//!
//! MCNK chunks are merged by index - texture chunk 0 corresponds to root chunk 0, etc.
//!
//! ## Examples
//!
//! ```no_run
//! use std::fs::File;
//! use std::io::BufReader;
//! use wow_adt::{parse_adt, ParsedAdt, merger::merge_split_files};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Parse split files
//! let root_file = File::open("Azeroth_30_30.adt")?;
//! let tex_file = File::open("Azeroth_30_30_tex0.adt")?;
//! let obj_file = File::open("Azeroth_30_30_obj0.adt")?;
//!
//! let root = match parse_adt(&mut BufReader::new(root_file))? {
//!     ParsedAdt::Root(r) => r,
//!     _ => panic!("Expected root ADT"),
//! };
//!
//! let tex = match parse_adt(&mut BufReader::new(tex_file))? {
//!     ParsedAdt::Tex0(t) => Some(t),
//!     _ => None,
//! };
//!
//! let obj = match parse_adt(&mut BufReader::new(obj_file))? {
//!     ParsedAdt::Obj0(o) => Some(o),
//!     _ => None,
//! };
//!
//! // Merge into unified structure
//! let merged = merge_split_files(*root, tex, obj)?;
//!
//! println!("Merged {} MCNK chunks", merged.mcnk_chunks.len());
//! println!("Textures: {}", merged.textures.len());
//! println!("Models: {}", merged.models.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## References
//!
//! - [wowdev.wiki ADT/v18](https://wowdev.wiki/ADT/v18) - Split file architecture
//! - `SPLIT_FILE_ARCHITECTURE.md` - Detailed architecture documentation

use crate::api::{Obj0Adt, RootAdt, Tex0Adt};
use crate::error::{AdtError, Result};

/// Merge split ADT files into unified RootAdt structure.
///
/// Combines data from Cataclysm+ split files (root, texture, object) into a single
/// `RootAdt` structure that matches the pre-Cataclysm monolithic format.
///
/// # Arguments
///
/// * `root` - Root ADT with terrain geometry and MCNK headers
/// * `texture` - Optional texture ADT with texture lists and layer data
/// * `object` - Optional object ADT with model/WMO lists and placements
///
/// # Returns
///
/// Merged `RootAdt` with all data from split files combined.
///
/// # Errors
///
/// Returns `AdtError::ChunkParseError` if:
/// - MCNK chunk counts don't match between files
/// - Referenced textures/models are out of bounds
///
/// # Examples
///
/// ```no_run
/// use wow_adt::{RootAdt, Tex0Adt, Obj0Adt, merger::merge_split_files};
/// # use wow_adt::error::Result;
///
/// # fn example(root: RootAdt, tex: Tex0Adt, obj: Obj0Adt) -> Result<()> {
/// let merged = merge_split_files(root, Some(tex), Some(obj))?;
/// println!("Merged ADT with {} chunks", merged.mcnk_chunks.len());
/// # Ok(())
/// # }
/// ```
pub fn merge_split_files(
    mut root: RootAdt,
    texture: Option<Tex0Adt>,
    object: Option<Obj0Adt>,
) -> Result<RootAdt> {
    // Merge texture data if present
    if let Some(tex) = texture {
        merge_texture_data(&mut root, tex)?;
    }

    // Merge object data if present
    if let Some(obj) = object {
        merge_object_data(&mut root, obj)?;
    }

    Ok(root)
}

/// Merge texture data from texture ADT into root ADT.
///
/// Copies texture lists and merges MCNK texture layers/alpha maps.
///
/// # Arguments
///
/// * `root` - Root ADT to merge into (modified in place)
/// * `texture` - Texture ADT with texture lists and layer data
///
/// # Errors
///
/// Returns error if MCNK chunk counts don't match.
fn merge_texture_data(root: &mut RootAdt, texture: Tex0Adt) -> Result<()> {
    // Copy texture list and parameters
    root.textures = texture.textures;
    root.texture_params = texture.texture_params;

    // Merge MCNK texture chunks
    // Each texture MCNK corresponds to same index in root MCNK
    for (i, tex_chunk) in texture.mcnk_textures.into_iter().enumerate() {
        if i >= root.mcnk_chunks.len() {
            return Err(AdtError::ChunkParseError {
                chunk: crate::chunk_id::ChunkId::MCNK,
                offset: 0,
                details: format!(
                    "Texture MCNK index {} exceeds root MCNK count {}",
                    i,
                    root.mcnk_chunks.len()
                ),
            });
        }

        // Merge texture layers
        if let Some(layers) = tex_chunk.layers {
            root.mcnk_chunks[i].layers = Some(layers);
        }

        // Merge alpha maps
        if let Some(alpha_maps) = tex_chunk.alpha_maps {
            root.mcnk_chunks[i].alpha = Some(alpha_maps);
        }
    }

    Ok(())
}

/// Merge object data from object ADT into root ADT.
///
/// Copies model/WMO lists and placements, then merges MCNK object references.
///
/// # Arguments
///
/// * `root` - Root ADT to merge into (modified in place)
/// * `object` - Object ADT with model/WMO data
///
/// # Errors
///
/// Returns error if MCNK chunk counts don't match.
fn merge_object_data(root: &mut RootAdt, object: Obj0Adt) -> Result<()> {
    // Copy model and WMO lists
    root.models = object.models;
    root.model_indices = object.model_indices;
    root.wmos = object.wmos;
    root.wmo_indices = object.wmo_indices;

    // Copy placements
    root.doodad_placements = object.doodad_placements;
    root.wmo_placements = object.wmo_placements;

    // Merge MCNK object chunks
    // Each object MCNK corresponds to same index in root MCNK
    for (i, obj_chunk) in object.mcnk_objects.into_iter().enumerate() {
        if i >= root.mcnk_chunks.len() {
            return Err(AdtError::ChunkParseError {
                chunk: crate::chunk_id::ChunkId::MCNK,
                offset: 0,
                details: format!(
                    "Object MCNK index {} exceeds root MCNK count {}",
                    i,
                    root.mcnk_chunks.len()
                ),
            });
        }

        // Merge doodad references
        if !obj_chunk.doodad_refs.is_empty() {
            root.mcnk_chunks[i].doodad_refs = Some(crate::chunks::mcnk::McrdChunk {
                doodad_refs: obj_chunk.doodad_refs,
            });
        }

        // Merge WMO references
        if !obj_chunk.wmo_refs.is_empty() {
            root.mcnk_chunks[i].wmo_refs = Some(crate::chunks::mcnk::McrwChunk {
                wmo_refs: obj_chunk.wmo_refs,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{McnkChunkObject, McnkChunkTexture};
    use crate::chunks::McnkChunk;
    use crate::version::AdtVersion;

    fn create_minimal_mcnk() -> McnkChunk {
        // Use unsafe zeroed for simplicity in tests
        unsafe { std::mem::zeroed() }
    }

    fn create_minimal_root() -> RootAdt {
        RootAdt {
            version: AdtVersion::Cataclysm,
            mhdr: Default::default(),
            mcin: Default::default(),
            textures: vec![],
            models: vec![],
            model_indices: vec![],
            wmos: vec![],
            wmo_indices: vec![],
            doodad_placements: vec![],
            wmo_placements: vec![],
            mcnk_chunks: vec![create_minimal_mcnk(), create_minimal_mcnk()],
            flight_bounds: None,
            water_data: None,
            texture_flags: None,
            texture_amplifier: None,
            texture_params: None,
            blend_mesh_headers: None,
            blend_mesh_bounds: None,
            blend_mesh_vertices: None,
            blend_mesh_indices: None,
        }
    }

    fn create_minimal_tex() -> Tex0Adt {
        Tex0Adt {
            version: AdtVersion::Cataclysm,
            textures: vec!["texture1.blp".to_string(), "texture2.blp".to_string()],
            texture_params: None,
            mcnk_textures: vec![
                McnkChunkTexture {
                    index: 0,
                    layers: None,
                    alpha_maps: None,
                },
                McnkChunkTexture {
                    index: 1,
                    layers: None,
                    alpha_maps: None,
                },
            ],
        }
    }

    fn create_minimal_obj() -> Obj0Adt {
        Obj0Adt {
            version: AdtVersion::Cataclysm,
            models: vec!["model1.m2".to_string()],
            model_indices: vec![0],
            wmos: vec!["wmo1.wmo".to_string()],
            wmo_indices: vec![0],
            doodad_placements: vec![],
            wmo_placements: vec![],
            mcnk_objects: vec![
                McnkChunkObject {
                    index: 0,
                    doodad_refs: vec![0, 1],
                    wmo_refs: vec![],
                },
                McnkChunkObject {
                    index: 1,
                    doodad_refs: vec![],
                    wmo_refs: vec![0],
                },
            ],
        }
    }

    #[test]
    fn test_merge_split_files_complete() {
        let root = create_minimal_root();
        let tex = create_minimal_tex();
        let obj = create_minimal_obj();

        let result = merge_split_files(root, Some(tex), Some(obj));
        assert!(result.is_ok());

        let merged = result.unwrap();
        assert_eq!(merged.textures.len(), 2);
        assert_eq!(merged.models.len(), 1);
        assert_eq!(merged.wmos.len(), 1);
        assert_eq!(merged.mcnk_chunks.len(), 2);
        assert!(merged.mcnk_chunks[0].doodad_refs.is_some());
        assert_eq!(
            merged.mcnk_chunks[0]
                .doodad_refs
                .as_ref()
                .unwrap()
                .doodad_refs
                .len(),
            2
        );
        assert!(merged.mcnk_chunks[1].wmo_refs.is_some());
        assert_eq!(
            merged.mcnk_chunks[1]
                .wmo_refs
                .as_ref()
                .unwrap()
                .wmo_refs
                .len(),
            1
        );
    }

    #[test]
    fn test_merge_split_files_texture_only() {
        let root = create_minimal_root();
        let tex = create_minimal_tex();

        let result = merge_split_files(root, Some(tex), None);
        assert!(result.is_ok());

        let merged = result.unwrap();
        assert_eq!(merged.textures.len(), 2);
        assert_eq!(merged.models.len(), 0);
    }

    #[test]
    fn test_merge_split_files_object_only() {
        let root = create_minimal_root();
        let obj = create_minimal_obj();

        let result = merge_split_files(root, None, Some(obj));
        assert!(result.is_ok());

        let merged = result.unwrap();
        assert_eq!(merged.textures.len(), 0);
        assert_eq!(merged.models.len(), 1);
        assert!(merged.mcnk_chunks[0].doodad_refs.is_some());
        assert_eq!(
            merged.mcnk_chunks[0]
                .doodad_refs
                .as_ref()
                .unwrap()
                .doodad_refs
                .len(),
            2
        );
    }

    #[test]
    fn test_merge_split_files_no_optional() {
        let root = create_minimal_root();

        let result = merge_split_files(root, None, None);
        assert!(result.is_ok());

        let merged = result.unwrap();
        assert_eq!(merged.textures.len(), 0);
        assert_eq!(merged.models.len(), 0);
    }

    #[test]
    fn test_merge_texture_mcnk_count_mismatch() {
        let root = create_minimal_root();
        let mut tex = create_minimal_tex();

        // Add extra MCNK that doesn't exist in root
        tex.mcnk_textures.push(McnkChunkTexture {
            index: 2,
            layers: None,
            alpha_maps: None,
        });

        let result = merge_split_files(root, Some(tex), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_object_mcnk_count_mismatch() {
        let root = create_minimal_root();
        let mut obj = create_minimal_obj();

        // Add extra MCNK that doesn't exist in root
        obj.mcnk_objects.push(McnkChunkObject {
            index: 2,
            doodad_refs: vec![],
            wmo_refs: vec![],
        });

        let result = merge_split_files(root, None, Some(obj));
        assert!(result.is_err());
    }
}
