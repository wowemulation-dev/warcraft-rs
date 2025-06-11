use crate::error::{Result, WmoError};
use crate::version::{WmoFeature, WmoVersion};
use crate::wmo_group_types::{WmoGroup, WmoLiquid};
use crate::wmo_types::{WmoFlags, WmoHeader, WmoMaterial, WmoMaterialFlags, WmoRoot};
use tracing::{info, warn};

// Use WmoGroupFlags from wmo_group_types since that's where WmoGroupHeader uses it
use crate::wmo_group_types::WmoGroupFlags;

/// Converter for WMO files between different versions
pub struct WmoConverter;

impl Default for WmoConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl WmoConverter {
    /// Create a new WMO converter
    pub fn new() -> Self {
        Self
    }

    /// Convert a WMO root file from its current version to a target version
    pub fn convert_root(&self, wmo: &mut WmoRoot, target_version: WmoVersion) -> Result<()> {
        if wmo.version == target_version {
            info!(
                "WMO is already at target version {}",
                target_version.to_raw()
            );
            return Ok(());
        }

        // Check if conversion is supported
        if target_version < WmoVersion::min_supported()
            || target_version > WmoVersion::max_supported()
        {
            return Err(WmoError::UnsupportedConversion {
                from: wmo.version.to_raw(),
                to: target_version.to_raw(),
            });
        }

        // Are we upgrading or downgrading?
        let is_upgrading = target_version > wmo.version;

        info!(
            "{} WMO from version {} to {}",
            if is_upgrading {
                "Upgrading"
            } else {
                "Downgrading"
            },
            wmo.version.to_raw(),
            target_version.to_raw()
        );

        // Update header flags based on version
        self.convert_header_flags(&mut wmo.header, wmo.version, target_version);

        // Update materials for version changes
        self.convert_materials(&mut wmo.materials, wmo.version, target_version)?;

        // Handle skybox changes
        if !target_version.supports_feature(WmoFeature::SkyboxReferences) && wmo.skybox.is_some() {
            warn!(
                "Target version {} does not support skyboxes, removing skybox",
                target_version.to_raw()
            );
            wmo.skybox = None;
            wmo.header.flags &= !WmoFlags::HAS_SKYBOX;
        }

        // Update version number
        wmo.version = target_version;

        Ok(())
    }

    /// Convert a WMO group file from its current version to a target version
    pub fn convert_group(
        &self,
        group: &mut WmoGroup,
        target_version: WmoVersion,
        current_version: WmoVersion,
    ) -> Result<()> {
        if current_version == target_version {
            info!(
                "WMO group is already at target version {}",
                target_version.to_raw()
            );
            return Ok(());
        }

        // Check if conversion is supported
        if target_version < WmoVersion::min_supported()
            || target_version > WmoVersion::max_supported()
        {
            return Err(WmoError::UnsupportedConversion {
                from: current_version.to_raw(),
                to: target_version.to_raw(),
            });
        }

        // Are we upgrading or downgrading?
        let is_upgrading = target_version > current_version;

        info!(
            "{} WMO group from version {} to {}",
            if is_upgrading {
                "Upgrading"
            } else {
                "Downgrading"
            },
            current_version.to_raw(),
            target_version.to_raw()
        );

        // Update group flags based on version
        self.convert_group_flags(&mut group.header.flags, current_version, target_version);

        // Handle liquid data changes if needed
        if group.liquid.is_some() {
            if target_version >= WmoVersion::Wod
                && !current_version.supports_feature(WmoFeature::LiquidV2)
            {
                // Upgrade to LiquidV2 format (more complex liquid data)
                self.upgrade_liquid_to_v2(group.liquid.as_mut().unwrap())?;
            } else if target_version < WmoVersion::Wod
                && current_version.supports_feature(WmoFeature::LiquidV2)
            {
                // Downgrade from LiquidV2 format (simplify liquid data)
                self.downgrade_liquid_from_v2(group.liquid.as_mut().unwrap())?;
            }
        }

        Ok(())
    }

    /// Convert header flags based on version changes
    fn convert_header_flags(
        &self,
        header: &mut WmoHeader,
        from_version: WmoVersion,
        to_version: WmoVersion,
    ) {
        // Example flag conversion logic
        if to_version >= WmoVersion::Wotlk
            && !from_version.supports_feature(WmoFeature::SkyboxReferences)
        {
            // Clear skybox flag if downgrading below WotLK
            header.flags &= !WmoFlags::HAS_SKYBOX;
        }

        // Note: Material flag conversion is handled in convert_materials method
    }

    /// Convert material definitions based on version changes
    fn convert_materials(
        &self,
        materials: &mut Vec<WmoMaterial>,
        from_version: WmoVersion,
        to_version: WmoVersion,
    ) -> Result<()> {
        // If downgrading to before MoP, ensure materials use old format
        if to_version < WmoVersion::Mop && from_version >= WmoVersion::Mop {
            for material in materials {
                // Clear flags not supported in older versions
                material.flags &=
                    !(WmoMaterialFlags::SHADOW_BATCH_1 | WmoMaterialFlags::SHADOW_BATCH_2);

                // Reset any extended material properties (would be fields beyond the basic format)
            }
        }

        // If upgrading to or beyond MoP, ensure extended material format
        if to_version >= WmoVersion::Mop && from_version < WmoVersion::Mop {
            // No special action needed, as writing will add the extended fields
        }

        Ok(())
    }

    /// Convert group flags based on version changes
    fn convert_group_flags(
        &self,
        flags: &mut WmoGroupFlags,
        _from_version: WmoVersion,
        to_version: WmoVersion,
    ) {
        // Example flag conversion logic
        if to_version < WmoVersion::Cataclysm {
            // Clear flags not supported in older versions
            *flags &= !(WmoGroupFlags::HAS_MORE_MOTION_TYPES
                | WmoGroupFlags::USE_SCENE_GRAPH
                | WmoGroupFlags::EXTERIOR_BSP);
        }

        if to_version < WmoVersion::Legion {
            // Clear flags not supported in older versions
            *flags &= !WmoGroupFlags::MOUNT_ALLOWED;
        }
    }

    /// Upgrade liquid data from old format to new LiquidV2 format (WoD+)
    fn upgrade_liquid_to_v2(&self, liquid: &mut WmoLiquid) -> Result<()> {
        // In real implementation, this would convert old liquid data format to new one
        // For our example, we'll just update the flags
        liquid.flags |= 0x2; // Set the V2 format flag

        Ok(())
    }

    /// Downgrade liquid data from LiquidV2 format to old format (pre-WoD)
    fn downgrade_liquid_from_v2(&self, liquid: &mut WmoLiquid) -> Result<()> {
        // In real implementation, this would simplify liquid data to older format
        // For our example, we'll just update the flags
        liquid.flags &= !0x2; // Clear the V2 format flag

        Ok(())
    }
}
