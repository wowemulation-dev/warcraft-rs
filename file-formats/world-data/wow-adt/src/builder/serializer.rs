//! ADT file serialization with automatic offset calculation.
//!
//! This module implements the two-pass serialization strategy:
//! 1. Write all chunks with placeholder offsets, tracking positions
//! 2. Seek back and update offset tables (MHDR, MCIN) with calculated values
//!
//! This approach ensures correct offset calculation without requiring
//! pre-calculation of chunk sizes.

use std::io::{Seek, SeekFrom, Write};

use binrw::BinWrite;

use crate::chunks::mcnk::{
    MccvChunk, MclyChunk, MclyLayer, McnkChunk, McnkFlags, McnkHeader, McnrChunk, McvtChunk,
};
use crate::chunks::mh2o::{Mh2oChunk, Mh2oHeader};
use crate::chunks::{
    McinChunk, McinEntry, MddfChunk, MhdrChunk, MmdxChunk, MmidChunk, ModfChunk, MtexChunk,
    MverChunk, MwidChunk, MwmoChunk,
};
use crate::error::{AdtError, Result};
use crate::{BuiltAdt, ChunkId};

/// Serialize BuiltAdt to a writer with automatic offset calculation.
///
/// Implements the two-pass serialization strategy:
/// - Pass 1: Write all chunks, track positions
/// - Pass 2: Update offset tables with calculated values
///
/// # Chunk Order
///
/// ```text
/// 1. MVER (version = 18)
/// 2. MHDR (placeholder, update later)
/// 3. MCIN (placeholder, update later)
/// 4. MTEX (texture filenames)
/// 5. MMDX (M2 model filenames)
/// 6. MMID (M2 model offsets)
/// 7. MWMO (WMO filenames)
/// 8. MWID (WMO offsets)
/// 9. MDDF (M2 placements)
/// 10. MODF (WMO placements)
/// 11. MFBO (if present, TBC+)
/// 12. MH2O (if present, WotLK+)
/// 13. MTXF (if present, WotLK+)
/// 14. MAMP (if present, Cataclysm+)
/// 15. MTXP (if present, MoP+)
/// 16-271. MCNK[0..255] (terrain chunks)
/// ```
///
/// # Arguments
///
/// * `adt` - Built ADT structure to serialize
/// * `writer` - Output writer (must support seeking)
///
/// # Errors
///
/// Returns error if:
/// - I/O operations fail
/// - Binary serialization fails
/// - Seeking fails during offset updates
pub fn serialize_to_writer<W: Write + Seek>(adt: &BuiltAdt, writer: &mut W) -> Result<()> {
    // Track chunk positions for offset calculation
    let mut chunk_positions = ChunkPositions::default();

    // PASS 1: Write all chunks with placeholders

    // 1. Write MVER chunk (always version 18)
    let mver = MverChunk { version: 18 };
    write_chunk(writer, ChunkId::MVER, &mver)?;

    // 2. Write MHDR chunk (placeholder with all zeros)
    chunk_positions.mhdr_data_start = writer.stream_position()? + 8; // After chunk header
    let mhdr_placeholder = MhdrChunk::default();
    write_chunk(writer, ChunkId::MHDR, &mhdr_placeholder)?;

    // 3. Write MCIN chunk (placeholder with all zeros)
    chunk_positions.mcin_data_start = writer.stream_position()? + 8;
    let mcin_placeholder = McinChunk::default();
    write_chunk(writer, ChunkId::MCIN, &mcin_placeholder)?;

    // 4. Write MTEX chunk (texture filenames)
    chunk_positions.mtex = writer.stream_position()?;
    let mtex = create_mtex_chunk(adt.textures());
    write_chunk(writer, ChunkId::MTEX, &mtex)?;

    // 5. Write MMDX chunk (M2 model filenames)
    chunk_positions.mmdx = writer.stream_position()?;
    let mmdx = create_mmdx_chunk(adt.models());
    write_chunk(writer, ChunkId::MMDX, &mmdx)?;

    // 6. Write MMID chunk (M2 model offsets)
    chunk_positions.mmid = writer.stream_position()?;
    let mmid = create_mmid_chunk(adt.models());
    write_chunk(writer, ChunkId::MMID, &mmid)?;

    // 7. Write MWMO chunk (WMO filenames)
    chunk_positions.mwmo = writer.stream_position()?;
    let mwmo = create_mwmo_chunk(adt.wmos());
    write_chunk(writer, ChunkId::MWMO, &mwmo)?;

    // 8. Write MWID chunk (WMO offsets)
    chunk_positions.mwid = writer.stream_position()?;
    let mwid = create_mwid_chunk(adt.wmos());
    write_chunk(writer, ChunkId::MWID, &mwid)?;

    // 9. Write MDDF chunk (M2 placements)
    chunk_positions.mddf = writer.stream_position()?;
    let mddf = create_mddf_chunk(adt.doodad_placements());
    write_chunk(writer, ChunkId::MDDF, &mddf)?;

    // 10. Write MODF chunk (WMO placements)
    chunk_positions.modf = writer.stream_position()?;
    let modf = create_modf_chunk(adt.wmo_placements());
    write_chunk(writer, ChunkId::MODF, &modf)?;

    // 11. Write MFBO chunk (if present, TBC+)
    if let Some(mfbo) = adt.flight_bounds() {
        chunk_positions.mfbo = Some(writer.stream_position()?);
        write_chunk(writer, ChunkId::MFBO, mfbo)?;
    }

    // 12. Write MH2O chunk (if present, WotLK+)
    if let Some(mh2o) = adt.water_data() {
        chunk_positions.mh2o = Some(writer.stream_position()?);
        write_mh2o_chunk(writer, mh2o)?;
    }

    // 13. Write MTXF chunk (WotLK+)
    // Always write for WotLK+ to enable proper version detection
    // MTXF is a root-level chunk, unlike MCCV which is a subchunk
    if matches!(
        adt.version(),
        crate::AdtVersion::WotLK | crate::AdtVersion::Cataclysm | crate::AdtVersion::MoP
    ) {
        chunk_positions.mtxf = Some(writer.stream_position()?);
        let mtxf = adt.texture_flags().cloned().unwrap_or_else(|| {
            // Generate empty MTXF with zeros for all textures
            crate::chunks::MtxfChunk {
                flags: vec![0; adt.textures().len()],
            }
        });
        write_chunk(writer, ChunkId::MTXF, &mtxf)?;
    }

    // 14. Write MAMP chunk (if present, Cataclysm+)
    if let Some(mamp) = adt.texture_amplifier() {
        chunk_positions.mamp = Some(writer.stream_position()?);
        write_chunk(writer, ChunkId::MAMP, mamp)?;
    }

    // 15. Write MTXP chunk (if present, MoP+)
    if let Some(mtxp) = adt.texture_params() {
        chunk_positions.mtxp = Some(writer.stream_position()?);
        write_chunk(writer, ChunkId::MTXP, mtxp)?;
    }

    // 16. Write MBMH chunk (blend mesh headers, MoP+)
    if let Some(mbmh) = adt.blend_mesh_headers() {
        chunk_positions.mbmh = Some(writer.stream_position()?);
        write_chunk(writer, ChunkId::MBMH, mbmh)?;
    }

    // 17. Write MBBB chunk (blend mesh bounds, MoP+)
    if let Some(mbbb) = adt.blend_mesh_bounds() {
        chunk_positions.mbbb = Some(writer.stream_position()?);
        write_chunk(writer, ChunkId::MBBB, mbbb)?;
    }

    // 18. Write MBNV chunk (blend mesh vertices, MoP+)
    if let Some(mbnv) = adt.blend_mesh_vertices() {
        chunk_positions.mbnv = Some(writer.stream_position()?);
        write_chunk(writer, ChunkId::MBNV, mbnv)?;
    }

    // 19. Write MBMI chunk (blend mesh indices, MoP+)
    if let Some(mbmi) = adt.blend_mesh_indices() {
        chunk_positions.mbmi = Some(writer.stream_position()?);
        write_chunk(writer, ChunkId::MBMI, mbmi)?;
    }

    // 20-275. Write MCNK chunks (terrain tiles)
    // Use user-provided chunks if available, otherwise generate 256 minimal chunks
    chunk_positions.mcnk_start = writer.stream_position()?;

    if adt.mcnk_chunks().is_empty() {
        // No user chunks - generate 256 minimal MCNK chunks (16x16 grid)
        for y in 0..16 {
            for x in 0..16 {
                let mcnk_start = writer.stream_position()?;
                write_minimal_mcnk_chunk(writer, x, y, adt.version())?;
                let mcnk_end = writer.stream_position()?;
                let mcnk_size = (mcnk_end - mcnk_start - 8) as u32;
                chunk_positions.mcnk_entries.push((mcnk_start, mcnk_size));
            }
        }
    } else {
        // Write user-provided MCNK chunks
        for mcnk in adt.mcnk_chunks() {
            let mcnk_start = writer.stream_position()?;
            write_mcnk_chunk(writer, mcnk)?;
            let mcnk_end = writer.stream_position()?;
            let mcnk_size = (mcnk_end - mcnk_start - 8) as u32;
            chunk_positions.mcnk_entries.push((mcnk_start, mcnk_size));
        }

        // Pad to 256 entries if fewer chunks provided
        while chunk_positions.mcnk_entries.len() < 256 {
            chunk_positions.mcnk_entries.push((0, 0));
        }
    }

    // PASS 2: Update offset tables

    // Calculate MHDR offsets (relative to MHDR data start)
    let mhdr = calculate_mhdr_offsets(&chunk_positions);

    // Seek to MHDR data position and write calculated offsets
    writer.seek(SeekFrom::Start(chunk_positions.mhdr_data_start))?;
    mhdr.write_le(writer)?;

    // Calculate MCIN entries (absolute file offsets)
    let mcin = calculate_mcin_entries(&chunk_positions);

    // Seek to MCIN data position and write calculated entries
    writer.seek(SeekFrom::Start(chunk_positions.mcin_data_start))?;
    mcin.write_le(writer)?;

    // Flush to ensure all data is written
    writer.flush()?;

    Ok(())
}

