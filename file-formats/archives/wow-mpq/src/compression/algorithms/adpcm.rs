//! IMA ADPCM compression algorithm implementation
//!
//! This module implements the IMA ADPCM (Interactive Multimedia Association Adaptive
//! Differential Pulse Code Modulation) compression algorithm used in MPQ archives
//! for compressing audio data, particularly WAV files.

use crate::error::{Error, Result};

/// Maximum number of channels supported
const MAX_ADPCM_CHANNEL_COUNT: usize = 2;

/// Initial step index for ADPCM compression
const INITIAL_ADPCM_STEP_INDEX: usize = 0x2C;

/// Table for determining the next step index
const NEXT_STEP_TABLE: [i8; 32] = [
    -1, 0, -1, 4, -1, 2, -1, 6, -1, 1, -1, 5, -1, 3, -1, 7, -1, 1, -1, 5, -1, 3, -1, 7, -1, 2, -1,
    4, -1, 6, -1, 8,
];

/// Step size table for ADPCM encoding/decoding
const STEP_SIZE_TABLE: [i32; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66,
    73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408, 449,
    494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066, 2272,
    2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630, 9493,
    10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794, 32767,
];

/// Compress data using IMA ADPCM algorithm
pub(crate) fn compress_mono(input: &[u8], compression_level: u8) -> Result<Vec<u8>> {
    compress_internal(input, compression_level, 1)
}

/// Compress data using IMA ADPCM algorithm (stereo audio)
pub(crate) fn compress_stereo(input: &[u8], compression_level: u8) -> Result<Vec<u8>> {
    compress_internal(input, compression_level, 2)
}

/// Internal compression function that handles both mono and stereo
fn compress_internal(input: &[u8], compression_level: u8, channel_count: usize) -> Result<Vec<u8>> {
    // Validate channel count
    if channel_count == 0 || channel_count > MAX_ADPCM_CHANNEL_COUNT {
        return Err(Error::compression(format!(
            "Invalid channel count: {}. ADPCM supports 1-{} channels",
            channel_count, MAX_ADPCM_CHANNEL_COUNT
        )));
    }

    // ADPCM works with 16-bit samples
    if input.len() % 2 != 0 {
        return Err(Error::compression("Input must be 16-bit aligned"));
    }

    let samples_count = input.len() / 2;
    if samples_count == 0 {
        return Ok(Vec::new());
    }

    let samples_per_channel = samples_count / channel_count;
    if samples_per_channel == 0 || samples_count % channel_count != 0 {
        return Err(Error::compression("Invalid sample count for channel count"));
    }

    // Calculate bit shift from compression level
    let bit_shift = if compression_level == 0 {
        0
    } else {
        compression_level - 1
    };

    // Allocate output buffer (worst case: same size as input + header)
    let mut output = Vec::with_capacity(input.len() + 4);

    // Write header: zero byte and compression level
    output.push(0);
    output.push(bit_shift);

    // Initialize state for each channel
    let mut predicted_samples = vec![0i16; channel_count];
    let mut step_indexes = vec![INITIAL_ADPCM_STEP_INDEX; channel_count];

    // Read and write initial samples for each channel
    for (ch, predicted_sample) in predicted_samples.iter_mut().enumerate().take(channel_count) {
        let initial_sample = read_sample(input, ch)?;
        *predicted_sample = initial_sample;
        write_sample(&mut output, initial_sample);
    }

    // Compress remaining samples
    let mut sample_index = channel_count;
    let mut channel_index = channel_count - 1;

    while sample_index < samples_count {
        // Alternate between channels
        channel_index = (channel_index + 1) % channel_count;

        let input_sample = read_sample(input, sample_index)?;

        // Calculate difference from predicted sample
        let mut difference = input_sample as i32 - predicted_samples[channel_index] as i32;
        let mut encoded_sample: u8 = 0;

        // If difference is negative, set sign bit
        if difference < 0 {
            difference = -difference;
            encoded_sample |= 0x40;
        }

        let mut step_size = STEP_SIZE_TABLE[step_indexes[channel_index]];

        // Check if difference is too small (below threshold)
        if difference < (step_size >> compression_level as i32) {
            if step_indexes[channel_index] > 0 {
                step_indexes[channel_index] -= 1;
            }
            output.push(0x80);
        } else {
            // Check if difference is too large
            while difference > (step_size << 1) {
                if step_indexes[channel_index] >= 0x58 {
                    break;
                }

                step_indexes[channel_index] += 8;
                if step_indexes[channel_index] > 0x58 {
                    step_indexes[channel_index] = 0x58;
                }

                // Update step size after modifying index
                step_size = STEP_SIZE_TABLE[step_indexes[channel_index]];
                output.push(0x81);
            }

            // Encode the sample
            let max_bit_mask = if bit_shift > 0 {
                1 << (bit_shift - 1)
            } else {
                0
            };
            let max_bit_mask = std::cmp::min(max_bit_mask, 0x20);
            let base_difference = step_size >> bit_shift as i32;
            let mut total_step_size = 0;

            // Encode bits starting from LSB
            if max_bit_mask > 0 {
                let mut step_size_work = step_size; // Start with full step_size
                let mut bit_val = 0x01;

                while bit_val <= max_bit_mask {
                    if total_step_size + step_size_work <= difference {
                        total_step_size += step_size_work;
                        encoded_sample |= bit_val;
                    }
                    step_size_work >>= 1;
                    bit_val <<= 1;
                }
            }

            // Update predicted sample
            predicted_samples[channel_index] = update_predicted_sample(
                predicted_samples[channel_index] as i32,
                encoded_sample,
                base_difference + total_step_size,
            ) as i16;

            output.push(encoded_sample);

            // Update step index for next sample
            step_indexes[channel_index] =
                get_next_step_index(step_indexes[channel_index], encoded_sample);
        }

        sample_index += 1;
    }

    Ok(output)
}

