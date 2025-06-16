//! Common types used throughout the library

/// Represents a key in a DBC file
pub type Key = u32;

/// Represents a string reference in a DBC file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StringRef(pub u32);

impl StringRef {
    /// Create a new string reference from an offset
    pub fn new(offset: u32) -> Self {
        Self(offset)
    }

    /// Get the offset of the string reference
    pub fn offset(&self) -> u32 {
        self.0
    }
}