/// Write a minimal MCNK chunk with required sub-chunks for valid parsing.
///
/// Creates a structurally valid MCNK chunk containing:
/// - MCNK header with proper offsets
/// - MCVT (heights): Flat terrain at height 0.0
/// - MCNR (normals): All normals pointing up [0, 0, 127]
/// - MCLY (layers): Single texture layer referencing texture ID 0
///
/// This minimal implementation allows ADT files to parse correctly without
/// requiring full terrain data. Full terrain support is deferred to Phase 5.
///
/// # Arguments
///
/// * `writer` - Output writer
/// * `x` - Tile X coordinate (0-15)
/// * `y` - Tile Y coordinate (0-15)
///
/// # Errors
///
/// Returns error if I/O or serialization fails.
fn write_minimal_mcnk_chunk<W: Write + Seek>(
    writer: &mut W,
    x: u32,
    y: u32,
    version: crate::AdtVersion,
) -> Result<()> {
    // Calculate world position for this tile
    // Each tile is 33.33333 yards, ADT origin is at (0, 0)
    let tile_size = 533.33 / 16.0;
    let pos_x = x as f32 * tile_size;
    let pos_y = y as f32 * tile_size;
    let pos_z = 0.0; // Flat terrain at sea level

    // Track sub-chunk positions (relative to MCNK start, including 8-byte header)
    let mcnk_start = writer.stream_position()?;

    // Reserve space for MCNK header (8-byte chunk header + 136-byte MCNK header)
    let header_start = writer.stream_position()?;
    let placeholder = vec![0u8; 8 + 136]; // Total 144 bytes
    writer.write_all(&placeholder)?;

    // Write MCVT sub-chunk (vertex heights)
    let mcvt_offset = (writer.stream_position()? - mcnk_start) as u32;
    let mcvt = McvtChunk {
        heights: vec![0.0; 145], // 9x9 outer + 8x8 inner vertices, all at height 0
    };
    // Write manually due to Vec serialization issues
    writer.write_all(&ChunkId::MCVT.0)?;
    let data_size = (mcvt.heights.len() * 4) as u32;
    writer.write_all(&data_size.to_le_bytes())?;
    for height in &mcvt.heights {
        writer.write_all(&height.to_le_bytes())?;
    }

    // Write MCNR sub-chunk (vertex normals)
    let mcnr_offset = (writer.stream_position()? - mcnk_start) as u32;
    let mcnr = McnrChunk::default(); // All normals pointing up
    // Write manually due to Vec serialization issues
    writer.write_all(&ChunkId::MCNR.0)?;
    let data_size = (mcnr.normals.len() * 3 + 13) as u32; // 3 bytes per normal + 13 padding
    writer.write_all(&data_size.to_le_bytes())?;
    for normal in &mcnr.normals {
        writer.write_all(&[normal.x as u8, normal.z as u8, normal.y as u8])?;
    }
    writer.write_all(&[0u8; 13])?; // Padding

    // Write MCLY sub-chunk (texture layers)
    let mcly_offset = (writer.stream_position()? - mcnk_start) as u32;
    let mcly = MclyChunk {
        layers: vec![MclyLayer {
            texture_id: 0,             // Reference to first texture in MTEX
            flags: Default::default(), // No special flags
            offset_in_mcal: 0,         // No alpha map
            effect_id: 0,              // No effect
        }],
    };
    write_chunk(writer, ChunkId::MCLY, &mcly)?;

    // Write MCCV sub-chunk (vertex colors) for VanillaLate+ to enable proper version detection
    let mccv_offset = if matches!(
        version,
        crate::AdtVersion::VanillaLate
            | crate::AdtVersion::TBC
            | crate::AdtVersion::WotLK
            | crate::AdtVersion::Cataclysm
            | crate::AdtVersion::MoP
    ) {
        let offset = (writer.stream_position()? - mcnk_start) as u32;
        let mccv = MccvChunk::default(); // Neutral colors (127, 127, 127, 127)
        // Write manually due to Vec serialization issues
        writer.write_all(&ChunkId::MCCV.0)?;
        let data_size = (mccv.colors.len() * 4) as u32; // 4 bytes per BGRA color
        writer.write_all(&data_size.to_le_bytes())?;
        for color in &mccv.colors {
            writer.write_all(&[color.b, color.g, color.r, color.a])?;
        }
        offset
    } else {
        0 // VanillaEarly has no MCCV
    };

    // Calculate MCNK chunk total size (excluding 8-byte chunk header)
    let mcnk_end = writer.stream_position()?;
    let mcnk_size = (mcnk_end - mcnk_start - 8) as u32;

    // Build MCNK header with calculated offsets
    let header = McnkHeader {
        flags: McnkFlags { value: 0 },
        index_x: x,
        index_y: y,
        n_layers: 1, // One texture layer
        n_doodad_refs: 0,
        multipurpose_field: McnkHeader::multipurpose_from_offsets(mcvt_offset, mcnr_offset),
        ofs_layer: mcly_offset,
        ofs_refs: 0,  // No MCRF
        ofs_alpha: 0, // No MCAL
        size_alpha: 0,
        ofs_shadow: 0, // No MCSH
        size_shadow: 0,
        area_id: 0,
        n_map_obj_refs: 0,
        holes_low_res: 0,
        unknown_but_used: 1, // Always 1 per spec
        pred_tex: [0; 8],
        no_effect_doodad: [0; 8],
        unknown_8bytes: [0; 8], // Unknown 8-byte field
        ofs_snd_emitters: 0,    // No MCSE
        n_snd_emitters: 0,
        ofs_liquid: 0, // No MCLQ
        size_liquid: 0,
        position: [pos_x, pos_y, pos_z],
        ofs_mccv: mccv_offset,
        ofs_mclv: 0, // No MCLV
        unused: 0,
        _padding: [0; 8],
    };

    // Seek back and write the actual header
    writer.seek(SeekFrom::Start(header_start))?;

    // Write chunk header (magic + size)
    ChunkId::MCNK.write_le(writer)?;
    writer.write_all(&mcnk_size.to_le_bytes())?;

    // Write MCNK header data
    header.write_le(writer)?;

    // Seek to end of chunk
    writer.seek(SeekFrom::Start(mcnk_end))?;

    Ok(())
}

