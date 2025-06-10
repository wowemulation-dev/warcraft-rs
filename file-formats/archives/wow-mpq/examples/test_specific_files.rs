//! Test specific files that showed errors in comprehensive test

use wow_mpq::Archive;

fn test_file(archive_path: &str, file_name: &str) {
    println!("\n=== Testing: {} ===", file_name);
    println!("Archive: {}", archive_path);
    
    match Archive::open(archive_path) {
        Ok(mut archive) => {
            println!("✓ Archive opened successfully");
            
            // Check if file exists
            match archive.list() {
                Ok(files) => {
                    let exists = files.iter().any(|f| f.name == file_name);
                    if exists {
                        println!("✓ File exists in archive");
                    } else {
                        println!("❌ File not found in archive!");
                        return;
                    }
                }
                Err(e) => {
                    println!("❌ Failed to list files: {}", e);
                    return;
                }
            }
            
            // Try to read the file
            match archive.read_file(file_name) {
                Ok(data) => {
                    println!("✓ Successfully read {} bytes", data.len());
                    
                    // Calculate simple checksum
                    let checksum: u32 = data.iter().map(|&b| b as u32).sum();
                    println!("Simple checksum: 0x{:08X}", checksum);
                    
                    // Show first few bytes
                    print!("First 16 bytes: ");
                    for i in 0..16.min(data.len()) {
                        print!("{:02X} ", data[i]);
                    }
                    println!();
                }
                Err(e) => {
                    println!("❌ Failed to read file: {}", e);
                    
                    // Try to get more details about the error
                    match e {
                        wow_mpq::Error::Compression(msg) => {
                            println!("  → Compression error: {}", msg);
                        }
                        wow_mpq::Error::Io(io_err) => {
                            println!("  → IO error: {}", io_err);
                        }
                        _ => {
                            println!("  → Error type: {:?}", e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to open archive: {}", e);
        }
    }
}

fn main() {
    println!("wow-mpq Specific File Test");
    println!("==========================");
    
    // Test the two files that showed errors
    test_file(
        "/home/danielsreichenbach/Downloads/wow/1.12.1/Data/interface.MPQ",
        "Interface\\Glues\\Credits\\TrollBanner4.blp"
    );
    
    test_file(
        "/home/danielsreichenbach/Downloads/wow/2.4.3/Data/common.MPQ",
        "Item\\TextureComponents\\LegLowerTexture\\Leather_Horde_A_01Blue_Pant_LL_M.blp"
    );
    
    // Test a known good file for comparison
    println!("\n=== Testing a known good file for comparison ===");
    test_file(
        "/home/danielsreichenbach/Downloads/wow/1.12.1/Data/fonts.MPQ",
        "Fonts\\MORPHEUS.TTF"
    );
}