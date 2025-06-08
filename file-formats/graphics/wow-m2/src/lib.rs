//! Parser for World of Warcraft M2 (model) files.
//!
//! This crate provides support for reading and writing M2 model files used in
//! World of Warcraft. M2 files contain 3D models, animations, particle effects,
//! and other visual data for characters, creatures, and objects.
//!
//! # Status
//!
//! 🚧 **Under Construction** - This crate is not yet fully implemented.
//!
//! # Examples
//!
//! ```no_run
//! use wow_m2::M2Model;
//!
//! // This is a placeholder example for future implementation
//! let model = M2Model::placeholder();
//! println!("Model name: {}", model.name());
//! println!("Vertices: {}", model.vertex_count());
//! ```

#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use std::fmt;

/// M2 model file
#[derive(Debug)]
pub struct M2Model {
    /// Model name
    name: String,
    /// Number of vertices
    vertex_count: u32,
    /// Number of triangles
    triangle_count: u32,
}

impl M2Model {
    /// Create a placeholder model for demonstration
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_m2::M2Model;
    ///
    /// let model = M2Model::placeholder();
    /// assert_eq!(model.name(), "Placeholder Model");
    /// ```
    pub fn placeholder() -> Self {
        Self {
            name: "Placeholder Model".to_string(),
            vertex_count: 100,
            triangle_count: 50,
        }
    }

    /// Get the model name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the vertex count
    pub fn vertex_count(&self) -> u32 {
        self.vertex_count
    }

    /// Get the triangle count
    pub fn triangle_count(&self) -> u32 {
        self.triangle_count
    }
}

impl fmt::Display for M2Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "M2 Model '{}' ({} vertices, {} triangles)",
            self.name, self.vertex_count, self.triangle_count
        )
    }
}

/// Animation sequence in an M2 model
#[derive(Debug, Clone)]
pub struct M2Animation {
    /// Animation ID
    pub id: u16,
    /// Animation name
    pub name: String,
    /// Duration in milliseconds
    pub duration: u32,
}

/// Error type for M2 operations
#[derive(Debug, thiserror::Error)]
pub enum M2Error {
    /// Invalid M2 file format
    #[error("Invalid M2 format: {0}")]
    InvalidFormat(String),

    /// Unsupported M2 version
    #[error("Unsupported M2 version: {0}")]
    UnsupportedVersion(u32),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for M2 operations
pub type Result<T> = std::result::Result<T, M2Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        let model = M2Model::placeholder();
        assert_eq!(model.name(), "Placeholder Model");
        assert_eq!(model.vertex_count(), 100);
        assert_eq!(model.triangle_count(), 50);
    }

    #[test]
    fn test_display() {
        let model = M2Model::placeholder();
        let display = format!("{}", model);
        assert!(display.contains("Placeholder Model"));
        assert!(display.contains("100 vertices"));
        assert!(display.contains("50 triangles"));
    }
}
