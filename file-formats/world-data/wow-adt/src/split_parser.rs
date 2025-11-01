//! Parser for Cataclysm+ split ADT files.
//!
//! Starting with Cataclysm (4.x), ADT files were split into specialized components:
//! - `_tex0.adt` - Texture definitions and layer data
//! - `_tex1.adt` - Additional texture data
//! - `_obj0.adt` - M2 model and WMO placement data
//! - `_obj1.adt` - Additional object data
//! - `_lod.adt` - Level-of-detail rendering data
//!
//! ## Cataclysm Chunk Support (Phase 4c)
//!
//! MCNK subchunks remain nested within MCNK structures across split files:
//!
//! **Root ADT (.adt)**:
//! - MCLV - Vertex lighting (145 ARGB entries)
//! - MCDD - Doodad disable bitmap (WoD+)
//!
//! **Texture files (_tex.adt)**:
//! - MCMT - Terrain material IDs (4 bytes per MCLY layer)
//!
//! **Object files (_obj.adt)**:
//! - MCRD - Doodad references (u32 indices into MDDF)
//! - MCRW - WMO references (u32 indices into MODF)
//!
//! These subchunks are automatically handled by the MCNK parser when processing
//! split files. No additional routing logic is needed as they are discovered
//! through chunk parsing within MCNK structures.

use std::io::{Cursor, Read, Seek, SeekFrom};

use binrw::BinRead;

use crate::api::{LodAdt, McnkChunkObject, McnkChunkTexture, Obj0Adt, Tex0Adt};
use crate::chunk_discovery::ChunkDiscovery;
use crate::chunk_header::ChunkHeader;
use crate::chunk_id::ChunkId;
use crate::chunks::mcnk::{McrdChunk, McrwChunk};
use crate::chunks::{
    McalChunk, MclyChunk, MddfChunk, MmdxChunk, MmidChunk, ModfChunk, MtexChunk, MtxpChunk,
    MwidChunk, MwmoChunk,
};
use crate::error::Result;
use crate::version::AdtVersion;

