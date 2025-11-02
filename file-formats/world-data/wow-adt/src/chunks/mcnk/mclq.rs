//! MCLQ - Legacy liquid data (Vanilla/TBC).
//!
//! MCLQ chunks store water, lava, and other liquid surfaces embedded in MCNK chunks.
//! Replaced by MH2O in WotLK (3.x).
//!
//! ## Structure
//!
//! ```text
//! MCLQ (minimum 720 bytes)
//! ├─ Header (8 bytes)
//! │  ├─ min_height: f32
//! │  └─ max_height: f32
//! ├─ Vertices (648 bytes = 81 × 8 bytes)
//! │  └─ 9×9 vertex grid
//! │     ├─ Water/Ocean/Slime: 4 bytes (depth, flow, flags) + 4 bytes (height)
//! │     └─ Magma: 4 bytes (s, t UVs) + 4 bytes (height)
//! └─ Tile Flags (64 bytes = 8×8 grid)
//! ```
//!
//! ## Liquid Type Detection
//!
//! Liquid type is determined from MCNK header flags:
//! - 0x04: River/Water
//! - 0x08: Ocean
//! - 0x10: Magma/Lava
//! - 0x20: Slime
//!
//! Reference: <https://wowdev.wiki/ADT/v18#MCLQ_sub-chunk>

use binrw::{BinRead, BinWrite};

/// Legacy liquid type (pre-WotLK).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LiquidType {
    /// Water/river
    Water = 0,

    /// Ocean
    Ocean = 1,

    /// Magma/lava
    Magma = 2,

    /// Slime
    Slime = 3,
}

impl LiquidType {
    /// Determine liquid type from MCNK flags.
    ///
    /// MCNK flags encoding:
    /// - 0x04: River/Water
    /// - 0x08: Ocean
    /// - 0x10: Magma
    /// - 0x20: Slime
    pub fn from_mcnk_flags(flags: u32) -> Self {
        if (flags & 0x08) != 0 {
            Self::Ocean
        } else if (flags & 0x10) != 0 {
            Self::Magma
        } else if (flags & 0x20) != 0 {
            Self::Slime
        } else {
            Self::Water
        }
    }
}

/// Liquid vertex (8 bytes, format varies by liquid type).
///
/// All vertices are 8 bytes total, but the first 4 bytes vary:
/// - **Water/Ocean/Slime**: depth(u8) + flow0(u8) + flow1(u8) + filler(u8)
/// - **Magma**: s(u16) + t(u16) texture coordinates
///
/// The last 4 bytes are always the height (f32).
#[derive(Debug, Clone, Copy, BinRead, BinWrite)]
pub struct LiquidVertex {
    /// Union data (4 bytes) - interpretation depends on liquid type
    ///
    /// Water/Ocean/Slime: [depth, flow0, flow1, filler]
    /// Magma: s(u16 LE) + t(u16 LE) as 4 bytes
    pub union_data: [u8; 4],

    /// Vertex height (absolute world height)
    pub height: f32,
}

impl LiquidVertex {
    /// Get depth byte (Water/Ocean/Slime only).
    pub fn depth_byte(&self) -> u8 {
        self.union_data[0]
    }

    /// Get flow0 byte (Water/Slime) or foam (Ocean).
    pub fn flow0_or_foam(&self) -> u8 {
        self.union_data[1]
    }

    /// Get flow1 byte (Water/Slime) or wet (Ocean).
    pub fn flow1_or_wet(&self) -> u8 {
        self.union_data[2]
    }

    /// Get s texture coordinate (Magma only).
    pub fn magma_s(&self) -> u16 {
        u16::from_le_bytes([self.union_data[0], self.union_data[1]])
    }

    /// Get t texture coordinate (Magma only).
    pub fn magma_t(&self) -> u16 {
        u16::from_le_bytes([self.union_data[2], self.union_data[3]])
    }

    /// Get relative depth from base height.
    pub fn relative_depth(&self, base_height: f32) -> f32 {
        self.height - base_height
    }
}

