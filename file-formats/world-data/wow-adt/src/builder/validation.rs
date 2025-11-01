//! Validation functions for ADT builder.
//!
//! Implements progressive validation strategy:
//! - Method-level: Basic parameter checks
//! - Build-level: Structural and reference validation

use crate::ChunkId;
use crate::chunks::DoodadPlacement;
use crate::chunks::WmoPlacement;
use crate::chunks::blend_mesh::{MbbbChunk, MbmhChunk, MbmiChunk, MbnvChunk};
use crate::error::{AdtError, Result};
use crate::version::AdtVersion;

/// Validate texture filename format.
///
/// # Validation Rules
///
/// - Filename not empty
/// - Uses forward slashes (not backslashes)
/// - Has .blp extension (case-insensitive)
///
/// # Examples
///
/// ```
/// # use wow_adt::builder::validation::validate_texture_filename;
/// assert!(validate_texture_filename("terrain/grass_01.blp").is_ok());
/// assert!(validate_texture_filename("").is_err());
/// assert!(validate_texture_filename("terrain\\grass.blp").is_err());
/// assert!(validate_texture_filename("terrain/grass.png").is_err());
/// ```
pub fn validate_texture_filename(filename: &str) -> Result<()> {
    if filename.is_empty() {
        return Err(AdtError::ChunkParseError {
            chunk: ChunkId::MTEX,
            offset: 0,
            details: "Texture filename cannot be empty".to_string(),
        });
    }

    if filename.contains('\\') {
        return Err(AdtError::ChunkParseError {
            chunk: ChunkId::MTEX,
            offset: 0,
            details: format!(
                "Texture filename '{}' contains backslashes - use forward slashes",
                filename
            ),
        });
    }

    if !filename.to_lowercase().ends_with(".blp") {
        return Err(AdtError::ChunkParseError {
            chunk: ChunkId::MTEX,
            offset: 0,
            details: format!("Texture filename '{}' must have .blp extension", filename),
        });
    }

    Ok(())
}

/// Validate M2 model filename format.
///
/// # Validation Rules
///
/// - Filename not empty
/// - Uses forward slashes (not backslashes)
/// - Has .m2 extension (case-insensitive)
pub fn validate_model_filename(filename: &str) -> Result<()> {
    if filename.is_empty() {
        return Err(AdtError::ChunkParseError {
            chunk: ChunkId::MMDX,
            offset: 0,
            details: "Model filename cannot be empty".to_string(),
        });
    }

    if filename.contains('\\') {
        return Err(AdtError::ChunkParseError {
            chunk: ChunkId::MMDX,
            offset: 0,
            details: format!(
                "Model filename '{}' contains backslashes - use forward slashes",
                filename
            ),
        });
    }

    if !filename.to_lowercase().ends_with(".m2") {
        return Err(AdtError::ChunkParseError {
            chunk: ChunkId::MMDX,
            offset: 0,
            details: format!("Model filename '{}' must have .m2 extension", filename),
        });
    }

    Ok(())
}

/// Validate WMO filename format.
///
/// # Validation Rules
///
/// - Filename not empty
/// - Uses forward slashes (not backslashes)
/// - Has .wmo extension (case-insensitive)
pub fn validate_wmo_filename(filename: &str) -> Result<()> {
    if filename.is_empty() {
        return Err(AdtError::ChunkParseError {
            chunk: ChunkId::MWMO,
            offset: 0,
            details: "WMO filename cannot be empty".to_string(),
        });
    }

    if filename.contains('\\') {
        return Err(AdtError::ChunkParseError {
            chunk: ChunkId::MWMO,
            offset: 0,
            details: format!(
                "WMO filename '{}' contains backslashes - use forward slashes",
                filename
            ),
        });
    }

    if !filename.to_lowercase().ends_with(".wmo") {
        return Err(AdtError::ChunkParseError {
            chunk: ChunkId::MWMO,
            offset: 0,
            details: format!("WMO filename '{}' must have .wmo extension", filename),
        });
    }

    Ok(())
}

