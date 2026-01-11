use crate::chunk_discovery::ChunkLocation;
use crate::chunk_id::ChunkId;
use std::collections::HashMap;

/// WoW client version detected from ADT chunk analysis.
///
/// ADT files across all WoW versions use `MVER = 18`, making traditional version
/// detection impossible. Instead, version is determined by analyzing which chunks
/// are present in the file, as different expansions introduced new chunk types.
///
/// Detection follows this hierarchy (most recent features first):
/// - `MTXP` (texture parameters) → Mists of Pandaria 5.x
/// - `MAMP` (texture amplitude) → Cataclysm 4.x
/// - `MH2O` (height-based water) → Wrath of the Lich King 3.x
/// - `MFBO` (flight bounds) → The Burning Crusade 2.x
/// - `MCCV` (vertex colors) → Vanilla 1.9+
/// - No distinguishing chunks → Vanilla early (1.0-1.8.4)
///
/// This approach is necessary because Blizzard never incremented the format
/// version number despite adding new features across 15+ years of development.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AdtVersion {
    /// Vanilla 1.x (early) - No MCCV vertex colors
    ///
    /// Basic terrain format without vertex coloring support.
    VanillaEarly,

    /// Vanilla 1.9+ - Has MCCV vertex colors
    ///
    /// Added per-vertex color support for terrain shading.
    VanillaLate,

    /// The Burning Crusade 2.x - Added MFBO flight bounds
    ///
    /// Introduced flight boundary data for flying mount restrictions.
    TBC,

    /// Wrath of the Lich King 3.x - Added MH2O water
    ///
    /// New height-based water system replacing old liquid chunks.
    WotLK,

    /// Cataclysm 4.x - Added MAMP, split files
    ///
    /// Introduced texture amplitude and split ADT into _obj0/_tex0 files.
    Cataclysm,

    /// Mists of Pandaria 5.x - Added MTXP
    ///
    /// Added texture parameter chunk for advanced texture blending.
    MoP,
}

impl AdtVersion {
    /// Detect version from chunk presence (primary detection method).
    ///
    /// Since all ADT versions use `MVER = 18`, version detection analyzes which
    /// chunks exist in the file. The algorithm checks for signature chunks
    /// introduced in each expansion, starting with the most recent.
    ///
    /// Detection algorithm:
    /// 1. `MTXP` present → Mists of Pandaria (5.x)
    /// 2. `MAMP` present OR (has `MCNK` but no `MCIN`) → Cataclysm (4.x)
    ///    - Split root files have `MCNK` but `MCIN` moved to _tex0.adt
    /// 3. `MH2O` present → Wrath of the Lich King (3.x)
    /// 4. `MFBO` present → The Burning Crusade (2.x)
    /// 5. `MCCV` present → Vanilla 1.9+
    /// 6. Otherwise → Vanilla early (1.0-1.8.4)
    ///
    /// # Arguments
    ///
    /// * `chunks` - Map of chunk IDs to their locations
    ///
    /// # Returns
    ///
    /// Detected ADT version based on chunk presence
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_adt::version::AdtVersion;
    /// use wow_adt::chunk_id::ChunkId;
    /// use wow_adt::chunk_discovery::ChunkLocation;
    /// use std::collections::HashMap;
    ///
    /// let mut chunks = HashMap::new();
    /// chunks.insert(ChunkId::MH2O, vec![ChunkLocation { offset: 0x1000, size: 100 }]);
    /// chunks.insert(ChunkId::MCCV, vec![ChunkLocation { offset: 0x2000, size: 200 }]);
    ///
    /// let version = AdtVersion::detect_from_chunks(&chunks);
    /// assert_eq!(version, AdtVersion::WotLK);
    /// ```
    pub fn detect_from_chunks(chunks: &HashMap<ChunkId, Vec<ChunkLocation>>) -> Self {
        // Check for split file architecture (Cataclysm+)
        // Split root files have MCNK but NO MCIN (MCIN moved to _tex0.adt)
        let has_mcnk = chunks.contains_key(&ChunkId::MCNK);
        let has_mcin = chunks.contains_key(&ChunkId::MCIN);
        let is_split_root = has_mcnk && !has_mcin;

        if chunks.contains_key(&ChunkId::MTXP) {
            Self::MoP
        } else if chunks.contains_key(&ChunkId::MAMP) || is_split_root {
            // Cataclysm: Either has MAMP or is split root file
            Self::Cataclysm
        } else if chunks.contains_key(&ChunkId::MH2O) || chunks.contains_key(&ChunkId::MTXF) {
            // WotLK introduced both MH2O (water) and MTXF (texture flags)
            // MTXF is used as fallback marker when MH2O is not present
            Self::WotLK
        } else if chunks.contains_key(&ChunkId::MFBO) {
            Self::TBC
        } else if chunks.contains_key(&ChunkId::MCCV) {
            Self::VanillaLate
        } else {
            Self::VanillaEarly
        }
    }