/// Write a user-provided MCNK chunk with all its subchunks.
///
/// Serializes a complete McnkChunk including:
/// - MCNK header with proper offset calculations
/// - All subchunks present in the chunk (MCVT, MCNR, MCLY, MCAL, MCRF, MCSH, MCLQ, MCCV, MCSE, etc.)
///
/// This preserves all user-provided terrain data during round-trip serialization.
///
/// # Arguments
///
/// * `writer` - Output writer (must support seeking)
/// * `mcnk` - User-provided MCNK chunk with all data
///
/// # Errors
///
/// Returns error if I/O or serialization fails.
fn write_mcnk_chunk<W: Write + Seek>(writer: &mut W, mcnk: &McnkChunk) -> Result<()> {
    let mcnk_start = writer.stream_position()?;

    // Reserve space for MCNK header (8-byte chunk header + 136-byte MCNK header)
    let header_start = writer.stream_position()?;
    let placeholder = vec![0u8; 8 + 136];
    writer.write_all(&placeholder)?;

    // Track offsets for header
    let mut header = mcnk.header.clone();

    // Clear all optional chunk offsets - they'll be set if chunks are actually written
    // This prevents stale offsets from the original header pointing to invalid locations
    header.ofs_layer = 0;
    header.n_layers = 0;
    header.ofs_refs = 0;
    header.ofs_alpha = 0;
    header.size_alpha = 0;
    header.ofs_shadow = 0;
    header.size_shadow = 0;
    header.ofs_liquid = 0;
    header.size_liquid = 0;
    header.ofs_mccv = 0;
    header.ofs_mclv = 0;
    header.ofs_snd_emitters = 0;
    header.n_snd_emitters = 0;
    header.multipurpose_field = [0u8; 8]; // ofs_height and ofs_normal will be set when written

    // Write MCVT (heights) if present
    if let Some(mcvt) = &mcnk.heights {
        let offset = (writer.stream_position()? - mcnk_start) as u32;
        // Update first 4 bytes of multipurpose_field with ofs_height
        header.multipurpose_field[0..4].copy_from_slice(&offset.to_le_bytes());
        // Write manually instead of using write_chunk due to Vec serialization issues
        writer.write_all(&ChunkId::MCVT.0)?; // Write chunk ID bytes
        let data_size = (mcvt.heights.len() * 4) as u32;
        writer.write_all(&data_size.to_le_bytes())?;
        for height in &mcvt.heights {
            writer.write_all(&height.to_le_bytes())?;
        }
    }

    // Write MCNR (normals) if present
    if let Some(mcnr) = &mcnk.normals {
        let offset = (writer.stream_position()? - mcnk_start) as u32;
        // Update last 4 bytes of multipurpose_field with ofs_normal
        header.multipurpose_field[4..8].copy_from_slice(&offset.to_le_bytes());
        // Write manually due to Vec serialization issues
        writer.write_all(&ChunkId::MCNR.0)?;
        let data_size = (mcnr.normals.len() * 3 + 13) as u32; // 3 bytes per normal + 13 padding
        writer.write_all(&data_size.to_le_bytes())?;
        for normal in &mcnr.normals {
            writer.write_all(&[normal.x as u8, normal.z as u8, normal.y as u8])?;
        }
        writer.write_all(&[0u8; 13])?; // Padding
    }

    // Write MCLY (texture layers) if present
    if let Some(mcly) = &mcnk.layers {
        header.ofs_layer = (writer.stream_position()? - mcnk_start) as u32;
        header.n_layers = mcly.layers.len() as u32;
        write_chunk(writer, ChunkId::MCLY, mcly)?;
    }

    // Write MCRF (object references) if present
    if let Some(mcrf) = &mcnk.refs {
        header.ofs_refs = (writer.stream_position()? - mcnk_start) as u32;
        write_chunk(writer, ChunkId::MCRF, mcrf)?;
    }

    // Write MCAL (alpha maps) if present
    if let Some(mcal) = &mcnk.alpha {
        header.ofs_alpha = (writer.stream_position()? - mcnk_start) as u32;
        let alpha_start = writer.stream_position()?;
        write_chunk(writer, ChunkId::MCAL, mcal)?;
        let alpha_end = writer.stream_position()?;
        header.size_alpha = (alpha_end - alpha_start - 8) as u32;
    }

    // Write MCSH (shadow map) if present
    if let Some(mcsh) = &mcnk.shadow {
        header.ofs_shadow = (writer.stream_position()? - mcnk_start) as u32;
        let shadow_start = writer.stream_position()?;
        write_chunk(writer, ChunkId::MCSH, mcsh)?;
        let shadow_end = writer.stream_position()?;
        header.size_shadow = (shadow_end - shadow_start - 8) as u32;
    }

    // Write MCLQ (liquid) if present
    if let Some(mclq) = &mcnk.liquid {
        header.ofs_liquid = (writer.stream_position()? - mcnk_start) as u32;
        let liquid_start = writer.stream_position()?;
        write_chunk(writer, ChunkId::MCLQ, mclq)?;
        let liquid_end = writer.stream_position()?;
        header.size_liquid = (liquid_end - liquid_start) as u32;
    }

    // Write MCCV (vertex colors) if present
    if let Some(mccv) = &mcnk.vertex_colors {
        header.ofs_mccv = (writer.stream_position()? - mcnk_start) as u32;
        // Write manually due to Vec serialization issues
        writer.write_all(&ChunkId::MCCV.0)?;
        let data_size = (mccv.colors.len() * 4) as u32; // 4 bytes per BGRA color
        writer.write_all(&data_size.to_le_bytes())?;
        for color in &mccv.colors {
            writer.write_all(&[color.b, color.g, color.r, color.a])?;
        }
    }

    // Write MCSE (sound emitters) if present
    if let Some(mcse) = &mcnk.sound_emitters {
        header.ofs_snd_emitters = (writer.stream_position()? - mcnk_start) as u32;
        header.n_snd_emitters = mcse.emitters.len() as u32;
        write_chunk(writer, ChunkId::MCSE, mcse)?;
    }

    // Write MCLV (vertex lighting, Cataclysm+) if present
    if let Some(mclv) = &mcnk.vertex_lighting {
        header.ofs_mclv = (writer.stream_position()? - mcnk_start) as u32;
        write_chunk(writer, ChunkId::MCLV, mclv)?;
    }

    // Write MCRD (doodad refs, split files, Cataclysm+) if present
    if let Some(mcrd) = &mcnk.doodad_refs {
        // MCRD shares ofs_refs with MCRF (Cataclysm+ split file architecture)
        if header.ofs_refs == 0 {
            header.ofs_refs = (writer.stream_position()? - mcnk_start) as u32;
        }
        write_chunk(writer, ChunkId::MCRD, mcrd)?;
    }

    // Write MCRW (WMO refs, split files, Cataclysm+) if present
    if let Some(mcrw) = &mcnk.wmo_refs {
        // MCRW shares ofs_refs with MCRF (Cataclysm+ split file architecture)
        if header.ofs_refs == 0 {
            header.ofs_refs = (writer.stream_position()? - mcnk_start) as u32;
        }
        write_chunk(writer, ChunkId::MCRW, mcrw)?;
    }

    // Write MCMT (material IDs, Cataclysm+) if present
    if let Some(mcmt) = &mcnk.materials {
        // MCMT has no dedicated header offset field (parsed via chunk discovery in split files)
        write_chunk(writer, ChunkId::MCMT, mcmt)?;
    }

    // Write MCDD (doodad disable, Cataclysm+) if present
    if let Some(mcdd) = &mcnk.doodad_disable {
        // MCDD has no dedicated header offset field (parsed via chunk discovery)
        write_chunk(writer, ChunkId::MCDD, mcdd)?;
    }

    // Write MCBB (blend batches, MoP+) if present
    if let Some(mcbb) = &mcnk.blend_batches {
        // MCBB has no dedicated header offset field (parsed via chunk discovery)
        write_chunk(writer, ChunkId::MCBB, mcbb)?;
    }

    // Calculate total MCNK size
    let mcnk_end = writer.stream_position()?;
    let mcnk_size = (mcnk_end - mcnk_start - 8) as u32;

    // Seek back and write header
    writer.seek(SeekFrom::Start(header_start))?;
    ChunkId::MCNK.write_le(writer)?;
    writer.write_all(&mcnk_size.to_le_bytes())?;
    header.write_le(writer)?;

    // Seek to end
    writer.seek(SeekFrom::Start(mcnk_end))?;

    Ok(())
}

