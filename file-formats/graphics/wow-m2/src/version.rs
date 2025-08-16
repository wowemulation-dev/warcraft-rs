use crate::error::Result;

/// M2 format versions across WoW expansions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum M2Version {
    /// Classic/Vanilla (1.x)
    Classic,

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

    /// The War Within (11.x+)
    TheWarWithin,
}

impl M2Version {
    /// Parse version from a string (e.g., "1.12.1", "3.3.5a", "4.3.4")
    pub fn from_string(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.is_empty() {
            return Err(crate::error::M2Error::UnsupportedVersion(format!(
                "Invalid version string: {s}"
            )));
        }

        let major = parts[0].parse::<u32>().map_err(|_| {
            crate::error::M2Error::UnsupportedVersion(format!(
                "Invalid major version: {}",
                parts[0]
            ))
        })?;

        Ok(match major {
            1 => M2Version::Classic,
            2 => M2Version::TBC,
            3 => M2Version::WotLK,
            4 => M2Version::Cataclysm,
            5 => M2Version::MoP,
            6 => M2Version::WoD,
            7 => M2Version::Legion,
            8 => M2Version::BfA,
            9 => M2Version::Shadowlands,
            10 => M2Version::Dragonflight,
            11 => M2Version::TheWarWithin,
            _ => {
                return Err(crate::error::M2Error::UnsupportedVersion(format!(
                    "Unknown WoW version: {major}"
                )));
            }
        })
    }

    /// Parse version from expansion short names or numeric versions
    /// Supports both numeric versions (e.g., "3.3.5a") and short names (e.g., "WotLK", "TBC")
    pub fn from_expansion_name(s: &str) -> Result<Self> {
        // First try parsing as a short name
        match s.to_lowercase().as_str() {
            "vanilla" | "classic" => Ok(M2Version::Classic),
            "tbc" | "bc" | "burningcrusade" | "burning_crusade" => Ok(M2Version::TBC),
            "wotlk" | "wrath" | "lichking" | "lich_king" | "wlk" => Ok(M2Version::WotLK),
            "cata" | "cataclysm" => Ok(M2Version::Cataclysm),
            "mop" | "pandaria" | "mists" | "mists_of_pandaria" => Ok(M2Version::MoP),
            "wod" | "draenor" | "warlords" | "warlords_of_draenor" => Ok(M2Version::WoD),
            "legion" => Ok(M2Version::Legion),
            "bfa" | "bfazeroth" | "battle_for_azeroth" | "battleforazeroth" => Ok(M2Version::BfA),
            "sl" | "shadowlands" => Ok(M2Version::Shadowlands),
            "df" | "dragonflight" => Ok(M2Version::Dragonflight),
            "tww" | "warwithin" | "the_war_within" | "thewarwithin" => Ok(M2Version::TheWarWithin),
            _ => {
                // If it's not a short name, try parsing as a numeric version
                Self::from_string(s)
            }
        }
    }

    /// Convert header version number to M2Version enum
    /// Based on empirical analysis of WoW versions 1.12.1 through 5.4.8
    pub fn from_header_version(version: u32) -> Option<Self> {
        match version {
            // Empirically verified exact versions from format analysis
            256 => Some(Self::Classic),   // Vanilla 1.12.1
            260 => Some(Self::TBC),       // The Burning Crusade 2.4.3
            264 => Some(Self::WotLK),     // Wrath of the Lich King 3.3.5a
            272 => Some(Self::Cataclysm), // Cataclysm 4.3.4 and MoP 5.4.8

            // Legacy support for version ranges (avoiding overlaps)
            257..=259 => Some(Self::Classic),
            261..=263 => Some(Self::TBC),
            265..=271 => Some(Self::WotLK),

            // MoP uses same version as Cataclysm based on empirical findings
            // Post-MoP versions (theoretical, not empirically verified)
            8 => Some(Self::MoP),
            10 => Some(Self::WoD),
            11 => Some(Self::Legion),
            16 => Some(Self::BfA),
            17 => Some(Self::Shadowlands),
            18 => Some(Self::Dragonflight),
            19 => Some(Self::TheWarWithin),

            _ => None,
        }
    }

    /// Convert M2Version enum to header version number
    /// Returns empirically verified version numbers for WoW 1.12.1 through 5.4.8
    pub fn to_header_version(&self) -> u32 {
        match self {
            // Empirically verified versions from format analysis
            Self::Classic => 256,   // Vanilla 1.12.1
            Self::TBC => 260,       // The Burning Crusade 2.4.3
            Self::WotLK => 264,     // Wrath of the Lich King 3.3.5a
            Self::Cataclysm => 272, // Cataclysm 4.3.4
            Self::MoP => 272,       // MoP 5.4.8 (same as Cataclysm)

            // Post-MoP versions (theoretical, not empirically verified)
            Self::WoD => 10,
            Self::Legion => 11,
            Self::BfA => 16,
            Self::Shadowlands => 17,
            Self::Dragonflight => 18,
            Self::TheWarWithin => 19,
        }
    }

