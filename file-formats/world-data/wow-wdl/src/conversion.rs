//! Version conversion functionality for WDL files

use std::collections::HashMap;

use crate::error::{Result, WdlError};
use crate::types::*;
use crate::version::WdlVersion;

/// Converts a WDL file from one version to another
pub fn convert_wdl_file(file: &WdlFile, target_version: WdlVersion) -> Result<WdlFile> {
    // Create a new file with the target version
    let mut new_file = WdlFile::with_version(target_version);

    // Copy basic data
    new_file.map_tile_offsets = file.map_tile_offsets;
    new_file.heightmap_tiles = file.heightmap_tiles.clone();

    // Handle holes data based on version
    if target_version.has_maho_chunk() {
        // Target supports holes data
        if file.version.has_maho_chunk() {
            // Source also has holes data, copy it
            new_file.holes_data = file.holes_data.clone();
        } else {
            // Source doesn't have holes data, create default holes (no holes)
            for &key in file.heightmap_tiles.keys() {
                new_file.holes_data.insert(key, HolesData::new());
            }
        }
    } else if file.version.has_maho_chunk() && !file.holes_data.is_empty() {
        // Target doesn't support holes, but source has them
        // This is a data loss situation, we should warn
        return Err(WdlError::VersionConversionError(
            "Cannot convert from a version with holes data to one without holes support"
                .to_string(),
        ));
    }

    // Handle model data conversion
    convert_model_data(file, &mut new_file)?;

    // Rebuild chunks list
    rebuild_chunks(&mut new_file)?;

    Ok(new_file)
}

/// Converts model data between different versions
fn convert_model_data(source: &WdlFile, target: &mut WdlFile) -> Result<()> {
    if target.version.has_wmo_chunks() {
        if source.version.has_wmo_chunks() {
            // Both versions use WMO chunks, direct copy
            target.wmo_filenames = source.wmo_filenames.clone();
            target.wmo_indices = source.wmo_indices.clone();
            target.wmo_placements = source.wmo_placements.clone();
        } else if source.version.has_ml_chunks() {
            // Convert from Legion+ format to pre-Legion format
            convert_ml_to_wmo(source, target)?;
        }
    }

    if target.version.has_ml_chunks() {
        if source.version.has_ml_chunks() {
            // Both versions use ML chunks, direct copy
            target.m2_placements = source.m2_placements.clone();
            target.m2_visibility = source.m2_visibility.clone();
            target.wmo_legion_placements = source.wmo_legion_placements.clone();
            target.wmo_legion_visibility = source.wmo_legion_visibility.clone();
        } else if source.version.has_wmo_chunks() {
            // Convert from pre-Legion format to Legion+ format
            convert_wmo_to_ml(source, target)?;
        }
    }

    Ok(())
}

/// Converts Legacy WMO format to Legion+ ML format
fn convert_wmo_to_ml(source: &WdlFile, target: &mut WdlFile) -> Result<()> {
    // In Legion+, WMOs use the FileDataID system instead of filenames
    // This is a simplistic conversion that assumes the WMO ID can be used as a FileDataID
    // In a real implementation, you'd need a mapping from filenames to FileDataIDs

    // Convert WMO placements to Legion format
    for placement in &source.wmo_placements {
        // Create an M2Placement
        let m2_placement = M2Placement {
            id: placement.id,
            m2_id: placement.wmo_id, // Use WMO ID as the FileDataID
            position: placement.position.clone(),
            rotation: placement.rotation.clone(),
            scale: 1.0, // Default scale
            flags: placement.flags as u32,
        };

        // Create a visibility info from the bounding box
        let visibility_info = M2VisibilityInfo {
            bounds: placement.bounds.clone(),
            radius: calculate_radius(&placement.bounds),
        };

        // Add to Legion WMO structures
        target.wmo_legion_placements.push(m2_placement);
        target.wmo_legion_visibility.push(visibility_info);
    }

    Ok(())
}