/// Tracked positions of chunks during serialization.
#[derive(Default)]
struct ChunkPositions {
    /// Position of MHDR data (after 8-byte header)
    mhdr_data_start: u64,
    /// Position of MCIN data (after 8-byte header)
    mcin_data_start: u64,
    /// Position of MTEX chunk
    mtex: u64,
    /// Position of MMDX chunk
    mmdx: u64,
    /// Position of MMID chunk
    mmid: u64,
    /// Position of MWMO chunk
    mwmo: u64,
    /// Position of MWID chunk
    mwid: u64,
    /// Position of MDDF chunk
    mddf: u64,
    /// Position of MODF chunk
    modf: u64,
    /// Position of MFBO chunk (if present)
    mfbo: Option<u64>,
    /// Position of MH2O chunk (if present)
    mh2o: Option<u64>,
    /// Position of MTXF chunk (if present)
    mtxf: Option<u64>,
    /// Position of MAMP chunk (if present)
    mamp: Option<u64>,
    /// Position of MTXP chunk (if present)
    mtxp: Option<u64>,
    /// Position of MBMH chunk (blend mesh headers, if present)
    mbmh: Option<u64>,
    /// Position of MBBB chunk (blend mesh bounds, if present)
    mbbb: Option<u64>,
    /// Position of MBNV chunk (blend mesh vertices, if present)
    mbnv: Option<u64>,
    /// Position of MBMI chunk (blend mesh indices, if present)
    mbmi: Option<u64>,
    /// Position of first MCNK chunk
    mcnk_start: u64,
    /// MCNK entries: (absolute offset, size)
    mcnk_entries: Vec<(u64, u32)>,
}

