// mcnk_writer.rs - Detailed implementation for writing MCNK subchunks

use crate::chunk::*;
use crate::error::Result;
use crate::io_helpers::WriteLittleEndian;
use crate::version::AdtVersion;
use std::io::{Seek, SeekFrom, Write};

/// Write a MCNK chunk with all its subchunks
pub fn write_mcnk<W: Write + Seek>(
    writer: &mut W,
    chunk: &McnkChunk,
    version: AdtVersion,
) -> Result<(u32, u32)> {
    // Remember the start position to calculate offsets
    let start_pos = writer.stream_position()? as u32;

    // Start with the MCNK header
    // We'll need to update this later with the correct offsets
    write_chunk_header(writer, b"MCNK", 0)?; // Size will be updated later

    // Write the MCNK header fields - we'll update the offsets later
    writer.write_u32_le(chunk.flags)?;
    writer.write_u32_le(chunk.ix)?;
    writer.write_u32_le(chunk.iy)?;
    writer.write_u32_le(chunk.n_layers)?;
    writer.write_u32_le(chunk.n_doodad_refs)?;

    // Placeholder for subchunk offsets
    let mcvt_offset_pos = writer.stream_position()? as u32;
    writer.write_u32_le(0)?; // MCVT offset

    let mcnr_offset_pos = writer.stream_position()? as u32;
    writer.write_u32_le(0)?; // MCNR offset

    let mcly_offset_pos = writer.stream_position()? as u32;
    writer.write_u32_le(0)?; // MCLY offset

    let mcrf_offset_pos = writer.stream_position()? as u32;
    writer.write_u32_le(0)?; // MCRF offset

    let mcal_offset_pos = writer.stream_position()? as u32;
    writer.write_u32_le(0)?; // MCAL offset

    writer.write_u32_le(0)?; // MCAL size (will be updated)

    let _mcsh_offset_pos = writer.stream_position()? as u32;
    writer.write_u32_le(0)?; // MCSH offset

    writer.write_u32_le(0)?; // MCSH size (will be updated)
    writer.write_u32_le(chunk.area_id)?;
    writer.write_u32_le(chunk.n_map_obj_refs)?;
    writer.write_u32_le(chunk.holes)?;
    writer.write_u16_le(chunk.s1)?;
    writer.write_u16_le(chunk.s2)?;

    // Write CMaNGOS d1, d2, d3, pred_tex fields
    writer.write_u32_le(chunk.d1)?;
    writer.write_u32_le(chunk.d2)?;
    writer.write_u32_le(chunk.d3)?;
    writer.write_u32_le(chunk.pred_tex)?;

    writer.write_u32_le(chunk.n_effect_doodad)?;

    let _mcse_offset_pos = writer.stream_position()? as u32;
    writer.write_u32_le(0)?; // MCSE offset

    writer.write_u32_le(chunk.n_sound_emitters)?;

    let _liquid_offset_pos = writer.stream_position()? as u32;
    writer.write_u32_le(0)?; // MCLQ/MH2O offset

    writer.write_u32_le(0)?; // Liquid size (will be updated)

    for i in 0..3 {
        writer.write_f32_le(chunk.position[i])?;
    }

    let _mccv_offset_pos = writer.stream_position()? as u32;
    writer.write_u32_le(0)?; // MCCV offset

    let _mclv_offset_pos = writer.stream_position()? as u32;
    writer.write_u32_le(0)?; // MCLV offset

    // Write the additional CMaNGOS fields
    writer.write_u32_le(chunk.texture_id)?;
    writer.write_u32_le(chunk.props)?;
    writer.write_u32_le(chunk.effect_id)?;

    // Store position after MCNK header
    let _after_header_pos = writer.stream_position()? as u32;

    // Now write the subchunks

    // MCVT - height map
    if !chunk.height_map.is_empty() {
        let mcvt_pos = writer.stream_position()? as u32;
        let rel_offset = mcvt_pos - start_pos;

        // Go back and update the offset in the header
        writer.seek(SeekFrom::Start(mcvt_offset_pos as u64))?;
        writer.write_u32_le(rel_offset)?;

        // Go back to our position and write the chunk
        writer.seek(SeekFrom::Start(mcvt_pos as u64))?;

        // Write MCVT
        write_chunk_header(writer, b"MCVT", 145 * 4)?; // 145 vertices * 4 bytes each

        for height in &chunk.height_map {
            writer.write_f32_le(*height)?;
        }
    }

    // MCNR - normals
    if !chunk.normals.is_empty() {
        let mcnr_pos = writer.stream_position()? as u32;
        let rel_offset = mcnr_pos - start_pos;

        // Update the offset in the header
        writer.seek(SeekFrom::Start(mcnr_offset_pos as u64))?;
        writer.write_u32_le(rel_offset)?;

        // Go back to our position
        writer.seek(SeekFrom::Start(mcnr_pos as u64))?;

        // Write MCNR
        let mut normal_data_size = chunk.normals.len() * 3; // 3 bytes per normal

        // Add padding to align to 4 bytes if needed
        let padding = (4 - (normal_data_size % 4)) % 4;
        normal_data_size += padding;

        write_chunk_header(writer, b"MCNR", normal_data_size as u32)?;

        for normal in &chunk.normals {
            writer.write_all(normal)?;
        }

        // Write padding if needed
        for _ in 0..padding {
            writer.write_u8(0)?;
        }
    }

    // MCLY - texture layers
    if !chunk.texture_layers.is_empty() {
        let mcly_pos = writer.stream_position()? as u32;
        let rel_offset = mcly_pos - start_pos;

        // Update the offset in the header
        writer.seek(SeekFrom::Start(mcly_offset_pos as u64))?;
        writer.write_u32_le(rel_offset)?;

        // Go back to our position
        writer.seek(SeekFrom::Start(mcly_pos as u64))?;

        // Write MCLY - each layer is 16 bytes
        let layer_size = chunk.texture_layers.len() * 16;
        write_chunk_header(writer, b"MCLY", layer_size as u32)?;

        for layer in &chunk.texture_layers {
            writer.write_u32_le(layer.texture_id)?;
            writer.write_u32_le(layer.flags)?;
            writer.write_u32_le(layer.alpha_map_offset)?;
            writer.write_u32_le(layer.effect_id)?;
        }
    }

    // MCRF - doodad references
    if !chunk.doodad_refs.is_empty() {
        let mcrf_pos = writer.stream_position()? as u32;
        let rel_offset = mcrf_pos - start_pos;

        // Update the offset in the header
        writer.seek(SeekFrom::Start(mcrf_offset_pos as u64))?;
        writer.write_u32_le(rel_offset)?;

        // Go back to our position
        writer.seek(SeekFrom::Start(mcrf_pos as u64))?;

        // Write MCRF - each ref is 4 bytes
        let refs_size = chunk.doodad_refs.len() * 4;
        write_chunk_header(writer, b"MCRF", refs_size as u32)?;

        for doodad_ref in &chunk.doodad_refs {
            writer.write_u32_le(*doodad_ref)?;
        }
    }

    // MCRD - map object references (comes after MCRF)
    if !chunk.map_obj_refs.is_empty() {
        // Write MCRD - each ref is 4 bytes
        let refs_size = chunk.map_obj_refs.len() * 4;
        write_chunk_header(writer, b"MCRD", refs_size as u32)?;

        for map_obj_ref in &chunk.map_obj_refs {
            writer.write_u32_le(*map_obj_ref)?;
        }
    }

    // MCSH - shadow map
    // For now we'll skip writing it if there's no data

    // MCAL - alpha maps
    if !chunk.alpha_maps.is_empty() {
        let mcal_pos = writer.stream_position()? as u32;
        let rel_offset = mcal_pos - start_pos;

        // Calculate total size of alpha maps
        let mut total_size = 0;
        for alpha_map in &chunk.alpha_maps {
            total_size += alpha_map.len();
        }

        // Update the offset and size in the header
        writer.seek(SeekFrom::Start(mcal_offset_pos as u64))?;
        writer.write_u32_le(rel_offset)?;
        writer.write_u32_le(total_size as u32)?;

        // Go back to our position
        writer.seek(SeekFrom::Start(mcal_pos as u64))?;

        // Write MCAL
        write_chunk_header(writer, b"MCAL", total_size as u32)?;

        for alpha_map in &chunk.alpha_maps {
            writer.write_all(alpha_map)?;
        }
    }

    // Write liquid data based on version
    if version < AdtVersion::WotLK {
        // Pre-WotLK uses MCLQ
        // We would need to implement MCLQ writing here
        // For now, just skip if no data
    } else {
        // WotLK+ uses the global MH2O chunk
        // We don't need to write anything here
    }

    // Calculate the total size of the MCNK chunk
    let end_pos = writer.stream_position()? as u32;
    let chunk_size = end_pos - start_pos - 8; // Subtract the chunk header size

    // Go back and update the chunk size
    writer.seek(SeekFrom::Start((start_pos + 4) as u64))?;
    writer.write_u32_le(chunk_size)?;

    // Return to the end
    writer.seek(SeekFrom::Start(end_pos as u64))?;

    // Return start position and size for indexing
    Ok((start_pos, chunk_size + 8))
}

/// Write a chunk header
fn write_chunk_header<W: Write>(writer: &mut W, magic: &[u8; 4], size: u32) -> Result<()> {
    // WoW files store magic bytes in reverse order
    let mut reversed_magic = *magic;
    reversed_magic.reverse();
    writer.write_all(&reversed_magic)?;
    writer.write_u32_le(size)?;
    Ok(())
}
