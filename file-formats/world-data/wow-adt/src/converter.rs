// converter.rs - Version conversion for ADT files

use crate::Adt;
use crate::chunk::*;
use crate::error::{AdtError, Result};
use crate::liquid_converter::{convert_mclq_to_mh2o, convert_mh2o_to_mclq};
use crate::mcnk_subchunks::MclqSubchunk;
use crate::version::AdtVersion;

/// Convert an ADT from one version to another
pub fn convert_adt(source: &Adt, target_version: AdtVersion) -> Result<Adt> {
    if source.version() == target_version {
        // No conversion needed
        return Ok(source.clone());
    }

    match (source.version(), target_version) {
        // Vanilla to TBC
        (AdtVersion::Vanilla, AdtVersion::TBC) => vanilla_to_tbc(source),

        // TBC to WotLK
        (AdtVersion::TBC, AdtVersion::WotLK) => tbc_to_wotlk(source),

        // WotLK to Cataclysm
        (AdtVersion::WotLK, AdtVersion::Cataclysm) => wotlk_to_cataclysm(source),

        // Downgrading
        (AdtVersion::TBC, AdtVersion::Vanilla) => tbc_to_vanilla(source),
        (AdtVersion::WotLK, AdtVersion::TBC) => wotlk_to_tbc(source),
        (AdtVersion::Cataclysm, AdtVersion::WotLK) => cataclysm_to_wotlk(source),

        // Multiple version jumps - chain conversions
        (from, to) if from < to => {
            // Upgrading through multiple versions
            let mut current = source.clone();

            // Chain conversions (convert to each intermediate version)
            let versions = upgrade_path(from, to);
            for (_v_from, v_to) in versions {
                current = convert_adt(&current, v_to)?;
            }

            Ok(current)
        }

        (from, to) if from > to => {
            // Downgrading through multiple versions
            let mut current = source.clone();

            // Chain conversions (convert to each intermediate version)
            let versions = downgrade_path(from, to);
            for (_v_from, v_to) in versions {
                current = convert_adt(&current, v_to)?;
            }

            Ok(current)
        }

        _ => {
            // Conversion not supported
            Err(AdtError::VersionConversionUnsupported {
                from: source.version().to_string(),
                to: target_version.to_string(),
            })
        }
    }
}

/// Generate a path of version pairs for upgrading
fn upgrade_path(from: AdtVersion, to: AdtVersion) -> Vec<(AdtVersion, AdtVersion)> {
    let mut path = Vec::new();
    let mut current = from;

    while current < to {
        let next = match current {
            AdtVersion::Vanilla => AdtVersion::TBC,
            AdtVersion::TBC => AdtVersion::WotLK,
            AdtVersion::WotLK => AdtVersion::Cataclysm,
            AdtVersion::Cataclysm => AdtVersion::MoP,
            AdtVersion::MoP => AdtVersion::WoD,
            AdtVersion::WoD => AdtVersion::Legion,
            AdtVersion::Legion => AdtVersion::BfA,
            AdtVersion::BfA => AdtVersion::Shadowlands,
            AdtVersion::Shadowlands => AdtVersion::Dragonflight,
            AdtVersion::Dragonflight => break, // No higher version
        };

        path.push((current, next));
        current = next;
    }

    path
}

/// Generate a path of version pairs for downgrading
fn downgrade_path(from: AdtVersion, to: AdtVersion) -> Vec<(AdtVersion, AdtVersion)> {
    let mut path = Vec::new();
    let mut current = from;

    while current > to {
        let next = match current {
            AdtVersion::Dragonflight => AdtVersion::Shadowlands,
            AdtVersion::Shadowlands => AdtVersion::BfA,
            AdtVersion::BfA => AdtVersion::Legion,
            AdtVersion::Legion => AdtVersion::WoD,
            AdtVersion::WoD => AdtVersion::MoP,
            AdtVersion::MoP => AdtVersion::Cataclysm,
            AdtVersion::Cataclysm => AdtVersion::WotLK,
            AdtVersion::WotLK => AdtVersion::TBC,
            AdtVersion::TBC => AdtVersion::Vanilla,
            AdtVersion::Vanilla => break, // No lower version
        };

        path.push((current, next));
        current = next;
    }

    path
}

/// Convert from Vanilla to TBC
fn vanilla_to_tbc(source: &Adt) -> Result<Adt> {
    // Create a clone of the source to modify
    let mut result = source.clone();

    // Update the version
    result.version = AdtVersion::TBC;

    // Update MVER chunk to match the new version
    result.mver.version = AdtVersion::TBC.to_mver_value();

    // Add TBC-specific chunks (empty ones)

    // Add MFBO (default flight boundaries with 9 values per plane)
    result.mfbo = Some(MfboChunk {
        max: [0; 9], // 9 int16 values for max plane
        min: [0; 9], // 9 int16 values for min plane
    });

    // Update MHDR to include TBC fields
    if let Some(ref mut mhdr) = result.mhdr {
        // Add MFBO offset field
        mhdr.mfbo_offset = Some(0); // Will be updated during writing
    }

    Ok(result)
}

