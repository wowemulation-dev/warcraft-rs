//! Version conversion functionality for WDT files

use crate::{
    WdtFile,
    chunks::{MaidChunk, MphdFlags, MwmoChunk},
    error::Result,
    version::WowVersion,
};

/// Convert a WDT file from one version to another
pub fn convert_wdt(
    wdt: &mut WdtFile,
    from_version: WowVersion,
    to_version: WowVersion,
) -> Result<()> {
    if from_version == to_version {
        return Ok(());
    }

    // Handle pre-Cataclysm to Cataclysm+ conversion
    if from_version < WowVersion::Cataclysm && to_version >= WowVersion::Cataclysm {
        convert_pre_cata_to_cata(wdt)?;
    }

    // Handle Cataclysm+ to pre-Cataclysm conversion
    if from_version >= WowVersion::Cataclysm && to_version < WowVersion::Cataclysm {
        convert_cata_to_pre_cata(wdt)?;
    }

    // Handle pre-BfA to BfA+ conversion
    if from_version < WowVersion::BfA && to_version >= WowVersion::BfA {
        convert_pre_bfa_to_bfa(wdt)?;
    }

    // Handle BfA+ to pre-BfA conversion
    if from_version >= WowVersion::BfA && to_version < WowVersion::BfA {
        convert_bfa_to_pre_bfa(wdt)?;
    }

    // Update version config
    wdt.version_config = crate::version::VersionConfig::new(to_version);

    // Update MODF scale values based on version
    if let Some(ref mut modf) = wdt.modf {
        let expected_scale = to_version.expected_modf_scale();
        for entry in &mut modf.entries {
            entry.scale = expected_scale;
        }
    }

    // Update MODF unique IDs based on version
    if let Some(ref mut modf) = wdt.modf {
        let expected_unique_id = to_version.expected_modf_unique_id();
        for entry in &mut modf.entries {
            entry.unique_id = expected_unique_id;
        }
    }

    // Update flags based on version
    update_flags_for_version(wdt, to_version);

    Ok(())
}

/// Convert pre-Cataclysm format to Cataclysm+ format
fn convert_pre_cata_to_cata(wdt: &mut WdtFile) -> Result<()> {
    // Remove MWMO chunk from terrain maps
    if !wdt.is_wmo_only() && wdt.mwmo.is_some() {
        // Check if MWMO is empty (as it should be for terrain maps)
        if let Some(ref mwmo) = wdt.mwmo {
            if mwmo.is_empty() {
                wdt.mwmo = None;
            }
        }
    }

    // Add universal flag 0x0040
    wdt.mphd.flags |= MphdFlags::UNK_FIRELANDS;

    Ok(())
}

/// Convert Cataclysm+ format to pre-Cataclysm format
fn convert_cata_to_pre_cata(wdt: &mut WdtFile) -> Result<()> {
    // Add empty MWMO chunk to terrain maps
    if !wdt.is_wmo_only() && wdt.mwmo.is_none() {
        wdt.mwmo = Some(MwmoChunk::new());
    }

    // Remove flag 0x0040 (not used pre-Cataclysm)
    wdt.mphd.flags.remove(MphdFlags::UNK_FIRELANDS);

    // Remove flag 0x0080 if present (not active pre-MoP)
    wdt.mphd.flags.remove(MphdFlags::ADT_HAS_HEIGHT_TEXTURING);

    Ok(())
}

/// Convert pre-BfA format to BfA+ format
fn convert_pre_bfa_to_bfa(wdt: &mut WdtFile) -> Result<()> {
    // We can't generate FileDataIDs from filenames
    // So we'll create an empty MAID chunk that needs to be populated
    if wdt.maid.is_none() {
        wdt.maid = Some(MaidChunk::new());
        wdt.mphd.flags |= MphdFlags::WDT_HAS_MAID;

        // Note: The user will need to populate FileDataIDs manually
        // or through a separate mapping system
    }

    // Clear the deprecated lighting vertices flag
    wdt.mphd.flags.remove(MphdFlags::ADT_HAS_LIGHTING_VERTICES);

    Ok(())
}

/// Convert BfA+ format to pre-BfA format
fn convert_bfa_to_pre_bfa(wdt: &mut WdtFile) -> Result<()> {
    // Remove MAID chunk
    if wdt.maid.is_some() {
        wdt.maid = None;
        wdt.mphd.flags.remove(MphdFlags::WDT_HAS_MAID);

        // Clear FileDataID fields in MPHD
        wdt.mphd.clear_file_data_ids();
    }

    Ok(())
}

