// version.rs - Version handling for ADT files

use crate::error::{AdtError, Result};

/// Represents the different World of Warcraft versions that ADT files can be from
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AdtVersion {
    /// Vanilla WoW (1.x)
    Vanilla,
    /// The Burning Crusade (2.x)
    TBC,
    /// Wrath of the Lich King (3.x)
    WotLK,
    /// Cataclysm (4.x)
    Cataclysm,
    /// Mists of Pandaria (5.x)
    MoP,
    /// Warlords of Draenor (6.x)
    WoD,
    /// Legion (7.x)
    Legion,
    /// Battle for Azeroth (8.x)
    BfA,
    /// Shadowlands (9.x)
    Shadowlands,
    /// Dragonflight (10.x)
    Dragonflight,
}

impl AdtVersion {
    /// Convert a version number from the MVER chunk to an AdtVersion enum
    pub fn from_mver(version: u32) -> Result<Self> {
        // According to wowdev.wiki, the MVER value doesn't seem to change between versions
        // The ADT version is determined more by the presence of certain chunks
        // However, the value is supposed to be 18 for most versions
        match version {
            18 => {
                // Default to Vanilla, but the caller should update based on chunks
                Ok(AdtVersion::Vanilla)
            }
            _ => Err(AdtError::UnsupportedVersion(version)),
        }
    }

    /// Get the corresponding MVER value for this version
    pub fn to_mver_value(self) -> u32 {
        // The MVER value is 18 for all known versions
        18
    }

    /// Detect version based on chunk presence
    /// This should be called after initial parsing to refine the version detection
    pub fn detect_from_chunks(
        has_mfbo: bool,
        has_mh2o: bool,
        has_mtfx: bool,
        has_mcnk_with_mccv: bool,
    ) -> Self {
        // Version detection based on chunk presence:
        // - MTFX: Cataclysm+ (4.x+)
        // - MH2O: WotLK+ (3.x+)
        // - MFBO: TBC+ (2.x+)
        // - MCCV in MCNK: WotLK+ (3.x+)

        if has_mtfx {
            // MTFX was introduced in Cataclysm
            // TODO: Further differentiate between Cata, MoP, WoD, etc. based on other indicators
            AdtVersion::Cataclysm
        } else if has_mh2o || has_mcnk_with_mccv {
            // MH2O and MCCV were introduced in WotLK
            AdtVersion::WotLK
        } else if has_mfbo {
            // MFBO was introduced in TBC
            AdtVersion::TBC
        } else {
            // No version-specific chunks found, assume Vanilla
            AdtVersion::Vanilla
        }
    }
}

impl std::fmt::Display for AdtVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AdtVersion::Vanilla => "Vanilla (1.x)",
            AdtVersion::TBC => "The Burning Crusade (2.x)",
            AdtVersion::WotLK => "Wrath of the Lich King (3.x)",
            AdtVersion::Cataclysm => "Cataclysm (4.x)",
            AdtVersion::MoP => "Mists of Pandaria (5.x)",
            AdtVersion::WoD => "Warlords of Draenor (6.x)",
            AdtVersion::Legion => "Legion (7.x)",
            AdtVersion::BfA => "Battle for Azeroth (8.x)",
            AdtVersion::Shadowlands => "Shadowlands (9.x)",
            AdtVersion::Dragonflight => "Dragonflight (10.x)",
        };
        write!(f, "{}", s)
    }
}
