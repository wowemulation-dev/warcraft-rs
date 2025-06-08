//! Sparse/RLE compression and decompression (StormLib compatible)
//!
//! This implements the exact sparse compression format used by StormLib and MPQ archives.
//! Based on StormLib's src/sparse/sparse.cpp implementation.

use crate::{Error, Result};

/// Decompress sparse/RLE compressed data (StormLib format)
pub(crate) fn decompress(data: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    // Don't decompress anything that is shorter than 5 bytes
    if data.len() < 5 {
        return Err(Error::compression(
            "Sparse decompression: input too small (< 5 bytes)",
        ));
    }

    // Get the 32-bits from the input stream (big-endian)
    let mut cb_out_buffer = 0u32;
    cb_out_buffer |= (data[0] as u32) << 0x18;
    cb_out_buffer |= (data[1] as u32) << 0x10;
    cb_out_buffer |= (data[2] as u32) << 0x08;
    cb_out_buffer |= data[3] as u32;

    // Verify the size of the stream against the output buffer size
    if cb_out_buffer as usize > expected_size {
        return Err(Error::compression(
            "Sparse decompression: stored size exceeds expected size",
        ));
    }

    let mut output = Vec::with_capacity(cb_out_buffer as usize);
    let mut pos = 4; // Skip the size header
    let mut cb_out_buffer_remaining = cb_out_buffer;

    // Process the input buffer
    while pos < data.len() {
        // Get (next) byte from the stream
        let one_byte = data[pos];
        pos += 1;

        // If highest bit, it means that normal data follow
        if one_byte & 0x80 != 0 {
            // Check the length of one chunk. Check for overflows
            let mut cb_chunk_size = ((one_byte & 0x7F) + 1) as u32;

            // Check for overflow like StormLib does
            if pos + cb_chunk_size as usize > data.len() {
                // StormLib returns 0 (failure) in this case
                // Some MPQ files might have malformed sparse data
                log::warn!(
                    "Sparse decompression: not enough data for copy (need {}, have {})",
                    cb_chunk_size,
                    data.len() - pos
                );
                // Try to recover by using available data
                cb_chunk_size = (data.len() - pos) as u32;
                if cb_chunk_size == 0 {
                    break; // No more data to process
                }
            }

            // Copy the chunk. Make sure that the buffer won't overflow
            cb_chunk_size = cb_chunk_size.min(cb_out_buffer_remaining);
            output.extend_from_slice(&data[pos..pos + cb_chunk_size as usize]);
            pos += cb_chunk_size as usize;
            cb_out_buffer_remaining -= cb_chunk_size;
        } else {
            let mut cb_chunk_size = ((one_byte & 0x7F) + 3) as u32;
            cb_chunk_size = cb_chunk_size.min(cb_out_buffer_remaining);
            output.resize(output.len() + cb_chunk_size as usize, 0);
            cb_out_buffer_remaining -= cb_chunk_size;
        }
    }

    Ok(output)
}