/// Calculate MHDR offsets relative to MHDR data start.
fn calculate_mhdr_offsets(positions: &ChunkPositions) -> MhdrChunk {
    let base = positions.mhdr_data_start;

    // Helper to calculate relative offset
    let relative_offset = |pos: u64| -> u32 { if pos == 0 { 0 } else { (pos - base) as u32 } };

    MhdrChunk {
        flags: calculate_mhdr_flags(positions),
        mcin_offset: relative_offset(positions.mcin_data_start - 8), // Offset to chunk start
        mtex_offset: relative_offset(positions.mtex),
        mmdx_offset: relative_offset(positions.mmdx),
        mmid_offset: relative_offset(positions.mmid),
        mwmo_offset: relative_offset(positions.mwmo),
        mwid_offset: relative_offset(positions.mwid),
        mddf_offset: relative_offset(positions.mddf),
        modf_offset: relative_offset(positions.modf),
        mfbo_offset: positions.mfbo.map_or(0, relative_offset),
        mh2o_offset: positions.mh2o.map_or(0, relative_offset),
        mtxf_offset: positions.mtxf.map_or(0, relative_offset),
        unused1: 0,
        unused2: 0,
        unused3: 0,
        unused4: 0,
    }
}

/// Calculate MHDR flags based on present optional chunks.
fn calculate_mhdr_flags(positions: &ChunkPositions) -> u32 {
    let mut flags = 0u32;

    // 0x01: MFBO present (flight bounds)
    if positions.mfbo.is_some() {
        flags |= 0x01;
    }

    // 0x02: MH2O present (water/lava)
    if positions.mh2o.is_some() {
        flags |= 0x02;
    }

    flags
}

/// Calculate MCIN entries with absolute file offsets.
fn calculate_mcin_entries(positions: &ChunkPositions) -> McinChunk {
    let mut entries = Vec::with_capacity(256);

    // Add actual MCNK entries
    for &(offset, size) in &positions.mcnk_entries {
        entries.push(McinEntry {
            offset: offset as u32,
            size,
            flags: 0,
            async_id: 0,
        });
    }

    // Pad to 256 entries with zeros
    while entries.len() < 256 {
        entries.push(McinEntry::default());
    }

    McinChunk { entries }
}

/// Write chunk header and data.
///
/// Writes 8-byte header (4-byte magic + 4-byte size) followed by chunk data.
fn write_chunk<W: Write + Seek, T: BinWrite>(
    writer: &mut W,
    chunk_id: ChunkId,
    data: &T,
) -> Result<()>
where
    for<'a> <T as BinWrite>::Args<'a>: Default,
{
    // Write chunk header
    writer.write_all(&chunk_id.0)?;

    // Write chunk data directly (size calculated by seeking)
    let data_start = writer.stream_position()?;
    writer.write_all(&[0u8; 4])?; // Placeholder for size

    // Write data
    data.write_le(writer)
        .map_err(|e| AdtError::BinrwError(format!("Failed to serialize {chunk_id}: {e}")))?;

    let data_end = writer.stream_position()?;
    let size = (data_end - data_start - 4) as u32;

    // Seek back and write size
    writer.seek(SeekFrom::Start(data_start))?;
    writer.write_all(&size.to_le_bytes())?;
    writer.seek(SeekFrom::Start(data_end))?;

    Ok(())
}

