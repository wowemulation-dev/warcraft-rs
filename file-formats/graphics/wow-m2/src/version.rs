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
    pub fn from_header_version(version: u32) -> Option<Self> {
        match version {
            // Classic through WotLK use versions 256-264
            256..=264 => Some(Self::Classic), // Covers all pre-Cata versions

            // Cataclysm uses 272 (actual files), but internally referenced as 4
            272 | 4 => Some(Self::Cataclysm),

            // MoP and later use sequential version numbers
            8 => Some(Self::MoP),
            10 => Some(Self::WoD),
            11 => Some(Self::Legion),
            16 => Some(Self::BfA),
            17 => Some(Self::Shadowlands),
            18 => Some(Self::Dragonflight),
            19..=u32::MAX => Some(Self::TheWarWithin),

            _ => None,
        }
    }

    /// Convert M2Version enum to header version number
    pub fn to_header_version(&self) -> u32 {
        match self {
            // For compatibility, we use the newer simplified version numbers
            Self::Classic | Self::TBC | Self::WotLK => 264, // Use the highest pre-Cata version
            Self::Cataclysm => 272,                         // Use the actual file version, not 4
            Self::MoP => 8,
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
        // Classic versions
        assert_eq!(
            M2Version::from_header_version(256),
            Some(M2Version::Classic)
        );
        assert_eq!(
            M2Version::from_header_version(264),
            Some(M2Version::Classic)
        );

        // Cataclysm
        assert_eq!(
            M2Version::from_header_version(272),
            Some(M2Version::Cataclysm)
        );
        assert_eq!(
            M2Version::from_header_version(4),
            Some(M2Version::Cataclysm)
        );

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
    }

    #[test]
    fn test_conversion_paths() {
        assert!(M2Version::Classic.has_direct_conversion_path(&M2Version::TBC));
        assert!(M2Version::TBC.has_direct_conversion_path(&M2Version::WotLK));
        assert!(M2Version::WotLK.has_direct_conversion_path(&M2Version::Cataclysm));
        assert!(!M2Version::Classic.has_direct_conversion_path(&M2Version::MoP));
        assert!(!M2Version::Classic.has_direct_conversion_path(&M2Version::TheWarWithin));
    }
}
