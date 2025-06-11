// mcnk_converter.rs - Convert MCNK chunks between different WoW versions

use crate::chunk::*;
use crate::error::Result;
// use crate::mcnk_subchunks::*;
use crate::texture_converter::{convert_alpha_maps, convert_area_id, convert_texture_layers};
use crate::version::AdtVersion;

/// Convert a MCNK chunk from one version to another
pub fn convert_mcnk(
    source: &McnkChunk,
    from_version: AdtVersion,
    to_version: AdtVersion,
) -> Result<McnkChunk> {
    // If same version, just clone
    if from_version == to_version {
        return Ok(source.clone());
    }

    // Create modified copy
    let mut result = source.clone();

    // Update area ID
    result.area_id = convert_area_id(source.area_id, from_version, to_version);

    // Convert texture layers
    result.texture_layers =
        convert_texture_layers(&source.texture_layers, from_version, to_version)?;

    // Convert alpha maps
    result.alpha_maps = convert_alpha_maps(
        &source.alpha_maps,
        &source.texture_layers,
        from_version,
        to_version,
    )?;

    // Update flags for version changes
    update_mcnk_flags(&mut result, from_version, to_version);

    // Version-specific conversions
    match (from_version, to_version) {
        // Vanilla to TBC
        (AdtVersion::Vanilla, AdtVersion::TBC) => {
            vanilla_to_tbc_mcnk(source, &mut result)?;
        }

        // TBC to WotLK
        (AdtVersion::TBC, AdtVersion::WotLK) => {
            tbc_to_wotlk_mcnk(source, &mut result)?;
        }

        // WotLK to Cataclysm
        (AdtVersion::WotLK, AdtVersion::Cataclysm) => {
            wotlk_to_cataclysm_mcnk(source, &mut result)?;
        }

        // Cataclysm to MoP
        (AdtVersion::Cataclysm, AdtVersion::MoP) => {
            cataclysm_to_mop_mcnk(source, &mut result)?;
        }

        // Downgrading
        (AdtVersion::TBC, AdtVersion::Vanilla) => {
            tbc_to_vanilla_mcnk(source, &mut result)?;
        }

        (AdtVersion::WotLK, AdtVersion::TBC) => {
            wotlk_to_tbc_mcnk(source, &mut result)?;
        }

        (AdtVersion::Cataclysm, AdtVersion::WotLK) => {
            cataclysm_to_wotlk_mcnk(source, &mut result)?;
        }

        (AdtVersion::MoP, AdtVersion::Cataclysm) => {
            mop_to_cataclysm_mcnk(source, &mut result)?;
        }

        // Other conversions - already handled by the basic conversion above
        _ => {}
    }

    Ok(result)
}

/// MCNK flags that changed between versions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum McnkFlags {
    /// Has height data (MCVT)
    HasMcvt = 0x1,
    /// Has normal vectors (MCNR)
    HasMcnr = 0x2,
    /// Has texture mapping (MCLY)
    HasMcly = 0x4,
    /// Has doodad references (MCRF)
    HasMcrf = 0x8,
    /// Has the color map (MCCV)
    HasMccv = 0x10,
    /// Has the area ID
    HasAreaId = 0x20,
    /// Has the low detail map
    IsLowDetail = 0x40,
    /// Has liquid data (MCLQ/MH2O)
    HasLiquid = 0x80,
    /// Has big alpha (MCAL)
    HasBigAlpha = 0x100,
    /// Has vertex shading (MCCV)
    HasVertexShading = 0x200,
    /// 0x400 - unused?
    Unused400 = 0x400,
    /// TBC+: Has the sound emitter (MCSE)
    HasMcse = 0x800,
    /// WotLK+: Has the water (MH2O)
    HasMh2o = 0x1000,
    /// Cataclysm+: Has the cliff texture (MCCR)
    HasMccr = 0x2000,
    /// MoP+: Has the texture scale (MCWS/MCSS)
    HasTextureScale = 0x4000,
    /// Legion+: Has the map objects (MCRD)
    HasMapObjects = 0x8000,
}

