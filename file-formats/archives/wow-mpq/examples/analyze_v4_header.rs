use std::env;
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <v4_archive.mpq>", args[0]);
        std::process::exit(1);
    }

    let archive_path = &args[1];
    println!("=== V4 Header Analysis ===");
    println!("Archive: {}", archive_path);

    let mut file = File::open(archive_path)?;

    // Read raw header bytes
    let mut header_bytes = vec![0u8; 512]; // V4 headers can be quite large
    file.read_exact(&mut header_bytes[..208])?; // Read minimum V4 header size

    // Parse header fields manually to avoid alignment issues
    let signature = &header_bytes[0..4];
    let header_size = u32::from_le_bytes([
        header_bytes[4],
        header_bytes[5],
        header_bytes[6],
        header_bytes[7],
    ]);
    let archive_size = u32::from_le_bytes([
        header_bytes[8],
        header_bytes[9],
        header_bytes[10],
        header_bytes[11],
    ]);
    let format_version = u16::from_le_bytes([header_bytes[12], header_bytes[13]]);
    let sector_size_shift = u16::from_le_bytes([header_bytes[14], header_bytes[15]]);
    let hash_table_offset = u32::from_le_bytes([
        header_bytes[16],
        header_bytes[17],
        header_bytes[18],
        header_bytes[19],
    ]);
    let block_table_offset = u32::from_le_bytes([
        header_bytes[20],
        header_bytes[21],
        header_bytes[22],
        header_bytes[23],
    ]);
    let hash_table_entries = u32::from_le_bytes([
        header_bytes[24],
        header_bytes[25],
        header_bytes[26],
        header_bytes[27],
    ]);
    let block_table_entries = u32::from_le_bytes([
        header_bytes[28],
        header_bytes[29],
        header_bytes[30],
        header_bytes[31],
    ]);

    println!("\n=== Basic Header Fields ===");
    println!(
        "Signature: {:?}",
        std::str::from_utf8(signature).unwrap_or("Invalid")
    );
    println!("Header size: {} (0x{:X})", header_size, header_size);
    println!("Archive size: {} (0x{:X})", archive_size, archive_size);
    println!(
        "Format version: {} (0x{:X})",
        format_version, format_version
    );
    println!("Sector size shift: {}", sector_size_shift);
    println!(
        "Hash table offset: {} (0x{:X})",
        hash_table_offset, hash_table_offset
    );
    println!(
        "Block table offset: {} (0x{:X})",
        block_table_offset, block_table_offset
    );
    println!("Hash table entries: {}", hash_table_entries);
    println!("Block table entries: {}", block_table_entries);

    if format_version >= 2 {
        let hi_block_table_offset = u64::from_le_bytes([
            header_bytes[32],
            header_bytes[33],
            header_bytes[34],
            header_bytes[35],
            header_bytes[36],
            header_bytes[37],
            header_bytes[38],
            header_bytes[39],
        ]);
        let hash_table_offset_high = u16::from_le_bytes([header_bytes[40], header_bytes[41]]);
        let block_table_offset_high = u16::from_le_bytes([header_bytes[42], header_bytes[43]]);

        println!("\n=== V2+ Extended Fields ===");
        println!(
            "Hi-block table offset: {} (0x{:X})",
            hi_block_table_offset, hi_block_table_offset
        );
        println!(
            "Hash table offset high: {} (0x{:X})",
            hash_table_offset_high, hash_table_offset_high
        );
        println!(
            "Block table offset high: {} (0x{:X})",
            block_table_offset_high, block_table_offset_high
        );
    }

    if format_version >= 3 {
        let archive_size_64 = u64::from_le_bytes([
            header_bytes[44],
            header_bytes[45],
            header_bytes[46],
            header_bytes[47],
            header_bytes[48],
            header_bytes[49],
            header_bytes[50],
            header_bytes[51],
        ]);
        let bet_table_offset = u64::from_le_bytes([
            header_bytes[52],
            header_bytes[53],
            header_bytes[54],
            header_bytes[55],
            header_bytes[56],
            header_bytes[57],
            header_bytes[58],
            header_bytes[59],
        ]);
        let het_table_offset = u64::from_le_bytes([
            header_bytes[60],
            header_bytes[61],
            header_bytes[62],
            header_bytes[63],
            header_bytes[64],
            header_bytes[65],
            header_bytes[66],
            header_bytes[67],
        ]);

        println!("\n=== V3+ Extended Fields ===");
        println!(
            "Archive size 64: {} (0x{:X})",
            archive_size_64, archive_size_64
        );
        println!(
            "BET table offset: {} (0x{:X})",
            bet_table_offset, bet_table_offset
        );
        println!(
            "HET table offset: {} (0x{:X})",
            het_table_offset, het_table_offset
        );
    }

    if format_version >= 3 && header_size >= 208 {
        let hash_table_size_64 = u64::from_le_bytes([
            header_bytes[68],
            header_bytes[69],
            header_bytes[70],
            header_bytes[71],
            header_bytes[72],
            header_bytes[73],
            header_bytes[74],
            header_bytes[75],
        ]);
        let block_table_size_64 = u64::from_le_bytes([
            header_bytes[76],
            header_bytes[77],
            header_bytes[78],
            header_bytes[79],
            header_bytes[80],
            header_bytes[81],
            header_bytes[82],
            header_bytes[83],
        ]);
        let hi_block_table_size_64 = u64::from_le_bytes([
            header_bytes[84],
            header_bytes[85],
            header_bytes[86],
            header_bytes[87],
            header_bytes[88],
            header_bytes[89],
            header_bytes[90],
            header_bytes[91],
        ]);
        let het_table_size_64 = u64::from_le_bytes([
            header_bytes[92],
            header_bytes[93],
            header_bytes[94],
            header_bytes[95],
            header_bytes[96],
            header_bytes[97],
            header_bytes[98],
            header_bytes[99],
        ]);
        let bet_table_size_64 = u64::from_le_bytes([
            header_bytes[100],
            header_bytes[101],
            header_bytes[102],
            header_bytes[103],
            header_bytes[104],
            header_bytes[105],
            header_bytes[106],
            header_bytes[107],
        ]);
        let raw_chunk_size = u32::from_le_bytes([
            header_bytes[108],
            header_bytes[109],
            header_bytes[110],
            header_bytes[111],
        ]);

        println!("\n=== V4+ Size Fields (CRITICAL) ===");
        println!(
            "Hash table size 64: {} (0x{:X})",
            hash_table_size_64, hash_table_size_64
        );
        println!(
            "Block table size 64: {} (0x{:X})",
            block_table_size_64, block_table_size_64
        );
        println!(
            "Hi-block table size 64: {} (0x{:X})",
            hi_block_table_size_64, hi_block_table_size_64
        );
        println!(
            "HET table size 64: {} (0x{:X})",
            het_table_size_64, het_table_size_64
        );
        println!(
            "BET table size 64: {} (0x{:X})",
            bet_table_size_64, bet_table_size_64
        );
        println!(
            "Raw chunk size: {} (0x{:X})",
            raw_chunk_size, raw_chunk_size
        );

        // Calculate expected sizes for comparison
        let expected_hash_table_size = hash_table_entries as u64 * 16; // Each hash entry is 16 bytes
        let expected_block_table_size = block_table_entries as u64 * 16; // Each block entry is 16 bytes

        println!("\n=== Size Validation ===");
        println!(
            "Expected hash table size: {} (0x{:X})",
            expected_hash_table_size, expected_hash_table_size
        );
        println!(
            "Actual hash table size 64: {} (0x{:X})",
            hash_table_size_64, hash_table_size_64
        );
        println!(
            "Hash table size match: {}",
            expected_hash_table_size == hash_table_size_64
        );

        println!(
            "Expected block table size: {} (0x{:X})",
            expected_block_table_size, expected_block_table_size
        );
        println!(
            "Actual block table size 64: {} (0x{:X})",
            block_table_size_64, block_table_size_64
        );
        println!(
            "Block table size match: {}",
            expected_block_table_size == block_table_size_64
        );

        // Check if any size fields are suspiciously large (potential causes of malloc crash)
        println!("\n=== Malloc Crash Analysis ===");
        let large_threshold = 100 * 1024 * 1024; // 100MB threshold
        if hash_table_size_64 > large_threshold {
            println!(
                "WARNING: Hash table size is suspiciously large: {} bytes",
                hash_table_size_64
            );
        }
        if block_table_size_64 > large_threshold {
            println!(
                "WARNING: Block table size is suspiciously large: {} bytes",
                block_table_size_64
            );
        }
        if het_table_size_64 > large_threshold {
            println!(
                "WARNING: HET table size is suspiciously large: {} bytes",
                het_table_size_64
            );
        }
        if bet_table_size_64 > large_threshold {
            println!(
                "WARNING: BET table size is suspiciously large: {} bytes",
                bet_table_size_64
            );
        }

        // MD5 hashes start at offset 112
        println!("\n=== V4 MD5 Hashes ===");
        let block_table_md5 = &header_bytes[112..128];
        let hash_table_md5 = &header_bytes[128..144];
        let hi_block_table_md5 = &header_bytes[144..160];
        let bet_table_md5 = &header_bytes[160..176];
        let het_table_md5 = &header_bytes[176..192];
        let mpq_header_md5 = &header_bytes[192..208];

        print!("Block table MD5: ");
        for &byte in block_table_md5 {
            print!("{:02x}", byte);
        }
        println!();

        print!("Hash table MD5: ");
        for &byte in hash_table_md5 {
            print!("{:02x}", byte);
        }
        println!();

        print!("Hi-block table MD5: ");
        for &byte in hi_block_table_md5 {
            print!("{:02x}", byte);
        }
        println!();

        print!("BET table MD5: ");
        for &byte in bet_table_md5 {
            print!("{:02x}", byte);
        }
        println!();

        print!("HET table MD5: ");
        for &byte in het_table_md5 {
            print!("{:02x}", byte);
        }
        println!();

        print!("MPQ header MD5: ");
        for &byte in mpq_header_md5 {
            print!("{:02x}", byte);
        }
        println!();
    }

    Ok(())
}
