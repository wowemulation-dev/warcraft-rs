// texture_converter.rs - Conversion of texture layers between ADT versions

use crate::chunk::*;
use crate::error::Result;
use crate::mcnk_subchunks::*;
use crate::version::AdtVersion;

/// Convert texture layers from one version to another
pub fn convert_texture_layers(
    source_layers: &[McnkTextureLayer],
    from_version: AdtVersion,
    to_version: AdtVersion,
) -> Result<Vec<McnkTextureLayer>> {
    match (from_version, to_version) {
        // Vanilla to TBC
        (AdtVersion::Vanilla, AdtVersion::TBC) => vanilla_to_tbc_layers(source_layers),

        // TBC to WotLK
        (AdtVersion::TBC, AdtVersion::WotLK) => tbc_to_wotlk_layers(source_layers),

        // WotLK to Cataclysm
        (AdtVersion::WotLK, AdtVersion::Cataclysm) => wotlk_to_cataclysm_layers(source_layers),

        // Cataclysm to MoP
        (AdtVersion::Cataclysm, AdtVersion::MoP) => cataclysm_to_mop_layers(source_layers),

        // Downgrading
        (AdtVersion::TBC, AdtVersion::Vanilla) => tbc_to_vanilla_layers(source_layers),

        (AdtVersion::WotLK, AdtVersion::TBC) => wotlk_to_tbc_layers(source_layers),

        (AdtVersion::Cataclysm, AdtVersion::WotLK) => cataclysm_to_wotlk_layers(source_layers),

        (AdtVersion::MoP, AdtVersion::Cataclysm) => mop_to_cataclysm_layers(source_layers),

        // Same version - just clone
        (from, to) if from == to => Ok(source_layers.to_vec()),

        // Multiple version jumps - handled by main converter
        // Just return as-is, they'll be processed step by step
        _ => Ok(source_layers.to_vec()),
    }
}

/// Convert texture layers from Vanilla to TBC
fn vanilla_to_tbc_layers(source_layers: &[McnkTextureLayer]) -> Result<Vec<McnkTextureLayer>> {
    // TBC added support for ADPCM compressed alpha maps
    // The flags used in TBC are mostly the same as Vanilla

    let mut result = Vec::with_capacity(source_layers.len());

    for layer in source_layers {
        let new_layer = layer.clone();

        // TBC supports compressed alpha maps - for conversion, we'll leave them uncompressed
        // Don't set the MCLY_FLAGS_COMPRESSED_ALPHA flag

        result.push(new_layer);
    }

    Ok(result)
}

/// Convert texture layers from TBC to WotLK
fn tbc_to_wotlk_layers(source_layers: &[McnkTextureLayer]) -> Result<Vec<McnkTextureLayer>> {
    // WotLK added new animation flags
    // MCLY_FLAGS_ANIMATE_FASTER (0x008)
    // MCLY_FLAGS_ANIMATE_FASTEST (0x010)

    let mut result = Vec::with_capacity(source_layers.len());

    for layer in source_layers {
        let mut new_layer = layer.clone();

        // Check for animation flags and update to the new format if needed
        if has_animation_flags(layer.flags) {
            // If using animation, add appropriate speed flags based on original animation
            // This is a heuristic - we don't know the exact desired speed
            let anim_type = layer.flags & 0x7; // Extract animation type (bits 0-2)

            if anim_type == MclyFlags::Animate3 as u32 {
                // Fast rotation (180°) gets ANIMATE_FASTER flag
                new_layer.flags |= MclyFlags::AnimateFaster as u32;
            }
        }

        result.push(new_layer);
    }

    Ok(result)
}

/// Convert texture layers from WotLK to Cataclysm
fn wotlk_to_cataclysm_layers(source_layers: &[McnkTextureLayer]) -> Result<Vec<McnkTextureLayer>> {
    // Cataclysm added MCLY_FLAGS_ANIMATE_USE_OTHER_LAYER (0x040)
    // This flag allows reusing animation settings from another layer

    let mut result = Vec::with_capacity(source_layers.len());

    for layer in source_layers.iter() {
        let new_layer = layer.clone();

        // For conversion, we don't need to set any new flags
        // The existing flags remain valid in Cataclysm

        result.push(new_layer);
    }

    Ok(result)
}

/// Convert texture layers from Cataclysm to MoP
fn cataclysm_to_mop_layers(source_layers: &[McnkTextureLayer]) -> Result<Vec<McnkTextureLayer>> {
    // MoP didn't significantly change the texture layer format
    // Just clone the layers

    Ok(source_layers.to_vec())
}

/// Convert texture layers from TBC to Vanilla
fn tbc_to_vanilla_layers(source_layers: &[McnkTextureLayer]) -> Result<Vec<McnkTextureLayer>> {
    // Need to remove any TBC-specific flags

    let mut result = Vec::with_capacity(source_layers.len());

    for layer in source_layers {
        let mut new_layer = layer.clone();

        // Remove compressed alpha flag if set
        // (MCLY_FLAGS_COMPRESSED_ALPHA = 0x080)
        new_layer.flags &= !(MclyFlags::CompressedAlpha as u32);

        result.push(new_layer);
    }

    Ok(result)
}

