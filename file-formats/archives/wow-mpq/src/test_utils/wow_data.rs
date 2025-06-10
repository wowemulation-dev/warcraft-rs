//! Utilities for finding WoW game data for examples and tests

use std::path::{Path, PathBuf};

/// Represents different WoW versions with their associated data paths
#[derive(Debug, Clone, Copy)]
pub enum WowVersion {
    /// World of Warcraft: Classic (1.12.1)
    Vanilla,
    /// The Burning Crusade (2.4.3)
    Tbc,
    /// Wrath of the Lich King (3.3.5a)
    Wotlk,
    /// Cataclysm (4.3.4)
    Cata,
    /// Mists of Pandaria (5.4.8)
    Mop,
}

impl WowVersion {
    /// Get the version string for this WoW version
    pub fn version_string(&self) -> &'static str {
        match self {
            WowVersion::Vanilla => "1.12.1",
            WowVersion::Tbc => "2.4.3",
            WowVersion::Wotlk => "3.3.5a",
            WowVersion::Cata => "4.3.4",
            WowVersion::Mop => "5.4.8",
        }
    }

    /// Get the environment variable name for this version
    pub fn env_var(&self) -> &'static str {
        match self {
            WowVersion::Vanilla => "WOW_VANILLA_DATA",
            WowVersion::Tbc => "WOW_TBC_DATA",
            WowVersion::Wotlk => "WOW_WOTLK_DATA",
            WowVersion::Cata => "WOW_CATA_DATA",
            WowVersion::Mop => "WOW_MOP_DATA",
        }
    }

    /// Get a human-readable name for this version
    pub fn display_name(&self) -> &'static str {
        match self {
            WowVersion::Vanilla => "World of Warcraft: Classic (1.12.1)",
            WowVersion::Tbc => "The Burning Crusade (2.4.3)",
            WowVersion::Wotlk => "Wrath of the Lich King (3.3.5a)",
            WowVersion::Cata => "Cataclysm (4.3.4)",
            WowVersion::Mop => "Mists of Pandaria (5.4.8)",
        }
    }
}

/// Attempts to locate WoW data for a specific version
pub fn find_wow_data(version: WowVersion) -> Option<PathBuf> {
    // Strategy 1: Check environment variable
    if let Ok(path) = std::env::var(version.env_var()) {
        let path = PathBuf::from(path);
        if path.exists() && path.is_dir() {
            return Some(path);
        }
    }

    // Strategy 2: Check common installation paths
    let common_paths = get_common_wow_paths(version);
    common_paths
        .into_iter()
        .find(|path| path.exists() && path.is_dir())
}

/// Get common installation paths for a WoW version
fn get_common_wow_paths(version: WowVersion) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Common base directories
    let base_dirs = if cfg!(windows) {
        vec![
            "C:\\Program Files\\World of Warcraft",
            "C:\\Program Files (x86)\\World of Warcraft",
            "C:\\Games\\World of Warcraft",
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            "/Applications/World of Warcraft",
            "~/Applications/World of Warcraft",
        ]
    } else {
        vec![
            "~/wow",
            "~/Downloads/wow",
            "/opt/wow",
            "/usr/local/games/wow",
        ]
    };

    for base in base_dirs {
        let base = PathBuf::from(base);

        // Handle different directory structures per version
        match version {
            WowVersion::Vanilla => {
                paths.push(base.join("1.12.1/Data"));
                paths.push(base.join("vanilla/Data"));
                paths.push(base.join("classic/Data"));
                paths.push(base.join("Data")); // Direct Data folder
            }
            WowVersion::Tbc => {
                paths.push(base.join("2.4.3/Data"));
                paths.push(base.join("tbc/Data"));
                paths.push(base.join("burning-crusade/Data"));
            }
            WowVersion::Wotlk => {
                paths.push(base.join("3.3.5a/Data"));
                paths.push(base.join("wotlk/Data"));
                paths.push(base.join("wrath/Data"));
            }
            WowVersion::Cata => {
                paths.push(base.join("4.3.4/4.3.4/Data"));
                paths.push(base.join("4.3.4/Data"));
                paths.push(base.join("cata/Data"));
                paths.push(base.join("cataclysm/Data"));
            }
            WowVersion::Mop => {
                paths.push(base.join("5.4.8/5.4.8/Data"));
                paths.push(base.join("5.4.8/Data"));
                paths.push(base.join("mop/Data"));
                paths.push(base.join("pandaria/Data"));
            }
        }
    }

    paths
}

/// Find any available WoW data directory from any version
pub fn find_any_wow_data() -> Option<(WowVersion, PathBuf)> {
    for &version in &[
        WowVersion::Vanilla,
        WowVersion::Tbc,
        WowVersion::Wotlk,
        WowVersion::Cata,
        WowVersion::Mop,
    ] {
        if let Some(path) = find_wow_data(version) {
            return Some((version, path));
        }
    }
    None
}

/// Get a specific MPQ file path for a version
pub fn get_mpq_path(version: WowVersion, mpq_name: &str) -> Option<PathBuf> {
    find_wow_data(version).map(|data_path| data_path.join(mpq_name))
}

/// Check if a WoW data directory contains expected MPQ files
pub fn validate_wow_data(path: &Path) -> bool {
    // Check for common MPQ files that should exist
    let expected_files = ["patch.MPQ", "patch.mpq", "dbc.MPQ", "misc.MPQ"];

    expected_files.iter().any(|&file| path.join(file).exists())
}

/// Print instructions for setting up WoW data paths
pub fn print_setup_instructions() {
    println!("WoW Data Setup Instructions:");
    println!("===========================");
    println!();
    println!("To run examples that require WoW game files, set one or more environment variables:");
    println!();

    for &version in &[
        WowVersion::Vanilla,
        WowVersion::Tbc,
        WowVersion::Wotlk,
        WowVersion::Cata,
        WowVersion::Mop,
    ] {
        println!(
            "  {} = /path/to/{}/Data",
            version.env_var(),
            version.version_string()
        );
        println!("    For: {}", version.display_name());
        println!();
    }

    println!("Example:");
    println!("  export WOW_VANILLA_DATA=\"/home/user/wow/1.12.1/Data\"");
    println!("  export WOW_TBC_DATA=\"/home/user/wow/2.4.3/Data\"");
    println!();
    println!("Alternatively, place WoW data in common locations:");

    if cfg!(windows) {
        println!("  C:\\Program Files\\World of Warcraft\\1.12.1\\Data");
        println!("  C:\\Games\\World of Warcraft\\vanilla\\Data");
    } else {
        println!("  ~/wow/1.12.1/Data");
        println!("  ~/Downloads/wow/vanilla/Data");
        println!("  /opt/wow/1.12.1/Data");
    }
}