/// MCLQ chunk - Legacy liquid data (pre-WotLK).
///
/// Stores liquid surface geometry for water, lava, ocean, and slime.
/// Always uses a fixed 9×9 vertex grid (81 vertices).
///
/// # Example
///
/// ```no_run
/// # use wow_adt::chunks::mcnk::MclqChunk;
/// # let chunk: MclqChunk = todo!();
/// // Access liquid properties
/// println!("Liquid type: {:?}", chunk.liquid_type);
/// println!("Base height: {}", chunk.base_height());
/// println!("Vertex count: {}", chunk.vertices.len()); // Always 81
///
/// // Calculate vertex heights
/// for vertex in &chunk.vertices {
///     let abs_height = vertex.height;
///     let rel_depth = vertex.relative_depth(chunk.base_height());
///     println!("Height: {}, Depth: {}", abs_height, rel_depth);
/// }
/// ```
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ✅ Introduced
/// - **TBC (2.4.3)**: ✅ Present
/// - **WotLK (3.3.5a)**: ❌ Deprecated (replaced by MH2O chunk)
/// - **Cataclysm (4.3.4)**: ❌ Not present
/// - **MoP (5.4.8)**: ❌ Not present
///
/// Reference: <https://wowdev.wiki/ADT/v18#MCLQ_sub-chunk>
#[derive(Debug, Clone)]
pub struct MclqChunk {
    /// Minimum height in the liquid surface
    pub min_height: f32,

    /// Maximum height in the liquid surface
    pub max_height: f32,

    /// Liquid vertices (always 81 = 9×9 grid)
    pub vertices: Vec<LiquidVertex>,

    /// Tile flags (64 bytes = 8×8 grid) - render enable flags
    pub tile_flags: [u8; 64],

    /// Liquid type determined from MCNK flags
    pub liquid_type: LiquidType,
}

impl MclqChunk {
    /// Fixed vertex grid size (9×9 = 81 vertices)
    pub const VERTEX_COUNT: usize = 81;

    /// Fixed tile grid size (8×8 = 64 tiles)
    pub const TILE_COUNT: usize = 64;

    /// Minimum valid chunk size: 8 (header) + 648 (vertices) + 64 (tiles) = 720 bytes
    pub const MIN_SIZE: usize = 8 + (Self::VERTEX_COUNT * 8) + Self::TILE_COUNT;

    /// Vertex grid width
    pub const GRID_WIDTH: usize = 9;

    /// Vertex grid height
    pub const GRID_HEIGHT: usize = 9;

    /// Calculate base height (average of min/max).
    pub fn base_height(&self) -> f32 {
        (self.min_height + self.max_height) / 2.0
    }

    /// Check if height range is valid.
    ///
    /// Valid heights must be:
    /// - Finite (not NaN or Infinity)
    /// - Within reasonable world bounds (±10000 units)
    /// - min_height <= max_height
    pub fn has_valid_heights(&self) -> bool {
        self.min_height.is_finite()
            && self.max_height.is_finite()
            && self.min_height.abs() <= 10000.0
            && self.max_height.abs() <= 10000.0
            && self.min_height <= self.max_height
    }

    /// Get vertex at grid position (x, y).
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-8)
    /// * `y` - Row index (0-8)
    pub fn get_vertex(&self, x: usize, y: usize) -> Option<&LiquidVertex> {
        if x >= Self::GRID_WIDTH || y >= Self::GRID_HEIGHT {
            return None;
        }
        let index = y * Self::GRID_WIDTH + x;
        self.vertices.get(index)
    }

    /// Get tile flag at position (x, y) in 8×8 grid.
    pub fn get_tile_flag(&self, x: usize, y: usize) -> Option<u8> {
        if x >= 8 || y >= 8 {
            return None;
        }
        let index = y * 8 + x;
        self.tile_flags.get(index).copied()
    }
}

impl BinRead for MclqChunk {
    type Args<'a> = u32; // MCNK flags for liquid type detection

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        mcnk_flags: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        // Read height range
        let min_height = f32::read_options(reader, endian, ())?;
        let max_height = f32::read_options(reader, endian, ())?;

        // Validate heights - corrupted chunks have invalid float values
        let heights_valid = min_height.is_finite()
            && max_height.is_finite()
            && min_height.abs() <= 10000.0
            && max_height.abs() <= 10000.0
            && min_height <= max_height;

        if !heights_valid {
            return Err(binrw::Error::AssertFail {
                pos: 0,
                message: format!(
                    "MCLQ has invalid height range: min={}, max={} (likely corrupted/empty)",
                    min_height, max_height
                ),
            });
        }

        // Determine liquid type from MCNK flags
        let liquid_type = LiquidType::from_mcnk_flags(mcnk_flags);