/// Decompress data using IMA ADPCM algorithm (mono audio)
pub(crate) fn decompress_mono(input: &[u8], output_size: usize) -> Result<Vec<u8>> {
    decompress_internal(input, output_size, 1)
}

/// Decompress data using IMA ADPCM algorithm (stereo audio)
pub(crate) fn decompress_stereo(input: &[u8], output_size: usize) -> Result<Vec<u8>> {
    decompress_internal(input, output_size, 2)
}

/// Internal decompression function that handles both mono and stereo
fn decompress_internal(input: &[u8], output_size: usize, channel_count: usize) -> Result<Vec<u8>> {
    // Validate channel count
    if channel_count == 0 || channel_count > MAX_ADPCM_CHANNEL_COUNT {
        return Err(Error::compression(format!(
            "Invalid channel count: {}. ADPCM supports 1-{} channels",
            channel_count, MAX_ADPCM_CHANNEL_COUNT
        )));
    }

    // Handle empty input
    if input.is_empty() && output_size == 0 {
        return Ok(Vec::new());
    }

    if input.len() < 4 {
        return Err(Error::compression("Input too small for ADPCM"));
    }

    let mut input_pos = 0;

    // Read header
    let _zero = input[input_pos];
    input_pos += 1;
    let bit_shift = input[input_pos];
    input_pos += 1;

    // Initialize state for each channel
    let mut predicted_samples = vec![0i16; channel_count];
    let mut step_indexes = vec![INITIAL_ADPCM_STEP_INDEX; channel_count];

    // Allocate output buffer
    let mut output = Vec::with_capacity(output_size);

    // Read initial samples for each channel
    for predicted_sample in predicted_samples.iter_mut().take(channel_count) {
        if input_pos + 2 > input.len() {
            return Err(Error::compression("Missing initial sample"));
        }

        let initial_sample = read_sample(input, input_pos / 2)?;
        *predicted_sample = initial_sample;
        write_sample(&mut output, initial_sample);
        input_pos += 2;
    }

    // Initialize channel index
    let mut channel_index = channel_count - 1;

    // Decompress remaining data
    while input_pos < input.len() && output.len() < output_size {
        let encoded_sample = input[input_pos];
        input_pos += 1;

        // Alternate between channels
        channel_index = (channel_index + 1) % channel_count;

        if encoded_sample == 0x80 {
            // Step index decrease marker
            if step_indexes[channel_index] > 0 {
                step_indexes[channel_index] -= 1;
            }
            write_sample(&mut output, predicted_samples[channel_index]);
        } else if encoded_sample == 0x81 {
            // Step index increase marker
            step_indexes[channel_index] += 8;
            if step_indexes[channel_index] > 0x58 {
                step_indexes[channel_index] = 0x58;
            }
            // For 0x81, we stay on the same channel for next sample
            channel_index = (channel_index + channel_count - 1) % channel_count;
        } else {
            // Decode the sample
            let step_size = STEP_SIZE_TABLE[step_indexes[channel_index]];
            predicted_samples[channel_index] = decode_sample(
                predicted_samples[channel_index] as i32,
                encoded_sample,
                step_size,
                step_size >> bit_shift as i32,
            ) as i16;

            write_sample(&mut output, predicted_samples[channel_index]);

            // Update step index
            step_indexes[channel_index] =
                get_next_step_index(step_indexes[channel_index], encoded_sample);
        }
    }

    Ok(output)
}

