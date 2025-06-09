//! Demonstrates the different hash algorithms used in MPQ archives
//!
//! This example shows how MPQ archives use different hash algorithms for
//! different table types and purposes.

use wow_mpq::crypto::{hash_string, hash_type, het_hash, jenkins_hash};

fn main() {
    println!("=== MPQ Hash Algorithms Demo ===\n");

    let test_files = vec![
        "Units\\Human\\Footman.mdx",
        "(listfile)",
        "(attributes)",
        "Interface\\Glue\\MainMenu.blp",
        "Sound\\Music\\ZoneMusic\\DMF_L70ETC01.mp3",
    ];

    for filename in &test_files {
        println!("File: {}", filename);
        println!("{}", "-".repeat(60));

        // Classic MPQ hash (used for hash tables in v1/v2)
        println!("MPQ Hash Algorithm:");
        let table_offset = hash_string(filename, hash_type::TABLE_OFFSET);
        let name_a = hash_string(filename, hash_type::NAME_A);
        let name_b = hash_string(filename, hash_type::NAME_B);
        let file_key = hash_string(filename, hash_type::FILE_KEY);

        println!("  TABLE_OFFSET: 0x{:08X}", table_offset);
        println!("  NAME_A:       0x{:08X}", name_a);
        println!("  NAME_B:       0x{:08X}", name_b);
        println!("  FILE_KEY:     0x{:08X} (for encryption)", file_key);

        // Jenkins one-at-a-time (used for BET tables in v3+)
        println!("\nJenkins one-at-a-time (BET tables):");
        let bet_hash = jenkins_hash(filename);
        println!("  BET hash:     0x{:016X}", bet_hash);

        // Jenkins hashlittle2 (used for HET tables in v3+)
        println!("\nJenkins hashlittle2 (HET tables):");
        for hash_bits in &[8u32, 48, 64] {
            let (file_hash, name_hash1) = het_hash(filename, *hash_bits);
            println!(
                "  {}-bit hash:  0x{:016X}, NameHash1: 0x{:02X}",
                hash_bits, file_hash, name_hash1
            );
        }

        println!();
    }

    // Demonstrate path normalization
    println!("=== Path Normalization ===");
    println!("All hash algorithms normalize paths:");
    println!();

    let variations = vec![
        "Units/Human/Footman.mdx",
        "Units\\Human\\Footman.mdx",
        "UNITS\\HUMAN\\FOOTMAN.MDX",
        "units\\human\\footman.mdx",
    ];

    println!("MPQ hash (TABLE_OFFSET) for path variations:");
    for path in &variations {
        let hash = hash_string(path, hash_type::TABLE_OFFSET);
        println!("  {:30} -> 0x{:08X}", path, hash);
    }

    println!("\nJenkins one-at-a-time for path variations:");
    for path in &variations {
        let hash = jenkins_hash(path);
        println!("  {:30} -> 0x{:016X}", path, hash);
    }

    println!("\nJenkins hashlittle2 (48-bit) for path variations:");
    for path in &variations {
        let (hash, _) = het_hash(path, 48);
        println!("  {:30} -> 0x{:016X}", path, hash);
    }

    // Show special files
    println!("\n=== Special Files ===");
    println!("MPQ archives have special metadata files with specific hash values:");
    println!();

    let special_files = vec!["(listfile)", "(attributes)", "(signature)", "(user data)"];

    for filename in &special_files {
        let hash_offset = hash_string(filename, hash_type::TABLE_OFFSET);
        let file_key = hash_string(filename, hash_type::FILE_KEY);
        let (het_hash_48, name_hash) = het_hash(filename, 48);

        println!("{}:", filename);
        println!("  MPQ TABLE_OFFSET: 0x{:08X}", hash_offset);
        println!("  MPQ FILE_KEY:     0x{:08X}", file_key);
        println!(
            "  HET 48-bit:       0x{:016X} (NameHash1: 0x{:02X})",
            het_hash_48, name_hash
        );
    }
}
