//! Decryption operations for MPQ files

use super::keys::ENCRYPTION_TABLE;

/// Decrypt a block of data
pub fn decrypt_block(data: &mut [u32], mut key: u32) {
    if key == 0 {
        return;
    }

    let mut seed: u32 = 0xEEEEEEEE;

    for value in data.iter_mut() {
        // Update seed using the encryption table and key
        seed = seed.wrapping_add(ENCRYPTION_TABLE[0x400 + (key & 0xFF) as usize]);

        // Decrypt the current DWORD
        let ch = *value ^ (key.wrapping_add(seed));
        *value = ch;

        // Update key for next round
        key = (!key << 0x15).wrapping_add(0x11111111) | (key >> 0x0B);

        // Update seed for next round
        seed = ch
            .wrapping_add(seed)
            .wrapping_add(seed << 5)
            .wrapping_add(3);
    }
}

/// Decrypt a single DWORD value
pub fn decrypt_dword(value: u32, key: u32) -> u32 {
    if key == 0 {
        return value;
    }

    let mut seed: u32 = 0xEEEEEEEE;
    seed = seed.wrapping_add(ENCRYPTION_TABLE[0x400 + (key & 0xFF) as usize]);

    value ^ (key.wrapping_add(seed))
}