/// Update MCNK flags for version conversion
fn update_mcnk_flags(chunk: &mut McnkChunk, from_version: AdtVersion, to_version: AdtVersion) {
    // Clear version-specific flags when downgrading
    if from_version > to_version {
        // TBC+ flags
        if to_version < AdtVersion::TBC {
            chunk.flags &= !(McnkFlags::HasMcse as u32);
        }

        // WotLK+ flags
        if to_version < AdtVersion::WotLK {
            chunk.flags &= !(McnkFlags::HasMh2o as u32);
        }

        // Cataclysm+ flags
        if to_version < AdtVersion::Cataclysm {
            chunk.flags &= !(McnkFlags::HasMccr as u32);
        }

        // MoP+ flags
        if to_version < AdtVersion::MoP {
            chunk.flags &= !(McnkFlags::HasTextureScale as u32);
        }

        // Legion+ flags
        if to_version < AdtVersion::Legion {
            chunk.flags &= !(McnkFlags::HasMapObjects as u32);
        }
    }

    // Set has_big_alpha flag for WotLK+
    if to_version >= AdtVersion::WotLK && from_version < AdtVersion::WotLK {
        chunk.flags |= McnkFlags::HasBigAlpha as u32;
    } else if to_version < AdtVersion::WotLK && from_version >= AdtVersion::WotLK {
        chunk.flags &= !(McnkFlags::HasBigAlpha as u32);
    }

    // Update liquid flag
    if to_version >= AdtVersion::WotLK {
        // WotLK+ uses MH2O
        if chunk.flags & (McnkFlags::HasLiquid as u32) != 0 {
            // Change from MCLQ to MH2O
            chunk.flags |= McnkFlags::HasMh2o as u32;
        }
    } else {
        // Pre-WotLK uses MCLQ
        chunk.flags &= !(McnkFlags::HasMh2o as u32);
    }
}

/// Convert a MCNK chunk from Vanilla to TBC
fn vanilla_to_tbc_mcnk(_source: &McnkChunk, _target: &mut McnkChunk) -> Result<()> {
    // TBC added MCSE (sound emitters)
    // No direct conversion needed - just update flags

    Ok(())
}

/// Convert a MCNK chunk from TBC to WotLK
fn tbc_to_wotlk_mcnk(_source: &McnkChunk, target: &mut McnkChunk) -> Result<()> {
    // WotLK changed liquid handling from MCLQ to MH2O
    // Actual conversion is done at the ADT level

    // Set the big alpha flag
    target.flags |= McnkFlags::HasBigAlpha as u32;

    // Clear liquid offset/size (handled by MH2O now)
    if (target.flags & (McnkFlags::HasLiquid as u32)) != 0 {
        target.liquid_offset = 0;
        target.liquid_size = 0;

        // Set MH2O flag
        target.flags |= McnkFlags::HasMh2o as u32;
    }

    Ok(())
}

/// Convert a MCNK chunk from WotLK to Cataclysm
fn wotlk_to_cataclysm_mcnk(_source: &McnkChunk, _target: &mut McnkChunk) -> Result<()> {
    // Cataclysm added cliff textures (MCCR)
    // Not all chunks have this - leave flag unset by default

    Ok(())
}

/// Convert a MCNK chunk from Cataclysm to MoP
fn cataclysm_to_mop_mcnk(_source: &McnkChunk, _target: &mut McnkChunk) -> Result<()> {
    // MoP added texture scaling
    // Not all chunks have this - leave flag unset by default

    Ok(())
}

/// Convert a MCNK chunk from TBC to Vanilla
fn tbc_to_vanilla_mcnk(_source: &McnkChunk, target: &mut McnkChunk) -> Result<()> {
    // Remove TBC-specific features
    target.flags &= !(McnkFlags::HasMcse as u32);

    Ok(())
}

/// Convert a MCNK chunk from WotLK to TBC
fn wotlk_to_tbc_mcnk(_source: &McnkChunk, target: &mut McnkChunk) -> Result<()> {
    // WotLK to TBC - revert liquid handling
    // Actual conversion is done at the ADT level

    // Clear WotLK-specific flags
    target.flags &= !((McnkFlags::HasMh2o as u32) | (McnkFlags::HasBigAlpha as u32));

    Ok(())
}

/// Convert a MCNK chunk from Cataclysm to WotLK
fn cataclysm_to_wotlk_mcnk(_source: &McnkChunk, target: &mut McnkChunk) -> Result<()> {
    // Remove Cataclysm-specific features
    target.flags &= !(McnkFlags::HasMccr as u32);

    Ok(())
}

/// Convert a MCNK chunk from MoP to Cataclysm
fn mop_to_cataclysm_mcnk(_source: &McnkChunk, target: &mut McnkChunk) -> Result<()> {
    // Remove MoP-specific features
    target.flags &= !(McnkFlags::HasTextureScale as u32);

    Ok(())
}

/// Convert multiple MCNK chunks
pub fn convert_mcnk_chunks(
    chunks: &[McnkChunk],
    from_version: AdtVersion,
    to_version: AdtVersion,
) -> Result<Vec<McnkChunk>> {
    let mut result = Vec::with_capacity(chunks.len());

    for chunk in chunks {
        let converted = convert_mcnk(chunk, from_version, to_version)?;
        result.push(converted);
    }

    Ok(result)
}