/// Converts Legion+ ML format to Legacy WMO format
fn convert_ml_to_wmo(source: &WdlFile, target: &mut WdlFile) -> Result<()> {
    // This is a simplistic conversion that assumes FileDataIDs can be directly used as indices
    // In a real implementation, you'd need to resolve FileDataIDs to filenames

    // Create dummy filenames for each unique FileDataID
    let mut file_ids = HashMap::new();
    let mut next_idx = 0;

    // Process M2 placements (these are doodads in pre-Legion)
    // Skip for now as they don't directly map to WMOs

    // Process WMO placements
    for (i, placement) in source.wmo_legion_placements.iter().enumerate() {
        // Get or create an index for this FileDataID
        let idx = match file_ids.get(&placement.m2_id) {
            Some(&idx) => idx,
            None => {
                let idx = next_idx;
                next_idx += 1;

                // Add a dummy filename
                target
                    .wmo_filenames
                    .push(format!("FileDataID_{}", placement.m2_id));
                file_ids.insert(placement.m2_id, idx);
                idx
            }
        };

        // Add the index to the list
        if !target.wmo_indices.contains(&idx) {
            target.wmo_indices.push(idx);
        }

        // Get visibility info
        let visibility = if i < source.wmo_legion_visibility.len() {
            &source.wmo_legion_visibility[i]
        } else {
            return Err(WdlError::VersionConversionError(
                "Missing visibility info for WMO placement".to_string(),
            ));
        };

        // Create a ModelPlacement
        let model_placement = ModelPlacement {
            id: placement.id,
            wmo_id: idx,
            position: placement.position.clone(),
            rotation: placement.rotation.clone(),
            bounds: visibility.bounds.clone(),
            flags: placement.flags as u16,
            doodad_set: 0, // Default
            name_set: 0,   // Default
            padding: 0,
        };

        // Add to legacy WMO structures
        target.wmo_placements.push(model_placement);
    }

    Ok(())
}

