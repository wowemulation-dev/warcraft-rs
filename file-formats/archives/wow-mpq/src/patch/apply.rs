//! Patch application logic
//!
//! This module implements patch application for both COPY and BSD0 patch types.

use super::header::{PatchFile, PatchType};
use crate::{Error, Result};

/// Apply a patch file to base data
///
/// # Arguments
///
/// * `patch` - The parsed patch file containing header and patch data
/// * `base_data` - The original file data to patch
///
/// # Returns
///
/// The patched file data
///
/// # Errors
///
/// Returns error if:
/// - Base file MD5 doesn't match expected hash
/// - Patch application fails
/// - Patched result MD5 doesn't match expected hash
/// - Unsupported patch type (BSD0 not yet implemented)
pub fn apply_patch(patch: &PatchFile, base_data: &[u8]) -> Result<Vec<u8>> {
    // Verify base file MD5 before patching
    patch.verify_base(base_data)?;

    // Apply patch based on type
    let patched_data = match patch.header.patch_type {
        PatchType::Copy => apply_copy_patch(patch, base_data)?,
        PatchType::Bsd0 => apply_bsd0_patch(patch, base_data)?,
    };

    // Verify patched result MD5
    patch.verify_patched(&patched_data)?;

    Ok(patched_data)
}

/// Apply a COPY patch
///
/// COPY patches are simple file replacements - the patch data is the complete
/// new file content. We ignore the base data entirely.
///
/// # Arguments
///
/// * `patch` - The parsed COPY patch file
/// * `base_data` - The original file data (used for size verification only)
///
/// # Returns
///
/// The complete new file data (from patch.data)
fn apply_copy_patch(patch: &PatchFile, base_data: &[u8]) -> Result<Vec<u8>> {
    // Verify base file size matches expected
    if base_data.len() != patch.header.size_before as usize {
        return Err(Error::invalid_format(format!(
            "Base file size mismatch: expected {}, got {}",
            patch.header.size_before,
            base_data.len()
        )));
    }

    // Verify patch data size matches expected output size
    if patch.data.len() != patch.header.size_after as usize {
        return Err(Error::invalid_format(format!(
            "Patch data size mismatch: expected {}, got {}",
            patch.header.size_after,
            patch.data.len()
        )));
    }

    log::debug!(
        "Applying COPY patch: {} -> {} bytes",
        base_data.len(),
        patch.data.len()
    );

    // COPY patch: patch data IS the complete new file
    Ok(patch.data.clone())
}