/// Convert from TBC to WotLK
fn tbc_to_wotlk(source: &Adt) -> Result<Adt> {
    // Create a clone of the source to modify
    let mut result = source.clone();

    // Update the version
    result.version = AdtVersion::WotLK;

    // Update MVER chunk to match the new version
    result.mver.version = AdtVersion::WotLK.to_mver_value();

    // Convert liquid data in MCNK chunks from MCLQ to MH2O format
    // Collect all MCLQ chunks from MCNKs
    let mut mclq_chunks = Vec::new();

    for _mcnk in &source.mcnk_chunks {
        // MCLQ chunks would need to be extracted from the original file
        // For now, create a placeholder based on the liquid data in the MCNKs
        let mclq = MclqSubchunk {
            x_vertices: 9, // Default grid size
            y_vertices: 9,
            base_height: 0.0,     // This would come from the actual MCLQ data
            vertices: Vec::new(), // This would be filled with actual vertices
        };

        mclq_chunks.push(mclq);
    }

    // Convert MCLQ data to MH2O format
    let mh2o = convert_mclq_to_mh2o(&mclq_chunks, &source.mcnk_chunks)?;
    result.mh2o = Some(mh2o);

    // Update MHDR to include WotLK fields
    if let Some(ref mut mhdr) = result.mhdr {
        // Add MH2O offset field
        mhdr.mh2o_offset = Some(0); // Will be updated during writing
    }

    Ok(result)
}

/// Convert from WotLK to Cataclysm
fn wotlk_to_cataclysm(source: &Adt) -> Result<Adt> {
    // Create a clone of the source to modify
    let mut result = source.clone();

    // Update the version
    result.version = AdtVersion::Cataclysm;

    // Update MVER chunk to match the new version
    result.mver.version = AdtVersion::Cataclysm.to_mver_value();

    // Add Cataclysm-specific chunks (empty ones)

    // Add MTFX (empty texture effects)
    result.mtfx = Some(MtfxChunk {
        effects: Vec::new(),
    });

    // Update MHDR to include Cataclysm fields
    if let Some(ref mut mhdr) = result.mhdr {
        // Add MTFX offset field
        mhdr.mtfx_offset = Some(0); // Will be updated during writing
    }

    Ok(result)
}

/// Convert from TBC to Vanilla
fn tbc_to_vanilla(source: &Adt) -> Result<Adt> {
    // Create a clone of the source to modify
    let mut result = source.clone();

    // Update the version
    result.version = AdtVersion::Vanilla;

    // Update MVER chunk to match the new version
    result.mver.version = AdtVersion::Vanilla.to_mver_value();

    // Remove TBC-specific chunks
    result.mfbo = None;

    // Update MHDR to remove TBC fields
    if let Some(ref mut mhdr) = result.mhdr {
        mhdr.mfbo_offset = None;
    }

    Ok(result)
}

/// Convert from WotLK to TBC
fn wotlk_to_tbc(source: &Adt) -> Result<Adt> {
    // Create a clone of the source to modify
    let mut result = source.clone();

    // Update the version
    result.version = AdtVersion::TBC;

    // Update MVER chunk to match the new version
    result.mver.version = AdtVersion::TBC.to_mver_value();

    // Convert water data in MCNK chunks from MH2O to MCLQ format
    if let Some(ref mh2o) = source.mh2o {
        let mclq_chunks = convert_mh2o_to_mclq(mh2o, &source.mcnk_chunks)?;

        // Update MCNKs with MCLQ data
        // In a real implementation, we'd update each MCNK's liquid data
        // For now, just store the liquid offset in the MCNK header
        for (i, mcnk) in result.mcnk_chunks.iter_mut().enumerate() {
            if i < mclq_chunks.len() && !mclq_chunks[i].vertices.is_empty() {
                // Mark this MCNK as having liquid data
                mcnk.liquid_offset = 1; // Placeholder, would be set during writing
                mcnk.liquid_size = 1; // Placeholder
            } else {
                // No liquid data
                mcnk.liquid_offset = 0;
                mcnk.liquid_size = 0;
            }
        }
    }

    // Remove WotLK-specific chunks
    result.mh2o = None;

    // Update MHDR to remove WotLK fields
    if let Some(ref mut mhdr) = result.mhdr {
        mhdr.mh2o_offset = None;
    }

    Ok(result)
}

/// Convert from Cataclysm to WotLK
fn cataclysm_to_wotlk(source: &Adt) -> Result<Adt> {
    // Create a clone of the source to modify
    let mut result = source.clone();

    // Update the version
    result.version = AdtVersion::WotLK;

    // Update MVER chunk to match the new version
    result.mver.version = AdtVersion::WotLK.to_mver_value();

    // Remove Cataclysm-specific chunks
    result.mtfx = None;

    // Update MHDR to remove Cataclysm fields
    if let Some(ref mut mhdr) = result.mhdr {
        mhdr.mtfx_offset = None;
    }

    Ok(result)
}