/// Calculates the radius from a bounding box
fn calculate_radius(bounds: &BoundingBox) -> f32 {
    let dx = (bounds.max.x - bounds.min.x) * 0.5;
    let dy = (bounds.max.y - bounds.min.y) * 0.5;
    let dz = (bounds.max.z - bounds.min.z) * 0.5;

    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Rebuilds the chunks list for a WDL file
fn rebuild_chunks(file: &mut WdlFile) -> Result<()> {
    // Clear existing chunks
    file.chunks.clear();

    // Add MVER chunk
    let mut mver_data = Vec::new();
    mver_data.extend_from_slice(&file.version_number.to_le_bytes());
    file.chunks.push(Chunk::new(MVER_MAGIC, mver_data));

    // Add WMO chunks if applicable
    if file.version.has_wmo_chunks() && !file.wmo_filenames.is_empty() {
        // Add MWMO chunk (WMO filenames)
        let mut mwmo_data = Vec::new();
        for name in &file.wmo_filenames {
            mwmo_data.extend_from_slice(name.as_bytes());
            mwmo_data.push(0); // Null terminator
        }
        file.chunks.push(Chunk::new(MWMO_MAGIC, mwmo_data));

        // Add MWID chunk (WMO indices)
        let mut mwid_data = Vec::new();
        for &idx in &file.wmo_indices {
            mwid_data.extend_from_slice(&idx.to_le_bytes());
        }
        file.chunks.push(Chunk::new(MWID_MAGIC, mwid_data));

        // Add MODF chunk (WMO placements)
        let mut modf_data = Vec::new();
        for placement in &file.wmo_placements {
            // Pack the data (simple example)
            modf_data.extend_from_slice(&placement.id.to_le_bytes());
            modf_data.extend_from_slice(&placement.wmo_id.to_le_bytes());
            // Position
            modf_data.extend_from_slice(&placement.position.x.to_le_bytes());
            modf_data.extend_from_slice(&placement.position.y.to_le_bytes());
            modf_data.extend_from_slice(&placement.position.z.to_le_bytes());
            // Rotation
            modf_data.extend_from_slice(&placement.rotation.x.to_le_bytes());
            modf_data.extend_from_slice(&placement.rotation.y.to_le_bytes());
            modf_data.extend_from_slice(&placement.rotation.z.to_le_bytes());
            // Bounds min
            modf_data.extend_from_slice(&placement.bounds.min.x.to_le_bytes());
            modf_data.extend_from_slice(&placement.bounds.min.y.to_le_bytes());
            modf_data.extend_from_slice(&placement.bounds.min.z.to_le_bytes());
            // Bounds max
            modf_data.extend_from_slice(&placement.bounds.max.x.to_le_bytes());
            modf_data.extend_from_slice(&placement.bounds.max.y.to_le_bytes());
            modf_data.extend_from_slice(&placement.bounds.max.z.to_le_bytes());
            // Flags
            modf_data.extend_from_slice(&placement.flags.to_le_bytes());
            // Doodad set
            modf_data.extend_from_slice(&placement.doodad_set.to_le_bytes());
            // Name set
            modf_data.extend_from_slice(&placement.name_set.to_le_bytes());
            // Padding
            modf_data.extend_from_slice(&placement.padding.to_le_bytes());
        }
        file.chunks.push(Chunk::new(MODF_MAGIC, modf_data));
    }

    // Add Legion+ chunks if applicable
    if file.version.has_ml_chunks() {
        // Add MLDD chunk (M2 placements)
        if !file.m2_placements.is_empty() {
            let mut mldd_data = Vec::new();
            for placement in &file.m2_placements {
                // Pack the data (simple example)
                mldd_data.extend_from_slice(&placement.id.to_le_bytes());
                mldd_data.extend_from_slice(&placement.m2_id.to_le_bytes());
                // Position
                mldd_data.extend_from_slice(&placement.position.x.to_le_bytes());
                mldd_data.extend_from_slice(&placement.position.y.to_le_bytes());
                mldd_data.extend_from_slice(&placement.position.z.to_le_bytes());
                // Rotation
                mldd_data.extend_from_slice(&placement.rotation.x.to_le_bytes());
                mldd_data.extend_from_slice(&placement.rotation.y.to_le_bytes());
                mldd_data.extend_from_slice(&placement.rotation.z.to_le_bytes());
                // Scale
                mldd_data.extend_from_slice(&placement.scale.to_le_bytes());
                // Flags
                mldd_data.extend_from_slice(&placement.flags.to_le_bytes());
            }
            file.chunks.push(Chunk::new(MLDD_MAGIC, mldd_data));
        }

        // Add MLDX chunk (M2 visibility info)
        if !file.m2_visibility.is_empty() {
            let mut mldx_data = Vec::new();
            for info in &file.m2_visibility {
                // Pack the data (simple example)
                // Bounds min
                mldx_data.extend_from_slice(&info.bounds.min.x.to_le_bytes());
                mldx_data.extend_from_slice(&info.bounds.min.y.to_le_bytes());
                mldx_data.extend_from_slice(&info.bounds.min.z.to_le_bytes());
                // Bounds max
                mldx_data.extend_from_slice(&info.bounds.max.x.to_le_bytes());
                mldx_data.extend_from_slice(&info.bounds.max.y.to_le_bytes());
                mldx_data.extend_from_slice(&info.bounds.max.z.to_le_bytes());
                // Radius
                mldx_data.extend_from_slice(&info.radius.to_le_bytes());
            }
            file.chunks.push(Chunk::new(MLDX_MAGIC, mldx_data));
        }

        // Add MLMD chunk (WMO Legion placements)
        if !file.wmo_legion_placements.is_empty() {
            let mut mlmd_data = Vec::new();
            for placement in &file.wmo_legion_placements {
                // Pack the data (simple example)
                mlmd_data.extend_from_slice(&placement.id.to_le_bytes());
                mlmd_data.extend_from_slice(&placement.m2_id.to_le_bytes());
                // Position
                mlmd_data.extend_from_slice(&placement.position.x.to_le_bytes());
                mlmd_data.extend_from_slice(&placement.position.y.to_le_bytes());
                mlmd_data.extend_from_slice(&placement.position.z.to_le_bytes());
                // Rotation
                mlmd_data.extend_from_slice(&placement.rotation.x.to_le_bytes());
                mlmd_data.extend_from_slice(&placement.rotation.y.to_le_bytes());
                mlmd_data.extend_from_slice(&placement.rotation.z.to_le_bytes());
                // Scale
                mlmd_data.extend_from_slice(&placement.scale.to_le_bytes());
                // Flags
                mlmd_data.extend_from_slice(&placement.flags.to_le_bytes());
            }
            file.chunks.push(Chunk::new(MLMD_MAGIC, mlmd_data));
        }

        // Add MLMX chunk (WMO Legion visibility info)
        if !file.wmo_legion_visibility.is_empty() {
            let mut mlmx_data = Vec::new();
            for info in &file.wmo_legion_visibility {
                // Pack the data (simple example)
                // Bounds min
                mlmx_data.extend_from_slice(&info.bounds.min.x.to_le_bytes());
                mlmx_data.extend_from_slice(&info.bounds.min.y.to_le_bytes());
                mlmx_data.extend_from_slice(&info.bounds.min.z.to_le_bytes());
                // Bounds max
                mlmx_data.extend_from_slice(&info.bounds.max.x.to_le_bytes());
                mlmx_data.extend_from_slice(&info.bounds.max.y.to_le_bytes());
                mlmx_data.extend_from_slice(&info.bounds.max.z.to_le_bytes());
                // Radius
                mlmx_data.extend_from_slice(&info.radius.to_le_bytes());
            }
            file.chunks.push(Chunk::new(MLMX_MAGIC, mlmx_data));
        }
    }

    // Add MAOF chunk (Map tile offsets)
    let mut maof_data = Vec::new();
    for &offset in &file.map_tile_offsets {
        maof_data.extend_from_slice(&offset.to_le_bytes());
    }
    file.chunks.push(Chunk::new(MAOF_MAGIC, maof_data));

    // We don't include the MARE and MAHO chunks directly in the chunks list
    // as they are referenced via offsets in the MAOF chunk
    // and will be written after the main chunk list

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BoundingBox, ModelPlacement, Vec3d, WdlFile};
    use crate::version::WdlVersion;

    #[test]
    fn test_calculate_radius() {
        let bounds = BoundingBox {
            min: Vec3d::new(-1.0, -1.0, -1.0),
            max: Vec3d::new(1.0, 1.0, 1.0),
        };

        let radius = calculate_radius(&bounds);
        // The bounding box has half-widths of 1.0 in each dimension
        // So the radius should be sqrt(1^2 + 1^2 + 1^2) = sqrt(3)
        let expected = (3.0f32).sqrt();

        assert!((radius - expected).abs() < 0.001);
    }

    #[test]
    fn test_convert_wotlk_to_legion() {
        // Create a WotLK-style file
        let mut wotlk_file = WdlFile::with_version(WdlVersion::Wotlk);

        // Add a WMO filename and index
        wotlk_file.wmo_filenames.push("test.wmo".to_string());
        wotlk_file.wmo_indices.push(0);

        // Add a WMO placement
        let placement = ModelPlacement {
            id: 1,
            wmo_id: 0,
            position: Vec3d::new(1.0, 2.0, 3.0),
            rotation: Vec3d::new(0.0, 0.0, 0.0),
            bounds: BoundingBox {
                min: Vec3d::new(-1.0, -1.0, -1.0),
                max: Vec3d::new(1.0, 1.0, 1.0),
            },
            flags: 0,
            doodad_set: 0,
            name_set: 0,
            padding: 0,
        };
        wotlk_file.wmo_placements.push(placement);

        // Convert to Legion
        let result = convert_wdl_file(&wotlk_file, WdlVersion::Legion);
        assert!(result.is_ok());

        let legion_file = result.unwrap();

        // Check conversion
        assert_eq!(legion_file.version, WdlVersion::Legion);
        assert!(legion_file.wmo_filenames.is_empty()); // Legion doesn't use these
        assert!(legion_file.wmo_indices.is_empty()); // Legion doesn't use these
        assert!(legion_file.wmo_placements.is_empty()); // Legion doesn't use these

        // Check that we have a converted WMO entry using Legion format
        assert_eq!(legion_file.wmo_legion_placements.len(), 1);
        assert_eq!(legion_file.wmo_legion_visibility.len(), 1);

        let legion_placement = &legion_file.wmo_legion_placements[0];
        assert_eq!(legion_placement.id, 1);
        assert_eq!(legion_placement.position.x, 1.0);
        assert_eq!(legion_placement.position.y, 2.0);
        assert_eq!(legion_placement.position.z, 3.0);
    }

    #[test]
    fn test_convert_legion_to_wotlk() {
        // Create a Legion-style file
        let mut legion_file = WdlFile::with_version(WdlVersion::Legion);

        // Add an M2 placement
        let m2_placement = M2Placement {
            id: 1,
            m2_id: 1234, // FileDataID
            position: Vec3d::new(1.0, 2.0, 3.0),
            rotation: Vec3d::new(0.0, 0.0, 0.0),
            scale: 1.0,
            flags: 0,
        };
        legion_file.m2_placements.push(m2_placement);

        // Add M2 visibility
        let visibility = M2VisibilityInfo {
            bounds: BoundingBox {
                min: Vec3d::new(-1.0, -1.0, -1.0),
                max: Vec3d::new(1.0, 1.0, 1.0),
            },
            radius: 1.732,
        };
        legion_file.m2_visibility.push(visibility);

        // Add a WMO placement
        let wmo_placement = M2Placement {
            id: 2,
            m2_id: 5678, // FileDataID
            position: Vec3d::new(4.0, 5.0, 6.0),
            rotation: Vec3d::new(0.0, 0.0, 0.0),
            scale: 1.0,
            flags: 0,
        };
        legion_file.wmo_legion_placements.push(wmo_placement);

        // Add WMO visibility
        let wmo_visibility = M2VisibilityInfo {
            bounds: BoundingBox {
                min: Vec3d::new(-2.0, -2.0, -2.0),
                max: Vec3d::new(2.0, 2.0, 2.0),
            },
            radius: 3.464,
        };
        legion_file.wmo_legion_visibility.push(wmo_visibility);

        // Convert to WotLK
        let result = convert_wdl_file(&legion_file, WdlVersion::Wotlk);
        assert!(result.is_ok());

        let wotlk_file = result.unwrap();

        // Check conversion
        assert_eq!(wotlk_file.version, WdlVersion::Wotlk);

        // Should have our WMO converted
        assert_eq!(wotlk_file.wmo_filenames.len(), 1);
        assert_eq!(wotlk_file.wmo_indices.len(), 1);
        assert_eq!(wotlk_file.wmo_placements.len(), 1);

        // Should have FileDataID in the filename
        assert!(wotlk_file.wmo_filenames[0].contains("5678"));

        // Position should match
        let wmo = &wotlk_file.wmo_placements[0];
        assert_eq!(wmo.position.x, 4.0);
        assert_eq!(wmo.position.y, 5.0);
        assert_eq!(wmo.position.z, 6.0);
    }
}