/// Read a 16-bit sample from the input buffer
fn read_sample(input: &[u8], sample_index: usize) -> Result<i16> {
    let byte_index = sample_index * 2;
    if byte_index + 1 >= input.len() {
        return Err(Error::compression("Sample index out of bounds"));
    }

    let sample = input[byte_index] as i16 | ((input[byte_index + 1] as i16) << 8);
    Ok(sample)
}

/// Write a 16-bit sample to the output buffer
fn write_sample(output: &mut Vec<u8>, sample: i16) {
    output.push((sample & 0xFF) as u8);
    output.push((sample >> 8) as u8);
}

/// Get the next step index based on the encoded sample
fn get_next_step_index(step_index: usize, encoded_sample: u8) -> usize {
    let index_change = NEXT_STEP_TABLE[(encoded_sample & 0x1F) as usize];
    let new_index = step_index as i32 + index_change as i32;

    if new_index < 0 {
        0
    } else if new_index > 88 {
        88
    } else {
        new_index as usize
    }
}

/// Update the predicted sample based on the encoded value
fn update_predicted_sample(predicted_sample: i32, encoded_sample: u8, difference: i32) -> i32 {
    let new_sample = if encoded_sample & 0x40 != 0 {
        predicted_sample - difference
    } else {
        predicted_sample + difference
    };

    // Clamp to 16-bit signed range
    new_sample.clamp(-32768, 32767)
}

