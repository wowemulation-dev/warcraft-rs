//! WoW version handling and version-specific behaviors

use crate::error::{Error, Result};
use std::fmt;

/// WoW expansion versions that affect WDT format
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WowVersion {
    /// Classic/Vanilla (1.x)
    Classic,
    /// The Burning Crusade (2.x)
    TBC,
    /// Wrath of the Lich King (3.x)
    WotLK,
    /// Cataclysm (4.x) - First major format change
    Cataclysm,
    /// Mists of Pandaria (5.x)
    MoP,
    /// Warlords of Draenor (6.x)
    WoD,
    /// Legion (7.x) - Added auxiliary WDT files
    Legion,
    /// Battle for Azeroth (8.x) - Added MAID chunk
    BfA,
    /// Shadowlands (9.x)
    Shadowlands,
    /// Dragonflight (10.x)
    Dragonflight,
}

impl WowVersion {
    /// Parse version from a string (e.g., "1.12.1", "3.3.5a", "4.3.4")
    pub fn from_string(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.is_empty() {
            return Err(Error::ValidationError(format!(
                "Invalid version string: {s}"
            )));
        }

        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| Error::ValidationError(format!("Invalid major version: {}", parts[0])))?;

        Ok(match major {
            1 => WowVersion::Classic,
            2 => WowVersion::TBC,
            3 => WowVersion::WotLK,
            4 => WowVersion::Cataclysm,
            5 => WowVersion::MoP,
            6 => WowVersion::WoD,
            7 => WowVersion::Legion,
            8 => WowVersion::BfA,
            9 => WowVersion::Shadowlands,
            10 => WowVersion::Dragonflight,
            _ => {
                return Err(Error::ValidationError(format!(
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
            "vanilla" | "classic" => Ok(WowVersion::Classic),
            "tbc" | "bc" | "burningcrusade" | "burning_crusade" => Ok(WowVersion::TBC),
            "wotlk" | "wrath" | "lichking" | "lich_king" | "wlk" => Ok(WowVersion::WotLK),
            "cata" | "cataclysm" => Ok(WowVersion::Cataclysm),
            "mop" | "pandaria" | "mists" | "mists_of_pandaria" => Ok(WowVersion::MoP),
            "wod" | "draenor" | "warlords" | "warlords_of_draenor" => Ok(WowVersion::WoD),
            "legion" => Ok(WowVersion::Legion),
            "bfa" | "bfazeroth" | "battle_for_azeroth" | "battleforazeroth" => Ok(WowVersion::BfA),
            "sl" | "shadowlands" => Ok(WowVersion::Shadowlands),
            "df" | "dragonflight" => Ok(WowVersion::Dragonflight),
            _ => {
                // If it's not a short name, try parsing as a numeric version
                Self::from_string(s)
            }
        }
    }

    /// Check if this version has terrain maps with MWMO chunks
    pub fn has_terrain_mwmo(&self) -> bool {
        // Pre-Cataclysm versions have empty MWMO chunks in terrain maps
        *self < WowVersion::Cataclysm
    }

    /// Check if this version supports MAID chunk
    pub fn has_maid_chunk(&self) -> bool {
        // MAID was introduced in BfA 8.1.0
        *self >= WowVersion::BfA
    }

    /// Check if this version has auxiliary WDT files
    pub fn has_auxiliary_files(&self) -> bool {
        // Auxiliary files (_lgt.wdt, etc.) were added in Legion
        *self >= WowVersion::Legion
    }

    /// Get the expected MODF scale value for this version
    pub fn expected_modf_scale(&self) -> u16 {
        match self {
            // Vanilla through WotLK use 0
            WowVersion::Classic | WowVersion::TBC | WowVersion::WotLK => 0,
            // Later versions typically use 1024 (1.0)
            _ => 1024,
        }
    }

    /// Get the expected MODF unique ID for this version
    pub fn expected_modf_unique_id(&self) -> u32 {
        match self {
            // Early versions use 0xFFFFFFFF (-1)
            WowVersion::Classic | WowVersion::TBC | WowVersion::WotLK => 0xFFFFFFFF,
            // Later versions may use 0 or other values
            _ => 0,
        }
    }

    /// Check if a specific MPHD flag is commonly used in this version
    pub fn is_flag_common(&self, flag: u32) -> bool {
        match flag {
            0x0001 => true,                       // WMO-only flag used in all versions
            0x0002 => *self >= WowVersion::WotLK, // MCCV widely used from WotLK
            0x0004 => *self >= WowVersion::WotLK, // Big alpha widely used from WotLK
            0x0008 => *self >= WowVersion::WotLK, // Sorted doodads from WotLK
            0x0010 => *self >= WowVersion::WotLK && *self < WowVersion::BfA, // Deprecated in BfA
            0x0040 => *self >= WowVersion::Cataclysm, // Universal from Cataclysm
            0x0080 => *self >= WowVersion::MoP,   // Height texturing active from MoP
            0x0200 => *self >= WowVersion::BfA,   // MAID flag from BfA
            _ => false,
        }
    }

    /// Get a descriptive name for this version
    pub fn name(&self) -> &'static str {
        match self {
            WowVersion::Classic => "Classic",
            WowVersion::TBC => "The Burning Crusade",
            WowVersion::WotLK => "Wrath of the Lich King",
            WowVersion::Cataclysm => "Cataclysm",
            WowVersion::MoP => "Mists of Pandaria",
            WowVersion::WoD => "Warlords of Draenor",
            WowVersion::Legion => "Legion",
            WowVersion::BfA => "Battle for Azeroth",
            WowVersion::Shadowlands => "Shadowlands",
            WowVersion::Dragonflight => "Dragonflight",
        }
    }
}

impl fmt::Display for WowVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Version-specific configuration for WDT handling
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionConfig {
    pub version: WowVersion,
}

impl VersionConfig {
    /// Create a new version configuration
    pub fn new(version: WowVersion) -> Self {
        Self { version }
    }

    /// Validate MPHD flags for this version
    pub fn validate_mphd_flags(&self, flags: u32) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check for flags that shouldn't be present in this version
        if (flags & 0x0200) != 0 && !self.version.has_maid_chunk() {
            warnings.push(format!(
                "Flag 0x0200 (MAID) present but not supported in {}",
                self.version
            ));
        }

        if (flags & 0x0040) != 0 && self.version < WowVersion::Cataclysm {
            warnings.push("Flag 0x0040 present but not expected before Cataclysm".to_string());
        }

        if (flags & 0x0080) != 0 && self.version < WowVersion::MoP {
            warnings.push(
                "Flag 0x0080 (height texturing) present but not active before MoP".to_string(),
            );
        }

        warnings
    }

    /// Check if a chunk should be present based on version and flags
    pub fn should_have_chunk(&self, chunk: &str, is_wmo_only: bool) -> bool {
        match chunk {
            "MVER" | "MPHD" | "MAIN" => true, // Always required
            "MWMO" => {
                // WMO-only maps always have MWMO
                // Terrain maps have MWMO only pre-Cataclysm
                is_wmo_only || self.version.has_terrain_mwmo()
            }
            "MODF" => is_wmo_only, // Only WMO-only maps have MODF
            "MAID" => self.version.has_maid_chunk(), // BfA+ only
            _ => false,
        }
    }
}