    /// Detect version from ChunkDiscovery result (convenience method).
    ///
    /// This method provides a direct way to detect version from a
    /// `ChunkDiscovery` result, delegating to `detect_from_chunks()`.
    ///
    /// # Arguments
    ///
    /// * `discovery` - Chunk discovery results from phase 1 parsing
    ///
    /// # Returns
    ///
    /// Detected ADT version based on chunk presence
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use wow_adt::chunk_discovery::discover_chunks;
    /// use wow_adt::version::AdtVersion;
    ///
    /// let mut file = File::open("Kalimdor_32_48.adt")?;
    /// let discovery = discover_chunks(&mut file)?;
    /// let version = AdtVersion::from_discovery(&discovery);
    ///
    /// println!("Detected version: {}", version);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_discovery(discovery: &crate::chunk_discovery::ChunkDiscovery) -> Self {
        Self::detect_from_chunks(&discovery.chunks)
    }

    /// Human-readable version string.
    ///
    /// Returns expansion name and major version number.
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_adt::version::AdtVersion;
    ///
    /// assert_eq!(AdtVersion::WotLK.as_str(), "Wrath of the Lich King 3.x");
    /// assert_eq!(AdtVersion::VanillaLate.as_str(), "Vanilla 1.9+");
    /// ```
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::VanillaEarly => "Vanilla 1.x (Early)",
            Self::VanillaLate => "Vanilla 1.9+",
            Self::TBC => "The Burning Crusade 2.x",
            Self::WotLK => "Wrath of the Lich King 3.x",
            Self::Cataclysm => "Cataclysm 4.x",
            Self::MoP => "Mists of Pandaria 5.x",
        }
    }

    /// Client version range supported by this ADT format.
    ///
    /// Returns the specific client version numbers that use this format.
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_adt::version::AdtVersion;
    ///
    /// assert_eq!(AdtVersion::WotLK.version_range(), "3.0.0 - 3.3.5a");
    /// assert_eq!(AdtVersion::VanillaEarly.version_range(), "1.0.0 - 1.8.4");
    /// ```
    #[must_use]
    pub const fn version_range(&self) -> &'static str {
        match self {
            Self::VanillaEarly => "1.0.0 - 1.8.4",
            Self::VanillaLate => "1.9.0 - 1.12.1",
            Self::TBC => "2.0.0 - 2.4.3",
            Self::WotLK => "3.0.0 - 3.3.5a",
            Self::Cataclysm => "4.0.0 - 4.3.4",
            Self::MoP => "5.0.0 - 5.4.8",
        }
    }
}

impl AdtVersion {
    /// Parse version from expansion short names.
    ///
    /// Supports short names like "WotLK", "TBC", "Classic", etc. for CLI consistency
    /// with other format converters (M2, WMO).
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_adt::AdtVersion;
    ///
    /// assert_eq!(AdtVersion::from_expansion_name("wotlk"), Some(AdtVersion::WotLK));
    /// assert_eq!(AdtVersion::from_expansion_name("Classic"), Some(AdtVersion::VanillaLate));
    /// assert_eq!(AdtVersion::from_expansion_name("invalid"), None);
    /// ```
    #[must_use]
    pub fn from_expansion_name(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "vanilla" | "classic" => Some(Self::VanillaLate),
            "vanilla_early" | "vanillaearly" => Some(Self::VanillaEarly),
            "tbc" | "bc" | "burningcrusade" | "burning_crusade" => Some(Self::TBC),
            "wotlk" | "wrath" | "lichking" | "lich_king" | "wlk" => Some(Self::WotLK),
            "cata" | "cataclysm" => Some(Self::Cataclysm),
            "mop" | "pandaria" | "mists" | "mists_of_pandaria" => Some(Self::MoP),
            _ => None,
        }
    }

    /// Get the short expansion name for display.
    ///
    /// Returns a concise name suitable for CLI output.
    #[must_use]
    pub const fn expansion_name(&self) -> &'static str {
        match self {
            Self::VanillaEarly => "Vanilla (Early)",
            Self::VanillaLate => "Classic/Vanilla",
            Self::TBC => "The Burning Crusade",
            Self::WotLK => "Wrath of the Lich King",
            Self::Cataclysm => "Cataclysm",
            Self::MoP => "Mists of Pandaria",
        }
    }
}

impl std::fmt::Display for AdtVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_mop_from_mtxp() {
        let mut chunks = HashMap::new();
        chunks.insert(
            ChunkId::MTXP,
            vec![ChunkLocation {
                offset: 0x1000,
                size: 100,
            }],
        );