/// Apply a BSD0 (bsdiff40) patch
///
/// BSD0 patches use binary diff algorithm to create space-efficient patches.
/// The patch data is RLE-compressed and must be decompressed before applying.
///
/// # Arguments
///
/// * `patch` - The parsed BSD0 patch file
/// * `base_data` - The original file data
///
/// # Returns
///
/// The patched file data
fn apply_bsd0_patch(patch: &PatchFile, base_data: &[u8]) -> Result<Vec<u8>> {
    use byteorder::{LittleEndian, ReadBytesExt};
    use std::io::Cursor;

    // Verify base file size
    if base_data.len() != patch.header.size_before as usize {
        return Err(Error::invalid_format(format!(
            "Base file size mismatch: expected {}, got {}",
            patch.header.size_before,
            base_data.len()
        )));
    }

    // BSD0 patch data is RLE-compressed - decompress it first
    // The RLE data has a 4-byte size header that should be skipped
    log::debug!(
        "RLE decompressing BSD0 patch data: {} bytes compressed",
        patch.data.len()
    );

    let bsdiff_data = crate::compression::rle::decompress(
        &patch.data,
        patch.header.patch_data_size as usize,
        true, // skip 4-byte size header
    )?;

    log::debug!(
        "RLE decompression complete: {} bytes → {} bytes",
        patch.data.len(),
        bsdiff_data.len()
    );

    let mut reader = Cursor::new(&bsdiff_data);

    // Parse Bsdiff40 header (32 bytes)
    let signature = reader.read_u64::<LittleEndian>()?;
    if signature != 0x3034464649445342 {
        // 'BSDIFF40'
        return Err(Error::invalid_format(format!(
            "Invalid BSDIFF40 signature: expected 0x3034464649445342, got 0x{signature:016X}"
        )));
    }

    let ctrl_block_size = reader.read_u64::<LittleEndian>()? as usize;
    let data_block_size = reader.read_u64::<LittleEndian>()? as usize;
    let new_file_size = reader.read_u64::<LittleEndian>()? as usize;

    // Verify new file size matches header
    if new_file_size != patch.header.size_after as usize {
        return Err(Error::invalid_format(format!(
            "BSD0 new file size mismatch: header says {}, bsdiff says {new_file_size}",
            patch.header.size_after
        )));
    }

    log::debug!(
        "BSD0 patch: {} -> {} bytes (ctrl: {}, data: {})",
        base_data.len(),
        new_file_size,
        ctrl_block_size,
        data_block_size
    );

    // Calculate block positions
    let ctrl_start = 32; // After bsdiff header
    let data_start = ctrl_start + ctrl_block_size;
    let extra_start = data_start + data_block_size;

    // Validate block sizes
    if extra_start > bsdiff_data.len() {
        return Err(Error::invalid_format(format!(
            "BSD0 patch data too small: need {extra_start} bytes, have {}",
            bsdiff_data.len()
        )));
    }

    // Extract data blocks
    let ctrl_block = &bsdiff_data[ctrl_start..data_start];
    let data_block = &bsdiff_data[data_start..extra_start];
    let extra_block = &bsdiff_data[extra_start..];

    // Number of control blocks (each is 12 bytes: 3x u32)
    let num_ctrl_blocks = ctrl_block_size / 12;

    // Allocate output buffer
    let mut new_data = vec![0u8; new_file_size];

    // Process control blocks
    let mut new_offset = 0usize;
    let mut old_offset = 0usize;
    let mut data_ptr = 0usize;
    let mut extra_ptr = 0usize;

    for i in 0..num_ctrl_blocks {
        let ctrl_offset = i * 12;
        if ctrl_offset + 12 > ctrl_block.len() {
            return Err(Error::invalid_format(
                "Control block extends beyond ctrl_block_size".to_string(),
            ));
        }

        // Read control block
        let mut ctrl_reader = Cursor::new(&ctrl_block[ctrl_offset..ctrl_offset + 12]);
        let add_data_length = ctrl_reader.read_u32::<LittleEndian>()? as usize;
        let mov_data_length = ctrl_reader.read_u32::<LittleEndian>()? as usize;
        let old_move_length_raw = ctrl_reader.read_u32::<LittleEndian>()?;

        // Step 1: Copy from data block and combine with old data
        if new_offset + add_data_length > new_file_size {
            return Err(Error::invalid_format(format!(
                "BSD0: add overflow at ctrl {i}: new_offset {new_offset} + add {add_data_length} > size {new_file_size}"
            )));
        }

        if data_ptr + add_data_length > data_block.len() {
            return Err(Error::invalid_format(format!(
                "BSD0: data block overflow at ctrl {i}: ptr {data_ptr} + len {add_data_length} > size {}",
                data_block.len()
            )));
        }

        // Copy from data block
        new_data[new_offset..new_offset + add_data_length]
            .copy_from_slice(&data_block[data_ptr..data_ptr + add_data_length]);
        data_ptr += add_data_length;

        // Combine with old data (wrapping addition)
        let combine_size = if old_offset + add_data_length >= base_data.len() {
            base_data.len().saturating_sub(old_offset)
        } else {
            add_data_length
        };

        for j in 0..combine_size {
            new_data[new_offset + j] =
                new_data[new_offset + j].wrapping_add(base_data[old_offset + j]);
        }

        new_offset += add_data_length;
        old_offset += add_data_length;

        // Step 2: Copy from extra block
        if new_offset + mov_data_length > new_file_size {
            return Err(Error::invalid_format(format!(
                "BSD0: mov overflow at ctrl {i}: new_offset {new_offset} + mov {mov_data_length} > size {new_file_size}"
            )));
        }

        if extra_ptr + mov_data_length > extra_block.len() {
            return Err(Error::invalid_format(format!(
                "BSD0: extra block overflow at ctrl {i}: ptr {extra_ptr} + len {mov_data_length} > size {}",
                extra_block.len()
            )));
        }

        new_data[new_offset..new_offset + mov_data_length]
            .copy_from_slice(&extra_block[extra_ptr..extra_ptr + mov_data_length]);
        extra_ptr += mov_data_length;
        new_offset += mov_data_length;

        // Step 3: Adjust old offset (signed!)
        let old_move_length = if old_move_length_raw & 0x80000000 != 0 {
            // Negative offset
            let neg_val = 0x80000000u32.wrapping_sub(old_move_length_raw);
            old_offset = old_offset.saturating_sub(neg_val as usize);
            0
        } else {
            old_move_length_raw as usize
        };

        old_offset += old_move_length;
    }

    // Verify final offset matches expected size
    if new_offset != new_file_size {
        return Err(Error::invalid_format(format!(
            "BSD0: final offset mismatch: got {new_offset}, expected {new_file_size}"
        )));
    }

    log::debug!("BSD0 patch applied successfully: {} bytes", new_data.len());

    Ok(new_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a test COPY patch
    fn create_copy_patch(base_size: u32, new_data: Vec<u8>) -> PatchFile {
        use md5::{Digest, Md5};

        // Calculate MD5 of dummy base data
        let base_data = vec![0u8; base_size as usize];
        let mut hasher = Md5::new();
        hasher.update(&base_data);
        let md5_before: [u8; 16] = hasher.finalize().into();

        // Calculate MD5 of new data
        let mut hasher = Md5::new();
        hasher.update(&new_data);
        let md5_after: [u8; 16] = hasher.finalize().into();

        // Build PTCH file
        let mut data = Vec::new();

        // PTCH Header
        data.extend_from_slice(&0x48435450u32.to_le_bytes());
        data.extend_from_slice(&(new_data.len() as u32).to_le_bytes());
        data.extend_from_slice(&base_size.to_le_bytes());
        data.extend_from_slice(&(new_data.len() as u32).to_le_bytes());

        // MD5 Block
        data.extend_from_slice(&0x5f35444du32.to_le_bytes());
        data.extend_from_slice(&40u32.to_le_bytes());
        data.extend_from_slice(&md5_before);
        data.extend_from_slice(&md5_after);

        // XFRM Block
        data.extend_from_slice(&0x4d524658u32.to_le_bytes());
        data.extend_from_slice(&(12 + new_data.len() as u32).to_le_bytes());
        data.extend_from_slice(&0x59504f43u32.to_le_bytes()); // COPY

        // Patch data
        data.extend_from_slice(&new_data);

        PatchFile::parse(&data).expect("Failed to create test patch")
    }

    #[test]
    fn test_apply_copy_patch_simple() {
        let base_data = vec![0u8; 100];
        let new_data = b"Hello, Warcraft!".to_vec();

        let patch = create_copy_patch(100, new_data.clone());
        let result = apply_patch(&patch, &base_data).expect("Failed to apply patch");

        assert_eq!(result, new_data);
    }

    #[test]
    fn test_copy_patch_size_increase() {
        let base_data = vec![0u8; 50];
        let new_data = vec![0x42u8; 200]; // Larger file

        let patch = create_copy_patch(50, new_data.clone());
        let result = apply_patch(&patch, &base_data).expect("Failed to apply patch");

        assert_eq!(result.len(), 200);
        assert_eq!(result, new_data);
    }

    #[test]
    fn test_copy_patch_size_decrease() {
        let base_data = vec![0u8; 200];
        let new_data = vec![0x42u8; 50]; // Smaller file

        let patch = create_copy_patch(200, new_data.clone());
        let result = apply_patch(&patch, &base_data).expect("Failed to apply patch");

        assert_eq!(result.len(), 50);
        assert_eq!(result, new_data);
    }

    #[test]
    fn test_copy_patch_wrong_base_size() {
        let base_data = vec![0u8; 100]; // Wrong size
        let new_data = b"Test".to_vec();

        let patch = create_copy_patch(50, new_data); // Expects 50 bytes
        let result = apply_patch(&patch, &base_data);

        assert!(result.is_err());
    }

    #[test]
    fn test_copy_patch_wrong_base_md5() {
        let base_data = vec![0x42u8; 100]; // Different content
        let new_data = b"Test".to_vec();

        let patch = create_copy_patch(100, new_data); // MD5 calculated for zeros
        let result = apply_patch(&patch, &base_data);

        assert!(result.is_err());
    }

    /// Simple RLE compress for test data
    /// RLE format: if byte has high bit set (0x80), copy (byte & 0x7F) + 1 literal bytes
    fn rle_compress_test(data: &[u8]) -> Vec<u8> {
        let mut compressed = Vec::new();

        // Add 4-byte size header (decompressed size)
        compressed.extend_from_slice(&(data.len() as u32).to_le_bytes());

        // Encode data in chunks of up to 128 bytes (0x7F + 1)
        for chunk in data.chunks(128) {
            let count = chunk.len() as u8;
            compressed.push(0x80 | (count - 1)); // High bit set + (count-1)
            compressed.extend_from_slice(chunk);
        }

        compressed
    }

    /// Helper to create a test BSD0 patch
    fn create_bsd0_patch(base_data: &[u8], new_data: &[u8], patch_data: Vec<u8>) -> PatchFile {
        use md5::{Digest, Md5};

        // Calculate MD5 hashes
        let mut hasher = Md5::new();
        hasher.update(base_data);
        let md5_before: [u8; 16] = hasher.finalize().into();

        let mut hasher = Md5::new();
        hasher.update(new_data);
        let md5_after: [u8; 16] = hasher.finalize().into();

        // RLE compress the patch data (BSD0 patches use RLE compression)
        let compressed_patch_data = rle_compress_test(&patch_data);

        // Build PTCH file with BSD0 type
        let mut data = Vec::new();

        // PTCH Header
        data.extend_from_slice(&0x48435450u32.to_le_bytes());
        data.extend_from_slice(&(patch_data.len() as u32).to_le_bytes()); // Decompressed size
        data.extend_from_slice(&(base_data.len() as u32).to_le_bytes());
        data.extend_from_slice(&(new_data.len() as u32).to_le_bytes());

        // MD5 Block
        data.extend_from_slice(&0x5f35444du32.to_le_bytes());
        data.extend_from_slice(&40u32.to_le_bytes());
        data.extend_from_slice(&md5_before);
        data.extend_from_slice(&md5_after);

        // XFRM Block
        data.extend_from_slice(&0x4d524658u32.to_le_bytes());
        data.extend_from_slice(&(12u32 + compressed_patch_data.len() as u32).to_le_bytes());
        data.extend_from_slice(&0x30445342u32.to_le_bytes()); // BSD0

        // Compressed patch data
        data.extend_from_slice(&compressed_patch_data);

        PatchFile::parse(&data).expect("Failed to create test BSD0 patch")
    }

    #[test]
    fn test_apply_bsd0_simple() {
        // Simple example: modify 3 bytes
        // Old: [0x10, 0x20, 0x30]
        // New: [0x15, 0x27, 0xFF]
        let old_data = vec![0x10u8, 0x20, 0x30];
        let new_data = vec![0x15u8, 0x27, 0xFF];

        // Build BSD0 patch manually
        let mut patch_data = Vec::new();

        // Bsdiff40 header
        patch_data.extend_from_slice(&0x3034464649445342u64.to_le_bytes()); // 'BSDIFF40'
        patch_data.extend_from_slice(&12u64.to_le_bytes()); // ctrl_block_size (1 block)
        patch_data.extend_from_slice(&2u64.to_le_bytes()); // data_block_size
        patch_data.extend_from_slice(&3u64.to_le_bytes()); // new_file_size

        // Control block: add=2, mov=1, old_move=1
        patch_data.extend_from_slice(&2u32.to_le_bytes()); // add_data_length
        patch_data.extend_from_slice(&1u32.to_le_bytes()); // mov_data_length
        patch_data.extend_from_slice(&1u32.to_le_bytes()); // old_move_length

        // Data block: [0x05, 0x07]
        // These will be added to old data: 0x05+0x10=0x15, 0x07+0x20=0x27
        patch_data.push(0x05);
        patch_data.push(0x07);

        // Extra block: [0xFF]
        patch_data.push(0xFF);

        let patch = create_bsd0_patch(&old_data, &new_data, patch_data);
        let result = apply_patch(&patch, &old_data).expect("Failed to apply BSD0 patch");

        assert_eq!(result, new_data);
    }

    #[test]
    fn test_bsd0_wrapping_addition() {
        // Test that addition wraps at 256
        // Old: [0xFF]
        // Diff: [0x02]
        // New: [0x01] (0xFF + 0x02 = 0x101 = 0x01 mod 256)
        let old_data = vec![0xFFu8];
        let new_data = vec![0x01u8];

        let mut patch_data = Vec::new();
        patch_data.extend_from_slice(&0x3034464649445342u64.to_le_bytes());
        patch_data.extend_from_slice(&12u64.to_le_bytes());
        patch_data.extend_from_slice(&1u64.to_le_bytes());
        patch_data.extend_from_slice(&1u64.to_le_bytes());

        // Control block
        patch_data.extend_from_slice(&1u32.to_le_bytes()); // add=1
        patch_data.extend_from_slice(&0u32.to_le_bytes()); // mov=0
        patch_data.extend_from_slice(&0u32.to_le_bytes()); // old_move=0

        // Data block
        patch_data.push(0x02); // Will wrap: 0xFF + 0x02 = 0x01

        let patch = create_bsd0_patch(&old_data, &new_data, patch_data);
        let result = apply_patch(&patch, &old_data).expect("Failed to apply BSD0 patch");

        assert_eq!(result, new_data);
        assert_eq!(result[0], 0x01);
    }

    #[test]
    fn test_bsd0_multiple_control_blocks() {
        // Test with 2 control blocks
        // Old: [0x10, 0x20, 0x30, 0x40]
        // New: [0x15, 0x27, 0xFF, 0xAA]
        let old_data = vec![0x10u8, 0x20, 0x30, 0x40];
        let new_data = vec![0x15u8, 0x27, 0xFF, 0xAA];

        let mut patch_data = Vec::new();
        patch_data.extend_from_slice(&0x3034464649445342u64.to_le_bytes());
        patch_data.extend_from_slice(&24u64.to_le_bytes()); // 2 control blocks
        patch_data.extend_from_slice(&2u64.to_le_bytes()); // data size
        patch_data.extend_from_slice(&4u64.to_le_bytes()); // new size

        // Control block 1
        patch_data.extend_from_slice(&2u32.to_le_bytes()); // add=2
        patch_data.extend_from_slice(&1u32.to_le_bytes()); // mov=1
        patch_data.extend_from_slice(&1u32.to_le_bytes()); // old_move=1

        // Control block 2
        patch_data.extend_from_slice(&0u32.to_le_bytes()); // add=0
        patch_data.extend_from_slice(&1u32.to_le_bytes()); // mov=1
        patch_data.extend_from_slice(&0u32.to_le_bytes()); // old_move=0

        // Data block
        patch_data.push(0x05); // 0x10 + 0x05 = 0x15
        patch_data.push(0x07); // 0x20 + 0x07 = 0x27

        // Extra block
        patch_data.push(0xFF);
        patch_data.push(0xAA);

        let patch = create_bsd0_patch(&old_data, &new_data, patch_data);
        let result = apply_patch(&patch, &old_data).expect("Failed to apply BSD0 patch");

        assert_eq!(result, new_data);
    }

    #[test]
    fn test_bsd0_invalid_signature() {
        let old_data = vec![0x10u8];
        let new_data = vec![0x11u8];

        let mut patch_data = Vec::new();
        patch_data.extend_from_slice(&0xDEADBEEFDEADBEEFu64.to_le_bytes()); // Bad signature
        patch_data.extend_from_slice(&12u64.to_le_bytes());
        patch_data.extend_from_slice(&1u64.to_le_bytes());
        patch_data.extend_from_slice(&1u64.to_le_bytes());
        patch_data.extend_from_slice(&1u32.to_le_bytes());
        patch_data.extend_from_slice(&0u32.to_le_bytes());
        patch_data.extend_from_slice(&0u32.to_le_bytes());
        patch_data.push(0x01);

        let patch = create_bsd0_patch(&old_data, &new_data, patch_data);
        let result = apply_patch(&patch, &old_data);

        assert!(result.is_err());
    }

    #[test]
    fn test_bsd0_size_mismatch() {
        // Wrong base size
        let old_data = vec![0x10u8, 0x20]; // 2 bytes
        let new_data = vec![0x11u8];

        let mut patch_data = Vec::new();
        patch_data.extend_from_slice(&0x3034464649445342u64.to_le_bytes());
        patch_data.extend_from_slice(&12u64.to_le_bytes());
        patch_data.extend_from_slice(&1u64.to_le_bytes());
        patch_data.extend_from_slice(&1u64.to_le_bytes()); // Says 1 byte result
        patch_data.extend_from_slice(&1u32.to_le_bytes());
        patch_data.extend_from_slice(&0u32.to_le_bytes());
        patch_data.extend_from_slice(&0u32.to_le_bytes());
        patch_data.push(0x01);

        // Patch expects 1-byte base, but we give 2 bytes
        let patch = create_bsd0_patch(&[0x10], &new_data, patch_data);
        let result = apply_patch(&patch, &old_data);

        assert!(result.is_err());
    }
}
