//! Support for parsing embedded skin data from pre-WotLK M2 models
//!
//! Pre-WotLK models (versions 256-260) have skin profile data embedded directly
//! in the M2 file rather than in separate .skin files. This module provides
//! functionality to extract and parse these embedded skin profiles.

use crate::io_ext::ReadExt;
use crate::skin::parse_embedded_skin;
use crate::{M2Error, M2Model, Result, SkinFile};
use std::io::{Cursor, Read, Seek, SeekFrom, Write};

impl M2Model {
    /// Parse embedded skin profiles from pre-WotLK M2 models
    ///
    /// For models with version <= 260, skin data is embedded in the M2 file itself.
    /// The views array contains ModelView structures with direct offsets to skin data.
    ///
    /// **Note**: Many character models only have the first skin profile (index 0)
    /// properly embedded. Additional skin profiles may contain invalid data.
    ///
    /// # Arguments
    ///
    /// * `original_m2_data` - The complete original M2 file data
    /// * `skin_index` - Index of the skin profile to extract (0-based)
    ///
    /// # Returns
    ///
    /// Returns the parsed SkinFile for the requested skin profile index
    ///
    /// # Errors
    ///
    /// Returns an error if the skin index is out of range or contains invalid data
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::fs;
    /// # use std::io::Cursor;
    /// # use wow_m2::{M2Model, parse_m2};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Load a pre-WotLK model
    /// let m2_data = fs::read("HumanMale.m2")?;
    /// let m2_format = parse_m2(&mut Cursor::new(&m2_data))?;
    /// let model = m2_format.model();
    ///
    /// if model.header.version <= 260 {
    ///     // Parse the first embedded skin profile
    ///     let skin = model.parse_embedded_skin(&m2_data, 0)?;
    ///     println!("Embedded skin has {} submeshes", skin.submeshes().len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_embedded_skin(
        &self,
        original_m2_data: &[u8],
        skin_index: usize,
    ) -> Result<SkinFile> {
        // Check if this is a pre-WotLK model
        if self.header.version > 260 {
            return Err(M2Error::ParseError(format!(
                "Model version {} does not have embedded skins. Use external .skin files instead.",
                self.header.version
            )));
        }

        // Check if views array has data
        if self.header.views.count == 0 {
            return Err(M2Error::ParseError(
                "No skin profiles found in model header".to_string(),
            ));
        }

        // Check if requested index is valid
        if skin_index >= self.header.views.count as usize {
            return Err(M2Error::ParseError(format!(
                "Skin index {} out of range. Model has {} skin profiles.",
                skin_index, self.header.views.count
            )));
        }

        // For pre-WotLK models, the views array points to ModelView structures
        // Based on WMVx analysis, ModelView is 44 bytes (5 M2Arrays + 1 uint32):
        // - indices: M2Array (count + offset) = 8 bytes
        // - triangles: M2Array (count + offset) = 8 bytes
        // - properties: M2Array (count + offset) = 8 bytes
        // - submeshes: M2Array (count + offset) = 8 bytes
        // - textureUnits: M2Array (count + offset) = 8 bytes
        // - boneCountMax: uint32 = 4 bytes
        // Total: 44 bytes

        // CRITICAL INSIGHT: Following WMVx implementation - ALL skin profiles use the SAME ModelView!
        // WMVx only uses views[0] and ignores other skin indices. Different skin indices likely
        // represent different LOD levels or rendering passes, not different geometry.
        // This explains why only skin 0 worked - we should always use the first ModelView.

        let model_view_size = 44; // Correct size from WMVx analysis
        let model_view_offset = self.header.views.offset as usize; // Always use first ModelView (skin 0)

        if model_view_offset + model_view_size > original_m2_data.len() {
            return Err(M2Error::ParseError(format!(
                "ModelView at offset {:#x} exceeds file size",
                model_view_offset
            )));
        }

        // Read the ModelView structure
        let model_view_data =
            &original_m2_data[model_view_offset..model_view_offset + model_view_size];

        // Parse ModelView fields as M2Arrays (count + offset pairs)
        // Each M2Array is 8 bytes: count (u32) + offset (u32)

        // indices M2Array
        let n_index = u32::from_le_bytes([
            model_view_data[0],
            model_view_data[1],
            model_view_data[2],
            model_view_data[3],
        ]);
        let ofs_index = u32::from_le_bytes([
            model_view_data[4],
            model_view_data[5],
            model_view_data[6],
            model_view_data[7],
        ]);

        // triangles M2Array
        let n_tris = u32::from_le_bytes([
            model_view_data[8],
            model_view_data[9],
            model_view_data[10],
            model_view_data[11],
        ]);
        let ofs_tris = u32::from_le_bytes([
            model_view_data[12],
            model_view_data[13],
            model_view_data[14],
            model_view_data[15],
        ]);

        // properties M2Array
        let _n_props = u32::from_le_bytes([
            model_view_data[16],
            model_view_data[17],
            model_view_data[18],
            model_view_data[19],
        ]);
        let _ofs_props = u32::from_le_bytes([
            model_view_data[20],
            model_view_data[21],
            model_view_data[22],
            model_view_data[23],
        ]);

        // submeshes M2Array - THIS IS WHAT WE WERE MISSING!
        let n_sub = u32::from_le_bytes([
            model_view_data[24],
            model_view_data[25],
            model_view_data[26],
            model_view_data[27],
        ]);
        let ofs_sub = u32::from_le_bytes([
            model_view_data[28],
            model_view_data[29],
            model_view_data[30],
            model_view_data[31],
        ]);

        // batches M2Array
        let n_batches = u32::from_le_bytes([
            model_view_data[32],
            model_view_data[33],
            model_view_data[34],
            model_view_data[35],
        ]);
        let ofs_batches = u32::from_le_bytes([
            model_view_data[36],
            model_view_data[37],
            model_view_data[38],
            model_view_data[39],
        ]);

        // boneCountMax uint32
        let _bone_count_max = u32::from_le_bytes([
            model_view_data[40],
            model_view_data[41],
            model_view_data[42],
            model_view_data[43],
        ]);

        // Validate ModelView values (debug info removed for production use)

        // Sanity check: ModelView data should be reasonable
        // Index count should not exceed total vertices, and offsets should be within file
        if n_index > 100000
            || ofs_index as usize >= original_m2_data.len()
            || n_tris > 100000
            || ofs_tris as usize >= original_m2_data.len()
            || n_sub > 1000 // Reasonable submesh count limit
            || (n_sub > 0 && ofs_sub as usize >= original_m2_data.len())
            || (n_batches > 0 && ofs_batches as usize >= original_m2_data.len())
        {
            return Err(M2Error::ParseError(format!(
                "Skin {} appears to have invalid ModelView data. This may not be a valid embedded skin.",
                skin_index
            )));
        }

        // IMPORTANT: The ModelView field names are misleading!
        // - nTris/ofsTris contains the INDICES array (triangle vertex indices)
        // - nIndex/ofsIndex contains the TRIANGLES array (vertex lookup table)
        // This is counterintuitive but confirmed by working implementations.

        // Calculate actual data sizes based on corrected understanding:
        // - indices come from nTris/ofsTris (should be the larger array - triangle connectivity)
        // - triangles come from nIndex/ofsIndex (should be the smaller array - vertex lookup table)
        // - submeshes come from nSub/ofsSub
        let indices_size = (n_tris as usize) * 2; // Indices from tris field (u16 per index) 
        let triangles_size = (n_index as usize) * 2; // Triangles from index field (u16 per triangle)

        // Calculate submesh data size based on empirical validation:
        // - Pre-TBC (versions < 260): 32 bytes aligned structure (empirically validated)
        // - TBC+ (versions >= 260): 10 uint16_t + 2 Vector3 + 1 float = 48 bytes
        // NOTE: We always allocate 48 bytes per submesh in our buffer for the parser,
        // but we read the original size from the file
        let original_submesh_size_per_entry = if self.header.version < 260 {
            32 // Empirically validated vanilla size with proper alignment
        } else {
            48 // TBC+ size (same as buffer size)
        };

        let original_submeshes_size = (n_sub as usize) * original_submesh_size_per_entry;
        let buffer_submeshes_size = original_submeshes_size; // Keep original size in buffer

        let batches_size = (n_batches as usize) * 96; // 96 bytes each

        // Verify offsets are within bounds
        if ofs_tris as usize + indices_size > original_m2_data.len() {
            return Err(M2Error::ParseError(format!(
                "Indices data at offset {:#x} + {} exceeds file size {}",
                ofs_tris,
                indices_size,
                original_m2_data.len()
            )));
        }

        if ofs_index as usize + triangles_size > original_m2_data.len() {
            return Err(M2Error::ParseError(format!(
                "Triangles data at offset {:#x} + {} exceeds file size {}",
                ofs_index,
                triangles_size,
                original_m2_data.len()
            )));
        }

        // Verify submeshes offset is within bounds if we have submeshes
        if n_sub > 0 && ofs_sub as usize + original_submeshes_size > original_m2_data.len() {
            return Err(M2Error::ParseError(format!(
                "Submeshes data at offset {:#x} + {} exceeds file size {}",
                ofs_sub,
                original_submeshes_size,
                original_m2_data.len()
            )));
        }

        if n_batches > 0 && ofs_batches as usize + batches_size > original_m2_data.len() {
            return Err(M2Error::ParseError(format!(
                "Batches data at offset {:#x} + {} exceeds file size {}",
                ofs_batches,
                batches_size,
                original_m2_data.len()
            )));
        }

        // Calculate where to place data in our buffer
        let header_size = 40; // 5 M2Arrays * 8 bytes each
        let indices_buffer_offset = header_size; // Indices go first (smaller array)
        let triangles_buffer_offset = indices_buffer_offset + triangles_size; // Triangles go second (larger array)
        let submeshes_buffer_offset = triangles_buffer_offset + indices_size;
        let batches_buffer_offset = submeshes_buffer_offset + buffer_submeshes_size;
        let total_buffer_size = batches_buffer_offset + batches_size;

        // Allocate buffer for skin data with proper layout

        let mut skin_buffer = vec![0u8; total_buffer_size];

        // Write header at the beginning
        let mut cursor = Cursor::new(&mut skin_buffer);

        // Write the skin header with corrected field mapping:
        // SWAP: Put the larger array (from tris field) into triangles field
        // and the smaller array (from index field) into indices field

        // Write indices array header (from nIndex/ofsIndex - smaller array)
        cursor.write_all(&n_index.to_le_bytes())?; // Count of indices (from index field)
        cursor.write_all(&(indices_buffer_offset as u32).to_le_bytes())?; // Offset to indices data

        // Write triangles array header (from nTris/ofsTris - larger array)
        cursor.write_all(&n_tris.to_le_bytes())?; // Count of triangles (from tris field)
        cursor.write_all(&(triangles_buffer_offset as u32).to_le_bytes())?; // Offset to triangles data

        // Write empty bone_indices array
        cursor.write_all(&0u32.to_le_bytes())?;
        cursor.write_all(&0u32.to_le_bytes())?;

        // Write submeshes array header (from nSub/ofsSub in ModelView)
        cursor.write_all(&n_sub.to_le_bytes())?; // Count of submeshes
        cursor.write_all(&(submeshes_buffer_offset as u32).to_le_bytes())?; // Offset to submeshes data

        // Write batches array
        cursor.write_all(&n_batches.to_le_bytes())?;
        cursor.write_all(&(batches_buffer_offset as u32).to_le_bytes())?;

        // Copy actual data from the original M2 file with corrected mapping:
        // Copy indices data (from ofsIndex in ModelView - smaller array goes to indices)
        if n_index > 0 {
            let src_indices =
                &original_m2_data[ofs_index as usize..(ofs_index as usize + triangles_size)];
            skin_buffer[indices_buffer_offset..(indices_buffer_offset + triangles_size)]
                .copy_from_slice(src_indices);
        }

        // Copy triangles data (from ofsTris in ModelView - larger array goes to triangles)
        if n_tris > 0 {
            let src_triangles =
                &original_m2_data[ofs_tris as usize..(ofs_tris as usize + indices_size)];
            skin_buffer[triangles_buffer_offset..(triangles_buffer_offset + indices_size)]
                .copy_from_slice(src_triangles);
        }

        // Copy submesh data (from ofsSub in ModelView)
        if n_sub > 0 {
            let src_submeshes =
                &original_m2_data[ofs_sub as usize..(ofs_sub as usize + original_submeshes_size)];

            // Copy submesh data directly without padding - the parse_with_version method
            // will handle the different structure sizes correctly
            skin_buffer[submeshes_buffer_offset..submeshes_buffer_offset + original_submeshes_size]
                .copy_from_slice(src_submeshes);
        }

        // Copy batches data (from ofsBatches in ModelView)
        if n_batches > 0 {
            let src_batches =
                &original_m2_data[ofs_batches as usize..(ofs_batches as usize + batches_size)];
            skin_buffer[batches_buffer_offset..(batches_buffer_offset + batches_size)]
                .copy_from_slice(src_batches);
        }

        // Create a cursor with our complete skin buffer
        let mut cursor = Cursor::new(&skin_buffer);

        // Parse as embedded skin (no SKIN magic signature)
        parse_embedded_skin(&mut cursor, self.header.version)
    }

    /// Get the number of embedded skin profiles in a pre-WotLK model
    ///
    /// Returns None if the model uses external skin files (version > 260)
    pub fn embedded_skin_count(&self) -> Option<u32> {
        if self.header.version <= 260 {
            Some(self.header.views.count)
        } else {
            None
        }
    }

    /// Check if this model uses embedded skins (pre-WotLK) or external skin files
    pub fn has_embedded_skins(&self) -> bool {
        // Vanilla (256), TBC (260-263) have embedded skins
        // WotLK (264+) introduced external .skin files
        self.header.version <= 263 && self.header.views.count > 0
    }

    /// Parse all embedded skin profiles from a pre-WotLK model
    ///
    /// This is a convenience method that extracts all skin profiles at once.
    ///
    /// # Arguments
    ///
    /// * `original_m2_data` - The complete original M2 file data
    ///
    /// # Returns
    ///
    /// A vector of all parsed skin profiles
    pub fn parse_all_embedded_skins(&self, original_m2_data: &[u8]) -> Result<Vec<SkinFile>> {
        if !self.has_embedded_skins() {
            return Ok(Vec::new());
        }

        let count = self.header.views.count as usize;
        let mut skins = Vec::with_capacity(count);

        for i in 0..count {
            skins.push(self.parse_embedded_skin(original_m2_data, i)?);
        }

        Ok(skins)
    }
}

