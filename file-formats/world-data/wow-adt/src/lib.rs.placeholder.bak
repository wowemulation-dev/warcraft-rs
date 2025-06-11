//! Parser for World of Warcraft ADT (terrain) files.
//!
//! This crate provides support for reading and writing ADT files used in
//! World of Warcraft. ADT files store terrain data including height maps,
//! texture layers, water, and object placement information for map tiles.
//!
//! # Status
//!
//! 🚧 **Under Construction** - This crate is not yet fully implemented.
//!
//! # Examples
//!
//! ```no_run
//! use wow_adt::TerrainTile;
//!
//! // This is a placeholder example for future implementation
//! let tile = TerrainTile::placeholder();
//! println!("Tile position: {:?}", tile.position());
//! ```

#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use std::fmt;

/// Represents a terrain chunk within an ADT tile
#[derive(Debug)]
pub struct TerrainChunk {
    /// Chunk index (0-255)
    pub index: u8,
    /// Minimum height value
    pub height_min: f32,
    /// Maximum height value
    pub height_max: f32,
    /// Number of texture layers
    pub layer_count: u32,
}

/// ADT terrain tile (32x32 chunks)
#[derive(Debug)]
pub struct TerrainTile {
    /// Map ID this tile belongs to
    map_id: u32,
    /// Tile X coordinate
    tile_x: u32,
    /// Tile Y coordinate
    tile_y: u32,
    /// Terrain chunks (16x16 grid)
    chunks: Vec<TerrainChunk>,
}

impl TerrainTile {
    /// Create a placeholder terrain tile for demonstration
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_adt::TerrainTile;
    ///
    /// let tile = TerrainTile::placeholder();
    /// assert_eq!(tile.position(), (32, 48));
    /// assert_eq!(tile.chunk_count(), 256);
    /// ```
    pub fn placeholder() -> Self {
        let mut chunks = Vec::with_capacity(256);
        for i in 0..256 {
            chunks.push(TerrainChunk {
                index: i as u8,
                height_min: -10.0,
                height_max: 50.0,
                layer_count: 3,
            });
        }

        Self {
            map_id: 0,
            tile_x: 32,
            tile_y: 48,
            chunks,
        }
    }

    /// Get the tile position (x, y)
    pub fn position(&self) -> (u32, u32) {
        (self.tile_x, self.tile_y)
    }

    /// Get the map ID
    pub fn map_id(&self) -> u32 {
        self.map_id
    }

    /// Get the number of chunks
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Get all chunks
    pub fn chunks(&self) -> &[TerrainChunk] {
        &self.chunks
    }

    /// Get a specific chunk by index
    pub fn chunk(&self, index: u8) -> Option<&TerrainChunk> {
        self.chunks.get(index as usize)
    }
}

impl fmt::Display for TerrainTile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ADT Tile [{},{}] (Map {}, {} chunks)",
            self.tile_x,
            self.tile_y,
            self.map_id,
            self.chunks.len()
        )
    }
}

/// Error type for ADT operations
#[derive(Debug, thiserror::Error)]
pub enum AdtError {
    /// Invalid ADT file format
    #[error("Invalid ADT format: {0}")]
    InvalidFormat(String),

    /// Missing required chunk
    #[error("Missing required chunk: {0}")]
    MissingChunk(String),

    /// Invalid chunk index
    #[error("Invalid chunk index: {0}")]
    InvalidChunkIndex(u8),

    /// Unsupported ADT version
    #[error("Unsupported ADT version: {0}")]
    UnsupportedVersion(u32),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for ADT operations
pub type Result<T> = std::result::Result<T, AdtError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        let tile = TerrainTile::placeholder();
        assert_eq!(tile.position(), (32, 48));
        assert_eq!(tile.map_id(), 0);
        assert_eq!(tile.chunk_count(), 256);
    }

    #[test]
    fn test_chunks() {
        let tile = TerrainTile::placeholder();
        let chunks = tile.chunks();
        assert_eq!(chunks.len(), 256);
        assert_eq!(chunks[0].index, 0);
        assert_eq!(chunks[255].index, 255);
    }

    #[test]
    fn test_chunk_access() {
        let tile = TerrainTile::placeholder();
        let chunk = tile.chunk(100).unwrap();
        assert_eq!(chunk.index, 100);
        assert_eq!(chunk.layer_count, 3);
    }

    #[test]
    fn test_display() {
        let tile = TerrainTile::placeholder();
        let display = format!("{}", tile);
        assert!(display.contains("ADT Tile [32,48]"));
        assert!(display.contains("Map 0"));
        assert!(display.contains("256 chunks"));
    }
}