/// Write MH2O chunk with multi-level offset structure.
///
/// MH2O has a complex structure:
/// 1. Chunk header (8 bytes: magic + size)
/// 2. 256 MH2O headers (12 bytes each = 3072 bytes)
/// 3. Variable data (instances, attributes) with offsets relative to data start
///
/// All offsets in headers are relative to the start of chunk data (after the 8-byte header).
fn write_mh2o_chunk<W: Write + Seek>(writer: &mut W, mh2o: &Mh2oChunk) -> Result<()> {
    use binrw::BinWrite;

    // Write chunk magic
    writer.write_all(&ChunkId::MH2O.0)?;

    // Placeholder for chunk size
    let size_pos = writer.stream_position()?;
    writer.write_all(&[0u8; 4])?;

    // Data start position (after 8-byte chunk header)
    let data_start = writer.stream_position()?;

    // Reserve space for 256 headers (will be updated later)
    let headers_start = writer.stream_position()?;
    const HEADER_SIZE: usize = 12; // sizeof(Mh2oHeader)
    const HEADER_COUNT: usize = 256;
    writer.write_all(&vec![0u8; HEADER_SIZE * HEADER_COUNT])?;

    // Track current position for writing variable data
    let mut current_pos = writer.stream_position()?;

    // Prepare headers with calculated offsets
    let mut final_headers = Vec::with_capacity(HEADER_COUNT);

    for entry in &mh2o.entries {
        let mut header = entry.header;

        // Write instances if present (check instances array, not header offset)
        if !entry.instances.is_empty() {
            header.offset_instances = (current_pos - data_start) as u32;
            writer.seek(SeekFrom::Start(current_pos))?;

            // Clone instances so we can update offsets before writing
            let mut instances_with_offsets = entry.instances.clone();

            // Calculate vertex data and exists bitmap offsets
            // Instances are written first, then vertex data + bitmaps for all instances
            let instances_size = instances_with_offsets.len() * 24; // Mh2oInstance::SIZE = 24
            let mut vertex_data_offset = current_pos - data_start + instances_size as u64;

            for (idx, inst) in instances_with_offsets.iter_mut().enumerate() {
                // Write exists bitmap if present
                if let Some(_bitmap) = entry.exists_bitmaps.get(idx).and_then(|b| b.as_ref()) {
                    inst.offset_exists_bitmap = vertex_data_offset as u32;
                    vertex_data_offset += 8; // u64 bitmap = 8 bytes
                } else {
                    inst.offset_exists_bitmap = 0;
                }

                // Write vertex data if present
                if let Some(vertex_data) = entry.vertex_data.get(idx).and_then(|v| v.as_ref()) {
                    inst.offset_vertex_data = vertex_data_offset as u32;
                    vertex_data_offset += vertex_data.byte_size() as u64;
                } else {
                    inst.offset_vertex_data = 0;
                }
            }

            // Write instances with updated offsets
            for instance in &instances_with_offsets {
                instance.write_le(writer).map_err(|e| {
                    AdtError::BinrwError(format!("Failed to write MH2O instance: {e}"))
                })?;
            }

            current_pos = writer.stream_position()?;

            // Write vertex data and exists bitmaps
            for idx in 0..entry.instances.len() {
                // Write exists bitmap if present
                if let Some(bitmap) = entry.exists_bitmaps.get(idx).and_then(|b| b.as_ref()) {
                    writer.write_all(&bitmap.to_le_bytes())?;
                    current_pos = writer.stream_position()?;
                }

                // Write vertex data if present
                if let Some(vertex_data) = entry.vertex_data.get(idx).and_then(|v| v.as_ref()) {
                    use crate::chunks::mh2o::VertexDataArray;

                    match vertex_data {
                        VertexDataArray::HeightDepth(vertices) => {
                            for vertex in vertices.as_ref().iter().filter_map(|v| v.as_ref()) {
                                vertex.write_le(writer).map_err(|e| {
                                    AdtError::BinrwError(format!(
                                        "Failed to write HeightDepth vertex: {e}"
                                    ))
                                })?;
                            }
                        }
                        VertexDataArray::HeightUv(vertices) => {
                            for vertex in vertices.as_ref().iter().filter_map(|v| v.as_ref()) {
                                vertex.write_le(writer).map_err(|e| {
                                    AdtError::BinrwError(format!(
                                        "Failed to write HeightUv vertex: {e}"
                                    ))
                                })?;
                            }
                        }
                        VertexDataArray::DepthOnly(vertices) => {
                            for vertex in vertices.as_ref().iter().filter_map(|v| v.as_ref()) {
                                vertex.write_le(writer).map_err(|e| {
                                    AdtError::BinrwError(format!(
                                        "Failed to write DepthOnly vertex: {e}"
                                    ))
                                })?;
                            }
                        }
                        VertexDataArray::HeightUvDepth(vertices) => {
                            for vertex in vertices.as_ref().iter().filter_map(|v| v.as_ref()) {
                                vertex.write_le(writer).map_err(|e| {
                                    AdtError::BinrwError(format!(
                                        "Failed to write HeightUvDepth vertex: {e}"
                                    ))
                                })?;
                            }
                        }
                    }
                    current_pos = writer.stream_position()?;
                }
            }

            header.layer_count = entry.instances.len() as u32;
        } else {
            header.offset_instances = 0;
            header.layer_count = 0;
        }

        // Write attributes if present
        if let Some(attrs) = &entry.attributes {
            header.offset_attributes = (current_pos - data_start) as u32;
            writer.seek(SeekFrom::Start(current_pos))?;

            attrs.write_le(writer).map_err(|e| {
                AdtError::BinrwError(format!("Failed to write MH2O attributes: {e}"))
            })?;

            current_pos = writer.stream_position()?;
        } else {
            header.offset_attributes = 0;
        }

        final_headers.push(header);
    }

    // Pad to 256 headers if needed
    while final_headers.len() < HEADER_COUNT {
        final_headers.push(Mh2oHeader::default());
    }

    // Seek back to write headers with correct offsets
    writer.seek(SeekFrom::Start(headers_start))?;
    for header in &final_headers {
        header
            .write_le(writer)
            .map_err(|e| AdtError::BinrwError(format!("Failed to write MH2O header: {e}")))?;
    }

    // Calculate total chunk size
    let data_end = current_pos;
    let chunk_size = (data_end - data_start) as u32;

    // Seek back and write size
    writer.seek(SeekFrom::Start(size_pos))?;
    writer.write_all(&chunk_size.to_le_bytes())?;

    // Seek to end of chunk
    writer.seek(SeekFrom::Start(data_end))?;

    Ok(())
}

/// Create MTEX chunk from texture filenames.
fn create_mtex_chunk(textures: &[String]) -> MtexChunk {
    MtexChunk {
        filenames: textures.to_vec(),
    }
}

/// Create MMDX chunk from M2 model filenames.
fn create_mmdx_chunk(models: &[String]) -> MmdxChunk {
    MmdxChunk {
        filenames: models.to_vec(),
    }
}

