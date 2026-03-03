//! Common utilities and traits for MPQ tables

/// Helper function to decrypt table data
///
/// Only processes full 4-byte DWORD chunks, matching StormLib's `DecryptMpqBlock`
/// behavior (`dwLength >>= 2`). Trailing bytes not aligned to a DWORD boundary
/// are left untouched.
pub(crate) fn decrypt_table_data(data: &mut [u8], key: u32) {
    use crate::crypto::decrypt_block;

    if data.is_empty() || key == 0 {
        return;
    }

    // Only process full u32 chunks — trailing bytes are left as-is
    let full_len = (data.len() / 4) * 4;
    let chunks = &mut data[..full_len];

    // Convert full chunks to u32 array for decryption
    let mut u32_buffer: Vec<u32> = chunks
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    decrypt_block(&mut u32_buffer, key);

    // Convert back to bytes
    for (i, &val) in u32_buffer.iter().enumerate() {
        let bytes = val.to_le_bytes();
        chunks[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
    }
}