    /// Get the WoW expansion name for this version
    pub fn expansion_name(&self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::TBC => "The Burning Crusade",
            Self::WotLK => "Wrath of the Lich King",
            Self::Cataclysm => "Cataclysm",
            Self::MoP => "Mists of Pandaria",
            Self::WoD => "Warlords of Draenor",
            Self::Legion => "Legion",
            Self::BfA => "Battle for Azeroth",
            Self::Shadowlands => "Shadowlands",
            Self::Dragonflight => "Dragonflight",
            Self::TheWarWithin => "The War Within",
        }
    }

    /// Get common version string representation (e.g., "3.3.5a" for WotLK)
    pub fn to_version_string(&self) -> &'static str {
        match self {
            Self::Classic => "1.12.1",
            Self::TBC => "2.4.3",
            Self::WotLK => "3.3.5a",
            Self::Cataclysm => "4.3.4",
            Self::MoP => "5.4.8",
            Self::WoD => "6.2.4",
            Self::Legion => "7.3.5",
            Self::BfA => "8.3.7",
            Self::Shadowlands => "9.2.7",
            Self::Dragonflight => "10.2.0",
            Self::TheWarWithin => "11.0.0",
        }
    }

    /// Check if a direct conversion path exists between two versions
    pub fn has_direct_conversion_path(&self, target: &Self) -> bool {
        // Adjacent versions typically have direct conversion paths
        let self_ord = *self as usize;
        let target_ord = *target as usize;

        (self_ord as isize - target_ord as isize).abs() == 1
    }

    /// Returns true if this version supports chunked format capability
    /// Based on empirical analysis: chunked format capability introduced in v264 (WotLK)
    /// but not actually used until post-MoP expansions
    pub fn supports_chunked_format(&self) -> bool {
        match self {
            Self::Classic | Self::TBC => false,
            Self::WotLK | Self::Cataclysm | Self::MoP => true, // Capability exists but unused
            _ => true, // Post-MoP versions may use chunked format
        }
    }

    /// Returns true if this version uses external chunks
    /// Based on empirical analysis: no external chunks found through MoP 5.4.8
    /// All data remains inline in the main M2 file
    pub fn uses_external_chunks(&self) -> bool {
        match self {
            Self::Classic | Self::TBC | Self::WotLK | Self::Cataclysm | Self::MoP => false,
            _ => false, // Even post-MoP versions may not use external chunks
        }
    }

    /// Returns true if this version uses inline data structure
    /// Based on empirical analysis: 100% inline data through MoP 5.4.8
    pub fn uses_inline_data(&self) -> bool {
        match self {
            Self::Classic | Self::TBC | Self::WotLK | Self::Cataclysm | Self::MoP => true,
            _ => true, // Assume inline data for newer versions too
        }
    }

    /// Get the empirically verified version number for this M2 version
    /// Returns None if the version was not part of the empirical analysis
    pub fn empirical_version_number(&self) -> Option<u32> {
        match self {
            Self::Classic => Some(256),
            Self::TBC => Some(260),
            Self::WotLK => Some(264),
            Self::Cataclysm => Some(272),
            Self::MoP => Some(272),
            _ => None, // Post-MoP versions not empirically verified
        }
    }
}

