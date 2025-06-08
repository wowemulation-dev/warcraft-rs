//! Parser for World of Warcraft WDL (World Detail Level) files.
//!
//! This crate provides support for reading and writing WDL files used in
//! World of Warcraft. WDL files store low-resolution versions of terrain
//! used for distant terrain rendering and the world map interface.
//!
//! # Status
//!
//! 🚧 **Under Construction** - This crate is not yet fully implemented.
//!
//! # Examples
//!
//! ```no_run
//! use wow_wdl::LowResTerrain;
//!
//! // This is a placeholder example for future implementation
//! let terrain = LowResTerrain::placeholder();
//! println!("Map size: {}x{}", terrain.width(), terrain.height());
//! ```

#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use std::fmt;

/// Low-resolution terrain data
#[derive(Debug)]
pub struct LowResTerrain {
    /// Map ID
    map_id: u32,
    /// Width in tiles (64x64 areas)
    width: u32,
    /// Height in tiles (64x64 areas)
    height: u32,
    /// Height data for each tile
    heights: Vec<f32>,
    /// Water level data for each tile
    water_levels: Vec<f32>,
}

impl LowResTerrain {
    /// Create a placeholder low-res terrain for demonstration
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_wdl::LowResTerrain;
    ///
    /// let terrain = LowResTerrain::placeholder();
    /// assert_eq!(terrain.width(), 64);
    /// assert_eq!(terrain.height(), 64);
    /// assert_eq!(terrain.map_id(), 0);
    /// ```
    pub fn placeholder() -> Self {
        let size = 64 * 64;
        let heights = vec![100.0; size];
        let water_levels = vec![0.0; size];

        Self {
            map_id: 0,
            width: 64,
            height: 64,
            heights,
            water_levels,
        }
    }

    /// Get the map ID
    pub fn map_id(&self) -> u32 {
        self.map_id
    }

    /// Get the terrain width in tiles
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the terrain height in tiles
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get height at a specific tile position
    pub fn height_at(&self, x: u32, y: u32) -> Option<f32> {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            self.heights.get(index).copied()
        } else {
            None
        }
    }

    /// Get water level at a specific tile position
    pub fn water_level_at(&self, x: u32, y: u32) -> Option<f32> {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            self.water_levels.get(index).copied()
        } else {
            None
        }
    }

    /// Check if a tile has water
    pub fn has_water_at(&self, x: u32, y: u32) -> bool {
        self.water_level_at(x, y)
            .map(|level| level > 0.0)
            .unwrap_or(false)
    }
}

impl fmt::Display for LowResTerrain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "WDL Low-res terrain (Map {}, {}x{} tiles)",
            self.map_id, self.width, self.height
        )
    }
}

/// Error type for WDL operations
#[derive(Debug, thiserror::Error)]
pub enum WdlError {
    /// Invalid WDL file format
    #[error("Invalid WDL format: {0}")]
    InvalidFormat(String),

    /// Invalid tile coordinates
    #[error("Invalid tile coordinates: ({0}, {1})")]
    InvalidCoordinates(u32, u32),

    /// Missing required data
    #[error("Missing required data: {0}")]
    MissingData(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for WDL operations
pub type Result<T> = std::result::Result<T, WdlError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        let terrain = LowResTerrain::placeholder();
        assert_eq!(terrain.width(), 64);
        assert_eq!(terrain.height(), 64);
        assert_eq!(terrain.map_id(), 0);
    }

    #[test]
    fn test_height_access() {
        let terrain = LowResTerrain::placeholder();
        assert_eq!(terrain.height_at(0, 0), Some(100.0));
        assert_eq!(terrain.height_at(63, 63), Some(100.0));
        assert_eq!(terrain.height_at(64, 0), None);
    }

    #[test]
    fn test_water_access() {
        let terrain = LowResTerrain::placeholder();
        assert_eq!(terrain.water_level_at(0, 0), Some(0.0));
        assert!(!terrain.has_water_at(0, 0));
    }

    #[test]
    fn test_display() {
        let terrain = LowResTerrain::placeholder();
        let display = format!("{}", terrain);
        assert!(display.contains("WDL Low-res terrain"));
        assert!(display.contains("Map 0"));
        assert!(display.contains("64x64"));
    }
}
