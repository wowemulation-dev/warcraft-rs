use binrw::{BinRead, BinWrite};

/// Sound emitter entry (28 bytes).
///
/// Defines a positioned ambient sound source within the terrain chunk.
/// Used for environmental audio like waterfalls, wind, animal sounds, etc.
///
/// # Binary Layout
///
/// ```text
/// Offset | Size | Field          | Description
/// -------|------|----------------|-----------------------------------
/// 0x00   |  4   | sound_entry_id | SoundEntries.dbc reference
/// 0x04   | 12   | position       | World position [X, Y, Z]
/// 0x10   | 12   | size_min       | Minimum size bounds [X, Y, Z]
/// 0x1C   |  0   | (size_max removed in modern clients)
/// ```
///
/// Total: 28 bytes (modern format)
///
/// Note: Older clients had size_max (12 bytes) for 40-byte entries.
/// Modern format uses only size_min for spherical bounds.
///
/// Reference: <https://wowdev.wiki/ADT/v18#MCSE_sub-chunk>
#[derive(Debug, Clone, Copy, BinRead, BinWrite)]
#[brw(little)]
pub struct SoundEmitter {
    /// Sound ID from SoundEntries.dbc
    pub sound_entry_id: u32,

    /// World position [X, Y, Z]
    pub position: [f32; 3],

    /// Minimum size bounds [X, Y, Z]
    ///
    /// Defines the attenuation radius for the sound source.
    /// In modern clients, this is treated as a spherical radius.
    pub size_min: [f32; 3],

    /// Padding to align to 28 bytes
    ///
    /// Modern format uses 28 bytes. Some older formats may use 40 bytes
    /// with an additional size_max field.
    #[brw(pad_after = 0)]
    pub _padding: [u8; 0],
}

impl Default for SoundEmitter {
    fn default() -> Self {
        Self {
            sound_entry_id: 0,
            position: [0.0; 3],
            size_min: [0.0; 3],
            _padding: [],
        }
    }
}

impl SoundEmitter {
    /// Get spherical radius from size_min.
    ///
    /// Uses the maximum component as the effective radius.
    pub fn radius(&self) -> f32 {
        self.size_min[0].max(self.size_min[1]).max(self.size_min[2])
    }

    /// Check if position is within emitter range.
    ///
    /// # Arguments
    ///
    /// * `pos` - Test position [X, Y, Z]
    ///
    /// # Returns
    ///
    /// `true` if position is within spherical radius
    pub fn is_in_range(&self, pos: [f32; 3]) -> bool {
        let dx = pos[0] - self.position[0];
        let dy = pos[1] - self.position[1];
        let dz = pos[2] - self.position[2];
        let dist_sq = dx * dx + dy * dy + dz * dz;
        let radius = self.radius();
        dist_sq <= radius * radius
    }
}

/// MCSE chunk - Sound emitters (Vanilla+).
///
/// Contains positioned ambient sound sources for environmental audio.
/// Count specified by `McnkHeader.n_snd_emitters`.
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ✅ Introduced
/// - **TBC (2.4.3)**: ✅ Present
/// - **WotLK (3.3.5a)**: ✅ Present
/// - **Cataclysm (4.3.4)**: ✅ Present
/// - **MoP (5.4.8)**: ✅ Present
///
/// Reference: <https://wowdev.wiki/ADT/v18#MCSE_sub-chunk>
#[derive(Debug, Clone, Default, BinRead, BinWrite)]
#[brw(little)]
pub struct McseChunk {
    /// Sound emitter entries
    #[br(parse_with = binrw::helpers::until_eof)]
    pub emitters: Vec<SoundEmitter>,
}

impl McseChunk {
    /// Entry size in bytes.
    pub const ENTRY_SIZE: usize = 28;

    /// Get number of sound emitters.
    pub fn count(&self) -> usize {
        self.emitters.len()
    }

    /// Validate emitter count matches header.
    ///
    /// # Arguments
    ///
    /// * `expected_count` - Count from `McnkHeader.n_snd_emitters`
    pub fn validate_count(&self, expected_count: usize) -> bool {
        self.emitters.len() == expected_count
    }

