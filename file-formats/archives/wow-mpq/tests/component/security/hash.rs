//! Integration tests for hash and crypto functionality

#[cfg(test)]
mod tests {
    use wow_mpq::crypto::{decrypt_block, encrypt_block};
    use wow_mpq::{hash_string, hash_type};

    #[test]
    fn test_hash_table_decryption_key() {
        // The hash table is encrypted with a key derived from "(hash table)"
        let key = hash_string("(hash table)", hash_type::FILE_KEY);

        // This is the actual key used by MPQ archives
        // You can verify this matches StormLib's implementation
        println!("Hash table key: 0x{key:08X}");

        // Create some dummy hash table data
        let mut data = vec![0x12345678u32, 0x9ABCDEF0, 0xFEDCBA98, 0x76543210];
        let original = data.clone();

        // Encrypt it
        encrypt_block(&mut data, key);
        assert_ne!(data, original, "Data should be encrypted");

        // Decrypt it
        decrypt_block(&mut data, key);
        assert_eq!(data, original, "Data should be decrypted back to original");
    }

    #[test]
    fn test_block_table_decryption_key() {
        // The block table is encrypted with a key derived from "(block table)"
        let key = hash_string("(block table)", hash_type::FILE_KEY);

        println!("Block table key: 0x{key:08X}");

        // Test encryption/decryption
        let mut data = vec![0xDEADBEEFu32; 16]; // Simulating block table entries
        let original = data.clone();

        encrypt_block(&mut data, key);
        decrypt_block(&mut data, key);
        assert_eq!(data, original);
    }

    #[test]
    fn test_file_encryption_key() {
        // Example: Calculate encryption key for a file
        let filename = "units\\human\\footman.mdx";
        let base_key = hash_string(filename, hash_type::FILE_KEY);

        // If the file has FLAG_FIX_KEY set, the key is modified:
        // key = (base_key + file_position) ^ file_size

        let file_position = 0x1000u32;
        let file_size = 0x2000u32;
        let fixed_key = (base_key.wrapping_add(file_position)) ^ file_size;

        println!("Base key for {filename}: 0x{base_key:08X}");
        println!("Fixed key: 0x{fixed_key:08X}");

        // Test that we can encrypt/decrypt with this key
        let mut data = vec![0u32; 256];
        for (i, val) in data.iter_mut().enumerate() {
            *val = i as u32;
        }
        let original = data.clone();

        encrypt_block(&mut data, fixed_key);
        decrypt_block(&mut data, fixed_key);
        assert_eq!(data, original);
    }

    #[test]
    fn test_special_file_keys() {
        // Test keys for special MPQ files
        let special_files = ["(listfile)", "(attributes)", "(signature)", "(user data)"];

        for filename in &special_files {
            let key = hash_string(filename, hash_type::FILE_KEY);
            println!("{filename} key: 0x{key:08X}");

            // Verify the key works for encryption
            let mut test_data = vec![0xCAFEBABEu32; 4];
            let original = test_data.clone();

            encrypt_block(&mut test_data, key);
            assert_ne!(test_data, original);

            decrypt_block(&mut test_data, key);
            assert_eq!(test_data, original);
        }
    }

    #[test]
    fn test_hash_for_empty_string() {
        // Edge case: empty filename
        let hash = hash_string("", hash_type::TABLE_OFFSET);

        // Empty string should still produce a valid hash
        assert_ne!(hash, 0);

        // The hash should be deterministic
        let hash2 = hash_string("", hash_type::TABLE_OFFSET);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_hash_for_long_paths() {
        // Test with a very long path
        let long_path = "folder1\\folder2\\folder3\\folder4\\folder5\\folder6\\folder7\\folder8\\very_long_filename_with_many_characters.txt";

        let hash_a = hash_string(long_path, hash_type::NAME_A);
        let hash_b = hash_string(long_path, hash_type::NAME_B);
        let hash_offset = hash_string(long_path, hash_type::TABLE_OFFSET);

        // All hashes should be non-zero
        assert_ne!(hash_a, 0);
        assert_ne!(hash_b, 0);
        assert_ne!(hash_offset, 0);

        // Hashes should be different for different types
        assert_ne!(hash_a, hash_b);
        assert_ne!(hash_a, hash_offset);
        assert_ne!(hash_b, hash_offset);
    }
}