/// Validate version-chunk compatibility.
///
/// # Validation Rules
///
/// - MFBO requires TBC+
/// - MH2O requires WotLK+
/// - MAMP requires Cataclysm+
/// - MTXP requires MoP+
/// - MBMH, MBBB, MBNV, MBMI require MoP+ (blend mesh system)
pub fn validate_version_chunk_compatibility(version: AdtVersion, chunk: ChunkId) -> Result<()> {
    match chunk {
        ChunkId::MFBO => {
            if !matches!(
                version,
                AdtVersion::TBC | AdtVersion::WotLK | AdtVersion::Cataclysm | AdtVersion::MoP
            ) {
                return Err(AdtError::ChunkParseError {
                    chunk: ChunkId::MFBO,
                    offset: 0,
                    details: format!(
                        "MFBO (flight bounds) requires TBC or later, but version is {:?}",
                        version
                    ),
                });
            }
        }
        ChunkId::MH2O => {
            if !matches!(
                version,
                AdtVersion::WotLK | AdtVersion::Cataclysm | AdtVersion::MoP
            ) {
                return Err(AdtError::ChunkParseError {
                    chunk: ChunkId::MH2O,
                    offset: 0,
                    details: format!(
                        "MH2O (advanced water) requires WotLK or later, but version is {:?}",
                        version
                    ),
                });
            }
        }
        ChunkId::MAMP => {
            if !matches!(version, AdtVersion::Cataclysm | AdtVersion::MoP) {
                return Err(AdtError::ChunkParseError {
                    chunk: ChunkId::MAMP,
                    offset: 0,
                    details: format!(
                        "MAMP (texture amplifier) requires Cataclysm or later, but version is {:?}",
                        version
                    ),
                });
            }
        }
        ChunkId::MTXP | ChunkId::MBMH | ChunkId::MBBB | ChunkId::MBNV | ChunkId::MBMI => {
            if version != AdtVersion::MoP {
                let chunk_name = match chunk {
                    ChunkId::MTXP => "MTXP (texture parameters)",
                    ChunkId::MBMH => "MBMH (blend mesh headers)",
                    ChunkId::MBBB => "MBBB (blend mesh bounds)",
                    ChunkId::MBNV => "MBNV (blend mesh vertices)",
                    ChunkId::MBMI => "MBMI (blend mesh indices)",
                    _ => unreachable!(),
                };
                return Err(AdtError::ChunkParseError {
                    chunk,
                    offset: 0,
                    details: format!(
                        "{} requires MoP, but version is {:?}",
                        chunk_name, version
                    ),
                });
            }
        }
        _ => {}
    }

    Ok(())
}

/// Validate doodad placement references.
///
/// # Validation Rules
///
/// - name_id < model_count
/// - scale > 0
pub fn validate_doodad_placement_references(
    placements: &[DoodadPlacement],
    model_count: usize,
) -> Result<()> {
    for (idx, placement) in placements.iter().enumerate() {
        if placement.name_id as usize >= model_count {
            return Err(AdtError::InvalidModelReference {
                index: placement.name_id,
                count: model_count as u32,
            });
        }

        if placement.scale == 0 {
            return Err(AdtError::ChunkParseError {
                chunk: ChunkId::MDDF,
                offset: 0,
                details: format!("Doodad placement {} has invalid scale 0 (must be > 0)", idx),
            });
        }
    }

    Ok(())
}

/// Validate WMO placement references.
///
/// # Validation Rules
///
/// - name_id < wmo_count
pub fn validate_wmo_placement_references(
    placements: &[WmoPlacement],
    wmo_count: usize,
) -> Result<()> {
    for placement in placements {
        if placement.name_id as usize >= wmo_count {
            return Err(AdtError::InvalidModelReference {
                index: placement.name_id,
                count: wmo_count as u32,
            });
        }
    }

    Ok(())
}

