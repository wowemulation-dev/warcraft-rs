// liquid_converter.rs - Convert between MCLQ and MH2O formats

use crate::chunk::*;
use crate::error::Result;
use crate::mcnk_subchunks::*;
use crate::mh2o::Mh2oChunk as AdvancedMh2oChunk;
use crate::mh2o::{
    Mh2oEntry, Mh2oHeader, Mh2oInstance, Mh2oRenderMask, WaterLevelData, WaterVertex,
    WaterVertexData,
};

/// Convert from MCLQ (pre-WotLK) to MH2O (WotLK+) format
pub fn convert_mclq_to_mh2o(
    mclq_chunks: &[MclqSubchunk],
    chunk_positions: &[McnkChunk],
) -> Result<AdvancedMh2oChunk> {
    // Each MCNK can have one MCLQ chunk, and there are 256 MCNKs
    // MH2O has 256 entries, one for each MCNK
    let mut mh2o_entries = Vec::with_capacity(256);

    // Process each MCNK/MCLQ pair
    for (i, chunk) in chunk_positions.iter().enumerate() {
        // Get the corresponding MCLQ if it exists
        let mclq = if i < mclq_chunks.len() {
            Some(&mclq_chunks[i])
        } else {
            None
        };

        // Create the MH2O entry for this chunk
        let entry = if let Some(mclq) = mclq {
            // MCLQ exists, convert it
            convert_chunk_mclq_to_mh2o(mclq, chunk, i)
        } else {
            // No MCLQ for this chunk, create an empty entry
            create_empty_mh2o_entry()
        };

        mh2o_entries.push(entry);
    }

    // Fill any remaining entries
    while mh2o_entries.len() < 256 {
        mh2o_entries.push(create_empty_mh2o_entry());
    }

    Ok(AdvancedMh2oChunk {
        chunks: mh2o_entries,
    })
}

/// Convert a single MCLQ chunk to MH2O format
fn convert_chunk_mclq_to_mh2o(
    mclq: &MclqSubchunk,
    chunk: &McnkChunk,
    _chunk_index: usize,
) -> Mh2oEntry {
    // Create the header for this entry
    let header = Mh2oHeader {
        offset_instances: 0, // Will be filled during serialization
        layer_count: if mclq.vertices.is_empty() { 0 } else { 1 },
        offset_render_mask: 0, // Will be filled during serialization
    };

    // If there's no liquid data, return an empty entry
    if mclq.vertices.is_empty() {
        return Mh2oEntry {
            header,
            instances: Vec::new(),
            render_mask: None,
        };
    }

    // Calculate min/max heights
    let (min_height, max_height) = calculate_min_max_heights(mclq, chunk);

    // Create water instance
    let instance = create_water_instance(mclq, min_height, max_height);

    // Create render mask
    let render_mask = create_render_mask(mclq);

    Mh2oEntry {
        header,
        instances: vec![instance],
        render_mask,
    }
}

/// Create an empty MH2O entry
fn create_empty_mh2o_entry() -> Mh2oEntry {
    Mh2oEntry {
        header: Mh2oHeader {
            offset_instances: 0,
            layer_count: 0,
            offset_render_mask: 0,
        },
        instances: Vec::new(),
        render_mask: None,
    }
}

/// Calculate minimum and maximum heights from MCLQ
fn calculate_min_max_heights(mclq: &MclqSubchunk, _chunk: &McnkChunk) -> (f32, f32) {
    // Start with the base height
    let mut min_height = mclq.base_height;
    let mut max_height = mclq.base_height;

    // Factor in vertex depths
    for vertex in &mclq.vertices {
        let height = mclq.base_height - vertex.depth;
        min_height = min_height.min(height);
        max_height = max_height.max(height);
    }

    (min_height, max_height)
}