        let version = AdtVersion::detect_from_chunks(&chunks);
        assert_eq!(version, AdtVersion::MoP);
    }

    #[test]
    fn detect_cataclysm_from_mamp() {
        let mut chunks = HashMap::new();
        chunks.insert(
            ChunkId::MAMP,
            vec![ChunkLocation {
                offset: 0x1000,
                size: 100,
            }],
        );
        chunks.insert(
            ChunkId::MH2O,
            vec![ChunkLocation {
                offset: 0x2000,
                size: 200,
            }],
        );

        let version = AdtVersion::detect_from_chunks(&chunks);
        assert_eq!(version, AdtVersion::Cataclysm);
    }

    #[test]
    fn detect_wotlk_from_mh2o() {
        let mut chunks = HashMap::new();
        chunks.insert(
            ChunkId::MH2O,
            vec![ChunkLocation {
                offset: 0x1000,
                size: 100,
            }],
        );
        chunks.insert(
            ChunkId::MCCV,
            vec![ChunkLocation {
                offset: 0x2000,
                size: 200,
            }],
        );

        let version = AdtVersion::detect_from_chunks(&chunks);
        assert_eq!(version, AdtVersion::WotLK);
    }

    #[test]
    fn detect_tbc_from_mfbo() {
        let mut chunks = HashMap::new();
        chunks.insert(
            ChunkId::MFBO,
            vec![ChunkLocation {
                offset: 0x1000,
                size: 100,
            }],
        );
        chunks.insert(
            ChunkId::MCCV,
            vec![ChunkLocation {
                offset: 0x2000,
                size: 200,
            }],
        );

        let version = AdtVersion::detect_from_chunks(&chunks);
        assert_eq!(version, AdtVersion::TBC);
    }

    #[test]
    fn detect_vanilla_late_from_mccv() {
        let mut chunks = HashMap::new();
        chunks.insert(
            ChunkId::MCCV,
            vec![ChunkLocation {
                offset: 0x1000,
                size: 100,
            }],
        );

        let version = AdtVersion::detect_from_chunks(&chunks);
        assert_eq!(version, AdtVersion::VanillaLate);
    }

    #[test]
    fn detect_vanilla_early_from_no_signature_chunks() {
        let chunks = HashMap::new();

        let version = AdtVersion::detect_from_chunks(&chunks);
        assert_eq!(version, AdtVersion::VanillaEarly);
    }

    #[test]
    fn version_strings_are_descriptive() {
        assert_eq!(AdtVersion::MoP.as_str(), "Mists of Pandaria 5.x");
        assert_eq!(AdtVersion::WotLK.as_str(), "Wrath of the Lich King 3.x");
        assert_eq!(AdtVersion::VanillaEarly.as_str(), "Vanilla 1.x (Early)");
    }

    #[test]
    fn version_ranges_are_accurate() {
        assert_eq!(AdtVersion::MoP.version_range(), "5.0.0 - 5.4.8");
        assert_eq!(AdtVersion::WotLK.version_range(), "3.0.0 - 3.3.5a");
        assert_eq!(AdtVersion::VanillaEarly.version_range(), "1.0.0 - 1.8.4");
    }

    #[test]
    fn display_implementation_matches_as_str() {
        let version = AdtVersion::Cataclysm;
        assert_eq!(format!("{version}"), version.as_str());
    }

    #[test]
    fn test_detect_from_discovery() {
        use crate::chunk_discovery::ChunkDiscovery;

        let mut discovery = ChunkDiscovery::new(1000);

        // Add MH2O chunk to indicate WotLK version
        discovery.chunks.insert(
            ChunkId::MH2O,
            vec![ChunkLocation {
                offset: 100,
                size: 200,
            }],
        );

        let version = AdtVersion::from_discovery(&discovery);
        assert_eq!(version, AdtVersion::WotLK);
    }

    #[test]
    fn test_from_discovery_with_multiple_versions() {
        use crate::chunk_discovery::ChunkDiscovery;

        let mut discovery = ChunkDiscovery::new(2000);

        // Add multiple version-indicating chunks (MTXP takes precedence)
        discovery.chunks.insert(
            ChunkId::MTXP,
            vec![ChunkLocation {
                offset: 100,
                size: 50,
            }],
        );
        discovery.chunks.insert(
            ChunkId::MAMP,
            vec![ChunkLocation {
                offset: 200,
                size: 100,
            }],
        );
        discovery.chunks.insert(
            ChunkId::MH2O,
            vec![ChunkLocation {
                offset: 300,
                size: 150,
            }],
        );

        let version = AdtVersion::from_discovery(&discovery);
        // Should detect as MoP (most recent)
        assert_eq!(version, AdtVersion::MoP);
    }

    #[test]
    fn test_from_discovery_vanilla_early() {
        use crate::chunk_discovery::ChunkDiscovery;

        let discovery = ChunkDiscovery::new(500);
        // No version-indicating chunks

        let version = AdtVersion::from_discovery(&discovery);
        assert_eq!(version, AdtVersion::VanillaEarly);
    }
}