        // Read 81 vertices (9×9 grid, always 8 bytes each)
        let mut vertices = Vec::with_capacity(Self::VERTEX_COUNT);
        for _ in 0..Self::VERTEX_COUNT {
            vertices.push(LiquidVertex::read_options(reader, endian, ())?);
        }

        // Read tile flags (8×8 = 64 bytes)
        let mut tile_flags = [0u8; 64];
        reader.read_exact(&mut tile_flags)?;

        // Note: Remaining data (flow vectors, etc.) is ignored
        // Most MCLQ chunks are exactly 720 bytes, but some have extra data

        Ok(Self {
            min_height,
            max_height,
            vertices,
            tile_flags,
            liquid_type,
        })
    }
}

impl BinWrite for MclqChunk {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        // Write header
        self.min_height.write_options(writer, endian, ())?;
        self.max_height.write_options(writer, endian, ())?;

        // Write vertices
        for vertex in &self.vertices {
            vertex.write_options(writer, endian, ())?;
        }

        // Write tile flags
        writer.write_all(&self.tile_flags)?;

        Ok(())
    }
}

impl Default for MclqChunk {
    fn default() -> Self {
        Self {
            min_height: 0.0,
            max_height: 0.0,
            vertices: Vec::new(),
            tile_flags: [0; 64],
            liquid_type: LiquidType::Water,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_liquid_type_values() {
        assert_eq!(LiquidType::Water as u8, 0);
        assert_eq!(LiquidType::Ocean as u8, 1);
        assert_eq!(LiquidType::Magma as u8, 2);
        assert_eq!(LiquidType::Slime as u8, 3);
    }

    #[test]
    fn test_liquid_type_from_flags() {
        assert_eq!(LiquidType::from_mcnk_flags(0x04), LiquidType::Water);
        assert_eq!(LiquidType::from_mcnk_flags(0x08), LiquidType::Ocean);
        assert_eq!(LiquidType::from_mcnk_flags(0x10), LiquidType::Magma);
        assert_eq!(LiquidType::from_mcnk_flags(0x20), LiquidType::Slime);
        assert_eq!(LiquidType::from_mcnk_flags(0x00), LiquidType::Water); // Default
    }

    #[test]
    fn test_liquid_vertex_accessors() {
        let vertex = LiquidVertex {
            union_data: [10, 20, 30, 40],
            height: 100.5,
        };

        assert_eq!(vertex.depth_byte(), 10);
        assert_eq!(vertex.flow0_or_foam(), 20);
        assert_eq!(vertex.flow1_or_wet(), 30);
        assert_eq!(vertex.magma_s(), u16::from_le_bytes([10, 20]));
        assert_eq!(vertex.magma_t(), u16::from_le_bytes([30, 40]));
        assert_eq!(vertex.relative_depth(50.0), 50.5);
    }

    #[test]
    fn test_mclq_base_height() {
        let chunk = MclqChunk {
            min_height: 10.0,
            max_height: 30.0,
            vertices: Vec::new(),
            tile_flags: [0; 64],
            liquid_type: LiquidType::Water,
        };

        assert_eq!(chunk.base_height(), 20.0);
    }

    #[test]
    fn test_mclq_has_valid_heights() {
        let valid = MclqChunk {
            min_height: 10.0,
            max_height: 30.0,
            vertices: Vec::new(),
            tile_flags: [0; 64],
            liquid_type: LiquidType::Water,
        };
        assert!(valid.has_valid_heights());

        let invalid_nan = MclqChunk {
            min_height: f32::NAN,
            max_height: 30.0,
            vertices: Vec::new(),
            tile_flags: [0; 64],
            liquid_type: LiquidType::Water,
        };
        assert!(!invalid_nan.has_valid_heights());

        let invalid_range = MclqChunk {
            min_height: 30.0,
            max_height: 10.0,
            vertices: Vec::new(),
            tile_flags: [0; 64],
            liquid_type: LiquidType::Water,
        };
        assert!(!invalid_range.has_valid_heights());
    }

    #[test]
    fn test_mclq_constants() {
        assert_eq!(MclqChunk::VERTEX_COUNT, 81);
        assert_eq!(MclqChunk::TILE_COUNT, 64);
        assert_eq!(MclqChunk::MIN_SIZE, 720);
        assert_eq!(MclqChunk::GRID_WIDTH, 9);
        assert_eq!(MclqChunk::GRID_HEIGHT, 9);
    }
}