impl std::fmt::Display for M2Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({})",
            self.expansion_name(),
            self.to_version_string()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_from_string() {
        assert_eq!(
            M2Version::from_string("1.12.1").unwrap(),
            M2Version::Classic
        );
        assert_eq!(M2Version::from_string("2.4.3").unwrap(), M2Version::TBC);
        assert_eq!(M2Version::from_string("3.3.5a").unwrap(), M2Version::WotLK);
        assert_eq!(
            M2Version::from_string("4.3.4").unwrap(),
            M2Version::Cataclysm
        );
        assert_eq!(M2Version::from_string("5.4.8").unwrap(), M2Version::MoP);
    }

    #[test]
    fn test_version_from_expansion_name() {
        assert_eq!(
            M2Version::from_expansion_name("classic").unwrap(),
            M2Version::Classic
        );
        assert_eq!(
            M2Version::from_expansion_name("TBC").unwrap(),
            M2Version::TBC
        );
        assert_eq!(
            M2Version::from_expansion_name("wotlk").unwrap(),
            M2Version::WotLK
        );
        assert_eq!(
            M2Version::from_expansion_name("cata").unwrap(),
            M2Version::Cataclysm
        );
        assert_eq!(
            M2Version::from_expansion_name("MoP").unwrap(),
            M2Version::MoP
        );

        // Test numeric fallback
        assert_eq!(
            M2Version::from_expansion_name("3.3.5a").unwrap(),
            M2Version::WotLK
        );
    }

    #[test]
    fn test_header_version_conversion() {
        // Empirically verified versions
        assert_eq!(
            M2Version::from_header_version(256),
            Some(M2Version::Classic)
        );
        assert_eq!(M2Version::from_header_version(260), Some(M2Version::TBC));
        assert_eq!(M2Version::from_header_version(264), Some(M2Version::WotLK));
        assert_eq!(
            M2Version::from_header_version(272),
            Some(M2Version::Cataclysm)
        );

        // Legacy support ranges
        assert_eq!(
            M2Version::from_header_version(257),
            Some(M2Version::Classic)
        );
        assert_eq!(M2Version::from_header_version(261), Some(M2Version::TBC));
        assert_eq!(M2Version::from_header_version(265), Some(M2Version::WotLK));

        // Later versions
        assert_eq!(M2Version::from_header_version(8), Some(M2Version::MoP));
        assert_eq!(M2Version::from_header_version(10), Some(M2Version::WoD));
        assert_eq!(M2Version::from_header_version(11), Some(M2Version::Legion));
        assert_eq!(M2Version::from_header_version(16), Some(M2Version::BfA));
        assert_eq!(
            M2Version::from_header_version(17),
            Some(M2Version::Shadowlands)
        );
        assert_eq!(
            M2Version::from_header_version(18),
            Some(M2Version::Dragonflight)
        );
        assert_eq!(
            M2Version::from_header_version(19),
            Some(M2Version::TheWarWithin)
        );

        // Unknown versions
        assert_eq!(M2Version::from_header_version(1), None);
        assert_eq!(M2Version::from_header_version(5), None);
        assert_eq!(M2Version::from_header_version(273), None);
    }

    #[test]
    fn test_conversion_paths() {
        assert!(M2Version::Classic.has_direct_conversion_path(&M2Version::TBC));
        assert!(M2Version::TBC.has_direct_conversion_path(&M2Version::WotLK));
        assert!(M2Version::WotLK.has_direct_conversion_path(&M2Version::Cataclysm));
        assert!(!M2Version::Classic.has_direct_conversion_path(&M2Version::MoP));
        assert!(!M2Version::Classic.has_direct_conversion_path(&M2Version::TheWarWithin));
    }

    #[test]
    fn test_empirical_version_features() {
        // Test chunked format support
        assert!(!M2Version::Classic.supports_chunked_format());
        assert!(!M2Version::TBC.supports_chunked_format());
        assert!(M2Version::WotLK.supports_chunked_format());
        assert!(M2Version::Cataclysm.supports_chunked_format());
        assert!(M2Version::MoP.supports_chunked_format());

        // Test external chunks usage (none found through MoP)
        assert!(!M2Version::Classic.uses_external_chunks());
        assert!(!M2Version::TBC.uses_external_chunks());
        assert!(!M2Version::WotLK.uses_external_chunks());
        assert!(!M2Version::Cataclysm.uses_external_chunks());
        assert!(!M2Version::MoP.uses_external_chunks());

        // Test inline data usage (100% through MoP)
        assert!(M2Version::Classic.uses_inline_data());
        assert!(M2Version::TBC.uses_inline_data());
        assert!(M2Version::WotLK.uses_inline_data());
        assert!(M2Version::Cataclysm.uses_inline_data());
        assert!(M2Version::MoP.uses_inline_data());
    }

    #[test]
    fn test_empirical_version_numbers() {
        assert_eq!(M2Version::Classic.empirical_version_number(), Some(256));
        assert_eq!(M2Version::TBC.empirical_version_number(), Some(260));
        assert_eq!(M2Version::WotLK.empirical_version_number(), Some(264));
        assert_eq!(M2Version::Cataclysm.empirical_version_number(), Some(272));
        assert_eq!(M2Version::MoP.empirical_version_number(), Some(272));

        // Post-MoP versions not empirically verified
        assert_eq!(M2Version::WoD.empirical_version_number(), None);
        assert_eq!(M2Version::Legion.empirical_version_number(), None);
    }

    #[test]
    fn test_header_version_roundtrip() {
        // Test that empirically verified versions roundtrip correctly
        assert_eq!(M2Version::Classic.to_header_version(), 256);
        assert_eq!(M2Version::TBC.to_header_version(), 260);
        assert_eq!(M2Version::WotLK.to_header_version(), 264);
        assert_eq!(M2Version::Cataclysm.to_header_version(), 272);
        assert_eq!(M2Version::MoP.to_header_version(), 272);

        // Verify roundtrip for empirically verified versions
        assert_eq!(
            M2Version::from_header_version(M2Version::Classic.to_header_version()),
            Some(M2Version::Classic)
        );
        assert_eq!(
            M2Version::from_header_version(M2Version::TBC.to_header_version()),
            Some(M2Version::TBC)
        );
        assert_eq!(
            M2Version::from_header_version(M2Version::WotLK.to_header_version()),
            Some(M2Version::WotLK)
        );
        assert_eq!(
            M2Version::from_header_version(M2Version::Cataclysm.to_header_version()),
            Some(M2Version::Cataclysm)
        );
    }
}
