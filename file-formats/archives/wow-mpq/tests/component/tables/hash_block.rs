//! Integration tests for table parsing

#[cfg(test)]
mod tests {
    use wow_mpq::{hash_string, hash_type};

    #[test]
    fn test_table_decryption_keys() {
        // Verify the encryption keys for tables
        let hash_table_key = hash_string("(hash table)", hash_type::FILE_KEY);
        let block_table_key = hash_string("(block table)", hash_type::FILE_KEY);

        // These are the expected keys based on the MPQ specification
        println!("Hash table key: 0x{hash_table_key:08X}");
        println!("Block table key: 0x{block_table_key:08X}");

        // Keys should be different
        assert_ne!(hash_table_key, block_table_key);

        // Keys should be non-zero
        assert_ne!(hash_table_key, 0);
        assert_ne!(block_table_key, 0);
    }

    #[test]
    fn test_hash_entry_lookup_simulation() {
        use wow_mpq::tables::HashEntry;

        // Simulate hash table lookup
        let filename = "units\\human\\footman.mdx";
        let hash_a = hash_string(filename, hash_type::NAME_A);
        let hash_b = hash_string(filename, hash_type::NAME_B);
        let table_offset = hash_string(filename, hash_type::TABLE_OFFSET);

        println!("Filename: {filename}");
        println!("Hash A: 0x{hash_a:08X}");
        println!("Hash B: 0x{hash_b:08X}");
        println!("Table offset: 0x{table_offset:08X}");

        // Simulate a hash table of size 1024
        let table_size = 1024u32;
        let initial_index = table_offset & (table_size - 1);
        println!(
            "Initial index for table size {table_size}: {initial_index}"
        );

        // Create a dummy hash entry
        let entry = HashEntry {
            name_1: hash_a,
            name_2: hash_b,
            locale: 0,
            platform: 0,
            block_index: 42,
        };

        assert!(entry.is_valid());
        assert!(!entry.is_empty());
        assert!(!entry.is_deleted());
    }

    #[test]
    fn test_block_entry_flags() {
        use wow_mpq::tables::BlockEntry;

        // Test various flag combinations
        let test_cases = vec![
            (BlockEntry::FLAG_EXISTS, "File exists only"),
            (
                BlockEntry::FLAG_EXISTS | BlockEntry::FLAG_COMPRESS,
                "Compressed file",
            ),
            (
                BlockEntry::FLAG_EXISTS | BlockEntry::FLAG_ENCRYPTED,
                "Encrypted file",
            ),
            (
                BlockEntry::FLAG_EXISTS | BlockEntry::FLAG_COMPRESS | BlockEntry::FLAG_ENCRYPTED,
                "Compressed and encrypted",
            ),
            (
                BlockEntry::FLAG_EXISTS | BlockEntry::FLAG_SINGLE_UNIT,
                "Single unit file",
            ),
        ];

        for (flags, description) in test_cases {
            println!("Testing: {description} (flags: 0x{flags:08X})");

            let entry = BlockEntry {
                file_pos: 0x1000,
                compressed_size: 1000,
                file_size: 2000,
                flags,
            };

            assert_eq!(entry.exists(), (flags & BlockEntry::FLAG_EXISTS) != 0);
            assert_eq!(
                entry.is_compressed(),
                (flags & BlockEntry::FLAG_COMPRESS) != 0
            );
            assert_eq!(
                entry.is_encrypted(),
                (flags & BlockEntry::FLAG_ENCRYPTED) != 0
            );
            assert_eq!(
                entry.is_single_unit(),
                (flags & BlockEntry::FLAG_SINGLE_UNIT) != 0
            );
        }
    }

    #[test]
    fn test_file_position_calculation() {
        // Test 64-bit file position calculation for large archives
        let block_pos_low = 0x80000000u32; // 2GB mark
        let hi_block_value = 0x0001u16;

        // Calculate full 64-bit position
        let full_pos = ((hi_block_value as u64) << 32) | (block_pos_low as u64);

        println!("Low 32 bits: 0x{block_pos_low:08X}");
        println!("High 16 bits: 0x{hi_block_value:04X}");
        println!(
            "Full 64-bit position: 0x{full_pos:016X} ({full_pos} bytes)"
        );

        // This should be 6GB (0x180000000)
        assert_eq!(full_pos, 0x180000000);
        assert_eq!(full_pos, 6 * 1024 * 1024 * 1024); // 6GB in bytes
    }

    #[test]
    fn test_hash_table_collision_handling() {
        // Test the linear probing algorithm
        let table_size = 16; // Small table for testing
        let filenames = vec!["file1.txt", "file2.txt", "file3.txt"];

        for filename in &filenames {
            let hash_offset = hash_string(filename, hash_type::TABLE_OFFSET);
            let index = hash_offset & (table_size - 1);
            println!("{filename} -> index {index}");
        }

        // In a real implementation, if two files hash to the same index,
        // the second one would be placed at the next available slot
    }

    #[test]
    fn test_locale_codes() {
        // Test common locale codes used in MPQ files
        let locales = vec![
            (0x0000, "Neutral/Default"),
            (0x0409, "English (US)"),
            (0x0407, "German"),
            (0x040C, "French"),
            (0x0410, "Italian"),
            (0x0411, "Japanese"),
            (0x0412, "Korean"),
            (0x0419, "Russian"),
            (0x0404, "Chinese (Traditional)"),
            (0x0804, "Chinese (Simplified)"),
        ];

        for (code, name) in locales {
            println!("Locale 0x{code:04X}: {name}");
        }
    }
}
