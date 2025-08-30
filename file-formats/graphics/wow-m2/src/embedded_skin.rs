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
        // Each ModelView is 28 bytes with the following structure:
        // nIndex, ofsIndex, nTris, ofsTris, nProps, ofsProps, nSub, ofsSub, nTex, ofsTex, lod (7*4 = 28 bytes)
        // We need to convert this to our SkinFile format

        let model_view_size = 28; // 7 * 4-byte fields
        let model_view_offset = self.header.views.offset as usize + (skin_index * model_view_size);

        if model_view_offset + model_view_size > original_m2_data.len() {
            return Err(M2Error::ParseError(format!(
                "ModelView at offset {:#x} exceeds file size",
                model_view_offset
            )));
        }

        // Read the ModelView structure
        let model_view_data =
            &original_m2_data[model_view_offset..model_view_offset + model_view_size];

        // Parse ModelView fields (all little-endian u32)
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

        // Sanity check: ModelView data should be reasonable
        // Index count should not exceed total vertices, and offsets should be within file
        if n_index > 100000
            || ofs_index as usize >= original_m2_data.len()
            || n_tris > 100000
            || ofs_tris as usize >= original_m2_data.len()
        {
            return Err(M2Error::ParseError(format!(
                "Skin {} appears to have invalid ModelView data. This may not be a valid embedded skin.",
                skin_index
            )));
        }

        // Create a complete skin buffer that includes both header and data
        // We need to calculate the total size required
        let indices_size = (n_index as usize) * 2; // u16 per index
        let triangles_size = (n_tris as usize) * 2; // u16 per triangle

        // Verify offsets are within bounds
        if ofs_index as usize + indices_size > original_m2_data.len() {
            return Err(M2Error::ParseError(format!(
                "Index data at offset {:#x} + {} exceeds file size {}",
                ofs_index,
                indices_size,
                original_m2_data.len()
            )));
        }

        if ofs_tris as usize + triangles_size > original_m2_data.len() {
            return Err(M2Error::ParseError(format!(
                "Triangle data at offset {:#x} + {} exceeds file size {}",
                ofs_tris,
                triangles_size,
                original_m2_data.len()
            )));
        }

        // Calculate where to place data in our buffer
        let header_size = 40; // 5 M2Arrays * 8 bytes each
        let indices_buffer_offset = header_size;
        let triangles_buffer_offset = indices_buffer_offset + indices_size;
        let total_buffer_size = triangles_buffer_offset + triangles_size;

        let mut skin_buffer = vec![0u8; total_buffer_size];

        // Write header at the beginning
        let mut cursor = Cursor::new(&mut skin_buffer);

        // Write indices array (count, buffer offset)
        cursor.write_all(&n_index.to_le_bytes())?;
        cursor.write_all(&(indices_buffer_offset as u32).to_le_bytes())?;

        // Write triangles array (count, buffer offset)
        cursor.write_all(&n_tris.to_le_bytes())?;
        cursor.write_all(&(triangles_buffer_offset as u32).to_le_bytes())?;

        // Write empty bone_indices array
        cursor.write_all(&0u32.to_le_bytes())?;
        cursor.write_all(&0u32.to_le_bytes())?;

        // Write empty submeshes array
        cursor.write_all(&0u32.to_le_bytes())?;
        cursor.write_all(&0u32.to_le_bytes())?;

        // Write empty material_lookup array
        cursor.write_all(&0u32.to_le_bytes())?;
        cursor.write_all(&0u32.to_le_bytes())?;

        // Copy actual data from the original M2 file
        // Copy indices data
        if n_index > 0 {
            let src_indices =
                &original_m2_data[ofs_index as usize..(ofs_index as usize + indices_size)];
            skin_buffer[indices_buffer_offset..triangles_buffer_offset]
                .copy_from_slice(src_indices);
        }

        // Copy triangles data
        if n_tris > 0 {
            let src_triangles =
                &original_m2_data[ofs_tris as usize..(ofs_tris as usize + triangles_size)];
            skin_buffer[triangles_buffer_offset..total_buffer_size].copy_from_slice(src_triangles);
        }

        // Create a cursor with our complete skin buffer
        let mut cursor = Cursor::new(&skin_buffer);

        // Parse as embedded skin (no SKIN magic signature)
        parse_embedded_skin(&mut cursor)
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
        self.header.version <= 260 && self.header.views.count > 0
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
        // ModelView is 28 bytes: nIndex, ofsIndex, nTris, ofsTris, nProps, ofsProps, nSub, ofsSub, nTex, ofsTex, lod
        let mut cursor = std::io::Cursor::new(&mut m2_data[0x1000..]);
        
        // Write ModelView fields
        cursor.write_all(&10u32.to_le_bytes()).unwrap(); // nIndex (10 indices)
        cursor.write_all(&0x1200u32.to_le_bytes()).unwrap(); // ofsIndex
        cursor.write_all(&6u32.to_le_bytes()).unwrap(); // nTris (6 triangles)  
        cursor.write_all(&0x1220u32.to_le_bytes()).unwrap(); // ofsTris
        cursor.write_all(&0u32.to_le_bytes()).unwrap(); // nProps
        cursor.write_all(&0u32.to_le_bytes()).unwrap(); // ofsProps
        cursor.write_all(&0u32.to_le_bytes()).unwrap(); // nSub
        cursor.write_all(&0u32.to_le_bytes()).unwrap(); // ofsSub
        cursor.write_all(&0u32.to_le_bytes()).unwrap(); // nTex
        cursor.write_all(&0u32.to_le_bytes()).unwrap(); // ofsTex
        cursor.write_all(&0u32.to_le_bytes()).unwrap(); // lod

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
        assert_eq!(skin.indices().len(), 10);
        assert_eq!(skin.triangles().len(), 6);
        // No submeshes in this simplified test (they were optional anyway)
        assert_eq!(skin.submeshes().len(), 0);
    }
}
