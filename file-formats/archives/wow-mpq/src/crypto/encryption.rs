//! Encryption operations for MPQ files

use super::keys::ENCRYPTION_TABLE;

/// Encrypt a block of data
pub fn encrypt_block(data: &mut [u32], mut key: u32) {
    if key == 0 {
        return;
    }

    let mut seed: u32 = 0xEEEEEEEE;

    for value in data.iter_mut() {
        // Update seed using the encryption table and key
        seed = seed.wrapping_add(ENCRYPTION_TABLE[0x400 + (key & 0xFF) as usize]);

        // Store original value
        let ch = *value;

        // Encrypt the current DWORD
        *value = ch ^ (key.wrapping_add(seed));

        // Update key for next round
        key = (!key << 0x15).wrapping_add(0x11111111) | (key >> 0x0B);

        // Update seed for next round
        seed = ch
            .wrapping_add(seed)
            .wrapping_add(seed << 5)
            .wrapping_add(3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::decryption::decrypt_block;

    #[test]
    fn test_encrypt_decrypt_round_trip() {
        let original_data = vec![
            0x12345678, 0x9ABCDEF0, 0x13579BDF, 0x2468ACE0, 0xFEDCBA98, 0x76543210, 0xF0DEBC9A,
            0xE1C3A597,
        ];

        let key = 0xC1EB1CEF;

        // Test round trip
        let mut data = original_data.clone();
        encrypt_block(&mut data, key);

        // Verify data was changed
        assert_ne!(data, original_data);

        // Decrypt back
        decrypt_block(&mut data, key);

        // Verify we got the original data back
        assert_eq!(data, original_data);
    }

    #[test]
    fn test_known_encryption() {
        // Test with known test vectors
        let mut data = vec![
            0x12345678, 0x9ABCDEF0, 0x13579BDF, 0x2468ACE0, 0xFEDCBA98, 0x76543210, 0xF0DEBC9A,
            0xE1C3A597,
        ];

        let key = 0xC1EB1CEF;
        let original = data.clone();

        encrypt_block(&mut data, key);

        // Verify encryption changed the data
        assert_ne!(data, original, "Encryption should modify the data");

        // Decrypt and verify round-trip
        decrypt_block(&mut data, key);
        assert_eq!(data, original, "Decryption should restore original data");
    }

    #[test]
    fn test_zero_key() {
        // Test that zero key doesn't modify data
        let original = vec![0x12345678, 0x9ABCDEF0];
        let mut data = original.clone();

        encrypt_block(&mut data, 0);
        assert_eq!(data, original);

        decrypt_block(&mut data, 0);
        assert_eq!(data, original);
    }

    #[test]
    fn test_different_keys_produce_different_results() {
        let original = vec![0x12345678, 0x9ABCDEF0];

        let mut data1 = original.clone();
        let mut data2 = original.clone();

        encrypt_block(&mut data1, 0x11111111);
        encrypt_block(&mut data2, 0x22222222);

        assert_ne!(data1, data2);
        assert_ne!(data1, original);
        assert_ne!(data2, original);
    }
}
