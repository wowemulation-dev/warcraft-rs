//! Test recompressing files that showed errors

use wow_mpq::{Archive, ArchiveBuilder, FormatVersion, ListfileOption, compression::CompressionMethod};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Recompression Test for Problematic Files");
    println!("========================================\n");
    
    // Files to test
    let test_files = vec![
        ("/home/danielsreichenbach/Downloads/wow/1.12.1/Data/interface.MPQ", 
         "Interface\\Glues\\Credits\\TrollBanner4.blp"),
        ("/home/danielsreichenbach/Downloads/wow/2.4.3/Data/common.MPQ",
         "Item\\TextureComponents\\LegLowerTexture\\Leather_Horde_A_01Blue_Pant_LL_M.blp"),
    ];
    
    for (archive_path, file_name) in test_files {
        println!("Testing: {}", file_name);
        println!("From: {}", archive_path);
        
        // Step 1: Read the original file
        let data = {
            let mut archive = Archive::open(archive_path)?;
            archive.read_file(file_name)?
        };
        
        println!("✓ Read {} bytes from original archive", data.len());
        
        // Step 2: Create a new archive with this file
        let test_archive = format!("test_recompress_{}.mpq", 
            file_name.split('\\').last().unwrap().replace(".blp", ""));
        
        println!("Creating test archive: {}", test_archive);
        
        let mut builder = ArchiveBuilder::new()
            .version(FormatVersion::V1)
            .listfile_option(ListfileOption::Generate);
        
        // Add with default compression (which should use Zlib)
        builder = builder.add_file_data(data.clone(), file_name);
        
        builder.build(&test_archive)?;
        println!("✓ Created archive with default compression");
        
        // Step 3: Try to read it back
        let mut new_archive = Archive::open(&test_archive)?;
        match new_archive.read_file(file_name) {
            Ok(read_data) => {
                if read_data == data {
                    println!("✓ File verified successfully after recompression!");
                } else {
                    println!("❌ Data mismatch after recompression");
                }
            }
            Err(e) => {
                println!("❌ Failed to read recompressed file: {}", e);
            }
        }
        
        // Step 4: Try different compression methods
        println!("\nTesting different compression methods:");
        
        // Try uncompressed
        let test_uncompressed = format!("test_uncompressed_{}.mpq",
            file_name.split('\\').last().unwrap().replace(".blp", ""));
        
        let mut builder = ArchiveBuilder::new()
            .version(FormatVersion::V1)
            .listfile_option(ListfileOption::Generate);
            
        builder = builder.add_file_data(data.clone(), file_name);
        
        builder.build(&test_uncompressed)?;
        
        let mut archive = Archive::open(&test_uncompressed)?;
        match archive.read_file(file_name) {
            Ok(_) => println!("  ✓ Uncompressed: Success"),
            Err(e) => println!("  ❌ Uncompressed: {}", e),
        }
        
        // Cleanup
        std::fs::remove_file(&test_archive).ok();
        std::fs::remove_file(&test_uncompressed).ok();
        
        println!("\n{}\n", "=".repeat(50));
    }
    
    Ok(())
}