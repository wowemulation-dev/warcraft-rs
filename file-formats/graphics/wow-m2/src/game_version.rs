use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GameVersion {
    Vanilla,
    TBC,
    WotLK,
    Cataclysm,
    MoP,
    WoD,
    Legion,
    BfA,
    Shadowlands,
    Dragonflight,
    TheWarWithin,
}

impl GameVersion {
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
            1 => GameVersion::Vanilla,
            2 => GameVersion::TBC,
            3 => GameVersion::WotLK,
            4 => GameVersion::Cataclysm,
            5 => GameVersion::MoP,
            6 => GameVersion::WoD,
            7 => GameVersion::Legion,
            8 => GameVersion::BfA,
            9 => GameVersion::Shadowlands,
            10 => GameVersion::Dragonflight,
            11 => GameVersion::TheWarWithin,
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
            "vanilla" | "classic" => Ok(GameVersion::Vanilla),
            "tbc" | "bc" | "burningcrusade" | "burning_crusade" => Ok(GameVersion::TBC),
            "wotlk" | "wrath" | "lichking" | "lich_king" | "wlk" => Ok(GameVersion::WotLK),
            "cata" | "cataclysm" => Ok(GameVersion::Cataclysm),
            "mop" | "pandaria" | "mists" | "mists_of_pandaria" => Ok(GameVersion::MoP),
            "wod" | "draenor" | "warlords" | "warlords_of_draenor" => Ok(GameVersion::WoD),
            "legion" => Ok(GameVersion::Legion),
            "bfa" | "bfazeroth" | "battle_for_azeroth" | "battleforazeroth" => Ok(GameVersion::BfA),
            "sl" | "shadowlands" => Ok(GameVersion::Shadowlands),
            "df" | "dragonflight" => Ok(GameVersion::Dragonflight),
            "tww" | "warwithin" | "the_war_within" | "thewarwithin" => {
                Ok(GameVersion::TheWarWithin)
            }
            _ => {
                // If it's not a short name, try parsing as a numeric version
                Self::from_string(s)
            }
        }
    }

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
}

impl std::fmt::Display for GameVersion {
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
            GameVersion::from_string("1.12.1").unwrap(),
            GameVersion::Vanilla
        );
        assert_eq!(GameVersion::from_string("2.4.3").unwrap(), GameVersion::TBC);
        assert_eq!(
            GameVersion::from_string("3.3.5a").unwrap(),
            GameVersion::WotLK
        );
        assert_eq!(
            GameVersion::from_string("4.3.4").unwrap(),
            GameVersion::Cataclysm
        );
        assert_eq!(GameVersion::from_string("5.4.8").unwrap(), GameVersion::MoP);
    }

    #[test]
    fn test_version_from_expansion_name() {
        assert_eq!(
            GameVersion::from_expansion_name("classic").unwrap(),
            GameVersion::Vanilla
        );
        assert_eq!(
            GameVersion::from_expansion_name("TBC").unwrap(),
            GameVersion::TBC
        );
        assert_eq!(
            GameVersion::from_expansion_name("wotlk").unwrap(),
            GameVersion::WotLK
        );
        assert_eq!(
            GameVersion::from_expansion_name("cata").unwrap(),
            GameVersion::Cataclysm
        );
        assert_eq!(
            GameVersion::from_expansion_name("MoP").unwrap(),
            GameVersion::MoP
        );

        // Test numeric fallback
        assert_eq!(
            GameVersion::from_expansion_name("3.3.5a").unwrap(),
            GameVersion::WotLK
        );
    }
}
