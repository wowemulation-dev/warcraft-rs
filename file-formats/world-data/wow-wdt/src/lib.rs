//! Parser for World of Warcraft WDT (World Data Table) files.
//!
//! This crate provides support for reading and writing WDT files used in
//! World of Warcraft. WDT files are the main map definition files that
//! reference all ADT tiles and define which areas of the map exist.
//!
//! # Status
//!
//! 🚧 **Under Construction** - This crate is not yet fully implemented.
//!
//! # Examples
//!
//! ```no_run
//! use wow_wdt::WorldDataTable;
//!
//! // This is a placeholder example for future implementation
//! let wdt = WorldDataTable::placeholder();
//! println!("Map name: {}", wdt.map_name());
//! ```

#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use std::fmt;

/// Flags for ADT tiles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TileFlags {
    /// Tile exists
    pub exists: bool,
    /// Tile is fully loaded
    pub loaded: bool,
    /// Tile contains water
    pub has_water: bool,
}

/// Information about a single ADT tile
#[derive(Debug)]
pub struct TileInfo {
    /// X coordinate (0-63)
    pub x: u8,
    /// Y coordinate (0-63)
    pub y: u8,
    /// Tile flags
    pub flags: TileFlags,
}

/// World Data Table - main map definition
#[derive(Debug)]
pub struct WorldDataTable {
    /// Map name
    map_name: String,
    /// Map ID
    map_id: u32,
    /// Tile information (64x64 grid)
    tiles: Vec<Vec<TileInfo>>,
}

impl WorldDataTable {
    /// Create a placeholder WDT for demonstration
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_wdt::WorldDataTable;
    ///
    /// let wdt = WorldDataTable::placeholder();
    /// assert_eq!(wdt.map_name(), "Eastern Kingdoms");
    /// assert_eq!(wdt.map_id(), 0);
    /// ```
    pub fn placeholder() -> Self {
        let mut tiles = Vec::with_capacity(64);
        for y in 0..64 {
            let mut row = Vec::with_capacity(64);
            for x in 0..64 {
                // Create a pattern where center tiles exist
                let exists = (20..44).contains(&x) && (20..44).contains(&y);
                row.push(TileInfo {
                    x: x as u8,
                    y: y as u8,
                    flags: TileFlags {
                        exists,
                        loaded: false,
                        has_water: exists && (x + y) % 7 == 0,
                    },
                });
            }
            tiles.push(row);
        }

        Self {
            map_name: "Eastern Kingdoms".to_string(),
            map_id: 0,
            tiles,
        }
    }

    /// Get the map name
    pub fn map_name(&self) -> &str {
        &self.map_name
    }

    /// Get the map ID
    pub fn map_id(&self) -> u32 {
        self.map_id
    }

    /// Get tile info at specific coordinates
    pub fn tile_at(&self, x: u8, y: u8) -> Option<&TileInfo> {
        self.tiles.get(y as usize)?.get(x as usize)
    }

    /// Check if a tile exists at coordinates
    pub fn tile_exists(&self, x: u8, y: u8) -> bool {
        self.tile_at(x, y).map(|t| t.flags.exists).unwrap_or(false)
    }

    /// Count existing tiles
    pub fn existing_tiles_count(&self) -> usize {
        self.tiles
            .iter()
            .flat_map(|row| row.iter())
            .filter(|tile| tile.flags.exists)
            .count()
    }
}

impl fmt::Display for WorldDataTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "WDT '{}' (Map {}, {} existing tiles)",
            self.map_name,
            self.map_id,
            self.existing_tiles_count()
        )
    }
}

/// Error type for WDT operations
#[derive(Debug, thiserror::Error)]
pub enum WdtError {
    /// Invalid WDT file format
    #[error("Invalid WDT format: {0}")]
    InvalidFormat(String),

    /// Invalid tile coordinates
    #[error("Invalid tile coordinates: ({0}, {1})")]
    InvalidCoordinates(u8, u8),

    /// Missing required chunk
    #[error("Missing required chunk: {0}")]
    MissingChunk(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for WDT operations
pub type Result<T> = std::result::Result<T, WdtError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        let wdt = WorldDataTable::placeholder();
        assert_eq!(wdt.map_name(), "Eastern Kingdoms");
        assert_eq!(wdt.map_id(), 0);
        assert!(wdt.existing_tiles_count() > 0);
    }

    #[test]
    fn test_tile_access() {
        let wdt = WorldDataTable::placeholder();

        // Check center tile exists
        assert!(wdt.tile_exists(30, 30));

        // Check edge tile doesn't exist
        assert!(!wdt.tile_exists(0, 0));

        // Check tile info
        let tile = wdt.tile_at(30, 30).unwrap();
        assert_eq!(tile.x, 30);
        assert_eq!(tile.y, 30);
        assert!(tile.flags.exists);
    }

    #[test]
    fn test_display() {
        let wdt = WorldDataTable::placeholder();
        let display = format!("{}", wdt);
        assert!(display.contains("Eastern Kingdoms"));
        assert!(display.contains("Map 0"));
        assert!(display.contains("existing tiles"));
    }
}
