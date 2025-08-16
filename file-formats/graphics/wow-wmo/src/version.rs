/// WMO format versions corresponding to different WoW expansions/patches
/// Based on empirical analysis: WMO version remains 17 across all analyzed versions (1.12.1-5.4.8)
/// Features are differentiated by chunk presence rather than version numbers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum WmoVersion {
    /// Classic/Vanilla (1.12.1) - Version 17, core chunks only
    Classic,

    /// The Burning Crusade (2.4.3) - Version 17, improved lighting
    Tbc,

    /// Wrath of the Lich King (3.3.5a) - Version 17, skybox support
    Wotlk,

    /// Cataclysm (4.3.4) - Version 17 + MCVP chunk for transport WMOs
    Cataclysm,

    /// Mists of Pandaria (5.4.8) - Version 17 + MCVP chunk, group MOCV support
    Mop,

    /// Warlords of Draenor (6.x) - Theoretical post-MoP versions
    Wod,

    /// Legion (7.x)
    Legion,

    /// Battle for Azeroth (8.x)
    Bfa,

    /// Shadowlands (9.x)
    Shadowlands,

    /// Dragonflight (10.x)
    Dragonflight,

    /// The War Within (11.x)
    WarWithin,
}

impl WmoVersion {
    /// Convert a raw version number to a WmoVersion
    /// Based on empirical analysis: version 17 spans Classic through MoP
    /// Features are determined by expansion context and chunk presence
    pub fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            17 => Some(Self::Classic), // Default to Classic for version 17
            18 => Some(Self::Wod),
            19 => Some(Self::Legion),
            20 => Some(Self::Bfa),
            21 => Some(Self::Shadowlands),
            22 => Some(Self::Dragonflight),
            23 => Some(Self::WarWithin),
            _ => None,
        }
    }

    /// Convert a raw version number to WmoVersion with expansion context
    /// Use this when you know the WoW expansion to get accurate feature detection
    pub fn from_raw_with_expansion(raw: u32, expansion: &str) -> Option<Self> {
        match (raw, expansion.to_lowercase().as_str()) {
            (17, exp) if exp.contains("vanilla") || exp.contains("classic") => Some(Self::Classic),
            (17, exp) if exp.contains("tbc") || exp.contains("burning") => Some(Self::Tbc),
            (17, exp) if exp.contains("wotlk") || exp.contains("wrath") => Some(Self::Wotlk),
            (17, exp) if exp.contains("cata") || exp.contains("cataclysm") => Some(Self::Cataclysm),
            (17, exp) if exp.contains("mop") || exp.contains("pandaria") => Some(Self::Mop),
            (17, _) => Some(Self::Classic), // Default to Classic for unknown v17
            _ => Self::from_raw(raw),       // Delegate to standard method for other versions
        }
    }

    /// Get the raw version number used in WMO files
    /// Returns the actual version number found in WMO headers
    pub fn to_raw(self) -> u32 {
        match self {
            // Empirically verified: all these use version 17
            Self::Classic | Self::Tbc | Self::Wotlk | Self::Cataclysm | Self::Mop => 17,
            // Theoretical post-MoP versions
            Self::Wod => 18,
            Self::Legion => 19,
            Self::Bfa => 20,
            Self::Shadowlands => 21,
            Self::Dragonflight => 22,
            Self::WarWithin => 23,
        }
    }

    /// Get the expansion name as a string
    pub fn expansion_name(self) -> &'static str {
        match self {
            Self::Classic => "Classic/Vanilla",
            Self::Tbc => "The Burning Crusade",
            Self::Wotlk => "Wrath of the Lich King",
            Self::Cataclysm => "Cataclysm",
            Self::Mop => "Mists of Pandaria",
            Self::Wod => "Warlords of Draenor",
            Self::Legion => "Legion",
            Self::Bfa => "Battle for Azeroth",
            Self::Shadowlands => "Shadowlands",
            Self::Dragonflight => "Dragonflight",
            Self::WarWithin => "The War Within",
        }
    }

    /// Get the minimum supported version
    pub fn min_supported() -> Self {
        Self::Classic
    }

    /// Get the maximum supported version
    pub fn max_supported() -> Self {
        Self::WarWithin
    }

    /// Check if this version supports a particular feature
    pub fn supports_feature(self, feature: WmoFeature) -> bool {
        self >= feature.min_version()
    }
}

/// Features introduced in different WMO versions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WmoFeature {
    /// Base WMO features (available in all versions)
    Base,

    /// Improved lighting introduced in TBC
    ImprovedLighting,

    /// Skybox references introduced in WotLK
    SkyboxReferences,

    /// Destructible objects introduced in Cataclysm
    DestructibleObjects,

    /// Convex volume planes (MCVP) introduced in Cataclysm for transport WMOs
    ConvexVolumePlanes,

    /// Extended materials introduced in MoP
    ExtendedMaterials,

    /// Liquid data v2 introduced in WoD
    LiquidV2,

    /// Global ambient color introduced in Legion
    GlobalAmbientColor,

    /// Particle systems introduced in BfA
    ParticleSystems,

    /// Shadow batches introduced in Shadowlands
    ShadowBatches,

    /// Ray-traced shadows introduced in Dragonflight
    RayTracedShadows,

    /// Enhanced materials introduced in The War Within
    EnhancedMaterials,
}

impl WmoFeature {
    /// Get the minimum version that supports this feature
    pub fn min_version(self) -> WmoVersion {
        match self {
            Self::Base => WmoVersion::Classic,
            Self::ImprovedLighting => WmoVersion::Tbc,
            Self::SkyboxReferences => WmoVersion::Wotlk,
            Self::DestructibleObjects => WmoVersion::Cataclysm,
            Self::ConvexVolumePlanes => WmoVersion::Cataclysm,
            Self::ExtendedMaterials => WmoVersion::Mop,
            Self::LiquidV2 => WmoVersion::Wod,
            Self::GlobalAmbientColor => WmoVersion::Legion,
            Self::ParticleSystems => WmoVersion::Bfa,
            Self::ShadowBatches => WmoVersion::Shadowlands,
            Self::RayTracedShadows => WmoVersion::Dragonflight,
            Self::EnhancedMaterials => WmoVersion::WarWithin,
        }
    }
}
