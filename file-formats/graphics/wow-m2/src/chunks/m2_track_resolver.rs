use crate::{
    Result,
    chunks::m2_track::{M2CompQuat, M2Track, M2TrackQuat, M2TrackVec3},
    common::{C3Vector, read_array},
    io_ext::ReadExt,
};
use std::io::{Read, Seek};

/// Resolver for M2Track animation data
/// This loads the actual keyframe data from M2Array offsets
pub struct M2TrackResolver;

impl M2TrackResolver {
    /// Resolve timestamp data for a track
    pub fn resolve_timestamps<R: Read + Seek>(
        reader: &mut R,
        track: &M2Track<impl Clone>,
    ) -> Result<Vec<u32>> {
        if track.timestamps.is_empty() {
            return Ok(Vec::new());
        }

        read_array(reader, &track.timestamps, |r| Ok(r.read_u32_le()?))
    }

    /// Resolve Vec3 track values (translation/scale)
    pub fn resolve_vec3_values<R: Read + Seek>(
        reader: &mut R,
        track: &M2TrackVec3,
    ) -> Result<Vec<C3Vector>> {
        if track.values.is_empty() {
            return Ok(Vec::new());
        }

        read_array(reader, &track.values, |r| C3Vector::parse(r))
    }

    /// Resolve quaternion track values (rotation)
    pub fn resolve_quat_values<R: Read + Seek>(
        reader: &mut R,
        track: &M2TrackQuat,
    ) -> Result<Vec<M2CompQuat>> {
        if track.values.is_empty() {
            return Ok(Vec::new());
        }

        read_array(reader, &track.values, |r| M2CompQuat::parse(r))
    }

    /// Resolve range data for pre-WotLK tracks
    /// Ranges are stored as pairs of u32 values (start, end)
    pub fn resolve_ranges<R: Read + Seek>(
        reader: &mut R,
        track: &M2Track<impl Clone>,
    ) -> Result<Option<Vec<u32>>> {
        if let Some(ref ranges) = track.ranges {
            if ranges.is_empty() {
                return Ok(Some(Vec::new()));
            }

            // Read ranges as raw u32 values (pairs of start/end)
            let range_data = read_array(reader, ranges, |r| Ok(r.read_u32_le()?))?;

            Ok(Some(range_data))
        } else {
            Ok(None)
        }
    }
}

// Type aliases to reduce complexity
type Vec3TrackData = (Vec<u32>, Vec<C3Vector>, Option<Vec<u32>>);
type QuatTrackData = (Vec<u32>, Vec<M2CompQuat>, Option<Vec<u32>>);

/// Extension trait for M2TrackVec3 to resolve animation data
pub trait M2TrackVec3Ext {
    /// Load the actual animation data from the file
    fn resolve_data<R: Read + Seek>(&self, reader: &mut R) -> Result<Vec3TrackData>;
}

impl M2TrackVec3Ext for M2TrackVec3 {
    fn resolve_data<R: Read + Seek>(&self, reader: &mut R) -> Result<Vec3TrackData> {
        let timestamps = M2TrackResolver::resolve_timestamps(reader, self)?;
        let values = M2TrackResolver::resolve_vec3_values(reader, self)?;
        let ranges = M2TrackResolver::resolve_ranges(reader, self)?;
        Ok((timestamps, values, ranges))
    }
}

/// Extension trait for M2TrackQuat to resolve animation data  
pub trait M2TrackQuatExt {
    /// Load the actual animation data from the file
    fn resolve_data<R: Read + Seek>(&self, reader: &mut R) -> Result<QuatTrackData>;
}

impl M2TrackQuatExt for M2TrackQuat {
    fn resolve_data<R: Read + Seek>(&self, reader: &mut R) -> Result<QuatTrackData> {
        let timestamps = M2TrackResolver::resolve_timestamps(reader, self)?;
        let values = M2TrackResolver::resolve_quat_values(reader, self)?;
        let ranges = M2TrackResolver::resolve_ranges(reader, self)?;
        Ok((timestamps, values, ranges))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunks::m2_track::M2TrackBase;
    use crate::common::M2Array;
    use std::io::Cursor;

    #[test]
    fn test_resolve_empty_track() {
        let track = M2TrackVec3 {
            base: M2TrackBase {
                interpolation_type: crate::chunks::animation::M2InterpolationType::None,
                global_sequence: 0xFFFF,
            },
            ranges: None,
            timestamps: M2Array::new(0, 0),
            values: M2Array::new(0, 0),
        };

        let data = vec![0u8; 100];
        let mut cursor = Cursor::new(data);

        let (timestamps, values, ranges) = track.resolve_data(&mut cursor).unwrap();
        assert!(timestamps.is_empty());
        assert!(values.is_empty());
        assert_eq!(ranges, None);
    }

    #[test]
    fn test_resolve_track_with_data() {
        // Create test data with timestamps and values
        let mut data = vec![0u8; 1000];

        // Write timestamps at offset 100 (3 timestamps)
        let timestamp_offset = 100;
        data[timestamp_offset..timestamp_offset + 4].copy_from_slice(&10u32.to_le_bytes());
        data[timestamp_offset + 4..timestamp_offset + 8].copy_from_slice(&20u32.to_le_bytes());
        data[timestamp_offset + 8..timestamp_offset + 12].copy_from_slice(&30u32.to_le_bytes());

        // Write Vec3 values at offset 200 (3 vectors)
        let value_offset = 200;
        data[value_offset..value_offset + 4].copy_from_slice(&1.0f32.to_le_bytes());
        data[value_offset + 4..value_offset + 8].copy_from_slice(&2.0f32.to_le_bytes());
        data[value_offset + 8..value_offset + 12].copy_from_slice(&3.0f32.to_le_bytes());

        data[value_offset + 12..value_offset + 16].copy_from_slice(&4.0f32.to_le_bytes());
        data[value_offset + 16..value_offset + 20].copy_from_slice(&5.0f32.to_le_bytes());
        data[value_offset + 20..value_offset + 24].copy_from_slice(&6.0f32.to_le_bytes());

        data[value_offset + 24..value_offset + 28].copy_from_slice(&7.0f32.to_le_bytes());
        data[value_offset + 28..value_offset + 32].copy_from_slice(&8.0f32.to_le_bytes());
        data[value_offset + 32..value_offset + 36].copy_from_slice(&9.0f32.to_le_bytes());

        let track = M2TrackVec3 {
            base: M2TrackBase {
                interpolation_type: crate::chunks::animation::M2InterpolationType::None,
                global_sequence: 0xFFFF,
            },
            ranges: None,
            timestamps: M2Array::new(3, timestamp_offset as u32),
            values: M2Array::new(3, value_offset as u32),
        };

        let mut cursor = Cursor::new(data);
        let (timestamps, values, _ranges) = track.resolve_data(&mut cursor).unwrap();

        assert_eq!(timestamps, vec![10, 20, 30]);
        assert_eq!(values.len(), 3);
        assert_eq!(
            values[0],
            C3Vector {
                x: 1.0,
                y: 2.0,
                z: 3.0
            }
        );
        assert_eq!(
            values[1],
            C3Vector {
                x: 4.0,
                y: 5.0,
                z: 6.0
            }
        );
        assert_eq!(
            values[2],
            C3Vector {
                x: 7.0,
                y: 8.0,
                z: 9.0
            }
        );
    }
}