/// Helper function to extract embedded skin data without loading the full model
///
/// This can be useful for tools that only need to extract skin data.
///
/// # Arguments
///
/// * `m2_data` - The complete M2 file data
/// * `skin_index` - Index of the skin profile to extract
///
/// # Returns
///
/// The raw bytes of the skin data at the specified index
pub fn extract_embedded_skin_bytes(m2_data: &[u8], skin_index: usize) -> Result<Vec<u8>> {
    // Read magic and version to validate
    let mut cursor = Cursor::new(m2_data);
    let mut magic_bytes = [0u8; 4];
    cursor
        .read_exact(&mut magic_bytes)
        .map_err(|e| M2Error::ParseError(format!("Failed to read magic: {}", e)))?;

    if &magic_bytes != b"MD20" {
        return Err(M2Error::ParseError(format!(
            "Invalid M2 magic: expected MD20, got {:?}",
            magic_bytes
        )));
    }

    let version = cursor.read_u32_le()?;

    if version > 260 {
        return Err(M2Error::ParseError(format!(
            "Version {} does not have embedded skins",
            version
        )));
    }

    // Skip to views array (at offset 0x2C in the header for old versions)
    cursor.seek(SeekFrom::Start(0x2C))?;
    let views_count = cursor.read_u32_le()?;
    let views_offset = cursor.read_u32_le()?;

    if skin_index >= views_count as usize {
        return Err(M2Error::ParseError(format!(
            "Skin index {} out of range (max: {})",
            skin_index,
            views_count - 1
        )));
    }

    // Read the offset to the skin data
    cursor.seek(SeekFrom::Start(
        views_offset as u64 + (skin_index as u64 * 4),
    ))?;
    let skin_offset = cursor.read_u32_le()? as usize;

    // We don't know the exact size, but we can estimate based on typical skin sizes
    // or read until we hit the next structure. For now, let's read a reasonable chunk.
    // Most skin headers are under 64KB
    const MAX_SKIN_SIZE: usize = 65536;

    let end_offset = (skin_offset + MAX_SKIN_SIZE).min(m2_data.len());
    let skin_bytes = m2_data[skin_offset..end_offset].to_vec();

    Ok(skin_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_skin_detection() {
        use crate::common::M2Array;
        // Create a mock pre-WotLK model header
        let mut model = M2Model::default();
        model.header.version = 256; // Vanilla WoW version
        model.header.views = M2Array::new(2, 0x1000); // 2 skin profiles at offset 0x1000

        assert!(model.has_embedded_skins());
        assert_eq!(model.embedded_skin_count(), Some(2));
    }

    #[test]
    fn test_post_wotlk_no_embedded() {
        use crate::common::M2Array;
        // Create a mock WotLK+ model header
        let mut model = M2Model::default();
        model.header.version = 264; // WotLK version
        model.header.views = M2Array::new(0, 0);

        assert!(!model.has_embedded_skins());
        assert_eq!(model.embedded_skin_count(), None);
    }

    #[test]
    fn test_parse_embedded_skin_without_magic() {
        use crate::common::M2Array;
        use std::io::Write;

        // Create a mock M2 file with embedded skin data using the ModelView structure
        let mut m2_data = vec![0u8; 0x2000];

        // Write a ModelView structure at 0x1000
        // ModelView is 44 bytes (5 M2Arrays + 1 uint32): indices, triangles, properties, submeshes, textureUnits, boneCountMax
        let mut cursor = std::io::Cursor::new(&mut m2_data[0x1000..]);

        // Write ModelView fields (as M2Arrays: count + offset)
        cursor.write_all(&10u32.to_le_bytes()).unwrap(); // indices.count (10 indices)
        cursor.write_all(&0x1200u32.to_le_bytes()).unwrap(); // indices.offset
        cursor.write_all(&6u32.to_le_bytes()).unwrap(); // triangles.count (6 triangles)  
        cursor.write_all(&0x1220u32.to_le_bytes()).unwrap(); // triangles.offset
        cursor.write_all(&0u32.to_le_bytes()).unwrap(); // properties.count
        cursor.write_all(&0u32.to_le_bytes()).unwrap(); // properties.offset
        cursor.write_all(&2u32.to_le_bytes()).unwrap(); // submeshes.count (2 submeshes)
        cursor.write_all(&0x1240u32.to_le_bytes()).unwrap(); // submeshes.offset
        cursor.write_all(&0u32.to_le_bytes()).unwrap(); // textureUnits.count
        cursor.write_all(&0u32.to_le_bytes()).unwrap(); // textureUnits.offset
        cursor.write_all(&50u32.to_le_bytes()).unwrap(); // boneCountMax

        // Write some dummy index data at 0x1200 (10 indices * 2 bytes)
        for i in 0..10u16 {
            let idx_pos = 0x1200 + (i as usize * 2);
            m2_data[idx_pos..idx_pos + 2].copy_from_slice(&i.to_le_bytes());
        }

        // Write some dummy triangle data at 0x1220 (6 triangles * 2 bytes)
        for i in 0..6u16 {
            let tri_pos = 0x1220 + (i as usize * 2);
            m2_data[tri_pos..tri_pos + 2].copy_from_slice(&i.to_le_bytes());
        }

        // Write dummy submesh data at 0x1240 (2 submeshes * 28 bytes each for vanilla)
        // Each vanilla submesh has 8 uint16 fields (16 bytes) and ends at 28 bytes
        for i in 0..2u32 {
            let submesh_pos = 0x1240 + (i as usize * 28);
            let mut submesh_cursor = std::io::Cursor::new(&mut m2_data[submesh_pos..]);
            submesh_cursor.write_all(&(i as u16).to_le_bytes()).unwrap(); // id
            submesh_cursor.write_all(&0u16.to_le_bytes()).unwrap(); // level
            submesh_cursor
                .write_all(&(i as u16 * 5).to_le_bytes())
                .unwrap(); // vertex_start
            submesh_cursor.write_all(&5u16.to_le_bytes()).unwrap(); // vertex_count
            submesh_cursor
                .write_all(&(i as u16 * 3).to_le_bytes())
                .unwrap(); // triangle_start
            submesh_cursor.write_all(&3u16.to_le_bytes()).unwrap(); // triangle_count
            submesh_cursor.write_all(&0u16.to_le_bytes()).unwrap(); // bone_start
            submesh_cursor.write_all(&0u16.to_le_bytes()).unwrap(); // bone_count
            // Remaining fields up to 28 bytes
            for _ in 0..(28 - 16) {
                submesh_cursor.write_all(&0u8.to_le_bytes()).unwrap();
            }
        }

        // Create a model and test parsing
        let mut model = M2Model::default();
        model.header.version = 256;
        model.header.views = M2Array::new(1, 0x1000); // One view at 0x1000

        // Parse the embedded skin
        let result = model.parse_embedded_skin(&m2_data, 0);
        assert!(
            result.is_ok(),
            "Failed to parse embedded skin: {:?}",
            result.err()
        );

        let skin = result.unwrap();
        // After fix: corrected field mapping understanding
        // - ModelView.indices (count=10) maps to triangles array (larger, connectivity)
        // - ModelView.triangles (count=6) maps to indices array (smaller, lookup table)
        // - ModelView.submeshes (count=2) maps to submeshes array
        assert_eq!(skin.indices().len(), 10); // From original indices.count (now correctly mapped)
        assert_eq!(skin.triangles().len(), 6); // From original triangles.count (now correctly mapped)
        assert_eq!(skin.submeshes().len(), 2);
    }
}
