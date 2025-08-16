//! Common utilities and traits for MPQ tables

/// Helper function to decrypt table data
pub(crate) fn decrypt_table_data(data: &mut [u8], key: u32) {
    use crate::crypto::{decrypt_block, decrypt_dword};

    if data.is_empty() || key == 0 {
        return;
    }

    // Process full u32 chunks
    let (chunks, remainder) = data.split_at_mut((data.len() / 4) * 4);

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

    // Handle remaining bytes (same way as encryption)
    if !remainder.is_empty() {
        let mut last_dword = [0u8; 4];
        last_dword[..remainder.len()].copy_from_slice(remainder);

        let encrypted_u32 = u32::from_le_bytes(last_dword);
        let decrypted_u32 =
            decrypt_dword(encrypted_u32, key.wrapping_add((chunks.len() / 4) as u32));

        let decrypted_bytes = decrypted_u32.to_le_bytes();
        remainder.copy_from_slice(&decrypted_bytes[..remainder.len()]);
    }
}