/// Parse Cataclysm+ texture file (_tex0.adt or _tex1.adt).
///
/// Texture files contain:
/// - MTEX - Texture filenames
/// - MCLY - Texture layer definitions (per-chunk)
/// - MCAL - Alpha maps for texture blending
/// - MTXP - Texture parameters (MoP+)
///
/// # Arguments
///
/// * `reader` - Seekable input stream
/// * `discovery` - Chunk discovery results
/// * `version` - Detected ADT version
///
/// # Returns
///
/// Tuple of (Tex0Adt, warnings)
pub fn parse_tex_adt<R: Read + Seek>(
    reader: &mut R,
    discovery: &ChunkDiscovery,
    version: AdtVersion,
) -> Result<(Tex0Adt, Vec<String>)> {
    let warnings = Vec::new();

    // Parse MTEX chunk (texture filenames)
    let textures = if let Some(chunks) = discovery.get_chunks(ChunkId::MTEX) {
        if let Some(chunk_info) = chunks.first() {
            reader.seek(SeekFrom::Start(chunk_info.offset + 8))?;
            // Read chunk data into buffer to prevent reading into next chunk
            let mut chunk_data = vec![0u8; chunk_info.size as usize];
            reader.read_exact(&mut chunk_data)?;
            let mut cursor = std::io::Cursor::new(chunk_data);
            let mtex = MtexChunk::read_le(&mut cursor)?;
            mtex.filenames
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Parse MTXP chunk (texture parameters, MoP+)
    let texture_params = if matches!(version, AdtVersion::MoP) {
        if let Some(chunks) = discovery.get_chunks(ChunkId::MTXP) {
            if let Some(chunk_info) = chunks.first() {
                reader.seek(SeekFrom::Start(chunk_info.offset + 8))?;
                Some(MtxpChunk::read_le(reader)?)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Parse MCNK chunks (texture layers and alpha maps)
    let mcnk_textures = parse_mcnk_texture_chunks(reader, discovery)?;

    let tex = Tex0Adt {
        version,
        textures,
        texture_params,
        mcnk_textures,
    };

    Ok((tex, warnings))
}

/// Parse Cataclysm+ object file (_obj0.adt or _obj1.adt).
///
/// Object files contain:
/// - MMDX - M2 model filenames
/// - MMID - M2 model filename offsets
/// - MWMO - WMO filenames
/// - MWID - WMO filename offsets
/// - MDDF - M2 model placements
/// - MODF - WMO placements
///
/// # Arguments
///
/// * `reader` - Seekable input stream
/// * `discovery` - Chunk discovery results
/// * `version` - Detected ADT version
///
/// # Returns
///
/// Tuple of (Obj0Adt, warnings)
pub fn parse_obj_adt<R: Read + Seek>(
    reader: &mut R,
    discovery: &ChunkDiscovery,
    version: AdtVersion,
) -> Result<(Obj0Adt, Vec<String>)> {
    let warnings = Vec::new();

    // Parse MMDX chunk (M2 model filenames)
    let models = if let Some(chunks) = discovery.get_chunks(ChunkId::MMDX) {
        if let Some(chunk_info) = chunks.first() {
            reader.seek(SeekFrom::Start(chunk_info.offset + 8))?;
            // Read chunk data into buffer to prevent reading into next chunk
            let mut chunk_data = vec![0u8; chunk_info.size as usize];
            reader.read_exact(&mut chunk_data)?;
            let mut cursor = Cursor::new(chunk_data);
            let mmdx = MmdxChunk::read_le(&mut cursor)?;
            mmdx.filenames
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Parse MMID chunk (M2 model indices)
    let model_indices = if let Some(chunks) = discovery.get_chunks(ChunkId::MMID) {
        if let Some(chunk_info) = chunks.first() {
            reader.seek(SeekFrom::Start(chunk_info.offset + 8))?;
            // Read chunk data into buffer to prevent until_eof from reading past chunk boundary
            let mut chunk_data = vec![0u8; chunk_info.size as usize];
            reader.read_exact(&mut chunk_data)?;
            let mut cursor = Cursor::new(chunk_data);
            let mmid = MmidChunk::read_le(&mut cursor)?;
            mmid.offsets
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Parse MWMO chunk (WMO filenames)
    let wmos = if let Some(chunks) = discovery.get_chunks(ChunkId::MWMO) {
        if let Some(chunk_info) = chunks.first() {
            reader.seek(SeekFrom::Start(chunk_info.offset + 8))?;
            // Read chunk data into buffer to prevent reading into next chunk
            let mut chunk_data = vec![0u8; chunk_info.size as usize];
            reader.read_exact(&mut chunk_data)?;
            let mut cursor = Cursor::new(chunk_data);
            let mwmo = MwmoChunk::read_le(&mut cursor)?;
            mwmo.filenames
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Parse MWID chunk (WMO indices)
    let wmo_indices = if let Some(chunks) = discovery.get_chunks(ChunkId::MWID) {
        if let Some(chunk_info) = chunks.first() {
            reader.seek(SeekFrom::Start(chunk_info.offset + 8))?;
            // Read chunk data into buffer to prevent until_eof from reading past chunk boundary
            let mut chunk_data = vec![0u8; chunk_info.size as usize];
            reader.read_exact(&mut chunk_data)?;
            let mut cursor = Cursor::new(chunk_data);
            let mwid = MwidChunk::read_le(&mut cursor)?;
            mwid.offsets
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Parse MDDF chunk (M2 model placements)
    let doodad_placements = if let Some(chunks) = discovery.get_chunks(ChunkId::MDDF) {
        if let Some(chunk_info) = chunks.first() {
            reader.seek(SeekFrom::Start(chunk_info.offset + 8))?;
            // Read chunk data into buffer to prevent until_eof from reading past chunk boundary
            let mut chunk_data = vec![0u8; chunk_info.size as usize];
            reader.read_exact(&mut chunk_data)?;
            let mut cursor = Cursor::new(chunk_data);
            let mddf = MddfChunk::read_le(&mut cursor)?;
            mddf.placements
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Parse MODF chunk (WMO placements)
    let wmo_placements = if let Some(chunks) = discovery.get_chunks(ChunkId::MODF) {
        if let Some(chunk_info) = chunks.first() {
            reader.seek(SeekFrom::Start(chunk_info.offset + 8))?;
            // Read chunk data into buffer to prevent until_eof from reading past chunk boundary
            let mut chunk_data = vec![0u8; chunk_info.size as usize];
            reader.read_exact(&mut chunk_data)?;
            let mut cursor = Cursor::new(chunk_data);
            let modf = ModfChunk::read_le(&mut cursor)?;
            modf.placements
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Parse MCNK chunks (object references)
    let mcnk_objects = parse_mcnk_object_chunks(reader, discovery)?;

    let obj = Obj0Adt {
        version,
        models,
        model_indices,
        wmos,
        wmo_indices,
        doodad_placements,
        wmo_placements,
        mcnk_objects,
    };

    Ok((obj, warnings))
}

/// Parse Cataclysm+ LOD file (_lod.adt).
///
/// LOD files contain simplified geometry for distant terrain rendering.
/// This format is minimally documented and this is a stub implementation.
///
/// # Arguments
///
/// * `reader` - Seekable input stream
/// * `discovery` - Chunk discovery results
/// * `version` - Detected ADT version
///
/// # Returns
///
/// Tuple of (LodAdt, warnings)
pub fn parse_lod_adt<R: Read + Seek>(
    _reader: &mut R,
    _discovery: &ChunkDiscovery,
    version: AdtVersion,
) -> Result<(LodAdt, Vec<String>)> {
    let warnings = vec!["LOD file format not yet fully implemented".to_string()];

    let lod = LodAdt { version };

    Ok((lod, warnings))
}

/// Parse MCNK chunks from texture file to extract texture layers and alpha maps.
///
/// In split texture files, MCNK chunks are simplified containers that only hold
/// MCLY (texture layers) and MCAL (alpha maps) subchunks. They do not have the
/// full MCNK header structure found in root files.
///
/// # Format
///
/// Each MCNK chunk contains:
/// - 8-byte chunk header (magic + size)
/// - MCLY subchunk (texture layer definitions)
/// - MCAL subchunk (alpha maps for blending)
///
/// # Arguments
///
/// * `reader` - Seekable input stream
/// * `discovery` - Chunk discovery results
///
/// # Returns
///
/// Vector of McnkChunkTexture containers (up to 256 chunks)
fn parse_mcnk_texture_chunks<R: Read + Seek>(
    reader: &mut R,
    discovery: &ChunkDiscovery,
) -> Result<Vec<McnkChunkTexture>> {
    let mut mcnk_textures = Vec::new();

    // Get MCNK chunk locations
    let mcnk_chunks = match discovery.get_chunks(ChunkId::MCNK) {
        Some(chunks) => chunks,
        None => return Ok(mcnk_textures), // No MCNK chunks in this file
    };

    // Parse each MCNK chunk
    for (index, chunk_info) in mcnk_chunks.iter().enumerate() {
        // Seek to MCNK chunk data (skip 8-byte header)
        reader.seek(SeekFrom::Start(chunk_info.offset + 8))?;

        let mcnk_end = chunk_info.offset + 8 + u64::from(chunk_info.size);
        let mut current_pos = chunk_info.offset + 8;

        let mut layers = None;
        let mut alpha_maps = None;

        // Parse subchunks within MCNK
        while current_pos < mcnk_end {
            reader.seek(SeekFrom::Start(current_pos))?;

            // Try to read subchunk header
            let subchunk_header = match ChunkHeader::read_le(reader) {
                Ok(header) => header,
                Err(_) => break, // End of MCNK or invalid data
            };

            current_pos += 8; // Move past header

            // Parse subchunk based on ID
            match subchunk_header.id {
                ChunkId::MCLY => {
                    // Read chunk data into buffer to prevent reading into next chunk
                    let mut chunk_data = vec![0u8; subchunk_header.size as usize];
                    reader.seek(SeekFrom::Start(current_pos))?;
                    reader.read_exact(&mut chunk_data)?;
                    let mut cursor = std::io::Cursor::new(chunk_data);
                    layers = Some(MclyChunk::read_le(&mut cursor)?);
                }
                ChunkId::MCAL => {
                    // Read chunk data into buffer to prevent reading into next chunk
                    let mut chunk_data = vec![0u8; subchunk_header.size as usize];
                    reader.seek(SeekFrom::Start(current_pos))?;
                    reader.read_exact(&mut chunk_data)?;
                    let mut cursor = std::io::Cursor::new(chunk_data);
                    alpha_maps = Some(McalChunk::read_le(&mut cursor)?);
                }
                _ => {
                    // Skip unknown subchunk
                }
            }

            current_pos += u64::from(subchunk_header.size);
        }

        mcnk_textures.push(McnkChunkTexture {
            index,
            layers,
            alpha_maps,
        });
    }

    Ok(mcnk_textures)
}

/// Parse MCNK chunks from object file to extract object references.
///
/// In split object files, MCNK chunks are simplified containers that only hold
/// MCRD (doodad references) and MCRW (WMO references) subchunks. They do not
/// have the full MCNK header structure found in root files.
///
/// # Format
///
/// Each MCNK chunk contains:
/// - 8-byte chunk header (magic + size)
/// - MCRD subchunk (M2 doodad reference indices)
/// - MCRW subchunk (WMO reference indices)
///
/// # Arguments
///
/// * `reader` - Seekable input stream
/// * `discovery` - Chunk discovery results
///
/// # Returns
///
/// Vector of McnkChunkObject containers (up to 256 chunks)
fn parse_mcnk_object_chunks<R: Read + Seek>(
    reader: &mut R,
    discovery: &ChunkDiscovery,
) -> Result<Vec<McnkChunkObject>> {
    let mut mcnk_objects = Vec::new();

    // Get MCNK chunk locations
    let mcnk_chunks = match discovery.get_chunks(ChunkId::MCNK) {
        Some(chunks) => chunks,
        None => return Ok(mcnk_objects), // No MCNK chunks in this file
    };

    // Parse each MCNK chunk
    for (index, chunk_info) in mcnk_chunks.iter().enumerate() {
        // Seek to MCNK chunk data (skip 8-byte header)
        reader.seek(SeekFrom::Start(chunk_info.offset + 8))?;

        let mcnk_end = chunk_info.offset + 8 + u64::from(chunk_info.size);
        let mut current_pos = chunk_info.offset + 8;

        let mut doodad_refs = Vec::new();
        let mut wmo_refs = Vec::new();

        // Parse subchunks within MCNK
        while current_pos < mcnk_end {
            reader.seek(SeekFrom::Start(current_pos))?;

            // Try to read subchunk header
            let subchunk_header = match ChunkHeader::read_le(reader) {
                Ok(header) => header,
                Err(_) => break, // End of MCNK or invalid data
            };

            current_pos += 8; // Move past header

            // Parse subchunk based on ID
            match subchunk_header.id {
                ChunkId::MCRD => {
                    reader.seek(SeekFrom::Start(current_pos))?;
                    let mcrd = McrdChunk::read_le(reader)?;
                    doodad_refs = mcrd.doodad_refs;
                }
                ChunkId::MCRW => {
                    reader.seek(SeekFrom::Start(current_pos))?;
                    let mcrw = McrwChunk::read_le(reader)?;
                    wmo_refs = mcrw.wmo_refs;
                }
                _ => {
                    // Skip unknown subchunk
                }
            }

            current_pos += u64::from(subchunk_header.size);
        }

        mcnk_objects.push(McnkChunkObject {
            index,
            doodad_refs,
            wmo_refs,
        });
    }

    Ok(mcnk_objects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_discovery::discover_chunks;
    use std::io::Cursor;

    /// Create minimal tex0 ADT data for testing.
    fn create_minimal_tex_adt() -> Vec<u8> {
        let mut data = Vec::new();

        // MVER chunk
        data.extend_from_slice(&ChunkId::MVER.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&18u32.to_le_bytes());

        // MTEX chunk (one texture)
        data.extend_from_slice(&ChunkId::MTEX.0);
        let texture = b"Tileset\\Terrain.blp\0";
        data.extend_from_slice(&(texture.len() as u32).to_le_bytes());
        data.extend_from_slice(texture);

        data
    }

    /// Create minimal obj0 ADT data for testing.
    fn create_minimal_obj_adt() -> Vec<u8> {
        let mut data = Vec::new();

        // MVER chunk
        data.extend_from_slice(&ChunkId::MVER.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&18u32.to_le_bytes());

        // MMDX chunk (one model)
        data.extend_from_slice(&ChunkId::MMDX.0);
        let model = b"World\\Doodad\\Model.m2\0";
        data.extend_from_slice(&(model.len() as u32).to_le_bytes());
        data.extend_from_slice(model);

        // MMID chunk (one index)
        data.extend_from_slice(&ChunkId::MMID.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());

        // MDDF chunk (empty - marks as object file for detection)
        data.extend_from_slice(&ChunkId::MDDF.0);
        data.extend_from_slice(&0u32.to_le_bytes());

        data
    }

    #[test]
    fn test_parse_tex_adt() {
        let data = create_minimal_tex_adt();
        let mut cursor = Cursor::new(data);

        let discovery = discover_chunks(&mut cursor).unwrap();
        let version = AdtVersion::from_discovery(&discovery);

        cursor.set_position(0);
        let result = parse_tex_adt(&mut cursor, &discovery, version);

        assert!(result.is_ok());
        let (tex, warnings) = result.unwrap();

        assert_eq!(tex.version, AdtVersion::VanillaEarly);
        assert_eq!(tex.textures.len(), 1);
        assert_eq!(tex.textures[0], "Tileset\\Terrain.blp");
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_parse_obj_adt() {
        let data = create_minimal_obj_adt();
        let mut cursor = Cursor::new(data);

        let discovery = discover_chunks(&mut cursor).unwrap();
        let version = AdtVersion::from_discovery(&discovery);

        cursor.set_position(0);
        let result = parse_obj_adt(&mut cursor, &discovery, version);

        assert!(result.is_ok());
        let (obj, warnings) = result.unwrap();

        assert_eq!(obj.version, AdtVersion::VanillaEarly);
        // MMDX chunk contains one model
        assert_eq!(obj.models.len(), 1);
        assert_eq!(obj.models[0], "World\\Doodad\\Model.m2");
        assert_eq!(obj.model_indices.len(), 1);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_parse_lod_adt() {
        let data = Vec::new();
        let mut cursor = Cursor::new(data);

        let discovery = crate::chunk_discovery::ChunkDiscovery::new(0);
        let version = AdtVersion::Cataclysm;

        let result = parse_lod_adt(&mut cursor, &discovery, version);

        assert!(result.is_ok());
        let (lod, warnings) = result.unwrap();

        assert_eq!(lod.version, AdtVersion::Cataclysm);
        assert!(!warnings.is_empty());
    }

    /// Create object ADT with MCNK chunks for testing.
    fn create_obj_adt_with_mcnk() -> Vec<u8> {
        let mut data = Vec::new();

        // MVER chunk
        data.extend_from_slice(&ChunkId::MVER.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&18u32.to_le_bytes());

        // MMDX chunk (two models)
        data.extend_from_slice(&ChunkId::MMDX.0);
        let model1 = b"World\\Doodad\\Tree.m2\0";
        let model2 = b"World\\Doodad\\Rock.m2\0";
        let mmdx_size = model1.len() + model2.len();
        data.extend_from_slice(&(mmdx_size as u32).to_le_bytes());
        data.extend_from_slice(model1);
        data.extend_from_slice(model2);

        // MMID chunk (two indices)
        data.extend_from_slice(&ChunkId::MMID.0);
        data.extend_from_slice(&8u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&(model1.len() as u32).to_le_bytes());

        // MDDF chunk (empty - just marks this as object file for detection)
        data.extend_from_slice(&ChunkId::MDDF.0);
        data.extend_from_slice(&0u32.to_le_bytes());

        // MCNK chunk with MCRD subchunk (doodad references)
        data.extend_from_slice(&ChunkId::MCNK.0);
        let mcnk_start = data.len();
        data.extend_from_slice(&0u32.to_le_bytes()); // Placeholder for size

        // MCRD subchunk (2 doodad references)
        data.extend_from_slice(&ChunkId::MCRD.0);
        data.extend_from_slice(&8u32.to_le_bytes()); // Size: 2 × 4 bytes
        data.extend_from_slice(&0u32.to_le_bytes()); // First doodad ref
        data.extend_from_slice(&1u32.to_le_bytes()); // Second doodad ref

        // Calculate and update MCNK size
        let mcnk_size = data.len() - mcnk_start - 4;
        let size_bytes = (mcnk_size as u32).to_le_bytes();
        data[mcnk_start..mcnk_start + 4].copy_from_slice(&size_bytes);

        data
    }

    #[test]
    fn test_parse_obj_adt_with_mcnk() {
        let data = create_obj_adt_with_mcnk();
        let mut cursor = Cursor::new(data);

        let discovery = discover_chunks(&mut cursor).unwrap();
        let version = AdtVersion::from_discovery(&discovery);

        cursor.set_position(0);
        let result = parse_obj_adt(&mut cursor, &discovery, version);

        assert!(result.is_ok());
        let (obj, warnings) = result.unwrap();

        assert_eq!(obj.models.len(), 2);
        assert_eq!(obj.model_indices.len(), 2);
        assert_eq!(obj.mcnk_objects.len(), 1);
        assert_eq!(obj.mcnk_objects[0].index, 0);
        assert_eq!(obj.mcnk_objects[0].doodad_refs.len(), 2);
        assert_eq!(obj.mcnk_objects[0].doodad_refs[0], 0);
        assert_eq!(obj.mcnk_objects[0].doodad_refs[1], 1);
        assert!(obj.mcnk_objects[0].wmo_refs.is_empty());
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_parse_obj_adt_file_type_detection() {
        let data = create_minimal_obj_adt();
        let mut cursor = Cursor::new(data);

        let discovery = discover_chunks(&mut cursor).unwrap();

        // Should detect as object file
        assert_eq!(
            crate::file_type::AdtFileType::from_discovery(&discovery),
            crate::file_type::AdtFileType::Obj0
        );
    }

    #[test]
    fn test_parse_obj_adt_empty_mcnk() {
        let mut data = Vec::new();

        // MVER chunk
        data.extend_from_slice(&ChunkId::MVER.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&18u32.to_le_bytes());

        // MMDX chunk
        data.extend_from_slice(&ChunkId::MMDX.0);
        let model = b"World\\Doodad\\Tree.m2\0";
        data.extend_from_slice(&(model.len() as u32).to_le_bytes());
        data.extend_from_slice(model);

        // MMID chunk
        data.extend_from_slice(&ChunkId::MMID.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());

        // MDDF chunk (empty - marks as object file for detection)
        data.extend_from_slice(&ChunkId::MDDF.0);
        data.extend_from_slice(&0u32.to_le_bytes());

        // MCNK chunk with no subchunks
        data.extend_from_slice(&ChunkId::MCNK.0);
        data.extend_from_slice(&0u32.to_le_bytes()); // Empty MCNK

        let mut cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut cursor).unwrap();
        let version = AdtVersion::from_discovery(&discovery);

        cursor.set_position(0);
        let result = parse_obj_adt(&mut cursor, &discovery, version);

        assert!(result.is_ok());
        let (obj, _) = result.unwrap();

        assert_eq!(obj.mcnk_objects.len(), 1);
        assert_eq!(obj.mcnk_objects[0].index, 0);
        assert!(obj.mcnk_objects[0].doodad_refs.is_empty());
        assert!(obj.mcnk_objects[0].wmo_refs.is_empty());
    }

    /// Create texture ADT with MCNK chunks for testing.
    fn create_tex_adt_with_mcnk() -> Vec<u8> {
        let mut data = Vec::new();

        // MVER chunk
        data.extend_from_slice(&ChunkId::MVER.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&18u32.to_le_bytes());

        // MTEX chunk (two textures)
        data.extend_from_slice(&ChunkId::MTEX.0);
        let texture1 = b"Tileset\\Ground.blp\0";
        let texture2 = b"Tileset\\Grass.blp\0";
        let mtex_size = texture1.len() + texture2.len();
        data.extend_from_slice(&(mtex_size as u32).to_le_bytes());
        data.extend_from_slice(texture1);
        data.extend_from_slice(texture2);

        // MCNK chunk with MCLY subchunk
        data.extend_from_slice(&ChunkId::MCNK.0);
        let mcnk_start = data.len();
        data.extend_from_slice(&0u32.to_le_bytes()); // Placeholder for size

        // MCLY subchunk (2 layers)
        data.extend_from_slice(&ChunkId::MCLY.0);
        data.extend_from_slice(&32u32.to_le_bytes()); // Size: 2 layers × 16 bytes
        // Layer 1
        data.extend_from_slice(&0u32.to_le_bytes()); // texture_id
        data.extend_from_slice(&0u32.to_le_bytes()); // flags
        data.extend_from_slice(&0u32.to_le_bytes()); // offset_in_mcal
        data.extend_from_slice(&0u32.to_le_bytes()); // effect_id
        // Layer 2
        data.extend_from_slice(&1u32.to_le_bytes()); // texture_id
        data.extend_from_slice(&0u32.to_le_bytes()); // flags
        data.extend_from_slice(&2048u32.to_le_bytes()); // offset_in_mcal
        data.extend_from_slice(&0u32.to_le_bytes()); // effect_id

        // Calculate and update MCNK size
        let mcnk_size = data.len() - mcnk_start - 4;
        let size_bytes = (mcnk_size as u32).to_le_bytes();
        data[mcnk_start..mcnk_start + 4].copy_from_slice(&size_bytes);

        data
    }

    #[test]
    fn test_parse_tex_adt_with_mcnk() {
        let data = create_tex_adt_with_mcnk();
        let mut cursor = Cursor::new(data);

        let discovery = discover_chunks(&mut cursor).unwrap();
        let version = AdtVersion::from_discovery(&discovery);

        cursor.set_position(0);
        let result = parse_tex_adt(&mut cursor, &discovery, version);

        assert!(result.is_ok());
        let (tex, warnings) = result.unwrap();

        assert_eq!(tex.textures.len(), 2);
        assert_eq!(tex.mcnk_textures.len(), 1);
        assert_eq!(tex.mcnk_textures[0].index, 0);
        assert!(tex.mcnk_textures[0].layers.is_some());

        let layers = tex.mcnk_textures[0].layers.as_ref().unwrap();
        assert_eq!(layers.layers.len(), 2);
        assert_eq!(layers.layers[0].texture_id, 0);
        assert_eq!(layers.layers[1].texture_id, 1);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_parse_tex_adt_file_type_detection() {
        let data = create_minimal_tex_adt();
        let mut cursor = Cursor::new(data);

        let discovery = discover_chunks(&mut cursor).unwrap();

        // Should detect as texture file
        assert_eq!(
            crate::file_type::AdtFileType::from_discovery(&discovery),
            crate::file_type::AdtFileType::Tex0
        );
    }

    #[test]
    fn test_parse_tex_adt_empty_mcnk() {
        let mut data = Vec::new();

        // MVER chunk
        data.extend_from_slice(&ChunkId::MVER.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&18u32.to_le_bytes());

        // MTEX chunk
        data.extend_from_slice(&ChunkId::MTEX.0);
        let texture = b"Tileset\\Terrain.blp\0";
        data.extend_from_slice(&(texture.len() as u32).to_le_bytes());
        data.extend_from_slice(texture);

        // MCNK chunk with no subchunks
        data.extend_from_slice(&ChunkId::MCNK.0);
        data.extend_from_slice(&0u32.to_le_bytes()); // Empty MCNK

        let mut cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut cursor).unwrap();
        let version = AdtVersion::from_discovery(&discovery);

        cursor.set_position(0);
        let result = parse_tex_adt(&mut cursor, &discovery, version);

        assert!(result.is_ok());
        let (tex, _) = result.unwrap();

        assert_eq!(tex.mcnk_textures.len(), 1);
        assert_eq!(tex.mcnk_textures[0].index, 0);
        assert!(tex.mcnk_textures[0].layers.is_none());
        assert!(tex.mcnk_textures[0].alpha_maps.is_none());
    }
}