/// Decode a sample using the ADPCM algorithm
fn decode_sample(
    predicted_sample: i32,
    encoded_sample: u8,
    step_size: i32,
    base_difference: i32,
) -> i32 {
    let mut difference = base_difference;

    if encoded_sample & 0x01 != 0 {
        difference += step_size;
    }
    if encoded_sample & 0x02 != 0 {
        difference += step_size >> 1;
    }
    if encoded_sample & 0x04 != 0 {
        difference += step_size >> 2;
    }
    if encoded_sample & 0x08 != 0 {
        difference += step_size >> 3;
    }
    if encoded_sample & 0x10 != 0 {
        difference += step_size >> 4;
    }
    if encoded_sample & 0x20 != 0 {
        difference += step_size >> 5;
    }

    update_predicted_sample(predicted_sample, encoded_sample, difference)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Extract samples from byte data for comparison
    fn extract_samples(data: &[u8]) -> Vec<i16> {
        data.chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect()
    }

    #[test]
    fn test_read_write_sample() {
        let mut buffer = Vec::new();
        write_sample(&mut buffer, 0x1234);
        write_sample(&mut buffer, -0x5678);

        assert_eq!(read_sample(&buffer, 0).unwrap(), 0x1234);
        assert_eq!(read_sample(&buffer, 1).unwrap(), -0x5678);
    }

    #[test]
    fn test_step_index_bounds() {
        assert_eq!(get_next_step_index(0, 0x00), 0); // Would go negative
        assert_eq!(get_next_step_index(88, 0x1F), 88); // Would go over max
        assert_eq!(get_next_step_index(44, 0x01), 44); // No change
    }

    #[test]
    fn test_predicted_sample_clamping() {
        assert_eq!(update_predicted_sample(30000, 0x00, 5000), 32767);
        assert_eq!(update_predicted_sample(-30000, 0x40, 5000), -32768);
    }

    #[test]
    fn test_empty_input() {
        let result = compress_mono(&[], 5);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_simple_values() {
        // Test with simple known values
        let mut input = Vec::new();
        write_sample(&mut input, 0);
        write_sample(&mut input, 1000);
        write_sample(&mut input, -1000);
        write_sample(&mut input, 5000);
        write_sample(&mut input, -5000);

        let compressed = compress_mono(&input, 5).unwrap();
        println!("Compressed: {:?}", compressed);

        let decompressed = decompress_mono(&compressed, input.len()).unwrap();

        for i in 0..5 {
            let original = read_sample(&input, i).unwrap();
            let decoded = read_sample(&decompressed, i).unwrap();
            println!("Sample {}: {} -> {}", i, original, decoded);
        }
    }

    #[test]
    fn test_basic_compress_decompress() {
        // Create a simple sine wave pattern
        let mut input = Vec::new();
        for i in 0..100 {
            let sample = ((i as f32 * 0.1).sin() * 10000.0) as i16;
            write_sample(&mut input, sample);
        }

        // Test with different compression levels
        for level in 1..=5 {
            let compressed = compress_mono(&input, level).unwrap();
            assert!(compressed.len() > 4); // At least header + initial sample

            let decompressed = decompress_mono(&compressed, input.len()).unwrap();
            assert_eq!(decompressed.len(), input.len());

            // ADPCM is lossy, so we check that samples are reasonably close
            for i in 0..50 {
                let original = read_sample(&input, i).unwrap();
                let decoded = read_sample(&decompressed, i).unwrap();
                let diff = (original - decoded).abs();

                // Allow some error due to lossy compression
                assert!(
                    diff < 1000,
                    "Sample {} differs too much: {} vs {}",
                    i,
                    original,
                    decoded
                );
            }
        }
    }

    #[test]
    fn test_stereo_basic() {
        // Create stereo audio data (interleaved: L, R, L, R, ...)
        let mut input = Vec::new();
        for i in 0..100 {
            // Left channel: sine wave
            let left = ((i as f32 * 0.1).sin() * 8000.0) as i16;
            write_sample(&mut input, left);

            // Right channel: cosine wave
            let right = ((i as f32 * 0.1).cos() * 8000.0) as i16;
            write_sample(&mut input, right);
        }

        let compressed = compress_stereo(&input, 5).unwrap();
        assert!(compressed.len() > 8); // At least header + 2 initial samples

        let decompressed = decompress_stereo(&compressed, input.len()).unwrap();
        assert_eq!(decompressed.len(), input.len());

        // Check quality
        for i in 0..100 {
            let left_orig = read_sample(&input, i * 2).unwrap();
            let left_dec = read_sample(&decompressed, i * 2).unwrap();
            let left_diff = (left_orig - left_dec).abs();

            let right_orig = read_sample(&input, i * 2 + 1).unwrap();
            let right_dec = read_sample(&decompressed, i * 2 + 1).unwrap();
            let right_diff = (right_orig - right_dec).abs();

            assert!(left_diff < 1000, "Left sample {} differs too much", i);
            assert!(right_diff < 1000, "Right sample {} differs too much", i);
        }
    }

    #[test]
    fn test_stereo_silence() {
        // Test stereo silence
        let silence = vec![0u8; 400]; // 100 stereo samples

        let compressed = compress_stereo(&silence, 5).unwrap();
        let decompressed = decompress_stereo(&compressed, silence.len()).unwrap();

        // Check that silence is preserved (allowing small errors)
        let samples = extract_samples(&decompressed);
        for (i, sample) in samples.iter().enumerate() {
            assert!(
                sample.abs() <= 1,
                "Stereo silence sample {} too large: {}",
                i,
                sample
            );
        }
    }

    #[test]
    fn test_stereo_channels_independent() {
        // Test that channels are compressed independently
        let mut input = Vec::new();
        for i in 0..50 {
            // Left channel: constant value
            write_sample(&mut input, 5000);

            // Right channel: varying values
            write_sample(&mut input, (i * 100) as i16);
        }

        let compressed = compress_stereo(&input, 5).unwrap();
        let decompressed = decompress_stereo(&compressed, input.len()).unwrap();

        // Left channel should be very close to constant
        for i in 0..50 {
            let left = read_sample(&decompressed, i * 2).unwrap();
            assert!(
                (left - 5000).abs() < 100,
                "Left channel not constant at sample {}",
                i
            );
        }

        // Right channel should vary
        let mut max_val = 0i16;
        let mut min_val = 32767i16;
        for i in 0..50 {
            let right = read_sample(&decompressed, i * 2 + 1).unwrap();
            max_val = max_val.max(right);
            min_val = min_val.min(right);
        }
        assert!(
            max_val - min_val > 2000,
            "Right channel doesn't vary enough"
        );
    }

    #[test]
    fn test_channel_count_validation() {
        // Test data - 8 bytes = 4 16-bit samples
        let test_data = vec![0u8; 8];

        // Test valid channel counts (1 and 2)
        let result = compress_internal(&test_data, 1, 1);
        assert!(result.is_ok(), "Mono compression should be valid");

        let result = compress_internal(&test_data, 1, 2);
        assert!(result.is_ok(), "Stereo compression should be valid");

        // Test invalid channel counts
        let result = compress_internal(&test_data, 1, 0);
        assert!(result.is_err(), "0 channels should be invalid");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Invalid channel count: 0"));
        assert!(err_msg.contains("ADPCM supports 1-2 channels"));

        let result = compress_internal(&test_data, 1, 3);
        assert!(result.is_err(), "3 channels should be invalid");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Invalid channel count: 3"));
        assert!(err_msg.contains("ADPCM supports 1-2 channels"));

        // Test decompression validation
        let compressed_data = vec![0u8; 10]; // Some dummy compressed data
        let result = decompress_internal(&compressed_data, 8, 0);
        assert!(
            result.is_err(),
            "0 channels should be invalid for decompression"
        );

        let result = decompress_internal(&compressed_data, 8, 3);
        assert!(
            result.is_err(),
            "3 channels should be invalid for decompression"
        );
    }

    #[test]
    fn test_max_channels_constant() {
        // This test ensures MAX_ADPCM_CHANNEL_COUNT is used correctly
        // Test that exactly MAX_ADPCM_CHANNEL_COUNT channels is accepted
        let test_data = vec![0u8; 16]; // 8 samples total

        // MAX_ADPCM_CHANNEL_COUNT is 2, so stereo should work
        let result = compress_internal(&test_data, 1, MAX_ADPCM_CHANNEL_COUNT);
        assert!(
            result.is_ok(),
            "MAX_ADPCM_CHANNEL_COUNT channels should be valid"
        );

        // One more than MAX should fail
        let result = compress_internal(&test_data, 1, MAX_ADPCM_CHANNEL_COUNT + 1);
        assert!(
            result.is_err(),
            "More than MAX_ADPCM_CHANNEL_COUNT should be invalid"
        );
    }
}