/// Convert texture layers from WotLK to TBC
fn wotlk_to_tbc_layers(source_layers: &[McnkTextureLayer]) -> Result<Vec<McnkTextureLayer>> {
    // Need to remove WotLK-specific flags

    let mut result = Vec::with_capacity(source_layers.len());

    for layer in source_layers {
        let mut new_layer = layer.clone();

        // Remove WotLK-specific animation flags
        // (MCLY_FLAGS_ANIMATE_FASTER = 0x008, MCLY_FLAGS_ANIMATE_FASTEST = 0x010)
        new_layer.flags &=
            !((MclyFlags::AnimateFaster as u32) | (MclyFlags::AnimateFastest as u32));

        result.push(new_layer);
    }

    Ok(result)
}

/// Convert texture layers from Cataclysm to WotLK
fn cataclysm_to_wotlk_layers(source_layers: &[McnkTextureLayer]) -> Result<Vec<McnkTextureLayer>> {
    // Need to remove Cataclysm-specific flags

    let mut result = Vec::with_capacity(source_layers.len());

    for layer in source_layers {
        let mut new_layer = layer.clone();

        // Remove Cataclysm-specific animation flags
        // (MCLY_FLAGS_ANIMATE_USE_OTHER_LAYER = 0x040)
        new_layer.flags &= !(MclyFlags::AnimateUseOtherLayer as u32);

        result.push(new_layer);
    }

    Ok(result)
}

/// Convert texture layers from MoP to Cataclysm
fn mop_to_cataclysm_layers(source_layers: &[McnkTextureLayer]) -> Result<Vec<McnkTextureLayer>> {
    // MoP didn't significantly change the texture layer format
    // Just clone the layers

    Ok(source_layers.to_vec())
}

/// Check if the layer has any animation flags set
fn has_animation_flags(flags: u32) -> bool {
    (flags
        & ((MclyFlags::Animate1 as u32)
            | (MclyFlags::Animate2 as u32)
            | (MclyFlags::Animate3 as u32)))
        != 0
}

/// Convert alpha maps between different versions
pub fn convert_alpha_maps(
    source_alpha_maps: &[Vec<u8>],
    source_layers: &[McnkTextureLayer],
    from_version: AdtVersion,
    to_version: AdtVersion,
) -> Result<Vec<Vec<u8>>> {
    match (from_version, to_version) {
        // WotLK+ uses 64x64 alpha maps (big alpha)
        (from, to) if from < AdtVersion::WotLK && to >= AdtVersion::WotLK => {
            upgrade_alpha_maps_to_big(source_alpha_maps, source_layers)
        }

        // Downgrade from WotLK+ to earlier versions
        (from, to) if from >= AdtVersion::WotLK && to < AdtVersion::WotLK => {
            downgrade_alpha_maps_from_big(source_alpha_maps, source_layers)
        }

        // Same version format - just clone
        _ => Ok(source_alpha_maps.to_vec()),
    }
}

/// Upgrade alpha maps from 32x32 to 64x64 (for WotLK+)
fn upgrade_alpha_maps_to_big(
    source_alpha_maps: &[Vec<u8>],
    source_layers: &[McnkTextureLayer],
) -> Result<Vec<Vec<u8>>> {
    let mut result = Vec::with_capacity(source_alpha_maps.len());

    for (i, alpha_map) in source_alpha_maps.iter().enumerate() {
        // Get the corresponding layer (skip the base layer)
        let layer_idx = i + 1;
        if layer_idx >= source_layers.len() {
            // No corresponding layer, just copy as-is
            result.push(alpha_map.clone());
            continue;
        }

        let _layer = &source_layers[layer_idx];

        // Check if the alpha map should be expanded from 32x32 to 64x64
        if alpha_map.len() == 32 * 32 {
            // Create a new 64x64 alpha map
            let mut new_alpha = vec![0u8; 64 * 64];

            // Scale the 32x32 map to 64x64 using bilinear interpolation
            for y in 0..64 {
                for x in 0..64 {
                    // Map from 64x64 to 32x32 coordinates
                    let src_x = (x as f32 * 31.0 / 63.0) as usize;
                    let src_y = (y as f32 * 31.0 / 63.0) as usize;

                    // Get source value
                    let value = alpha_map[src_y * 32 + src_x];

                    // Set the new value
                    new_alpha[y * 64 + x] = value;
                }
            }

            result.push(new_alpha);
        } else {
            // Unknown size or already 64x64, just copy
            result.push(alpha_map.clone());
        }
    }

    Ok(result)
}

