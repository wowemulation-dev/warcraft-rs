//! Validation functions for WDL files

use crate::error::{Result, WdlError};
use crate::types::{
    MLDD_MAGIC, MLDX_MAGIC, MLMD_MAGIC, MLMX_MAGIC, MODF_MAGIC, MWID_MAGIC, MWMO_MAGIC, WdlFile,
};

/// Validates a WDL file for various constraints and correctness
pub fn validate_wdl_file(file: &WdlFile) -> Result<()> {
    // Basic file validation
    file.validate()?;

    // Validate map tile indices
    validate_map_tiles(file)?;

    // Validate model references
    validate_model_references(file)?;

    // Version-specific validations
    validate_version_specific(file)?;

    Ok(())
}

/// Validates map tiles in the WDL file
fn validate_map_tiles(file: &WdlFile) -> Result<()> {
    // Ensure all coordinate keys are within bounds
    for &(x, y) in file.heightmap_tiles.keys() {
        if x >= 64 || y >= 64 {
            return Err(WdlError::ValidationError(format!(
                "Map tile coordinates out of bounds: ({}, {})",
                x, y
            )));
        }
    }

    // Ensure all coordinate keys are within bounds for holes
    for &(x, y) in file.holes_data.keys() {
        if x >= 64 || y >= 64 {
            return Err(WdlError::ValidationError(format!(
                "Holes data coordinates out of bounds: ({}, {})",
                x, y
            )));
        }
    }

    // Verify that the map_tile_offsets match with the heightmap_tiles
    for y in 0..64 {
        for x in 0..64 {
            let index = y * 64 + x;
            let has_offset = file.map_tile_offsets[index] != 0;
            let has_heightmap = file.heightmap_tiles.contains_key(&(x as u32, y as u32));

            if has_offset != has_heightmap {
                return Err(WdlError::ValidationError(format!(
                    "Map tile offset mismatch at ({}, {}): offset={}, heightmap={}",
                    x, y, has_offset, has_heightmap
                )));
            }
        }
    }

    Ok(())
}

/// Validates model references in the WDL file
fn validate_model_references(file: &WdlFile) -> Result<()> {
    // MWID contains offsets into MWMO chunk data, not indices
    // We don't need to validate them as indices

    // Validate that MODF references valid indices into the MWID array
    for placement in &file.wmo_placements {
        if (placement.wmo_id as usize) >= file.wmo_indices.len() {
            return Err(WdlError::ValidationError(format!(
                "WMO placement references invalid MWID index: {} (max: {})",
                placement.wmo_id,
                file.wmo_indices.len().saturating_sub(1)
            )));
        }
    }

    // For Legion+, validate that MLDX entries match with MLDD entries
    if file.version.has_ml_chunks() {
        if file.m2_visibility.len() != file.m2_placements.len() {
            return Err(WdlError::ValidationError(format!(
                "M2 visibility count ({}) doesn't match M2 placement count ({})",
                file.m2_visibility.len(),
                file.m2_placements.len()
            )));
        }

        // Validate that MLMX entries match with MLMD entries
        if file.wmo_legion_visibility.len() != file.wmo_legion_placements.len() {
            return Err(WdlError::ValidationError(format!(
                "WMO visibility count ({}) doesn't match WMO placement count ({})",
                file.wmo_legion_visibility.len(),
                file.wmo_legion_placements.len()
            )));
        }
    }

    Ok(())
}

/// Validates version-specific constraints for the WDL file
fn validate_version_specific(file: &WdlFile) -> Result<()> {
    // Check if the file has the appropriate chunks for its version

    // Pre-WotLK should not have MAHO chunks
    if !file.version.has_maho_chunk() && !file.holes_data.is_empty() {
        return Err(WdlError::ValidationError(
            "Pre-WotLK WDL files should not have MAHO chunks".to_string(),
        ));
    }

    // Pre-Legion should not have ML* chunks
    if !file.version.has_ml_chunks()
        && (!file.m2_placements.is_empty()
            || !file.m2_visibility.is_empty()
            || !file.wmo_legion_placements.is_empty()
            || !file.wmo_legion_visibility.is_empty())
    {
        return Err(WdlError::ValidationError(
            "Pre-Legion WDL files should not have MLDD, MLDX, MLMD, or MLMX chunks".to_string(),
        ));
    }

    // Legion+ should not have WMO chunks
    if file.version.has_ml_chunks()
        && (!file.wmo_filenames.is_empty()
            || !file.wmo_indices.is_empty()
            || !file.wmo_placements.is_empty())
    {
        return Err(WdlError::ValidationError(
            "Legion+ WDL files should not have MWMO, MWID, or MODF chunks".to_string(),
        ));
    }

    // Pre-Legion should not have ML* chunks
    if !file.version.has_ml_chunks()
        && file.chunks.iter().any(|c| {
            c.magic == MLDD_MAGIC
                || c.magic == MLDX_MAGIC
                || c.magic == MLMD_MAGIC
                || c.magic == MLMX_MAGIC
        })
    {
        return Err(WdlError::ValidationError(
            "Pre-Legion WDL files should not have MLDD, MLDX, MLMD, or MLMX chunks".to_string(),
        ));
    }

    // Legion+ should not have WMO chunks
    if file.version.has_ml_chunks()
        && file
            .chunks
            .iter()
            .any(|c| c.magic == MWMO_MAGIC || c.magic == MWID_MAGIC || c.magic == MODF_MAGIC)
    {
        return Err(WdlError::ValidationError(
            "Legion+ WDL files should not have MWMO, MWID, or MODF chunks".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{HeightMapTile, HolesData};
    use crate::version::WdlVersion;

    #[test]
    fn test_validate_map_tiles() {
        let mut file = WdlFile::new();

        // Valid map tile
        file.heightmap_tiles.insert((5, 10), HeightMapTile::new());
        file.map_tile_offsets[10 * 64 + 5] = 1; // Non-zero value

        // This should pass validation
        assert!(validate_map_tiles(&file).is_ok());

        // Add an invalid map tile (out of bounds)
        file.heightmap_tiles.insert((65, 10), HeightMapTile::new());

        // This should fail validation
        assert!(validate_map_tiles(&file).is_err());

        // Remove the invalid one and add a mismatch
        file.heightmap_tiles.remove(&(65, 10));
        file.map_tile_offsets[64 + 1] = 1; // Offset set but no heightmap

        // This should fail validation
        assert!(validate_map_tiles(&file).is_err());
    }

    #[test]
    fn test_validate_version_specific() {
        // Test WotLK file
        let mut wotlk_file = WdlFile::with_version(WdlVersion::Wotlk);
        wotlk_file.holes_data.insert((1, 1), HolesData::new());

        // Should be valid
        assert!(validate_version_specific(&wotlk_file).is_ok());

        // Test Vanilla file with holes (invalid)
        let mut vanilla_file = WdlFile::with_version(WdlVersion::Vanilla);
        vanilla_file.holes_data.insert((1, 1), HolesData::new());

        // Should be invalid
        assert!(validate_version_specific(&vanilla_file).is_err());
    }
}
