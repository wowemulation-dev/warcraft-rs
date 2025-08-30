use crate::chunks::animation::M2InterpolationType;
use crate::common::{C3Vector, M2Array};
use crate::error::Result;
use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Write};

/// Compressed quaternion using 16-bit integers
/// Used for rotation animations to save space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct M2CompQuat {
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub w: i16,
}

impl M2CompQuat {
    /// Parse a compressed quaternion from a reader
    /// Note: X component is negated to match Python reference implementation (pywowlib)
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let x = reader.read_i16_le()?;
        let y = reader.read_i16_le()?;
        let z = reader.read_i16_le()?;
        let w = reader.read_i16_le()?;

        // Negate X component to match Python reference implementation
        Ok(Self { x: -x, y, z, w })
    }

    /// Write a compressed quaternion to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_i16_le(self.x)?;
        writer.write_i16_le(self.y)?;
        writer.write_i16_le(self.z)?;
        writer.write_i16_le(self.w)?;
        Ok(())
    }

    /// Convert compressed quaternion to normalized float quaternion
    /// The compressed values are typically normalized to [-1.0, 1.0] range
    pub fn to_float_quaternion(&self) -> (f32, f32, f32, f32) {
        const SCALE: f32 = 1.0 / 32767.0;
        (
            self.x as f32 * SCALE,
            self.y as f32 * SCALE,
            self.z as f32 * SCALE,
            self.w as f32 * SCALE,
        )
    }

    /// Create compressed quaternion from normalized float values
    pub fn from_float_quaternion(x: f32, y: f32, z: f32, w: f32) -> Self {
        const SCALE: f32 = 32767.0;
        Self {
            x: (x * SCALE) as i16,
            y: (y * SCALE) as i16,
            z: (z * SCALE) as i16,
            w: (w * SCALE) as i16,
        }
    }
}

/// Base structure for M2Track that contains common fields
/// This represents the header part of an M2Track before the value data
#[derive(Debug, Clone)]
pub struct M2TrackBase {
    /// Type of interpolation to use between keyframes
    pub interpolation_type: M2InterpolationType,
    /// Global sequence index (65535 = no global sequence)
    pub global_sequence: u16,
}

impl M2TrackBase {
    /// Parse the base track structure from a reader
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let interpolation_type = M2InterpolationType::from_u16(reader.read_u16_le()?);
        let interpolation_type = interpolation_type.unwrap_or(M2InterpolationType::Linear);
        let global_sequence = reader.read_u16_le()?;

        Ok(Self {
            interpolation_type,
            global_sequence,
        })
    }

    /// Write the base track structure to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u16_le(self.interpolation_type as u16)?;
        writer.write_u16_le(self.global_sequence)?;
        Ok(())
    }

    /// Check if this track uses a global sequence
    pub fn uses_global_sequence(&self) -> bool {
        self.global_sequence != 65535
    }
}

/// Generic M2Track structure for handling animated values
/// Supports version-aware parsing for pre-WotLK vs WotLK+ formats
#[derive(Debug, Clone)]
pub struct M2Track<T> {
    /// Base track information (interpolation type, global sequence)
    pub base: M2TrackBase,
    /// Animation ranges (pre-Wrath only, v263 and earlier)
    /// This field was removed in Wrath (v264+)
    pub ranges: Option<M2Array<u32>>,
    /// Timestamps for animation keyframes
    /// Pre-264: Single M2Array with all timestamps
    /// 264+: M2Array pointing to per-animation timestamp arrays
    pub timestamps: M2Array<u32>,
    /// Values for animation keyframes
    /// Pre-264: Single M2Array with all values
    /// 264+: M2Array pointing to per-animation value arrays
    pub values: M2Array<T>,
}

impl<T> M2Track<T> {
    /// Parse an M2Track from a reader
    ///
    /// # Arguments
    /// * `reader` - The reader to parse from
    /// * `version` - M2 version to determine parsing format
    pub fn parse<R: Read>(reader: &mut R, version: u32) -> Result<Self> {
        let base = M2TrackBase::parse(reader)?;

        // WMVx M2Definitions.h shows: Pre-WotLK versions (< 264) include ranges field
        // This includes both Vanilla (256-257) AND TBC (260-263)
        let (ranges, timestamps, values) = if version < 264 {
            // Pre-WotLK format: ranges + timestamps + values
            let ranges = M2Array::parse(reader)?;
            let timestamps = M2Array::parse(reader)?;
            let values = M2Array::parse(reader)?;
            (Some(ranges), timestamps, values)
        } else {
            // WotLK+ format (v264+): removed ranges field
            let timestamps = M2Array::parse(reader)?;
            let values = M2Array::parse(reader)?;
            (None, timestamps, values)
        };

        Ok(Self {
            base,
            ranges,
            timestamps,
            values,
        })
    }

