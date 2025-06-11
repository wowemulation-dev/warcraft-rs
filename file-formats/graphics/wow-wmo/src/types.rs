use std::fmt;

/// A 4-byte chunk identifier (magic)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkId(pub [u8; 4]);

impl ChunkId {
    /// Create a new chunk identifier from a 4-byte array
    pub const fn new(array: [u8; 4]) -> Self {
        Self(array)
    }

    /// Create a new chunk identifier from a static string
    ///
    /// # Panics
    ///
    /// Panics if the string is not exactly 4 bytes
    pub const fn from_str(s: &str) -> Self {
        assert!(s.len() == 4, "ChunkId must be exactly 4 bytes");
        let bytes = s.as_bytes();
        Self([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    /// Get the raw bytes of this chunk identifier
    pub fn as_bytes(&self) -> &[u8; 4] {
        &self.0
    }
}

impl fmt::Display for ChunkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Convert to string if all bytes are ASCII printable, otherwise show hex
        if self.0.iter().all(|&b| b.is_ascii_graphic()) {
            write!(f, "{}", std::str::from_utf8(&self.0).unwrap_or("????"))
        } else {
            write!(
                f,
                "0x{:02X}{:02X}{:02X}{:02X}",
                self.0[0], self.0[1], self.0[2], self.0[3]
            )
        }
    }
}

/// Represents a 3D vector
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Represents a bounding box defined by min and max points
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

/// Represents RGBA color
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