/// Create MMID chunk from M2 model filenames.
///
/// Calculates byte offsets for each filename in the MMDX chunk.
fn create_mmid_chunk(models: &[String]) -> MmidChunk {
    let mut offsets = Vec::new();
    let mut current_offset = 0u32;

    for model in models {
        offsets.push(current_offset);
        current_offset += model.len() as u32 + 1; // +1 for null terminator
    }

    MmidChunk { offsets }
}

/// Create MWMO chunk from WMO filenames.
fn create_mwmo_chunk(wmos: &[String]) -> MwmoChunk {
    MwmoChunk {
        filenames: wmos.to_vec(),
    }
}

/// Create MWID chunk from WMO filenames.
///
/// Calculates byte offsets for each filename in the MWMO chunk.
fn create_mwid_chunk(wmos: &[String]) -> MwidChunk {
    let mut offsets = Vec::new();
    let mut current_offset = 0u32;

    for wmo in wmos {
        offsets.push(current_offset);
        current_offset += wmo.len() as u32 + 1; // +1 for null terminator
    }

    MwidChunk { offsets }
}

/// Create MDDF chunk from doodad placements.
fn create_mddf_chunk(placements: &[crate::chunks::DoodadPlacement]) -> MddfChunk {
    MddfChunk {
        placements: placements.to_vec(),
    }
}