    /// Write an M2Track to a writer
    ///
    /// # Arguments
    /// * `writer` - The writer to write to
    /// * `version` - M2 version to determine writing format
    pub fn write<W: Write>(&self, writer: &mut W, version: u32) -> Result<()> {
        self.base.write(writer)?;

        // Write ranges field for all Pre-WotLK versions (< 264) - includes vanilla AND TBC
        if version < 264 {
            if let Some(ref ranges) = self.ranges {
                ranges.write(writer)?;
            } else {
                // Write empty ranges if not present
                M2Array::<u32>::new(0, 0).write(writer)?;
            }
        }

        self.timestamps.write(writer)?;
        self.values.write(writer)?;
        Ok(())
    }

    /// Check if this track has any animation data
    pub fn has_data(&self) -> bool {
        !self.timestamps.is_empty() && !self.values.is_empty()
    }

    /// Check if this track is static (no animation)
    pub fn is_static(&self) -> bool {
        self.timestamps.count <= 1
    }

    /// Get the interpolation type for this track
    pub fn interpolation_type(&self) -> M2InterpolationType {
        self.base.interpolation_type
    }

    /// Check if this track uses a global sequence
    pub fn uses_global_sequence(&self) -> bool {
        self.base.uses_global_sequence()
    }

    /// Create a new empty M2Track
    pub fn new() -> Self {
        Self {
            base: M2TrackBase {
                interpolation_type: M2InterpolationType::None,
                global_sequence: 65535, // No global sequence
            },
            ranges: None, // Modern format doesn't use ranges
            timestamps: M2Array::new(0, 0),
            values: M2Array::new(0, 0),
        }
    }

    /// Create a new M2Track with specified interpolation type
    pub fn new_with_interpolation(interpolation_type: M2InterpolationType) -> Self {
        Self {
            base: M2TrackBase {
                interpolation_type,
                global_sequence: 65535,
            },
            ranges: None, // Modern format doesn't use ranges
            timestamps: M2Array::new(0, 0),
            values: M2Array::new(0, 0),
        }
    }
}

impl<T> Default for M2Track<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Type aliases for common M2Track usage patterns
pub type M2TrackVec3 = M2Track<C3Vector>;
pub type M2TrackQuat = M2Track<M2CompQuat>;
pub type M2TrackFloat = M2Track<f32>;
pub type M2TrackUint16 = M2Track<u16>;

/// Extended M2Track structure for tracks that include interpolation ranges
/// Used in more complex animation systems that support cubic interpolation
#[derive(Debug, Clone)]
pub struct M2TrackWithRanges<T> {
    /// Base M2Track data
    pub track: M2Track<T>,
    /// Interpolation ranges for cubic spline interpolation
    /// Only used when interpolation_type is CubicBezier or CubicHermite
    pub interpolation_ranges: M2Array<u32>,
}

impl<T> M2TrackWithRanges<T> {
    /// Parse an extended M2Track with interpolation ranges
    pub fn parse<R: Read>(reader: &mut R, version: u32) -> Result<Self> {
        let track = M2Track::parse(reader, version)?;
        let interpolation_ranges = M2Array::parse(reader)?;

        Ok(Self {
            track,
            interpolation_ranges,
        })
    }

    /// Write an extended M2Track with interpolation ranges
    pub fn write<W: Write>(&self, writer: &mut W, version: u32) -> Result<()> {
        self.track.write(writer, version)?;
        self.interpolation_ranges.write(writer)?;
        Ok(())
    }