/// Compress using sparse/RLE compression (StormLib format)
pub(crate) fn compress(data: &[u8]) -> Result<Vec<u8>> {
    let cb_in_buffer = data.len();
    // Reserve enough space for worst case (data doesn't compress)
    let mut output = Vec::with_capacity(cb_in_buffer + 4 + (cb_in_buffer / 128) + 1);

    // Put the original data length (in big-endian)
    output.push((cb_in_buffer >> 0x18) as u8);
    output.push((cb_in_buffer >> 0x10) as u8);
    output.push((cb_in_buffer >> 0x08) as u8);
    output.push(cb_in_buffer as u8);

    let pb_in_buffer_end = data.len();
    let mut pb_in_buffer = 0;

    // If there is at least 3 bytes in the input buffer, do this loop
    while pb_in_buffer < pb_in_buffer_end.saturating_sub(3) {
        // Reset the zero count and frontal pointer
        let mut pb_last_non_zero = pb_in_buffer;
        let mut pb_in_buff_ptr = pb_in_buffer;
        let mut number_of_zeros = 0usize;

        if pb_in_buff_ptr < pb_in_buffer_end {
            loop {
                // Count number of zeros
                if data[pb_in_buff_ptr] == 0 {
                    number_of_zeros += 1;
                } else {
                    // Were there at least 3 zeros before? If yes, we need to flush the data
                    if number_of_zeros >= 3 {
                        break;
                    }
                    pb_last_non_zero = pb_in_buff_ptr + 1;
                    number_of_zeros = 0;
                }
                pb_in_buff_ptr += 1;
                if pb_in_buff_ptr >= pb_in_buffer_end {
                    break;
                }
            }
        }

        // Get number of nonzeros that we found so far and flush them
        let mut number_of_non_zeros = pb_last_non_zero - pb_in_buffer;
        if number_of_non_zeros != 0 {
            // Process blocks that are longer than 0x81 nonzero bytes
            while number_of_non_zeros > 0x81 {
                // Put marker that means "0x80 of nonzeros"
                output.push(0xFF);
                output.extend_from_slice(&data[pb_in_buffer..pb_in_buffer + 0x80]);

                // Adjust counter of nonzeros and both pointers
                number_of_non_zeros -= 0x80;
                pb_in_buffer += 0x80;
            }

            // BUGBUG: The following code will be triggered if the NumberOfNonZeros
            // was 0x81 before. It will copy just one byte. This seems like a bug to me,
            // but since I want StormLib to be exact like Blizzard code is, I will keep
            // it that way here
            if number_of_non_zeros > 0x80 {
                // Put marker that means "1 nonzero byte"
                output.push(0x80);
                output.push(data[pb_in_buffer]);

                // Adjust counter of nonzeros and both pointers
                number_of_non_zeros -= 1;
                pb_in_buffer += 1;
            }

            // If there is 1 nonzero or more, put the block
            if number_of_non_zeros >= 0x01 {
                // Put marker that means "Several nonzero bytes"
                output.push(0x80 | (number_of_non_zeros - 1) as u8);
                output.extend_from_slice(&data[pb_in_buffer..pb_in_buffer + number_of_non_zeros]);

                // Adjust pointers
                pb_in_buffer += number_of_non_zeros;
            }
        }

        // Now flush all zero bytes
        while number_of_zeros > 0x85 {
            // Put marker that means "0x82 zeros"
            output.push(0x7F);

            // Adjust zero counter and input pointer
            number_of_zeros -= 0x82;
            pb_in_buffer += 0x82;
        }

        // If we got more than 0x82 zeros, flush 3 of them now
        if number_of_zeros > 0x82 {
            // Put marker that means "0x03 zeros"
            output.push(0);

            // Adjust zero counter and input pointer
            number_of_zeros -= 0x03;
            pb_in_buffer += 0x03;
        }

        // Is there at least three zeros?
        if number_of_zeros >= 3 {
            // Put marker that means "Several zeros"
            output.push((number_of_zeros - 3) as u8);

            // Adjust pointer
            pb_in_buffer += number_of_zeros;
        }
    }

    // Flush last three bytes
    if pb_in_buffer < pb_in_buffer_end {
        let mut pb_in_buff_ptr = pb_in_buffer;

        loop {
            if pb_in_buff_ptr < pb_in_buffer_end && data[pb_in_buff_ptr] != 0 {
                // Get number of bytes remaining
                let number_of_non_zeros = pb_in_buffer_end - pb_in_buffer;

                // Use the correct marker for the actual number of bytes
                if number_of_non_zeros <= 0x80 {
                    output.push(0x80 | (number_of_non_zeros - 1) as u8);
                } else {
                    // For larger chunks, use 0xFF
                    output.push(0xFF);
                }
                output.extend_from_slice(&data[pb_in_buffer..pb_in_buffer + number_of_non_zeros]);
                break;
            } else {
                pb_in_buff_ptr += 1;
                // Is there are more chars in the input buffer
                if pb_in_buff_ptr < pb_in_buffer_end {
                    continue;
                }

                // Terminate with a chunk that means "0x82 of zeros"
                output.push(0x7F);
                break;
            }
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompress() {
        // Test StormLib sparse format
        // Format: [4-byte size BE] [control bytes + data]
        // Control byte: 0x80+ = literal data, 0x00-0x7F = zeros
        let mut compressed = vec![];

        // Size header (15 bytes, big-endian)
        compressed.extend_from_slice(&[0x00, 0x00, 0x00, 0x0F]);

        // "Hello" = 5 bytes literal
        compressed.push(0x84); // 0x80 | (5-1)
        compressed.extend_from_slice(b"Hello");

        // 5 zeros
        compressed.push(0x02); // 5-3 = 2

        // "World" = 5 bytes literal
        compressed.push(0x84); // 0x80 | (5-1)
        compressed.extend_from_slice(b"World");

        let decompressed = decompress(&compressed, 15).expect("Decompression failed");
        let expected = b"Hello\0\0\0\0\0World";

        assert_eq!(decompressed, expected);
    }

    #[test]
    fn test_round_trip() {
        let original = b"Hello\0\0\0\0\0World\0\0\0!!!";

        let compressed = compress(original).expect("Compression failed");
        println!(
            "Original len: {}, Compressed: {:?}",
            original.len(),
            compressed
        );
        let decompressed = decompress(&compressed, original.len()).expect("Decompression failed");

        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_all_zeros() {
        let original = vec![0u8; 100];

        let compressed = compress(&original).expect("Compression failed");
        assert!(compressed.len() < original.len()); // Should compress well

        let decompressed = decompress(&compressed, original.len()).expect("Decompression failed");
        assert_eq!(decompressed, original);
    }
}