    /// Find emitters within range of a position.
    ///
    /// # Arguments
    ///
    /// * `pos` - Test position [X, Y, Z]
    ///
    /// # Returns
    ///
    /// Indices of emitters within range
    pub fn find_in_range(&self, pos: [f32; 3]) -> Vec<usize> {
        self.emitters
            .iter()
            .enumerate()
            .filter(|(_, e)| e.is_in_range(pos))
            .map(|(i, _)| i)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::{BinRead, BinWrite};
    use std::io::Cursor;

    #[test]
    fn test_sound_emitter_size() {
        assert_eq!(
            std::mem::size_of::<SoundEmitter>(),
            28,
            "SoundEmitter must be exactly 28 bytes"
        );
    }

    #[test]
    fn test_sound_emitter_parsing() {
        let data: Vec<u8> = vec![
            // sound_entry_id: 42
            0x2A, 0x00, 0x00, 0x00, // position: [100.0, 200.0, 300.0]
            0x00, 0x00, 0xC8, 0x42, // 100.0
            0x00, 0x00, 0x48, 0x43, // 200.0
            0x00, 0x00, 0x96, 0x43, // 300.0
            // size_min: [10.0, 20.0, 30.0]
            0x00, 0x00, 0x20, 0x41, // 10.0
            0x00, 0x00, 0xA0, 0x41, // 20.0
            0x00, 0x00, 0xF0, 0x41, // 30.0
        ];

        let mut cursor = Cursor::new(&data);
        let emitter = SoundEmitter::read(&mut cursor).unwrap();

        assert_eq!(emitter.sound_entry_id, 42);
        assert_eq!(emitter.position, [100.0, 200.0, 300.0]);
        assert_eq!(emitter.size_min, [10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_sound_emitter_default() {
        let emitter = SoundEmitter::default();

        assert_eq!(emitter.sound_entry_id, 0);
        assert_eq!(emitter.position, [0.0, 0.0, 0.0]);
        assert_eq!(emitter.size_min, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_sound_emitter_radius() {
        let emitter = SoundEmitter {
            sound_entry_id: 1,
            position: [0.0; 3],
            size_min: [10.0, 20.0, 15.0],
            _padding: [],
        };

        assert_eq!(emitter.radius(), 20.0);
    }

    #[test]
    fn test_sound_emitter_radius_equal_components() {
        let emitter = SoundEmitter {
            sound_entry_id: 1,
            position: [0.0; 3],
            size_min: [25.0, 25.0, 25.0],
            _padding: [],
        };

        assert_eq!(emitter.radius(), 25.0);
    }

    #[test]
    fn test_sound_emitter_is_in_range_inside() {
        let emitter = SoundEmitter {
            sound_entry_id: 1,
            position: [0.0, 0.0, 0.0],
            size_min: [10.0, 10.0, 10.0],
            _padding: [],
        };

        // Point at origin (within range)
        assert!(emitter.is_in_range([0.0, 0.0, 0.0]));

        // Point 5 units away (within 10-unit radius)
        assert!(emitter.is_in_range([5.0, 0.0, 0.0]));
        assert!(emitter.is_in_range([0.0, 5.0, 0.0]));
        assert!(emitter.is_in_range([0.0, 0.0, 5.0]));

        // Point at edge (exactly 10 units away)
        assert!(emitter.is_in_range([10.0, 0.0, 0.0]));
    }

    #[test]
    fn test_sound_emitter_is_in_range_outside() {
        let emitter = SoundEmitter {
            sound_entry_id: 1,
            position: [0.0, 0.0, 0.0],
            size_min: [10.0, 10.0, 10.0],
            _padding: [],
        };

        // Points outside 10-unit radius
        assert!(!emitter.is_in_range([15.0, 0.0, 0.0]));
        assert!(!emitter.is_in_range([0.0, 15.0, 0.0]));
        assert!(!emitter.is_in_range([0.0, 0.0, 15.0]));

        // Diagonal point outside radius
        assert!(!emitter.is_in_range([8.0, 8.0, 8.0]));
    }

    #[test]
    fn test_sound_emitter_is_in_range_offset_position() {
        let emitter = SoundEmitter {
            sound_entry_id: 1,
            position: [100.0, 200.0, 300.0],
            size_min: [25.0, 25.0, 25.0],
            _padding: [],
        };

        // Point at emitter position
        assert!(emitter.is_in_range([100.0, 200.0, 300.0]));

        // Point within range
        assert!(emitter.is_in_range([110.0, 200.0, 300.0]));

        // Point outside range
        assert!(!emitter.is_in_range([130.0, 200.0, 300.0]));
    }

    #[test]
    fn test_mcse_chunk_parsing_single() {
        let data: Vec<u8> = vec![
            // Single emitter
            0x01, 0x00, 0x00, 0x00, // sound_entry_id: 1
            0x00, 0x00, 0x00, 0x00, // position X: 0.0
            0x00, 0x00, 0x00, 0x00, // position Y: 0.0
            0x00, 0x00, 0x00, 0x00, // position Z: 0.0
            0x00, 0x00, 0x20, 0x41, // size_min X: 10.0
            0x00, 0x00, 0x20, 0x41, // size_min Y: 10.0
            0x00, 0x00, 0x20, 0x41, // size_min Z: 10.0
        ];

        let mut cursor = Cursor::new(&data);
        let chunk = McseChunk::read(&mut cursor).unwrap();

        assert_eq!(chunk.count(), 1);
        assert_eq!(chunk.emitters[0].sound_entry_id, 1);
        assert_eq!(chunk.emitters[0].position, [0.0, 0.0, 0.0]);
        assert_eq!(chunk.emitters[0].size_min, [10.0, 10.0, 10.0]);
    }

    #[test]
    fn test_mcse_chunk_parsing_multiple() {
        let mut data = Vec::new();

        // First emitter
        data.extend_from_slice(&[
            0x01, 0x00, 0x00, 0x00, // sound_entry_id: 1
            0x00, 0x00, 0x00, 0x00, // position X: 0.0
            0x00, 0x00, 0x00, 0x00, // position Y: 0.0
            0x00, 0x00, 0x00, 0x00, // position Z: 0.0
            0x00, 0x00, 0x20, 0x41, // size_min X: 10.0
            0x00, 0x00, 0x20, 0x41, // size_min Y: 10.0
            0x00, 0x00, 0x20, 0x41, // size_min Z: 10.0
        ]);

        // Second emitter
        data.extend_from_slice(&[
            0x02, 0x00, 0x00, 0x00, // sound_entry_id: 2
            0x00, 0x00, 0xC8, 0x42, // position X: 100.0
            0x00, 0x00, 0x48, 0x43, // position Y: 200.0
            0x00, 0x00, 0x96, 0x43, // position Z: 300.0
            0x00, 0x00, 0xA0, 0x41, // size_min X: 20.0
            0x00, 0x00, 0xA0, 0x41, // size_min Y: 20.0
            0x00, 0x00, 0xA0, 0x41, // size_min Z: 20.0
        ]);

        let mut cursor = Cursor::new(&data);
        let chunk = McseChunk::read(&mut cursor).unwrap();

        assert_eq!(chunk.count(), 2);

        assert_eq!(chunk.emitters[0].sound_entry_id, 1);
        assert_eq!(chunk.emitters[0].position, [0.0, 0.0, 0.0]);
        assert_eq!(chunk.emitters[0].size_min, [10.0, 10.0, 10.0]);

        assert_eq!(chunk.emitters[1].sound_entry_id, 2);
        assert_eq!(chunk.emitters[1].position, [100.0, 200.0, 300.0]);
        assert_eq!(chunk.emitters[1].size_min, [20.0, 20.0, 20.0]);
    }

    #[test]
    fn test_mcse_chunk_empty() {
        let data: Vec<u8> = vec![];

        let mut cursor = Cursor::new(&data);
        let chunk = McseChunk::read(&mut cursor).unwrap();

        assert_eq!(chunk.count(), 0);
        assert!(chunk.emitters.is_empty());
    }

    #[test]
    fn test_mcse_chunk_validate_count() {
        let data: Vec<u8> = vec![
            0x01, 0x00, 0x00, 0x00, // sound_entry_id: 1
            0x00, 0x00, 0x00, 0x00, // position X: 0.0
            0x00, 0x00, 0x00, 0x00, // position Y: 0.0
            0x00, 0x00, 0x00, 0x00, // position Z: 0.0
            0x00, 0x00, 0x20, 0x41, // size_min X: 10.0
            0x00, 0x00, 0x20, 0x41, // size_min Y: 10.0
            0x00, 0x00, 0x20, 0x41, // size_min Z: 10.0
        ];

        let mut cursor = Cursor::new(&data);
        let chunk = McseChunk::read(&mut cursor).unwrap();

        assert!(chunk.validate_count(1));
        assert!(!chunk.validate_count(0));
        assert!(!chunk.validate_count(2));
    }

    #[test]
    fn test_mcse_chunk_find_in_range_single() {
        let data: Vec<u8> = vec![
            0x01, 0x00, 0x00, 0x00, // sound_entry_id: 1
            0x00, 0x00, 0x00, 0x00, // position X: 0.0
            0x00, 0x00, 0x00, 0x00, // position Y: 0.0
            0x00, 0x00, 0x00, 0x00, // position Z: 0.0
            0x00, 0x00, 0x20, 0x41, // size_min X: 10.0
            0x00, 0x00, 0x20, 0x41, // size_min Y: 10.0
            0x00, 0x00, 0x20, 0x41, // size_min Z: 10.0
        ];

        let mut cursor = Cursor::new(&data);
        let chunk = McseChunk::read(&mut cursor).unwrap();

        // Position within range
        let indices = chunk.find_in_range([5.0, 0.0, 0.0]);
        assert_eq!(indices, vec![0]);

        // Position outside range
        let indices = chunk.find_in_range([20.0, 0.0, 0.0]);
        assert!(indices.is_empty());
    }

    #[test]
    fn test_mcse_chunk_find_in_range_multiple() {
        let mut data = Vec::new();

        // First emitter at origin, radius 10
        data.extend_from_slice(&[
            0x01, 0x00, 0x00, 0x00, // sound_entry_id: 1
            0x00, 0x00, 0x00, 0x00, // position X: 0.0
            0x00, 0x00, 0x00, 0x00, // position Y: 0.0
            0x00, 0x00, 0x00, 0x00, // position Z: 0.0
            0x00, 0x00, 0x20, 0x41, // size_min X: 10.0
            0x00, 0x00, 0x20, 0x41, // size_min Y: 10.0
            0x00, 0x00, 0x20, 0x41, // size_min Z: 10.0
        ]);

        // Second emitter at [100, 0, 0], radius 20
        data.extend_from_slice(&[
            0x02, 0x00, 0x00, 0x00, // sound_entry_id: 2
            0x00, 0x00, 0xC8, 0x42, // position X: 100.0
            0x00, 0x00, 0x00, 0x00, // position Y: 0.0
            0x00, 0x00, 0x00, 0x00, // position Z: 0.0
            0x00, 0x00, 0xA0, 0x41, // size_min X: 20.0
            0x00, 0x00, 0xA0, 0x41, // size_min Y: 20.0
            0x00, 0x00, 0xA0, 0x41, // size_min Z: 20.0
        ]);

        let mut cursor = Cursor::new(&data);
        let chunk = McseChunk::read(&mut cursor).unwrap();

        // Position near origin - only first emitter
        let indices = chunk.find_in_range([5.0, 0.0, 0.0]);
        assert_eq!(indices, vec![0]);

        // Position near second emitter - only second emitter
        let indices = chunk.find_in_range([110.0, 0.0, 0.0]);
        assert_eq!(indices, vec![1]);

        // Position far from both
        let indices = chunk.find_in_range([50.0, 0.0, 0.0]);
        assert!(indices.is_empty());
    }

    #[test]
    fn test_mcse_chunk_default() {
        let chunk = McseChunk::default();

        assert_eq!(chunk.count(), 0);
        assert!(chunk.emitters.is_empty());
    }

    #[test]
    fn test_sound_emitter_round_trip() {
        let original = SoundEmitter {
            sound_entry_id: 42,
            position: [100.0, 200.0, 300.0],
            size_min: [10.0, 20.0, 30.0],
            _padding: [],
        };

        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        original.write(&mut cursor).unwrap();

        let mut cursor = Cursor::new(&buffer);
        let parsed = SoundEmitter::read(&mut cursor).unwrap();

        assert_eq!(parsed.sound_entry_id, original.sound_entry_id);
        assert_eq!(parsed.position, original.position);
        assert_eq!(parsed.size_min, original.size_min);
    }

    #[test]
    fn test_mcse_chunk_round_trip() {
        let original = McseChunk {
            emitters: vec![
                SoundEmitter {
                    sound_entry_id: 1,
                    position: [0.0, 0.0, 0.0],
                    size_min: [10.0, 10.0, 10.0],
                    _padding: [],
                },
                SoundEmitter {
                    sound_entry_id: 2,
                    position: [100.0, 200.0, 300.0],
                    size_min: [20.0, 20.0, 20.0],
                    _padding: [],
                },
            ],
        };

        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        original.write(&mut cursor).unwrap();

        let mut cursor = Cursor::new(&buffer);
        let parsed = McseChunk::read(&mut cursor).unwrap();

        assert_eq!(parsed.count(), original.count());
        for (p, o) in parsed.emitters.iter().zip(original.emitters.iter()) {
            assert_eq!(p.sound_entry_id, o.sound_entry_id);
            assert_eq!(p.position, o.position);
            assert_eq!(p.size_min, o.size_min);
        }
    }

    #[test]
    fn test_mcse_entry_size_constant() {
        assert_eq!(McseChunk::ENTRY_SIZE, 28);
    }
}