/// Create MODF chunk from WMO placements.
fn create_modf_chunk(placements: &[crate::chunks::WmoPlacement]) -> ModfChunk {
    ModfChunk {
        placements: placements.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AdtVersion;
    use crate::builder::AdtBuilder;
    use crate::chunks::{McnkChunk, McnkFlags, McnkHeader};
    use std::io::{Cursor, Read};

    /// Create minimal MCNK chunk for testing.
    fn create_minimal_mcnk() -> McnkChunk {
        McnkChunk {
            header: McnkHeader {
                flags: McnkFlags { value: 0 },
                index_x: 0,
                index_y: 0,
                n_layers: 0,
                n_doodad_refs: 0,
                multipurpose_field: McnkHeader::multipurpose_from_offsets(0, 0),
                ofs_layer: 0,
                ofs_refs: 0,
                ofs_alpha: 0,
                size_alpha: 0,
                ofs_shadow: 0,
                size_shadow: 0,
                area_id: 0,
                n_map_obj_refs: 0,
                holes_low_res: 0,
                unknown_but_used: 0,
                pred_tex: [0; 8],
                no_effect_doodad: [0; 8],
                unknown_8bytes: [0; 8],
                ofs_snd_emitters: 0,
                n_snd_emitters: 0,
                ofs_liquid: 0,
                size_liquid: 0,
                position: [0.0, 0.0, 0.0],
                ofs_mccv: 0,
                ofs_mclv: 0,
                unused: 0,
                _padding: [0; 8],
            },
            heights: None,
            normals: None,
            layers: None,
            materials: None,
            refs: None,
            doodad_refs: None,
            wmo_refs: None,
            alpha: None,
            shadow: None,
            vertex_colors: None,
            vertex_lighting: None,
            sound_emitters: None,
            liquid: None,
            doodad_disable: None,
            blend_batches: None,
        }
    }

    #[test]
    fn test_serialize_minimal_adt() {
        let adt = AdtBuilder::new()
            .with_version(AdtVersion::VanillaEarly)
            .add_texture("terrain/grass.blp")
            .add_mcnk_chunk(create_minimal_mcnk())
            .build()
            .expect("Failed to build ADT");

        let mut buffer = Cursor::new(Vec::new());
        serialize_to_writer(&adt, &mut buffer).expect("Failed to serialize ADT");

        let data = buffer.into_inner();

        // Verify file starts with MVER chunk
        assert_eq!(&data[0..4], b"REVM"); // MVER reversed
        assert_eq!(u32::from_le_bytes([data[4], data[5], data[6], data[7]]), 4); // Size = 4
        assert_eq!(
            u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
            18
        ); // Version = 18
    }

    #[test]
    fn test_chunk_ordering() {
        let adt = AdtBuilder::new()
            .add_texture("test.blp")
            .add_mcnk_chunk(create_minimal_mcnk())
            .build()
            .expect("Failed to build ADT");

        let mut buffer = Cursor::new(Vec::new());
        serialize_to_writer(&adt, &mut buffer).expect("Failed to serialize ADT");

        let data = buffer.into_inner();
        let mut cursor = Cursor::new(&data);

        // Verify chunk order by reading magic bytes
        let mut read_magic = || {
            let mut magic = [0u8; 4];
            cursor.read_exact(&mut magic).unwrap();
            let mut size_bytes = [0u8; 4];
            cursor.read_exact(&mut size_bytes).unwrap();
            let size = u32::from_le_bytes(size_bytes);
            cursor.seek(SeekFrom::Current(size as i64)).unwrap();
            ChunkId(magic)
        };

        assert_eq!(read_magic(), ChunkId::MVER);
        assert_eq!(read_magic(), ChunkId::MHDR);
        assert_eq!(read_magic(), ChunkId::MCIN);
        assert_eq!(read_magic(), ChunkId::MTEX);
        assert_eq!(read_magic(), ChunkId::MMDX);
        assert_eq!(read_magic(), ChunkId::MMID);
        assert_eq!(read_magic(), ChunkId::MWMO);
        assert_eq!(read_magic(), ChunkId::MWID);
        assert_eq!(read_magic(), ChunkId::MDDF);
        assert_eq!(read_magic(), ChunkId::MODF);
    }

    #[test]
    fn test_offset_calculation() {
        let adt = AdtBuilder::new()
            .add_texture("test.blp")
            .add_mcnk_chunk(create_minimal_mcnk())
            .build()
            .expect("Failed to build ADT");

        let mut buffer = Cursor::new(Vec::new());
        serialize_to_writer(&adt, &mut buffer).expect("Failed to serialize ADT");

        let data = buffer.into_inner();

        // Find MHDR chunk and verify offsets are non-zero
        let mhdr_offset = 12; // After MVER (4+4+4 bytes)
        let mhdr_data_offset = mhdr_offset + 8; // After MHDR header

        // Read MHDR offsets
        let mcin_offset = u32::from_le_bytes([
            data[mhdr_data_offset + 4],
            data[mhdr_data_offset + 5],
            data[mhdr_data_offset + 6],
            data[mhdr_data_offset + 7],
        ]);
        let mtex_offset = u32::from_le_bytes([
            data[mhdr_data_offset + 8],
            data[mhdr_data_offset + 9],
            data[mhdr_data_offset + 10],
            data[mhdr_data_offset + 11],
        ]);

        // Verify offsets are non-zero
        assert!(mcin_offset > 0, "MCIN offset should be non-zero");
        assert!(mtex_offset > 0, "MTEX offset should be non-zero");
    }

    #[test]
    fn test_mmid_offset_calculation() {
        let models = vec!["model1.m2".to_string(), "model2.m2".to_string()];
        let mmid = create_mmid_chunk(&models);

        assert_eq!(mmid.offsets.len(), 2);
        assert_eq!(mmid.offsets[0], 0);
        assert_eq!(mmid.offsets[1], 10); // "model1.m2\0" = 10 bytes
    }

    #[test]
    fn test_mwid_offset_calculation() {
        let wmos = vec!["building.wmo".to_string(), "castle.wmo".to_string()];
        let mwid = create_mwid_chunk(&wmos);

        assert_eq!(mwid.offsets.len(), 2);
        assert_eq!(mwid.offsets[0], 0);
        assert_eq!(mwid.offsets[1], 13); // "building.wmo\0" = 13 bytes
    }

    #[test]
    fn test_write_chunk() {
        let mut buffer = Cursor::new(Vec::new());
        let mver = MverChunk { version: 18 };

        write_chunk(&mut buffer, ChunkId::MVER, &mver).expect("Failed to write chunk");

        let data = buffer.into_inner();

        // Verify chunk header
        assert_eq!(&data[0..4], b"REVM"); // Magic
        assert_eq!(u32::from_le_bytes([data[4], data[5], data[6], data[7]]), 4); // Size
        assert_eq!(
            u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
            18
        ); // Version
    }

    #[test]
    fn test_mhdr_flags_no_optional_chunks() {
        let positions = ChunkPositions::default();
        let flags = calculate_mhdr_flags(&positions);
        assert_eq!(flags, 0);
    }

    #[test]
    fn test_mhdr_flags_with_mfbo() {
        let positions = ChunkPositions {
            mfbo: Some(1000),
            ..Default::default()
        };
        let flags = calculate_mhdr_flags(&positions);
        assert_eq!(flags, 0x01);
    }

    #[test]
    fn test_mhdr_flags_with_mh2o() {
        let positions = ChunkPositions {
            mh2o: Some(2000),
            ..Default::default()
        };
        let flags = calculate_mhdr_flags(&positions);
        assert_eq!(flags, 0x02);
    }

    #[test]
    fn test_mhdr_flags_with_both() {
        let positions = ChunkPositions {
            mfbo: Some(1000),
            mh2o: Some(2000),
            ..Default::default()
        };
        let flags = calculate_mhdr_flags(&positions);
        assert_eq!(flags, 0x03);
    }

    #[test]
    fn test_mh2o_serialization() {
        use crate::chunks::mh2o::{Mh2oAttributes, Mh2oEntry, Mh2oInstance};

        // Create MH2O chunk with water in one entry
        let mut entries = vec![Mh2oEntry::default(); 256];

        // Add water to entry 0 (first chunk)
        entries[0] = Mh2oEntry {
            header: Mh2oHeader {
                offset_instances: 0, // Will be calculated during serialization
                layer_count: 1,
                offset_attributes: 0, // Will be calculated
            },
            instances: vec![Mh2oInstance {
                liquid_type: 5,          // Water
                liquid_object_or_lvf: 0, // LVF 0
                min_height_level: 100.0,
                max_height_level: 105.0,
                x_offset: 0,
                y_offset: 0,
                width: 8,
                height: 8,
                offset_exists_bitmap: 0, // No bitmap (full coverage)
                offset_vertex_data: 0,   // No vertex data
            }],
            vertex_data: vec![None], // No vertex data (uses min/max height)
            exists_bitmaps: vec![None], // No exists bitmap (full coverage)
            attributes: Some(Mh2oAttributes {
                fishable: 0xFFFFFFFFFFFFFFFF, // All fishable
                deep: 0x0000000000000000,     // No deep water
            }),
        };

        let mh2o = Mh2oChunk { entries };

        // Serialize
        let mut buffer = Cursor::new(Vec::new());
        write_mh2o_chunk(&mut buffer, &mh2o).expect("Failed to write MH2O chunk");

        let data = buffer.into_inner();

        // Verify chunk header
        assert_eq!(&data[0..4], b"O2HM"); // Magic (reversed)
        let chunk_size = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);

        // Size should be: 256 headers (3072 bytes) + 1 instance (24 bytes) + 1 attributes (16 bytes) = 3112 bytes
        assert_eq!(chunk_size, 3112);

        // Verify first header has offsets
        let header_offset = 8; // After chunk header
        let offset_instances = u32::from_le_bytes([
            data[header_offset],
            data[header_offset + 1],
            data[header_offset + 2],
            data[header_offset + 3],
        ]);
        let layer_count = u32::from_le_bytes([
            data[header_offset + 4],
            data[header_offset + 5],
            data[header_offset + 6],
            data[header_offset + 7],
        ]);
        let offset_attributes = u32::from_le_bytes([
            data[header_offset + 8],
            data[header_offset + 9],
            data[header_offset + 10],
            data[header_offset + 11],
        ]);

        // First header should point to data after all 256 headers
        assert_eq!(offset_instances, 3072); // After 256 * 12 bytes of headers
        assert_eq!(layer_count, 1);
        assert_eq!(offset_attributes, 3096); // After headers + instance (3072 + 24)

        // Verify second header is empty
        let second_header_offset = header_offset + 12;
        let second_offset_instances = u32::from_le_bytes([
            data[second_header_offset],
            data[second_header_offset + 1],
            data[second_header_offset + 2],
            data[second_header_offset + 3],
        ]);
        let second_layer_count = u32::from_le_bytes([
            data[second_header_offset + 4],
            data[second_header_offset + 5],
            data[second_header_offset + 6],
            data[second_header_offset + 7],
        ]);

        assert_eq!(second_offset_instances, 0);
        assert_eq!(second_layer_count, 0);
    }
}
