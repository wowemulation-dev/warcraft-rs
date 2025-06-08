//! Parser for World of Warcraft WMO (World Map Object) files.
//!
//! This crate provides support for reading and writing WMO files used in
//! World of Warcraft. WMO files define large building models such as inns,
//! dungeons, towers, and other architectural structures.
//!
//! # Status
//!
//! 🚧 **Under Construction** - This crate is not yet fully implemented.
//!
//! # Examples
//!
//! ```no_run
//! use wow_wmo::WorldMapObject;
//!
//! // This is a placeholder example for future implementation
//! let wmo = WorldMapObject::placeholder();
//! println!("WMO groups: {}", wmo.group_count());
//! ```

#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use std::fmt;

/// A WMO group containing geometry data
#[derive(Debug)]
pub struct WmoGroup {
    /// Group index
    pub index: u32,
    /// Number of vertices
    pub vertex_count: u32,
    /// Number of triangles
    pub triangle_count: u32,
}

/// World Map Object (building/structure)
#[derive(Debug)]
pub struct WorldMapObject {
    /// Object name
    name: String,
    /// WMO groups
    groups: Vec<WmoGroup>,
    /// Bounding box minimum
    bbox_min: [f32; 3],
    /// Bounding box maximum
    bbox_max: [f32; 3],
}

impl WorldMapObject {
    /// Create a placeholder WMO for demonstration
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_wmo::WorldMapObject;
    ///
    /// let wmo = WorldMapObject::placeholder();
    /// assert_eq!(wmo.name(), "Placeholder WMO");
    /// assert_eq!(wmo.group_count(), 2);
    /// ```
    pub fn placeholder() -> Self {
        Self {
            name: "Placeholder WMO".to_string(),
            groups: vec![
                WmoGroup {
                    index: 0,
                    vertex_count: 100,
                    triangle_count: 50,
                },
                WmoGroup {
                    index: 1,
                    vertex_count: 200,
                    triangle_count: 100,
                },
            ],
            bbox_min: [-100.0, -100.0, -100.0],
            bbox_max: [100.0, 100.0, 100.0],
        }
    }

    /// Get the WMO name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the number of groups
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    /// Get all groups
    pub fn groups(&self) -> &[WmoGroup] {
        &self.groups
    }

    /// Get the bounding box
    pub fn bounding_box(&self) -> ([f32; 3], [f32; 3]) {
        (self.bbox_min, self.bbox_max)
    }
}

impl fmt::Display for WorldMapObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WMO '{}' ({} groups)", self.name, self.groups.len())
    }
}

/// Error type for WMO operations
#[derive(Debug, thiserror::Error)]
pub enum WmoError {
    /// Invalid WMO file format
    #[error("Invalid WMO format: {0}")]
    InvalidFormat(String),

    /// Missing required chunk
    #[error("Missing required chunk: {0}")]
    MissingChunk(String),

    /// Invalid group reference
    #[error("Invalid group reference: {0}")]
    InvalidGroup(u32),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for WMO operations
pub type Result<T> = std::result::Result<T, WmoError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        let wmo = WorldMapObject::placeholder();
        assert_eq!(wmo.name(), "Placeholder WMO");
        assert_eq!(wmo.group_count(), 2);
        assert_eq!(wmo.groups().len(), 2);
    }

    #[test]
    fn test_groups() {
        let wmo = WorldMapObject::placeholder();
        let groups = wmo.groups();
        assert_eq!(groups[0].index, 0);
        assert_eq!(groups[0].vertex_count, 100);
        assert_eq!(groups[1].index, 1);
        assert_eq!(groups[1].triangle_count, 100);
    }

    #[test]
    fn test_bounding_box() {
        let wmo = WorldMapObject::placeholder();
        let (min, max) = wmo.bounding_box();
        assert_eq!(min, [-100.0, -100.0, -100.0]);
        assert_eq!(max, [100.0, 100.0, 100.0]);
    }

    #[test]
    fn test_display() {
        let wmo = WorldMapObject::placeholder();
        let display = format!("{}", wmo);
        assert!(display.contains("Placeholder WMO"));
        assert!(display.contains("2 groups"));
    }
}