/// Downgrade alpha maps from 64x64 to 32x32 (for pre-WotLK)
fn downgrade_alpha_maps_from_big(
    source_alpha_maps: &[Vec<u8>],
    source_layers: &[McnkTextureLayer],
) -> Result<Vec<Vec<u8>>> {
    let mut result = Vec::with_capacity(source_alpha_maps.len());

    for (i, alpha_map) in source_alpha_maps.iter().enumerate() {
        // Get the corresponding layer (skip the base layer)
        let layer_idx = i + 1;
        if layer_idx >= source_layers.len() {
            // No corresponding layer, just copy as-is
            result.push(alpha_map.clone());
            continue;
        }

        let _layer = &source_layers[layer_idx];

        // Check if the alpha map should be reduced from 64x64 to 32x32
        if alpha_map.len() == 64 * 64 {
            // Create a new 32x32 alpha map
            let mut new_alpha = vec![0u8; 32 * 32];

            // Scale the 64x64 map to 32x32 by averaging 2x2 blocks
            for y in 0..32 {
                for x in 0..32 {
                    // Map to 64x64 coordinates
                    let src_x = x * 2;
                    let src_y = y * 2;

                    // Average 2x2 block
                    let sum = alpha_map[src_y * 64 + src_x] as u32
                        + alpha_map[src_y * 64 + src_x + 1] as u32
                        + alpha_map[(src_y + 1) * 64 + src_x] as u32
                        + alpha_map[(src_y + 1) * 64 + src_x + 1] as u32;

                    // Set the new value (average of 2x2 block)
                    new_alpha[y * 32 + x] = (sum / 4) as u8;
                }
            }

            result.push(new_alpha);
        } else {
            // Unknown size or already 32x32, just copy
            result.push(alpha_map.clone());
        }
    }

    Ok(result)
}

/// Convert area IDs between different versions
pub fn convert_area_id(area_id: u32, from_version: AdtVersion, to_version: AdtVersion) -> u32 {
    // Area IDs changed significantly between expansions
    // This function provides mappings for known area ID changes

    // If same version, no conversion needed
    if from_version == to_version {
        return area_id;
    }

    // Handle special area ID conversions
    match (from_version, to_version, area_id) {
        // Examples of area ID changes between Vanilla and TBC
        (AdtVersion::Vanilla, AdtVersion::TBC, 0) => 0, // Default
        (AdtVersion::Vanilla, AdtVersion::TBC, 1) => 1, // Dun Morogh
        (AdtVersion::Vanilla, AdtVersion::TBC, 12) => 12, // Elwynn Forest
        (AdtVersion::Vanilla, AdtVersion::TBC, 14) => 14, // Durotar

        // Examples of area ID changes between TBC and WotLK
        (AdtVersion::TBC, AdtVersion::WotLK, 0) => 0, // Default
        (AdtVersion::TBC, AdtVersion::WotLK, 3430) => 3430, // Eversong Woods
        (AdtVersion::TBC, AdtVersion::WotLK, 3433) => 3433, // Ghostlands

        // Examples of area ID changes between WotLK and Cataclysm
        (AdtVersion::WotLK, AdtVersion::Cataclysm, 0) => 0, // Default
        (AdtVersion::WotLK, AdtVersion::Cataclysm, 4742) => 4742, // Hrothgar's Landing

        // Handle downgrading area IDs

        // TBC to Vanilla
        (AdtVersion::TBC, AdtVersion::Vanilla, 3430) => 0, // Eversong Woods (didn't exist)
        (AdtVersion::TBC, AdtVersion::Vanilla, 3433) => 0, // Ghostlands (didn't exist)

        // WotLK to TBC
        (AdtVersion::WotLK, AdtVersion::TBC, 4742) => 0, // Hrothgar's Landing (didn't exist)

        // Cataclysm to WotLK
        (AdtVersion::Cataclysm, AdtVersion::WotLK, 5042) => 0, // Deepholm (didn't exist)
        (AdtVersion::Cataclysm, AdtVersion::WotLK, 5095) => 0, // Tol Barad (didn't exist)

        // Default case - keep the same ID if no special conversion exists
        _ => area_id,
    }
}

/// Area ID mapping entry
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AreaIdMapping {
    pub from_version: AdtVersion,
    pub to_version: AdtVersion,
    pub from_id: u32,
    pub to_id: u32,
    pub name: String,
}

/// Get a list of known area ID mappings
#[allow(dead_code)]
fn get_area_id_mappings() -> Vec<AreaIdMapping> {
    vec![
        // Vanilla to TBC
        AreaIdMapping {
            from_version: AdtVersion::Vanilla,
            to_version: AdtVersion::TBC,
            from_id: 1,
            to_id: 1,
            name: "Dun Morogh".to_string(),
        },
        // TBC to WotLK
        AreaIdMapping {
            from_version: AdtVersion::TBC,
            to_version: AdtVersion::WotLK,
            from_id: 3430,
            to_id: 3430,
            name: "Eversong Woods".to_string(),
        },
        // Examples of new areas that didn't exist in previous versions
        AreaIdMapping {
            from_version: AdtVersion::TBC,
            to_version: AdtVersion::Vanilla,
            from_id: 3430,
            to_id: 0,
            name: "Eversong Woods (didn't exist in Vanilla)".to_string(),
        },
        AreaIdMapping {
            from_version: AdtVersion::WotLK,
            to_version: AdtVersion::TBC,
            from_id: 4742,
            to_id: 0,
            name: "Hrothgar's Landing (didn't exist in TBC)".to_string(),
        },
        // Add more mappings as needed
    ]
}
