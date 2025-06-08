//! Integration tests for crypto functionality

use wow_mpq::crypto::{ENCRYPTION_TABLE, decrypt_block, decrypt_dword, encrypt_block};

#[test]
fn test_encryption_table_is_initialized() {
    // Access the table to ensure it's properly initialized
    assert_eq!(ENCRYPTION_TABLE.len(), 0x500);

    // Verify it's not all zeros
    let sum: u64 = ENCRYPTION_TABLE.iter().map(|&x| x as u64).sum();
    assert_ne!(sum, 0);
}

#[test]
fn test_cross_thread_access() {
    use std::thread;

    // Test that the encryption table can be accessed from multiple threads
    let handles: Vec<_> = (0..4)
        .map(|i| {
            thread::spawn(move || {
                // Each thread accesses different parts of the table
                let offset = i * 0x100;
                let value = ENCRYPTION_TABLE[offset];
                assert_ne!(value, 0);
                value
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_large_data_encryption() {
    // Test with larger data sets
    let mut large_data: Vec<u32> = (0..10000).collect();
    let original = large_data.clone();
    let key = 0xDEADBEEF;

    // Encrypt
    encrypt_block(&mut large_data, key);
    assert_ne!(large_data, original);

    // Decrypt
    decrypt_block(&mut large_data, key);
    assert_eq!(large_data, original);
}

#[test]
fn test_different_data_patterns() {
    let test_patterns = [
        vec![0u32; 8],                      // All zeros
        vec![0xFFFFFFFF; 8],                // All ones
        vec![0x55555555; 8],                // Alternating bits
        vec![0xAAAAAAAA; 8],                // Alternating bits (inverted)
        (0..8).map(|i| i as u32).collect(), // Sequential
    ];

    for (i, original) in test_patterns.iter().enumerate() {
        let key = 0x12345678 + i as u32;
        let mut data = original.clone();

        encrypt_block(&mut data, key);
        assert_ne!(&data, original, "Pattern {} should be encrypted", i);

        decrypt_block(&mut data, key);
        assert_eq!(&data, original, "Pattern {} should decrypt correctly", i);
    }
}

#[test]
fn test_empty_data() {
    let mut empty: Vec<u32> = vec![];
    let key = 0x12345678;

    // Should handle empty data gracefully
    encrypt_block(&mut empty, key);
    decrypt_block(&mut empty, key);

    assert_eq!(empty.len(), 0);
}

#[test]
fn test_single_dword_consistency() {
    // Test that decrypt_dword matches decrypt_block for single values
    let test_values = vec![0x12345678, 0xDEADBEEF, 0xCAFEBABE, 0x00000000, 0xFFFFFFFF];
    let key = 0xABCDEF00;

    for &value in &test_values {
        // Encrypt using block function
        let mut block_data = vec![value];
        encrypt_block(&mut block_data, key);
        let encrypted = block_data[0];

        // Decrypt using single dword function
        let decrypted_single = decrypt_dword(encrypted, key);

        // Decrypt using block function to verify
        decrypt_block(&mut block_data, key);
        let decrypted_block = block_data[0];

        assert_eq!(decrypted_single, value);
        assert_eq!(decrypted_block, value);
    }
}
