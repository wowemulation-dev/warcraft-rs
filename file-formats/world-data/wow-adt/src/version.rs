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
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mver_parsing() {
        // Valid version (18)
        assert!(matches!(AdtVersion::from_mver(18), Ok(AdtVersion::Vanilla)));

        // Invalid version
        assert!(AdtVersion::from_mver(19).is_err());
        assert!(AdtVersion::from_mver(0).is_err());
        assert!(AdtVersion::from_mver(99).is_err());
    }

    #[test]
    fn test_mver_value() {
        // All versions should return 18
        assert_eq!(AdtVersion::Vanilla.to_mver_value(), 18);
        assert_eq!(AdtVersion::TBC.to_mver_value(), 18);
        assert_eq!(AdtVersion::WotLK.to_mver_value(), 18);
        assert_eq!(AdtVersion::Cataclysm.to_mver_value(), 18);
        assert_eq!(AdtVersion::MoP.to_mver_value(), 18);
        assert_eq!(AdtVersion::Legion.to_mver_value(), 18);
    }

    #[test]
    fn test_version_detection() {
        // Vanilla - no special chunks
        assert_eq!(
            AdtVersion::detect_from_chunks(false, false, false, false),
            AdtVersion::Vanilla
        );

        // TBC - has MFBO
        assert_eq!(
            AdtVersion::detect_from_chunks(true, false, false, false),
            AdtVersion::TBC
        );

        // WotLK - has MH2O
        assert_eq!(
            AdtVersion::detect_from_chunks(false, true, false, false),
            AdtVersion::WotLK
        );

        // WotLK - has MCCV
        assert_eq!(
            AdtVersion::detect_from_chunks(false, false, false, true),
            AdtVersion::WotLK
        );

        // Cataclysm - has MTFX
        assert_eq!(
            AdtVersion::detect_from_chunks(false, false, true, false),
            AdtVersion::Cataclysm
        );

        // Cataclysm - has all chunks
        assert_eq!(
            AdtVersion::detect_from_chunks(true, true, true, true),
            AdtVersion::Cataclysm
        );
    }

    #[test]
    fn test_version_comparison() {
        assert!(AdtVersion::Vanilla < AdtVersion::TBC);
        assert!(AdtVersion::TBC < AdtVersion::WotLK);
        assert!(AdtVersion::WotLK < AdtVersion::Cataclysm);
        assert!(AdtVersion::Cataclysm < AdtVersion::MoP);
        assert!(AdtVersion::MoP < AdtVersion::WoD);
        assert!(AdtVersion::WoD < AdtVersion::Legion);
        assert!(AdtVersion::Legion < AdtVersion::BfA);
        assert!(AdtVersion::BfA < AdtVersion::Shadowlands);
        assert!(AdtVersion::Shadowlands < AdtVersion::Dragonflight);

        // Test equality
        assert_eq!(AdtVersion::Vanilla, AdtVersion::Vanilla);
        assert_eq!(AdtVersion::Legion, AdtVersion::Legion);
    }

    #[test]
    fn test_version_to_string() {
        assert_eq!(AdtVersion::Vanilla.to_string(), "Vanilla (1.x)");
        assert_eq!(AdtVersion::TBC.to_string(), "The Burning Crusade (2.x)");
        assert_eq!(
            AdtVersion::WotLK.to_string(),
            "Wrath of the Lich King (3.x)"
        );
        assert_eq!(AdtVersion::Cataclysm.to_string(), "Cataclysm (4.x)");
        assert_eq!(AdtVersion::MoP.to_string(), "Mists of Pandaria (5.x)");
        assert_eq!(AdtVersion::WoD.to_string(), "Warlords of Draenor (6.x)");
        assert_eq!(AdtVersion::Legion.to_string(), "Legion (7.x)");
        assert_eq!(AdtVersion::BfA.to_string(), "Battle for Azeroth (8.x)");
        assert_eq!(AdtVersion::Shadowlands.to_string(), "Shadowlands (9.x)");
        assert_eq!(AdtVersion::Dragonflight.to_string(), "Dragonflight (10.x)");
    }

    #[test]
    fn test_version_ordering() {
        let versions = vec![
            AdtVersion::Dragonflight,
            AdtVersion::Vanilla,
            AdtVersion::Legion,
            AdtVersion::TBC,
            AdtVersion::Cataclysm,
        ];

        let mut sorted = versions.clone();
        sorted.sort();

        assert_eq!(
            sorted,
            vec![
                AdtVersion::Vanilla,
                AdtVersion::TBC,
                AdtVersion::Cataclysm,
                AdtVersion::Legion,
                AdtVersion::Dragonflight,
            ]
        );
    }
}
