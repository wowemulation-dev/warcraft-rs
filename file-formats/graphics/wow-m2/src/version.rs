use crate::error::Result;

/// M2 format versions across WoW expansions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum M2Version {
    /// Vanilla (1.x)
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
            1 => M2Version::Vanilla,
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
            "vanilla" | "classic" => Ok(M2Version::Vanilla),
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
    /// Updated to handle Legion+ chunked format versions (272+)
    pub fn from_header_version(version: u32) -> Option<Self> {
        match version {
            // Empirically verified exact versions from format analysis
            256 => Some(Self::Vanilla),   // Vanilla 1.12.1
            260 => Some(Self::TBC),       // The Burning Crusade 2.4.3
            264 => Some(Self::WotLK),     // Wrath of the Lich King 3.3.5a
            272 => Some(Self::Cataclysm), // Cataclysm 4.3.4 and MoP 5.4.8

            // Legacy support for version ranges (avoiding overlaps)
            257..=259 => Some(Self::Vanilla),
            261..=263 => Some(Self::TBC),
            265..=271 => Some(Self::WotLK),

            // Legion+ versions (272+ with chunked format support)
            273..=279 => Some(Self::Legion), // Legion 7.x versions
            280..=289 => Some(Self::BfA),    // Battle for Azeroth 8.x versions
            290..=299 => Some(Self::Shadowlands), // Shadowlands 9.x versions
            300..=309 => Some(Self::Dragonflight), // Dragonflight 10.x versions
            310..=399 => Some(Self::TheWarWithin), // The War Within 11.x versions

            // MoP uses same version as Cataclysm based on empirical findings
            // Alternative version numbering for post-MoP (legacy compatibility)
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
    /// Updated to handle Legion+ chunked format versions
    pub fn to_header_version(&self) -> u32 {
        match self {
            // Empirically verified versions from format analysis
            Self::Vanilla => 256,   // Vanilla 1.12.1
            Self::TBC => 260,       // The Burning Crusade 2.4.3
            Self::WotLK => 264,     // Wrath of the Lich King 3.3.5a
            Self::Cataclysm => 272, // Cataclysm 4.3.4
            Self::MoP => 272,       // MoP 5.4.8 (same as Cataclysm)

            // Post-MoP versions with chunked format support
            Self::WoD => 275,          // Theoretical WoD chunked version
            Self::Legion => 276,       // Legion chunked format base version
            Self::BfA => 280,          // Battle for Azeroth chunked version
            Self::Shadowlands => 290,  // Shadowlands chunked version
            Self::Dragonflight => 300, // Dragonflight chunked version
            Self::TheWarWithin => 310, // The War Within chunked version
        }
    }

    /// Get the WoW expansion name for this version
    pub fn expansion_name(&self) -> &'static str {
        match self {
            Self::Vanilla => "Vanilla",
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
            Self::Vanilla => "1.12.1",
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
    /// but not actually used until Legion (versions 272+)
    pub fn supports_chunked_format(&self) -> bool {
        match self {
            Self::Vanilla | Self::TBC => false,
            Self::WotLK | Self::Cataclysm | Self::MoP => true, // Capability exists but unused
            Self::WoD
            | Self::Legion
            | Self::BfA
            | Self::Shadowlands
            | Self::Dragonflight
            | Self::TheWarWithin => true,
        }
    }

    /// Returns true if this version uses external chunks
    /// Based on empirical analysis: no external chunks found through MoP 5.4.8
    /// External chunks introduced with Legion+ (versions 272+)
    pub fn uses_external_chunks(&self) -> bool {
        match self {
            Self::Vanilla | Self::TBC | Self::WotLK | Self::Cataclysm | Self::MoP | Self::WoD => {
                false
            }
            Self::Legion
            | Self::BfA
            | Self::Shadowlands
            | Self::Dragonflight
            | Self::TheWarWithin => true,
        }
    }

    /// Returns true if this version uses inline data structure
    /// Based on empirical analysis: 100% inline data through MoP 5.4.8
    /// Legion+ versions use chunked data with FileDataID references
    pub fn uses_inline_data(&self) -> bool {
        match self {
            Self::Vanilla | Self::TBC | Self::WotLK | Self::Cataclysm | Self::MoP | Self::WoD => {
                true
            }
            Self::Legion
            | Self::BfA
            | Self::Shadowlands
            | Self::Dragonflight
            | Self::TheWarWithin => false,
        }
    }

    /// Returns true if this version uses the new skin format (with version field)
    ///
    /// WotLK introduced external .skin files but used the old format (no version field).
    /// Cataclysm introduced the new skin format with a version field.
    ///
    /// - Old format: magic + arrays (no version field) - used by WotLK and earlier
    /// - New format: magic + version + name + vertex_count + arrays - used by Cataclysm+
    pub fn uses_new_skin_format(&self) -> bool {
        match self {
            Self::Vanilla | Self::TBC | Self::WotLK => false,
            Self::Cataclysm
            | Self::MoP
            | Self::WoD
            | Self::Legion
            | Self::BfA
            | Self::Shadowlands
            | Self::Dragonflight
            | Self::TheWarWithin => true,
        }
    }

    /// Get the empirically verified version number for this M2 version
    /// Returns None if the version was not part of the empirical analysis
    pub fn empirical_version_number(&self) -> Option<u32> {
        match self {
            Self::Vanilla => Some(256),
            Self::TBC => Some(260),
            Self::WotLK => Some(264),
            Self::Cataclysm => Some(272),
            Self::MoP => Some(272),
            _ => None, // Post-MoP versions not empirically verified but have theoretical versions
        }
    }

    /// Returns true if this version requires chunked format parsing (MD21)
    /// Legion+ versions (272+) use chunked format exclusively
    pub fn requires_chunked_format(&self) -> bool {
        matches!(
            self,
            Self::Legion | Self::BfA | Self::Shadowlands | Self::Dragonflight | Self::TheWarWithin
        )
    }

    /// Detect expansion from version number including Legion+ support
    /// Updated to handle versions 272+ as Legion+
    pub fn detect_expansion(version: u32) -> M2Version {
        match version {
            256..=259 => M2Version::Vanilla,
            260..=263 => M2Version::TBC,
            264..=271 => M2Version::WotLK,
            272 => M2Version::Cataclysm, // Could also be MoP
            273..=279 => M2Version::Legion,
            280..=289 => M2Version::BfA,
            290..=299 => M2Version::Shadowlands,
            300..=309 => M2Version::Dragonflight,
            310.. => M2Version::TheWarWithin,
            _ => M2Version::Vanilla, // Default fallback
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
            M2Version::Vanilla
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
            M2Version::Vanilla
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
            Some(M2Version::Vanilla)
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
            Some(M2Version::Vanilla)
        );
        assert_eq!(M2Version::from_header_version(261), Some(M2Version::TBC));
        assert_eq!(M2Version::from_header_version(265), Some(M2Version::WotLK));

        // Legion+ versions (chunked format)
        assert_eq!(M2Version::from_header_version(273), Some(M2Version::Legion));
        assert_eq!(M2Version::from_header_version(276), Some(M2Version::Legion));
        assert_eq!(M2Version::from_header_version(280), Some(M2Version::BfA));
        assert_eq!(
            M2Version::from_header_version(290),
            Some(M2Version::Shadowlands)
        );
        assert_eq!(
            M2Version::from_header_version(300),
            Some(M2Version::Dragonflight)
        );
        assert_eq!(
            M2Version::from_header_version(310),
            Some(M2Version::TheWarWithin)
        );

        // Legacy alternative versions
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
        assert_eq!(M2Version::from_header_version(999), None); // Use a version clearly outside any range
    }

    #[test]
    fn test_conversion_paths() {
        assert!(M2Version::Vanilla.has_direct_conversion_path(&M2Version::TBC));
        assert!(M2Version::TBC.has_direct_conversion_path(&M2Version::WotLK));
        assert!(M2Version::WotLK.has_direct_conversion_path(&M2Version::Cataclysm));
        assert!(!M2Version::Vanilla.has_direct_conversion_path(&M2Version::MoP));
        assert!(!M2Version::Vanilla.has_direct_conversion_path(&M2Version::TheWarWithin));
    }

    #[test]
    fn test_empirical_version_features() {
        // Test chunked format support
        assert!(!M2Version::Vanilla.supports_chunked_format());
        assert!(!M2Version::TBC.supports_chunked_format());
        assert!(M2Version::WotLK.supports_chunked_format());
        assert!(M2Version::Cataclysm.supports_chunked_format());
        assert!(M2Version::MoP.supports_chunked_format());

        // Test external chunks usage (introduced in Legion+)
        assert!(!M2Version::Vanilla.uses_external_chunks());
        assert!(!M2Version::TBC.uses_external_chunks());
        assert!(!M2Version::WotLK.uses_external_chunks());
        assert!(!M2Version::Cataclysm.uses_external_chunks());
        assert!(!M2Version::MoP.uses_external_chunks());
        assert!(!M2Version::WoD.uses_external_chunks());
        assert!(M2Version::Legion.uses_external_chunks());
        assert!(M2Version::BfA.uses_external_chunks());

        // Test inline data usage (100% through WoD, chunked after)
        assert!(M2Version::Vanilla.uses_inline_data());
        assert!(M2Version::TBC.uses_inline_data());
        assert!(M2Version::WotLK.uses_inline_data());
        assert!(M2Version::Cataclysm.uses_inline_data());
        assert!(M2Version::MoP.uses_inline_data());
        assert!(M2Version::WoD.uses_inline_data());
        assert!(!M2Version::Legion.uses_inline_data());
        assert!(!M2Version::BfA.uses_inline_data());
    }

    #[test]
    fn test_empirical_version_numbers() {
        assert_eq!(M2Version::Vanilla.empirical_version_number(), Some(256));
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
        assert_eq!(M2Version::Vanilla.to_header_version(), 256);
        assert_eq!(M2Version::TBC.to_header_version(), 260);
        assert_eq!(M2Version::WotLK.to_header_version(), 264);
        assert_eq!(M2Version::Cataclysm.to_header_version(), 272);
        assert_eq!(M2Version::MoP.to_header_version(), 272);

        // Test Legion+ versions
        assert_eq!(M2Version::Legion.to_header_version(), 276);
        assert_eq!(M2Version::BfA.to_header_version(), 280);
        assert_eq!(M2Version::Shadowlands.to_header_version(), 290);

        // Verify roundtrip for empirically verified versions
        assert_eq!(
            M2Version::from_header_version(M2Version::Vanilla.to_header_version()),
            Some(M2Version::Vanilla)
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

        // Test Legion+ roundtrip
        assert_eq!(
            M2Version::from_header_version(M2Version::Legion.to_header_version()),
            Some(M2Version::Legion)
        );
    }

    #[test]
    fn test_chunked_format_detection() {
        // Test chunked format requirement detection
        assert!(!M2Version::Vanilla.requires_chunked_format());
        assert!(!M2Version::TBC.requires_chunked_format());
        assert!(!M2Version::WotLK.requires_chunked_format());
        assert!(!M2Version::Cataclysm.requires_chunked_format());
        assert!(!M2Version::MoP.requires_chunked_format());
        assert!(!M2Version::WoD.requires_chunked_format());
        assert!(M2Version::Legion.requires_chunked_format());
        assert!(M2Version::BfA.requires_chunked_format());
    }

    #[test]
    fn test_expansion_detection() {
        assert_eq!(M2Version::detect_expansion(256), M2Version::Vanilla);
        assert_eq!(M2Version::detect_expansion(260), M2Version::TBC);
        assert_eq!(M2Version::detect_expansion(264), M2Version::WotLK);
        assert_eq!(M2Version::detect_expansion(272), M2Version::Cataclysm);
        assert_eq!(M2Version::detect_expansion(273), M2Version::Legion);
        assert_eq!(M2Version::detect_expansion(276), M2Version::Legion);
        assert_eq!(M2Version::detect_expansion(280), M2Version::BfA);
        assert_eq!(M2Version::detect_expansion(310), M2Version::TheWarWithin);
    }
}