/// Create a water instance from MCLQ data
fn create_water_instance(mclq: &MclqSubchunk, min_height: f32, max_height: f32) -> Mh2oInstance {
    // Extract liquid type from vertices (use the first one)
    let liquid_type = if !mclq.vertices.is_empty() {
        mclq.vertices[0].liquid_id
    } else {
        0 // Default liquid type
    };

    // Extract flow data from vertices
    let vertex_data = if mclq.x_vertices > 0 && mclq.y_vertices > 0 {
        // Create water vertices
        let mut vertices = Vec::with_capacity(mclq.vertices.len());

        for vertex in &mclq.vertices {
            vertices.push(WaterVertex {
                depth: vertex.depth,
                flow: [0, 0], // MCLQ doesn't have flow data
            });
        }

        Some(WaterVertexData {
            offset_vertex_data: 0, // Will be filled during serialization
            x_vertices: mclq.x_vertices as u8,
            y_vertices: mclq.y_vertices as u8,
            vertices: Some(vertices),
        })
    } else {
        None
    };

    // Determine level data type
    let level_data = if vertex_data.is_some() {
        // Variable height water
        WaterLevelData::Variable {
            min_height,
            max_height,
            offset_height_map: 0, // Will be filled during serialization
            heights: None,        // Will be generated during serialization
        }
    } else {
        // Uniform height water
        WaterLevelData::Uniform {
            min_height,
            max_height,
        }
    };

    Mh2oInstance {
        liquid_type,
        liquid_object: 0, // Default liquid object ID
        level_data,
        vertex_data,
        attributes: Vec::new(), // No attributes in older versions
    }
}

/// Create a render mask from MCLQ data
fn create_render_mask(mclq: &MclqSubchunk) -> Option<Mh2oRenderMask> {
    if mclq.vertices.is_empty() {
        return None;
    }

    // In MCLQ, we don't have explicit render masks
    // We need to derive them from the vertex data

    // The render mask is an 8x8 grid, 1 bit per cell
    // We'll set bits for areas that have liquid
    let mut mask = [0u8; 8];

    // If MCLQ exists, we'll assume the entire chunk has liquid
    // A more accurate conversion would look at vertex depths
    // and only set bits for areas with non-zero depth

    // Set all bits to indicate liquid is present everywhere
    mask.fill(0xFF);

    Some(Mh2oRenderMask { mask })
}

/// Convert from MH2O (WotLK+) to MCLQ (pre-WotLK) format
pub fn convert_mh2o_to_mclq(
    mh2o: &AdvancedMh2oChunk,
    chunk_positions: &[McnkChunk],
) -> Result<Vec<MclqSubchunk>> {
    let mut mclq_chunks = Vec::with_capacity(chunk_positions.len());

    // Process each MCNK/MH2O pair
    for (i, chunk) in chunk_positions.iter().enumerate() {
        // Get the corresponding MH2O entry
        let mh2o_entry = if i < mh2o.chunks.len() {
            &mh2o.chunks[i]
        } else {
            // No MH2O entry for this chunk
            mclq_chunks.push(create_empty_mclq());
            continue;
        };

        // Convert the MH2O entry to MCLQ
        let mclq = convert_chunk_mh2o_to_mclq(mh2o_entry, chunk);
        mclq_chunks.push(mclq);
    }

    Ok(mclq_chunks)
}

/// Convert a single MH2O entry to MCLQ format
fn convert_chunk_mh2o_to_mclq(mh2o_entry: &Mh2oEntry, _chunk: &McnkChunk) -> MclqSubchunk {
    // If there's no liquid data, return an empty MCLQ
    if mh2o_entry.instances.is_empty() {
        return create_empty_mclq();
    }

    // Get the first liquid instance
    let instance = &mh2o_entry.instances[0];

    // Extract height information
    let (base_height, _min_height, _max_height) = match &instance.level_data {
        WaterLevelData::Uniform {
            min_height,
            max_height,
        } => (*max_height, *min_height, *max_height),
        WaterLevelData::Variable {
            min_height,
            max_height,
            ..
        } => (*max_height, *min_height, *max_height),
    };

    // Extract vertex data
    let (x_vertices, y_vertices, vertices) = if let Some(ref vertex_data) = instance.vertex_data {
        if let Some(ref vertices) = vertex_data.vertices {
            (
                vertex_data.x_vertices as u32,
                vertex_data.y_vertices as u32,
                vertices.as_slice(),
            )
        } else {
            (0, 0, &[] as &[WaterVertex])
        }
    } else {
        (0, 0, &[] as &[WaterVertex])
    };

    // Convert water vertices to MCLQ format
    let mut mclq_vertices = Vec::with_capacity(vertices.len());

    for vertex in vertices {
        mclq_vertices.push(LiquidVertex {
            depth: vertex.depth,
            liquid_id: instance.liquid_type,
            flags: 0, // No flags in newer versions
        });
    }

    // Create the MCLQ
    MclqSubchunk {
        x_vertices,
        y_vertices,
        base_height,
        vertices: mclq_vertices,
    }
}

/// Create an empty MCLQ chunk
fn create_empty_mclq() -> MclqSubchunk {
    MclqSubchunk {
        x_vertices: 0,
        y_vertices: 0,
        base_height: 0.0,
        vertices: Vec::new(),
    }
}
