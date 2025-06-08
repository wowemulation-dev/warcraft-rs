//! Hash algorithms for MPQ file name hashing

use super::keys::{ASCII_TO_LOWER, ASCII_TO_UPPER, ENCRYPTION_TABLE};

/// Hash a string using the MPQ hash algorithm
pub fn hash_string(filename: &str, hash_type: u32) -> u32 {
    let mut seed1: u32 = 0x7FED7FED;
    let mut seed2: u32 = 0xEEEEEEEE;

    for &byte in filename.as_bytes() {
        // Get the next character and normalize it
        let mut ch = byte;

        // Convert path separators to backslash
        if ch == b'/' {
            ch = b'\\';
        }

        // Convert to uppercase using the table
        ch = ASCII_TO_UPPER[ch as usize];

        // Update the hash
        let table_idx = (hash_type * 0x100 + ch as u32) as usize;
        seed1 = ENCRYPTION_TABLE[table_idx] ^ (seed1.wrapping_add(seed2));
        seed2 = (ch as u32)
            .wrapping_add(seed1)
            .wrapping_add(seed2)
            .wrapping_add(seed2 << 5)
            .wrapping_add(3);
    }

    seed1
}

/// Jenkins hash function for HET tables
pub fn jenkins_hash(filename: &str) -> u64 {
    let mut hash: u64 = 0;

    for &byte in filename.as_bytes() {
        // Get the next character and normalize it
        let mut ch = byte;

        // Convert path separators to backslash
        if ch == b'/' {
            ch = b'\\';
        }

        // Convert to lowercase using the table
        ch = ASCII_TO_LOWER[ch as usize];

        // Jenkins one-at-a-time hash algorithm
        hash = hash.wrapping_add(ch as u64);
        hash = hash.wrapping_add(hash << 10);
        hash ^= hash >> 6;
    }

    hash = hash.wrapping_add(hash << 3);
    hash ^= hash >> 11;
    hash = hash.wrapping_add(hash << 15);

    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::types::hash_type;

    #[test]
    fn test_hash_string_test_vectors() {
        // Test vectors from the MPQ format documentation

        // Test 1: "(listfile)"
        assert_eq!(
            hash_string("(listfile)", hash_type::TABLE_OFFSET),
            0x5F3DE859
        );

        // Test 2: "(hash table)"
        assert_eq!(hash_string("(hash table)", hash_type::FILE_KEY), 0xC3AF3770);

        // Test 3: "(block table)"
        assert_eq!(
            hash_string("(block table)", hash_type::FILE_KEY),
            0xEC83B3A3
        );
    }

    #[test]
    fn test_path_separator_normalization() {
        // Both paths should produce the same hash
        let hash1 = hash_string("path/to/file.txt", hash_type::TABLE_OFFSET);
        let hash2 = hash_string("path\\to\\file.txt", hash_type::TABLE_OFFSET);
        assert_eq!(hash1, hash2);

        // Test from documentation
        assert_eq!(
            hash_string("path\\to\\file", hash_type::TABLE_OFFSET),
            hash_string("path/to/file", hash_type::TABLE_OFFSET)
        );
        assert_eq!(
            hash_string("path\\to\\file", hash_type::TABLE_OFFSET),
            0x534CC8EE
        );

        // Test interface path
        assert_eq!(
            hash_string("interface\\glue\\mainmenu.blp", hash_type::TABLE_OFFSET),
            hash_string("interface/glue/mainmenu.blp", hash_type::TABLE_OFFSET)
        );
        assert_eq!(
            hash_string("interface\\glue\\mainmenu.blp", hash_type::TABLE_OFFSET),
            0x2BBE7C09
        );
    }

    #[test]
    fn test_case_insensitivity() {
        // Different cases should produce the same hash
        let hash1 = hash_string("File.txt", hash_type::TABLE_OFFSET);
        let hash2 = hash_string("FILE.TXT", hash_type::TABLE_OFFSET);
        assert_eq!(hash1, hash2);

        // Test from documentation
        assert_eq!(
            hash_string("file.txt", hash_type::TABLE_OFFSET),
            hash_string("FILE.TXT", hash_type::TABLE_OFFSET)
        );
        assert_eq!(hash_string("file.txt", hash_type::TABLE_OFFSET), 0x3EA98D7A);

        assert_eq!(
            hash_string("path\\to\\FILE", hash_type::TABLE_OFFSET),
            hash_string("PATH\\TO\\file", hash_type::TABLE_OFFSET)
        );
        assert_eq!(
            hash_string("path\\to\\FILE", hash_type::TABLE_OFFSET),
            0x534CC8EE
        );
    }

    #[test]
    fn test_jenkins_hash() {
        // Test Jenkins hash for HET tables
        let hash = jenkins_hash("unit\\neutral\\chicken.mdx");
        assert_ne!(hash, 0); // Basic sanity check

        // Test case insensitivity - Jenkins uses lowercase conversion
        let hash1 = jenkins_hash("File.txt");
        let hash2 = jenkins_hash("FILE.TXT");
        assert_eq!(hash1, hash2, "Jenkins hash should be case-insensitive");

        // Test path normalization
        let hash1 = jenkins_hash("path/to/file");
        let hash2 = jenkins_hash("path\\to\\file");
        assert_eq!(hash1, hash2, "Path separators should be normalized");

        // Test that different files produce different hashes
        let hash1 = jenkins_hash("file1.txt");
        let hash2 = jenkins_hash("file2.txt");
        assert_ne!(
            hash1, hash2,
            "Different files should produce different hashes"
        );
    }

    #[test]
    fn test_hash_table_lookup_process() {
        // Example of how hash values are used in practice
        let filename = "(listfile)";

        // Calculate the three hash values needed for file lookup
        let hash_a = hash_string(filename, hash_type::NAME_A);
        let hash_b = hash_string(filename, hash_type::NAME_B);
        let hash_offset = hash_string(filename, hash_type::TABLE_OFFSET);

        // Example with hash table size of 0x1000 (4096)
        let hash_table_size = 0x1000u32;
        let index = hash_offset & (hash_table_size - 1);

        // Print for debugging
        println!("Hash A: 0x{:08X}", hash_a);
        println!("Hash B: 0x{:08X}", hash_b);
        println!("Hash offset: 0x{:08X}", hash_offset);
        println!("Table index: 0x{:04X}", index);

        // Verify we get consistent hash values
        assert_eq!(hash_offset, 0x5F3DE859);
        // The index should be the lower bits of the hash offset
        assert_eq!(index, 0x0859); // 0x5F3DE859 & 0xFFF = 0x0859

        // These hash values are used to find the file in the hash table
        assert_ne!(hash_a, 0); // Just verify they're non-zero
        assert_ne!(hash_b, 0);
    }

    #[test]
    fn test_encryption_key_calculation() {
        // Test file key calculation for encryption
        let filename = "(hash table)";
        let key = hash_string(filename, hash_type::FILE_KEY);

        // This key would be used to decrypt the hash table
        assert_eq!(key, 0xC3AF3770);

        // Test block table key
        let filename = "(block table)";
        let key = hash_string(filename, hash_type::FILE_KEY);
        assert_eq!(key, 0xEC83B3A3);
    }
}