/// Update flags based on target version
fn update_flags_for_version(wdt: &mut WdtFile, version: WowVersion) {
    // Remove flags that shouldn't exist in the target version
    let _flags_bits = wdt.mphd.flags.bits();

    // Check each flag
    if !version.is_flag_common(0x0002) {
        wdt.mphd.flags.remove(MphdFlags::ADT_HAS_MCCV);
    }
    if !version.is_flag_common(0x0004) {
        wdt.mphd.flags.remove(MphdFlags::ADT_HAS_BIG_ALPHA);
    }
    if !version.is_flag_common(0x0008) {
        wdt.mphd
            .flags
            .remove(MphdFlags::ADT_HAS_DOODADREFS_SORTED_BY_SIZE_CAT);
    }
    if !version.is_flag_common(0x0040) {
        wdt.mphd.flags.remove(MphdFlags::UNK_FIRELANDS);
    }
    if !version.is_flag_common(0x0080) {
        wdt.mphd.flags.remove(MphdFlags::ADT_HAS_HEIGHT_TEXTURING);
    }
    if !version.is_flag_common(0x0200) {
        wdt.mphd.flags.remove(MphdFlags::WDT_HAS_MAID);
    }
}

/// Generate a summary of changes that will be made during conversion
pub fn get_conversion_summary(
    from_version: WowVersion,
    to_version: WowVersion,
    is_wmo_only: bool,
) -> Vec<String> {
    let mut changes = Vec::new();

    if from_version == to_version {
        changes.push("No conversion needed - versions are the same".to_string());
        return changes;
    }

    // Pre-Cataclysm to Cataclysm+
    if from_version < WowVersion::Cataclysm && to_version >= WowVersion::Cataclysm {
        if !is_wmo_only {
            changes.push("Remove empty MWMO chunk from terrain map".to_string());
        }
        changes.push("Add universal flag 0x0040".to_string());
    }

    // Cataclysm+ to pre-Cataclysm
    if from_version >= WowVersion::Cataclysm && to_version < WowVersion::Cataclysm {
        if !is_wmo_only {
            changes.push("Add empty MWMO chunk to terrain map".to_string());
        }
        changes.push("Remove flag 0x0040".to_string());
        changes.push("Remove flag 0x0080 (height texturing)".to_string());
    }

    // Pre-BfA to BfA+
    if from_version < WowVersion::BfA && to_version >= WowVersion::BfA {
        changes.push("Add MAID chunk (FileDataIDs will need to be populated)".to_string());
        changes.push("Add flag 0x0200 (WDT_HAS_MAID)".to_string());
        changes.push("Remove deprecated flag 0x0010 (lighting vertices)".to_string());
    }

    // BfA+ to pre-BfA
    if from_version >= WowVersion::BfA && to_version < WowVersion::BfA {
        changes.push("Remove MAID chunk".to_string());
        changes.push("Remove flag 0x0200 (WDT_HAS_MAID)".to_string());
        changes.push("Clear FileDataID fields in MPHD".to_string());
    }

    // MODF changes
    if from_version.expected_modf_scale() != to_version.expected_modf_scale() {
        changes.push(format!(
            "Update MODF scale values from {} to {}",
            from_version.expected_modf_scale(),
            to_version.expected_modf_scale()
        ));
    }

    if from_version.expected_modf_unique_id() != to_version.expected_modf_unique_id() {
        changes.push(format!(
            "Update MODF unique IDs from 0x{:08X} to 0x{:08X}",
            from_version.expected_modf_unique_id(),
            to_version.expected_modf_unique_id()
        ));
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_cata_to_cata_terrain() {
        let mut wdt = WdtFile::new(WowVersion::WotLK);
        wdt.mwmo = Some(MwmoChunk::new()); // Empty MWMO

        convert_wdt(&mut wdt, WowVersion::WotLK, WowVersion::Cataclysm).unwrap();

        assert!(wdt.mwmo.is_none());
        assert!(wdt.mphd.flags.contains(MphdFlags::UNK_FIRELANDS));
    }

    #[test]
    fn test_cata_to_pre_cata_terrain() {
        let mut wdt = WdtFile::new(WowVersion::Cataclysm);
        wdt.mphd.flags |= MphdFlags::UNK_FIRELANDS;

        convert_wdt(&mut wdt, WowVersion::Cataclysm, WowVersion::WotLK).unwrap();

        assert!(wdt.mwmo.is_some());
        assert!(!wdt.mphd.flags.contains(MphdFlags::UNK_FIRELANDS));
    }

    #[test]
    fn test_pre_bfa_to_bfa() {
        let mut wdt = WdtFile::new(WowVersion::Legion);

        convert_wdt(&mut wdt, WowVersion::Legion, WowVersion::BfA).unwrap();

        assert!(wdt.maid.is_some());
        assert!(wdt.mphd.flags.contains(MphdFlags::WDT_HAS_MAID));
    }

    #[test]
    fn test_conversion_summary() {
        let summary = get_conversion_summary(WowVersion::WotLK, WowVersion::Cataclysm, false);

        assert!(summary.iter().any(|s| s.contains("Remove empty MWMO")));
        assert!(summary.iter().any(|s| s.contains("Add universal flag")));
    }
}
