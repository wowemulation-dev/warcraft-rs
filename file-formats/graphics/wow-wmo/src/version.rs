/// WMO format versions corresponding to different WoW expansions/patches
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
#[repr(u32)]
pub enum WmoVersion {
    /// Classic/Vanilla (1.x)
    Classic = 17,

    /// The Burning Crusade (2.x)
    Tbc = 18,

    /// Wrath of the Lich King (3.x)
    Wotlk = 19,

    /// Cataclysm (4.x)
    Cataclysm = 20,

    /// Mists of Pandaria (5.x)
    Mop = 21,

    /// Warlords of Draenor (6.x)
    Wod = 22,

    /// Legion (7.x)
    Legion = 23,

    /// Battle for Azeroth (8.x)
    Bfa = 24,

    /// Shadowlands (9.x)
    Shadowlands = 25,

    /// Dragonflight (10.x)
    Dragonflight = 26,

    /// The War Within (11.x)
    WarWithin = 27,
}

impl WmoVersion {
    /// Convert a raw version number to a WmoVersion
    pub fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            17 => Some(Self::Classic),
            18 => Some(Self::Tbc),
            19 => Some(Self::Wotlk),
            20 => Some(Self::Cataclysm),
            21 => Some(Self::Mop),
            22 => Some(Self::Wod),
            23 => Some(Self::Legion),
            24 => Some(Self::Bfa),
            25 => Some(Self::Shadowlands),
            26 => Some(Self::Dragonflight),
            27 => Some(Self::WarWithin),
            _ => None,
        }
    }

    /// Get the raw version number
    pub fn to_raw(self) -> u32 {
        self as u32
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