    /// Check if this track uses interpolation ranges
    pub fn uses_interpolation_ranges(&self) -> bool {
        matches!(
            self.track.interpolation_type(),
            M2InterpolationType::Bezier | M2InterpolationType::Hermite
        ) && !self.interpolation_ranges.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_interpolation_type_conversion() {
        assert_eq!(
            M2InterpolationType::from_u16(0),
            Some(M2InterpolationType::None)
        );
        assert_eq!(
            M2InterpolationType::from_u16(1),
            Some(M2InterpolationType::Linear)
        );
        assert_eq!(
            M2InterpolationType::from_u16(2),
            Some(M2InterpolationType::Bezier)
        );
        assert_eq!(
            M2InterpolationType::from_u16(3),
            Some(M2InterpolationType::Hermite)
        );

        // Test unknown value returns None
        assert_eq!(M2InterpolationType::from_u16(999), None);

        // Test enum to u16 conversion
        assert_eq!(M2InterpolationType::None as u16, 0);
        assert_eq!(M2InterpolationType::Linear as u16, 1);
        assert_eq!(M2InterpolationType::Bezier as u16, 2);
        assert_eq!(M2InterpolationType::Hermite as u16, 3);
    }

    #[test]
    fn test_compressed_quaternion_parse() {
        let data = [
            0x00, 0x40, // x = 16384 (0x4000) -> becomes -16384 after negation
            0xFF, 0x7F, // y = 32767 (0x7FFF)
            0x00, 0x00, // z = 0
            0xFF, 0x3F, // w = 16383 (0x3FFF)
        ];

        let mut cursor = Cursor::new(data);
        let quat = M2CompQuat::parse(&mut cursor).unwrap();

        // X component is negated during parsing to match Python reference
        assert_eq!(quat.x, -16384); // -(16384) = -16384
        assert_eq!(quat.y, 32767);
        assert_eq!(quat.z, 0);
        assert_eq!(quat.w, 16383);
    }

    #[test]
    fn test_compressed_quaternion_float_conversion() {
        let quat = M2CompQuat {
            x: 16384,  // ~0.5 when normalized
            y: -16384, // ~-0.5 when normalized
            z: 0,
            w: 32767, // ~1.0 when normalized
        };

        let (x, y, z, w) = quat.to_float_quaternion();

        // Check approximate values (allowing for float precision)
        // Note: X value from stored data is used as-is in to_float_quaternion()
        assert!((x - 0.5).abs() < 0.01);
        assert!((y + 0.5).abs() < 0.01);
        assert!(z.abs() < 0.001);
        assert!((w - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_track_base_parse() {
        let data = [
            0x01, 0x00, // interpolation_type = Linear (1)
            0xFF, 0xFF, // global_sequence = 65535 (no global sequence)
        ];

        let mut cursor = Cursor::new(data);
        let base = M2TrackBase::parse(&mut cursor).unwrap();

        assert_eq!(base.interpolation_type, M2InterpolationType::Linear);
        assert_eq!(base.global_sequence, 65535);
        assert!(!base.uses_global_sequence());
    }

    #[test]
    fn test_track_base_with_global_sequence() {
        let data = [
            0x02, 0x00, // interpolation_type = CubicBezier (2)
            0x05, 0x00, // global_sequence = 5
        ];

        let mut cursor = Cursor::new(data);
        let base = M2TrackBase::parse(&mut cursor).unwrap();

        assert_eq!(base.interpolation_type, M2InterpolationType::Bezier);
        assert_eq!(base.global_sequence, 5);
        assert!(base.uses_global_sequence());
    }

    #[test]
    fn test_m2track_parse() {
        let mut data = Vec::new();

        // M2TrackBase
        data.extend_from_slice(&1u16.to_le_bytes()); // Linear interpolation
        data.extend_from_slice(&65535u16.to_le_bytes()); // No global sequence

        // Timestamps M2Array
        data.extend_from_slice(&3u32.to_le_bytes()); // count = 3
        data.extend_from_slice(&100u32.to_le_bytes()); // offset = 100

        // Values M2Array
        data.extend_from_slice(&3u32.to_le_bytes()); // count = 3
        data.extend_from_slice(&200u32.to_le_bytes()); // offset = 200

        let mut cursor = Cursor::new(data);
        let track: M2Track<f32> = M2Track::parse(&mut cursor, 264).unwrap();

        assert_eq!(track.base.interpolation_type, M2InterpolationType::Linear);
        assert_eq!(track.base.global_sequence, 65535);
        assert_eq!(track.timestamps.count, 3);
        assert_eq!(track.timestamps.offset, 100);
        assert_eq!(track.values.count, 3);
        assert_eq!(track.values.offset, 200);
        assert!(track.has_data());
        assert!(!track.is_static());
    }

    #[test]
    fn test_m2track_new() {
        let track: M2Track<f32> = M2Track::new();

        assert_eq!(track.base.interpolation_type, M2InterpolationType::None);
        assert_eq!(track.base.global_sequence, 65535);
        assert!(!track.has_data());
        assert!(track.is_static());
    }

    #[test]
    fn test_m2track_with_interpolation() {
        let track: M2Track<M2CompQuat> =
            M2Track::new_with_interpolation(M2InterpolationType::Hermite);

        assert_eq!(track.base.interpolation_type, M2InterpolationType::Hermite);
        assert_eq!(track.base.global_sequence, 65535);
    }

    #[test]
    fn test_compressed_quaternion_roundtrip() {
        // Test that from_float_quaternion -> to_float_quaternion roundtrip works correctly
        let original_x = 0.707;
        let original_y = 0.0;
        let original_z = 0.707;
        let original_w = 0.0;

        let quat =
            M2CompQuat::from_float_quaternion(original_x, original_y, original_z, original_w);
        let (recovered_x, recovered_y, recovered_z, recovered_w) = quat.to_float_quaternion();

        // Should recover original values (accounting for quantization loss)
        assert!(
            (recovered_x - original_x).abs() < 0.01,
            "X component: expected {}, got {}",
            original_x,
            recovered_x
        );
        assert!(
            (recovered_y - original_y).abs() < 0.01,
            "Y component: expected {}, got {}",
            original_y,
            recovered_y
        );
        assert!(
            (recovered_z - original_z).abs() < 0.01,
            "Z component: expected {}, got {}",
            original_z,
            recovered_z
        );
        assert!(
            (recovered_w - original_w).abs() < 0.01,
            "W component: expected {}, got {}",
            original_w,
            recovered_w
        );
    }
}