/// Validate blend mesh data completeness and consistency (MoP+).
///
/// # Validation Rules
///
/// - All four blend mesh chunks must be present together or all absent
/// - Headers cannot be empty if present
/// - Bounds count must match headers count
/// - Total vertices/indices counts must match headers' ranges
///
/// # Examples
///
/// ```
/// # use wow_adt::builder::validation::validate_blend_mesh_data;
/// # use wow_adt::chunks::blend_mesh::{MbmhChunk, MbbbChunk, MbnvChunk, MbmiChunk};
/// // All None - valid
/// assert!(validate_blend_mesh_data(&None, &None, &None, &None).is_ok());
///
/// // All present with matching counts - valid
/// let headers = MbmhChunk::default();
/// let bounds = MbbbChunk::default();
/// let vertices = MbnvChunk::default();
/// let indices = MbmiChunk::default();
/// // This would validate if counts match
/// ```
pub fn validate_blend_mesh_data(
    headers: &Option<MbmhChunk>,
    bounds: &Option<MbbbChunk>,
    vertices: &Option<MbnvChunk>,
    indices: &Option<MbmiChunk>,
) -> Result<()> {
    // Check completeness: all 4 chunks present or all 4 absent
    let present = [
        headers.is_some(),
        bounds.is_some(),
        vertices.is_some(),
        indices.is_some(),
    ];

    let some_count = present.iter().filter(|&&x| x).count();
    if some_count != 0 && some_count != 4 {
        return Err(AdtError::ChunkParseError {
            chunk: ChunkId::MBMH,
            offset: 0,
            details: format!(
                "Blend mesh chunks must all be present or all absent (found {} of 4)",
                some_count
            ),
        });
    }

    // If all absent, validation complete
    if some_count == 0 {
        return Ok(());
    }

    // All 4 are present - validate consistency
    let headers = headers.as_ref().unwrap();
    let bounds = bounds.as_ref().unwrap();
    let vertices = vertices.as_ref().unwrap();
    let indices = indices.as_ref().unwrap();

    // Headers cannot be empty
    if headers.entries.is_empty() {
        return Err(AdtError::ChunkParseError {
            chunk: ChunkId::MBMH,
            offset: 0,
            details: "Blend mesh headers cannot be empty".to_string(),
        });
    }

    // Bounds count must match headers count
    if bounds.entries.len() != headers.entries.len() {
        return Err(AdtError::ChunkParseError {
            chunk: ChunkId::MBBB,
            offset: 0,
            details: format!(
                "Blend mesh bounds count {} does not match headers count {}",
                bounds.entries.len(),
                headers.entries.len()
            ),
        });
    }

    // Validate that index/vertex counts in headers match actual data
    let total_indices: u32 = headers.entries.iter().map(|h| h.mbmi_count).sum();
    let total_vertices: u32 = headers.entries.iter().map(|h| h.mbnv_count).sum();

    if indices.indices.len() != total_indices as usize {
        return Err(AdtError::ChunkParseError {
            chunk: ChunkId::MBMI,
            offset: 0,
            details: format!(
                "Blend mesh indices count {} does not match headers total {}",
                indices.indices.len(),
                total_indices
            ),
        });
    }

    if vertices.vertices.len() != total_vertices as usize {
        return Err(AdtError::ChunkParseError {
            chunk: ChunkId::MBNV,
            offset: 0,
            details: format!(
                "Blend mesh vertices count {} does not match headers total {}",
                vertices.vertices.len(),
                total_vertices
            ),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_texture_filename_valid() {
        assert!(validate_texture_filename("terrain/grass_01.blp").is_ok());
        assert!(validate_texture_filename("Terrain/Grass.BLP").is_ok());
        assert!(validate_texture_filename("a/b/c/texture.blp").is_ok());
    }

    #[test]
    fn test_validate_texture_filename_empty() {
        assert!(validate_texture_filename("").is_err());
    }

    #[test]
    fn test_validate_texture_filename_backslash() {
        assert!(validate_texture_filename("terrain\\grass.blp").is_err());
    }

    #[test]
    fn test_validate_texture_filename_wrong_extension() {
        assert!(validate_texture_filename("terrain/grass.png").is_err());
        assert!(validate_texture_filename("terrain/grass.jpg").is_err());
    }

    #[test]
    fn test_validate_model_filename_valid() {
        assert!(validate_model_filename("doodad/tree_01.m2").is_ok());
        assert!(validate_model_filename("Doodad/Tree.M2").is_ok());
    }

    #[test]
    fn test_validate_model_filename_wrong_extension() {
        assert!(validate_model_filename("doodad/tree.mdx").is_err());
    }

    #[test]
    fn test_validate_wmo_filename_valid() {
        assert!(validate_wmo_filename("building/house_01.wmo").is_ok());
        assert!(validate_wmo_filename("Building/House.WMO").is_ok());
    }

    #[test]
    fn test_validate_version_chunk_mfbo() {
        assert!(validate_version_chunk_compatibility(AdtVersion::TBC, ChunkId::MFBO).is_ok());
        assert!(
            validate_version_chunk_compatibility(AdtVersion::VanillaEarly, ChunkId::MFBO).is_err()
        );
    }

    #[test]
    fn test_validate_version_chunk_mh2o() {
        assert!(validate_version_chunk_compatibility(AdtVersion::WotLK, ChunkId::MH2O).is_ok());
        assert!(validate_version_chunk_compatibility(AdtVersion::TBC, ChunkId::MH2O).is_err());
    }

    #[test]
    fn test_validate_version_chunk_mamp() {
        assert!(validate_version_chunk_compatibility(AdtVersion::Cataclysm, ChunkId::MAMP).is_ok());
        assert!(validate_version_chunk_compatibility(AdtVersion::WotLK, ChunkId::MAMP).is_err());
    }

    #[test]
    fn test_validate_version_chunk_mtxp() {
        assert!(validate_version_chunk_compatibility(AdtVersion::MoP, ChunkId::MTXP).is_ok());
        assert!(
            validate_version_chunk_compatibility(AdtVersion::Cataclysm, ChunkId::MTXP).is_err()
        );
    }

    #[test]
    fn test_validate_doodad_placement_valid() {
        let placements = vec![
            DoodadPlacement {
                name_id: 0,
                unique_id: 1,
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: 1024,
                flags: 0,
            },
            DoodadPlacement {
                name_id: 1,
                unique_id: 2,
                position: [100.0, 100.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: 512,
                flags: 0,
            },
        ];

        assert!(validate_doodad_placement_references(&placements, 2).is_ok());
    }

    #[test]
    fn test_validate_doodad_placement_invalid_reference() {
        let placements = vec![DoodadPlacement {
            name_id: 5,
            unique_id: 1,
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: 1024,
            flags: 0,
        }];

        assert!(validate_doodad_placement_references(&placements, 2).is_err());
    }

    #[test]
    fn test_validate_doodad_placement_zero_scale() {
        let placements = vec![DoodadPlacement {
            name_id: 0,
            unique_id: 1,
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: 0,
            flags: 0,
        }];

        assert!(validate_doodad_placement_references(&placements, 1).is_err());
    }

    #[test]
    fn test_validate_wmo_placement_valid() {
        let placements = vec![WmoPlacement {
            name_id: 0,
            unique_id: 1,
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            extents_min: [-100.0, -100.0, 0.0],
            extents_max: [100.0, 100.0, 200.0],
            flags: 0,
            doodad_set: 0,
            name_set: 0,
            scale: 1024,
        }];

        assert!(validate_wmo_placement_references(&placements, 1).is_ok());
    }

    #[test]
    fn test_validate_wmo_placement_invalid_reference() {
        let placements = vec![WmoPlacement {
            name_id: 3,
            unique_id: 1,
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            extents_min: [-100.0, -100.0, 0.0],
            extents_max: [100.0, 100.0, 200.0],
            flags: 0,
            doodad_set: 0,
            name_set: 0,
            scale: 1024,
        }];

        assert!(validate_wmo_placement_references(&placements, 1).is_err());
    }
}
