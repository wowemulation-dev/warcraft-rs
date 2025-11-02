//! Parser for root ADT files (main terrain files).
//!
//! Implements parsing logic for root ADT files which contain:
//! - Terrain chunks (MCNK with heightmaps, normals, layers)
//! - Texture definitions (MTEX)
//! - Model and WMO filenames and placements
//! - Version-specific features (water, flight bounds, etc.)
//!
//! This parser handles all ADT versions from Vanilla through MoP.

use std::io::{Read, Seek, SeekFrom};

use binrw::BinRead;

use crate::api::RootAdt;
use crate::chunk_discovery::ChunkDiscovery;
use crate::chunk_id::ChunkId;
use crate::chunks::mh2o::{Mh2oAttributes, Mh2oChunk, Mh2oEntry, Mh2oHeader, Mh2oInstance};
use crate::chunks::{
    MampChunk, MbbbChunk, MbmhChunk, MbmiChunk, MbnvChunk, McinChunk, McnkChunk, MddfChunk,
    MfboChunk, MhdrChunk, MmdxChunk, MmidChunk, ModfChunk, MtexChunk, MtxfChunk, MtxpChunk,
    MwidChunk, MwmoChunk,
};
use crate::error::{AdtError, Result};
use crate::version::AdtVersion;

/// Parse root ADT file from reader using discovery results.
///
/// This function implements the second pass of two-pass parsing, extracting
/// type-safe chunk data based on discovery phase results.
///
/// # Arguments
///
/// * `reader` - Seekable input stream positioned at file start
/// * `discovery` - Chunk discovery results from phase 1
/// * `version` - Detected ADT version
///
/// # Returns
///
/// Tuple of (RootAdt, warnings) where warnings contains non-critical issues
///
/// # Errors
///
/// Returns `AdtError` for:
/// - Missing required chunks (MHDR, MCIN, MTEX, MCNK)
/// - Invalid chunk sizes or offsets
/// - Corrupted chunk data
/// - Binary parsing failures
pub fn parse_root_adt<R: Read + Seek>(
    reader: &mut R,
    discovery: &ChunkDiscovery,
    version: AdtVersion,
) -> Result<(RootAdt, Vec<String>)> {
    let warnings = Vec::new();

    // Detect if this is a split file (Cataclysm+ root without MCIN/MTEX)
    let is_split_root = !discovery.has_chunk(ChunkId::MCIN) && !discovery.has_chunk(ChunkId::MTEX);

    // Verify required chunks are present
    if !discovery.has_chunk(ChunkId::MHDR) {
        return Err(AdtError::MissingRequiredChunk(ChunkId::MHDR));
    }
    if !discovery.has_chunk(ChunkId::MCNK) {
        return Err(AdtError::MissingRequiredChunk(ChunkId::MCNK));
    }

    // MCIN and MTEX are required for monolithic files but absent in split root files
    if !is_split_root {
        if !discovery.has_chunk(ChunkId::MCIN) {
            return Err(AdtError::MissingRequiredChunk(ChunkId::MCIN));
        }
        if !discovery.has_chunk(ChunkId::MTEX) {
            return Err(AdtError::MissingRequiredChunk(ChunkId::MTEX));
        }
    }

    // Parse MHDR chunk
    let mhdr = parse_simple_chunk::<MhdrChunk, _>(reader, discovery, ChunkId::MHDR)?;

    // Parse MCIN chunk (or create empty for split files)
    let mcin = if is_split_root {
        // Split root files don't have MCIN - create empty structure
        McinChunk { entries: vec![] }
    } else {
        parse_simple_chunk::<McinChunk, _>(reader, discovery, ChunkId::MCIN)?
    };

    // Parse MTEX chunk (texture filenames)
    let textures = if let Some(chunks) = discovery.get_chunks(ChunkId::MTEX) {
        if let Some(chunk_info) = chunks.first() {
            reader.seek(SeekFrom::Start(chunk_info.offset + 8))?; // Skip header
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

    // Parse MMDX chunk (M2 model filenames)
    let models = if let Some(chunks) = discovery.get_chunks(ChunkId::MMDX) {
        if let Some(chunk_info) = chunks.first() {
            reader.seek(SeekFrom::Start(chunk_info.offset + 8))?;
            // Read chunk data into buffer to prevent reading into next chunk
            let mut chunk_data = vec![0u8; chunk_info.size as usize];
            reader.read_exact(&mut chunk_data)?;
            let mut cursor = std::io::Cursor::new(chunk_data);
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
            let mut cursor = std::io::Cursor::new(chunk_data);
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
            let mut cursor = std::io::Cursor::new(chunk_data);
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
            let mut cursor = std::io::Cursor::new(chunk_data);
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
            let mut cursor = std::io::Cursor::new(chunk_data);
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
            let mut cursor = std::io::Cursor::new(chunk_data);
            let modf = ModfChunk::read_le(&mut cursor)?;
            modf.placements
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Parse MCNK chunks (terrain tiles)
    let mcnk_chunks = parse_mcnk_chunks(reader, discovery)?;

    // Parse version-specific chunks

    // MFBO - Flight boundaries (TBC+)
    let flight_bounds = if matches!(
        version,
        AdtVersion::TBC | AdtVersion::WotLK | AdtVersion::Cataclysm | AdtVersion::MoP
    ) {
        if let Some(chunks) = discovery.get_chunks(ChunkId::MFBO) {
            if let Some(chunk_info) = chunks.first() {
                reader.seek(SeekFrom::Start(chunk_info.offset + 8))?;
                Some(MfboChunk::read_le(reader)?)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // MH2O - Advanced water (WotLK+)
    let water_data = if matches!(
        version,
        AdtVersion::WotLK | AdtVersion::Cataclysm | AdtVersion::MoP
    ) {
        if let Some(chunks) = discovery.get_chunks(ChunkId::MH2O) {
            if let Some(chunk_info) = chunks.first() {
                parse_mh2o_chunk(reader, chunk_info.offset, chunk_info.size)?
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // MTXF - Texture flags (WotLK 3.x+)
    let texture_flags = if matches!(
        version,
        AdtVersion::WotLK | AdtVersion::Cataclysm | AdtVersion::MoP
    ) {
        if let Some(chunks) = discovery.get_chunks(ChunkId::MTXF) {
            if let Some(chunk_info) = chunks.first() {
                reader.seek(SeekFrom::Start(chunk_info.offset + 8))?;
                Some(MtxfChunk::read_le(reader)?)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // MAMP - Texture amplifier (Cataclysm+)
    let texture_amplifier = if matches!(version, AdtVersion::Cataclysm | AdtVersion::MoP) {
        if let Some(chunks) = discovery.get_chunks(ChunkId::MAMP) {
            if let Some(chunk_info) = chunks.first() {
                reader.seek(SeekFrom::Start(chunk_info.offset + 8))?;
                Some(MampChunk::read_le(reader)?)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // MTXP - Texture parameters (MoP+)
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

    // MBMH - Blend mesh headers (MoP 5.x+)
    let blend_mesh_headers = if matches!(version, AdtVersion::MoP) {
        if let Some(chunks) = discovery.get_chunks(ChunkId::MBMH) {
            if let Some(chunk_info) = chunks.first() {
                reader.seek(SeekFrom::Start(chunk_info.offset + 8))?;
                Some(MbmhChunk::read_le(reader)?)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // MBBB - Blend mesh bounding boxes (MoP 5.x+)
    let blend_mesh_bounds = if matches!(version, AdtVersion::MoP) {
        if let Some(chunks) = discovery.get_chunks(ChunkId::MBBB) {
            if let Some(chunk_info) = chunks.first() {
                reader.seek(SeekFrom::Start(chunk_info.offset + 8))?;
                Some(MbbbChunk::read_le(reader)?)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // MBNV - Blend mesh vertices (MoP 5.x+)
    let blend_mesh_vertices = if matches!(version, AdtVersion::MoP) {
        if let Some(chunks) = discovery.get_chunks(ChunkId::MBNV) {
            if let Some(chunk_info) = chunks.first() {
                reader.seek(SeekFrom::Start(chunk_info.offset + 8))?;
                Some(MbnvChunk::read_le(reader)?)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // MBMI - Blend mesh indices (MoP 5.x+)
    let blend_mesh_indices = if matches!(version, AdtVersion::MoP) {
        if let Some(chunks) = discovery.get_chunks(ChunkId::MBMI) {
            if let Some(chunk_info) = chunks.first() {
                reader.seek(SeekFrom::Start(chunk_info.offset + 8))?;
                Some(MbmiChunk::read_le(reader)?)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let root = RootAdt {
        version,
        mhdr,
        mcin,
        textures,
        models,
        model_indices,
        wmos,
        wmo_indices,
        doodad_placements,
        wmo_placements,
        mcnk_chunks,
        flight_bounds,
        water_data,
        texture_flags,
        texture_amplifier,
        texture_params,
        blend_mesh_headers,
        blend_mesh_bounds,
        blend_mesh_vertices,
        blend_mesh_indices,
    };

    Ok((root, warnings))
}

/// Parse MH2O chunk with full 256-header structure.
///
/// MH2O contains 256 headers (one per MCNK chunk), each with offsets to
/// instances and attributes. This function parses all headers and their
/// associated data.
///
/// # Arguments
///
/// * `reader` - Seekable input stream
/// * `chunk_offset` - Offset to MH2O chunk header
/// * `chunk_size` - Size of MH2O chunk data
///
/// # Returns
///
/// Parsed Mh2oChunk or None if no water data exists
fn parse_mh2o_chunk<R: Read + Seek>(
    reader: &mut R,
    chunk_offset: u64,
    chunk_size: u32,
) -> Result<Option<Mh2oChunk>> {
    // Seek to chunk data (skip 8-byte header)
    let data_start = chunk_offset + 8;
    reader.seek(SeekFrom::Start(data_start))?;

    // Parse all 256 headers (12 bytes each = 3072 bytes total)
    let mut headers = Vec::with_capacity(Mh2oChunk::ENTRY_COUNT);
    for _ in 0..Mh2oChunk::ENTRY_COUNT {
        let header = Mh2oHeader::read_le(reader)?;
        headers.push(header);
    }

    // Parse instances and attributes for each header
    let mut entries = Vec::with_capacity(Mh2oChunk::ENTRY_COUNT);

    for header in headers {
        let mut instances = Vec::new();
        let mut attributes = None;

        // Parse instances if present
        if header.has_liquid() {
            // Validate offset is within chunk bounds
            if header.offset_instances < chunk_size {
                reader.seek(SeekFrom::Start(
                    data_start + u64::from(header.offset_instances),
                ))?;

                // Parse each instance
                for _ in 0..header.layer_count {
                    match Mh2oInstance::read_le(reader) {
                        Ok(instance) => instances.push(instance),
                        Err(_) => break, // Stop on parse error
                    }
                }
            }
        }

        // Parse attributes if present
        if header.has_attributes() && header.offset_attributes < chunk_size {
            reader.seek(SeekFrom::Start(
                data_start + u64::from(header.offset_attributes),
            ))?;
            if let Ok(attrs) = Mh2oAttributes::read_le(reader) {
                attributes = Some(attrs);
            }
        }

        // Parse vertex data and exists bitmaps for each instance
        let mut vertex_data = Vec::with_capacity(instances.len());
        let mut exists_bitmaps = Vec::with_capacity(instances.len());

        for instance in &instances {
            // Parse exists bitmap if present
            let exists_bitmap = if instance.offset_exists_bitmap != 0
                && u64::from(instance.offset_exists_bitmap) < u64::from(chunk_size)
            {
                reader.seek(SeekFrom::Start(
                    data_start + u64::from(instance.offset_exists_bitmap),
                ))?;
                u64::read_le(reader).ok()
            } else {
                None
            };
            exists_bitmaps.push(exists_bitmap);

            // Parse vertex data if present
            let vdata = if instance.offset_vertex_data != 0
                && u64::from(instance.offset_vertex_data) < u64::from(chunk_size)
            {
                reader.seek(SeekFrom::Start(
                    data_start + u64::from(instance.offset_vertex_data),
                ))?;

                use crate::chunks::mh2o::{
                    DepthOnlyVertex, HeightDepthVertex, HeightUvDepthVertex, HeightUvVertex,
                    VertexDataArray,
                };

                // Determine LVF format from instance (WotLK mode)
                let lvf_opt = instance.get_lvf_wotlk();

                // Calculate vertex count
                let vertex_count = instance.vertex_count();

                // Parse vertices based on format (if format is known)
                match lvf_opt {
                    Some(crate::chunks::mh2o::LiquidVertexFormat::HeightDepth) => {
                        let mut vertices = Vec::with_capacity(vertex_count);
                        for _ in 0..vertex_count {
                            match HeightDepthVertex::read_le(reader) {
                                Ok(v) => vertices.push(v),
                                Err(_) => break,
                            }
                        }
                        if vertices.len() == vertex_count {
                            Some(VertexDataArray::HeightDepth(vertices))
                        } else {
                            None
                        }
                    }
                    Some(crate::chunks::mh2o::LiquidVertexFormat::HeightUv) => {
                        let mut vertices = Vec::with_capacity(vertex_count);
                        for _ in 0..vertex_count {
                            match HeightUvVertex::read_le(reader) {
                                Ok(v) => vertices.push(v),
                                Err(_) => break,
                            }
                        }
                        if vertices.len() == vertex_count {
                            Some(VertexDataArray::HeightUv(vertices))
                        } else {
                            None
                        }
                    }
                    Some(crate::chunks::mh2o::LiquidVertexFormat::DepthOnly) => {
                        let mut vertices = Vec::with_capacity(vertex_count);
                        for _ in 0..vertex_count {
                            match DepthOnlyVertex::read_le(reader) {
                                Ok(v) => vertices.push(v),
                                Err(_) => break,
                            }
                        }
                        if vertices.len() == vertex_count {
                            Some(VertexDataArray::DepthOnly(vertices))
                        } else {
                            None
                        }
                    }
                    Some(crate::chunks::mh2o::LiquidVertexFormat::HeightUvDepth) => {
                        let mut vertices = Vec::with_capacity(vertex_count);
                        for _ in 0..vertex_count {
                            match HeightUvDepthVertex::read_le(reader) {
                                Ok(v) => vertices.push(v),
                                Err(_) => break,
                            }
                        }
                        if vertices.len() == vertex_count {
                            Some(VertexDataArray::HeightUvDepth(vertices))
                        } else {
                            None
                        }
                    }
                    None => None, // Unknown LVF format
                }
            } else {
                None
            };
            vertex_data.push(vdata);
        }

        entries.push(Mh2oEntry {
            header,
            instances,
            vertex_data,
            exists_bitmaps,
            attributes,
        });
    }

    let chunk = Mh2oChunk { entries };

    // Return None if no water data exists
    if chunk.has_any_liquid() {
        Ok(Some(chunk))
    } else {
        Ok(None)
    }
}

/// Parse MCNK chunks using two-level parsing.
///
/// MCNK chunks require special handling because they contain subchunks
/// with relative offsets. We use the `parse_with_offset` method to
/// handle the offset calculations properly.
fn parse_mcnk_chunks<R: Read + Seek>(
    reader: &mut R,
    discovery: &ChunkDiscovery,
) -> Result<Vec<McnkChunk>> {
    let mcnk_locations = discovery
        .get_chunks(ChunkId::MCNK)
        .ok_or(AdtError::MissingRequiredChunk(ChunkId::MCNK))?;

    let mut mcnk_chunks = Vec::with_capacity(mcnk_locations.len());

    for (index, chunk_info) in mcnk_locations.iter().enumerate() {
        // Seek to MCNK chunk (skip the 8-byte header to get to data)
        reader.seek(SeekFrom::Start(chunk_info.offset + 8))?;

        // Parse using the special two-level parser
        let mcnk = McnkChunk::parse_with_offset(reader, chunk_info.offset).map_err(|e| {
            log::error!(
                "Failed to parse MCNK chunk {} at offset {}: {:?}",
                index,
                chunk_info.offset,
                e
            );
            e
        })?;

        mcnk_chunks.push(mcnk);
    }

    log::debug!("Parsed {} MCNK chunks", mcnk_chunks.len());

    Ok(mcnk_chunks)
}

/// Parse a simple chunk by ID from discovery results.
///
/// Helper function to seek to chunk location and parse it using binrw.
/// Works for chunks with simple binrw derives.
fn parse_simple_chunk<T, R>(
    reader: &mut R,
    discovery: &ChunkDiscovery,
    chunk_id: ChunkId,
) -> Result<T>
where
    T: for<'a> BinRead<Args<'a> = ()>,
    R: Read + Seek,
{
    let chunks = discovery
        .get_chunks(chunk_id)
        .ok_or(AdtError::MissingRequiredChunk(chunk_id))?;

    let chunk_info = chunks
        .first()
        .ok_or(AdtError::MissingRequiredChunk(chunk_id))?;

    // Seek to chunk data (skip 8-byte header)
    reader.seek(SeekFrom::Start(chunk_info.offset + 8))?;

    // Parse chunk using binrw
    T::read_le(reader).map_err(|e| AdtError::ChunkParseError {
        chunk: chunk_id,
        offset: chunk_info.offset,
        details: format!("{e}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_discovery::discover_chunks;
    use std::io::Cursor;

    /// Create minimal valid root ADT data for testing.
    fn create_minimal_root_adt() -> Vec<u8> {
        let mut data = Vec::new();

        // MVER chunk
        data.extend_from_slice(&ChunkId::MVER.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&18u32.to_le_bytes());

        // MHDR chunk (64 bytes)
        data.extend_from_slice(&ChunkId::MHDR.0);
        data.extend_from_slice(&64u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 64]);

        // MCIN chunk (4096 bytes = 256 entries × 16 bytes)
        data.extend_from_slice(&ChunkId::MCIN.0);
        data.extend_from_slice(&4096u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 4096]);

        // MTEX chunk (empty texture list)
        data.extend_from_slice(&ChunkId::MTEX.0);
        data.extend_from_slice(&0u32.to_le_bytes());

        // One minimal MCNK chunk
        data.extend_from_slice(&ChunkId::MCNK.0);
        data.extend_from_slice(&136u32.to_le_bytes()); // Size: just header

        // MCNK header (136 bytes, all zeros)
        data.extend_from_slice(&[0u8; 136]);

        data
    }

    #[test]
    fn test_parse_minimal_root_adt() {
        let data = create_minimal_root_adt();
        let mut cursor = Cursor::new(data);

        let discovery = discover_chunks(&mut cursor).unwrap();
        let version = AdtVersion::from_discovery(&discovery);

        cursor.set_position(0);
        let result = parse_root_adt(&mut cursor, &discovery, version);

        assert!(result.is_ok());
        let (root, warnings) = result.unwrap();

        assert_eq!(root.version, AdtVersion::VanillaEarly);
        // MTEX with size 0 correctly produces zero textures (fixed parser boundary issue)
        assert_eq!(root.textures.len(), 0);
        assert_eq!(root.mcnk_chunks.len(), 1);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_parse_root_missing_mhdr() {
        let mut data = Vec::new();

        // MVER chunk only
        data.extend_from_slice(&ChunkId::MVER.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&18u32.to_le_bytes());

        let mut cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut cursor).unwrap();
        let version = AdtVersion::from_discovery(&discovery);

        cursor.set_position(0);
        let result = parse_root_adt(&mut cursor, &discovery, version);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AdtError::MissingRequiredChunk(_)
        ));
    }

    #[test]
    fn test_parse_root_missing_mcin() {
        let mut data = Vec::new();

        // MVER chunk
        data.extend_from_slice(&ChunkId::MVER.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&18u32.to_le_bytes());

        // MHDR chunk
        data.extend_from_slice(&ChunkId::MHDR.0);
        data.extend_from_slice(&64u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 64]);

        let mut cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut cursor).unwrap();
        let version = AdtVersion::from_discovery(&discovery);

        cursor.set_position(0);
        let result = parse_root_adt(&mut cursor, &discovery, version);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AdtError::MissingRequiredChunk(_)
        ));
    }

    /// Create minimal Cataclysm+ split root ADT (no MCIN/MTEX).
    fn create_split_root_adt() -> Vec<u8> {
        let mut data = Vec::new();

        // MVER chunk
        data.extend_from_slice(&ChunkId::MVER.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&18u32.to_le_bytes());

        // MHDR chunk (64 bytes)
        data.extend_from_slice(&ChunkId::MHDR.0);
        data.extend_from_slice(&64u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 64]);

        // One minimal MCNK chunk (split root still has MCNK but with different structure)
        data.extend_from_slice(&ChunkId::MCNK.0);
        data.extend_from_slice(&136u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 136]);

        data
    }

    #[test]
    fn test_parse_split_root_adt() {
        let data = create_split_root_adt();
        let mut cursor = Cursor::new(data);

        let discovery = discover_chunks(&mut cursor).unwrap();
        let version = AdtVersion::from_discovery(&discovery);

        cursor.set_position(0);
        let result = parse_root_adt(&mut cursor, &discovery, version);

        assert!(result.is_ok());
        let (root, warnings) = result.unwrap();

        // Split root files don't have MCIN or MTEX
        assert_eq!(root.mcin.entries.len(), 0);
        assert_eq!(root.textures.len(), 0);
        assert_eq!(root.mcnk_chunks.len(), 1);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_split_root_detection() {
        let data = create_split_root_adt();
        let mut cursor = Cursor::new(data);

        let discovery = discover_chunks(&mut cursor).unwrap();

        // Verify split root is detected (no MCIN/MTEX)
        assert!(!discovery.has_chunk(ChunkId::MCIN));
        assert!(!discovery.has_chunk(ChunkId::MTEX));
        assert!(discovery.has_chunk(ChunkId::MHDR));
        assert!(discovery.has_chunk(ChunkId::MCNK));
    }

    #[test]
    fn test_split_root_without_mcnk_fails() {
        let mut data = Vec::new();

        // MVER chunk
        data.extend_from_slice(&ChunkId::MVER.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&18u32.to_le_bytes());

        // MHDR chunk
        data.extend_from_slice(&ChunkId::MHDR.0);
        data.extend_from_slice(&64u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 64]);

        // No MCIN, no MTEX (split root), but also no MCNK

        let mut cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut cursor).unwrap();
        let version = AdtVersion::from_discovery(&discovery);

        cursor.set_position(0);
        let result = parse_root_adt(&mut cursor, &discovery, version);

        // Should fail because MCNK is always required
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AdtError::MissingRequiredChunk(ChunkId::MCNK)
        ));
    }

    #[test]
    fn test_monolithic_root_requires_mcin_and_mtex() {
        let mut data = Vec::new();

        // MVER chunk
        data.extend_from_slice(&ChunkId::MVER.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&18u32.to_le_bytes());

        // MHDR chunk
        data.extend_from_slice(&ChunkId::MHDR.0);
        data.extend_from_slice(&64u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 64]);

        // MCIN present (indicates monolithic file)
        data.extend_from_slice(&ChunkId::MCIN.0);
        data.extend_from_slice(&4096u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 4096]);

        // MTEX missing (required for monolithic)

        // MCNK chunk
        data.extend_from_slice(&ChunkId::MCNK.0);
        data.extend_from_slice(&128u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 128]);

        let mut cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut cursor).unwrap();
        let version = AdtVersion::from_discovery(&discovery);

        cursor.set_position(0);
        let result = parse_root_adt(&mut cursor, &discovery, version);

        // Should fail because MTEX is required for monolithic files
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AdtError::MissingRequiredChunk(ChunkId::MTEX)
        ));
    }
}
