//! Comprehensive analysis tool for embedded skin data in M2 models
//!
//! This tool analyzes the skin data structure across different WoW versions

use std::fs;
use std::io::{Cursor, Read, Seek, SeekFrom};
use wow_m2::{M2Model, parse_m2};

fn analyze_views_array(m2_data: &[u8], model: &M2Model) {
    println!("\n=== Views Array Analysis ===");
    println!("Version: {}", model.header.version);
    println!("Views count: {}", model.header.views.count);
    println!("Views offset: 0x{:08X}", model.header.views.offset);

    if model.header.views.count == 0 {
        println!("No views/skins defined");
        return;
    }

    // Read the views array data
    let mut cursor = Cursor::new(m2_data);
    if cursor
        .seek(SeekFrom::Start(model.header.views.offset as u64))
        .is_ok()
    {
        println!("\nViews array content:");
        for i in 0..model.header.views.count.min(10) {
            let mut bytes = [0u8; 4];
            if cursor.read_exact(&mut bytes).is_ok() {
                let value = u32::from_le_bytes(bytes);
                println!("  View[{}]: 0x{:08X} ({})", i, value, value);

                // For pre-WotLK, try to interpret this as a skin offset
                if model.header.version <= 260 && value < m2_data.len() as u32 {
                    // Check what's at this offset
                    let mut test_cursor = Cursor::new(m2_data);
                    if test_cursor.seek(SeekFrom::Start(value as u64)).is_ok() {
                        let mut test_bytes = [0u8; 8];
                        if test_cursor.read_exact(&mut test_bytes).is_ok() {
                            let count = u32::from_le_bytes([
                                test_bytes[0],
                                test_bytes[1],
                                test_bytes[2],
                                test_bytes[3],
                            ]);
                            let offset = u32::from_le_bytes([
                                test_bytes[4],
                                test_bytes[5],
                                test_bytes[6],
                                test_bytes[7],
                            ]);
                            println!(
                                "    -> At 0x{:08X}: count={}, offset=0x{:08X}",
                                value, count, offset
                            );
                        }
                    }
                }
            }
        }
    }
}

fn analyze_skin_data_locations(m2_data: &[u8], _model: &M2Model) {
    println!("\n=== Searching for Skin Data Patterns ===");

    // Look for typical skin data patterns (M2Array structures)
    // Skin data typically starts with indices array (count, offset)

    for offset in (0..m2_data.len().min(0x100000)).step_by(4) {
        if offset + 20 >= m2_data.len() {
            break;
        }

        let indices_count = u32::from_le_bytes([
            m2_data[offset],
            m2_data[offset + 1],
            m2_data[offset + 2],
            m2_data[offset + 3],
        ]);

        let indices_offset = u32::from_le_bytes([
            m2_data[offset + 4],
            m2_data[offset + 5],
            m2_data[offset + 6],
            m2_data[offset + 7],
        ]);

        // Check if this looks like a valid skin header
        // Indices count is typically > 0 and < 100000
        // Indices offset should be valid
        if indices_count > 0
            && indices_count < 100000
            && indices_offset > 0
            && (indices_offset as usize) < m2_data.len()
            && (indices_offset as usize + indices_count as usize * 2) < m2_data.len()
        {
            // Check next arrays (triangles)
            let triangles_count = u32::from_le_bytes([
                m2_data[offset + 8],
                m2_data[offset + 9],
                m2_data[offset + 10],
                m2_data[offset + 11],
            ]);

            if triangles_count > 0 && triangles_count < 100000 {
                println!("Potential skin header at 0x{:08X}:", offset);
                println!(
                    "  Indices: count={}, offset=0x{:08X}",
                    indices_count, indices_offset
                );
                println!("  Triangles: count={}", triangles_count);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <m2_file>", args[0]);
        std::process::exit(1);
    }

    let m2_path = &args[1];
    println!("Analyzing: {}", m2_path);

    // Read the complete M2 file data
    let m2_data = fs::read(m2_path)?;
    println!("File size: {} bytes (0x{:X})", m2_data.len(), m2_data.len());

    // Parse the M2 model
    let mut cursor = Cursor::new(&m2_data);
    let m2_format = parse_m2(&mut cursor)?;
    let model = m2_format.model();

    // Basic info
    println!("\n=== Basic Information ===");
    println!("Model name: {:?}", model.name);
    println!("Version: {}", model.header.version);
    println!("Is pre-WotLK: {}", model.header.version <= 260);
    println!("Has embedded skins: {}", model.has_embedded_skins());

    // Analyze views array
    analyze_views_array(&m2_data, model);

    // Search for skin data patterns
    if model.header.version <= 260 {
        analyze_skin_data_locations(&m2_data, model);
    }

    // Try to parse embedded skins with detailed error reporting
    if model.has_embedded_skins() {
        println!("\n=== Embedded Skin Parsing Attempts ===");
        for i in 0..model.header.views.count.min(4) {
            println!("\nAttempting to parse skin {}:", i);
            match model.parse_embedded_skin(&m2_data, i as usize) {
                Ok(skin) => {
                    println!("  SUCCESS!");
                    println!("  Indices: {}", skin.indices().len());
                    println!("  Triangles: {}", skin.triangles().len());
                    println!("  Submeshes: {}", skin.submeshes().len());
                }
                Err(e) => {
                    println!("  FAILED: {}", e);

                    // Try to manually read the offset
                    let offset_position = model.header.views.offset as usize + (i as usize * 4);
                    if offset_position + 4 <= m2_data.len() {
                        let skin_offset = u32::from_le_bytes([
                            m2_data[offset_position],
                            m2_data[offset_position + 1],
                            m2_data[offset_position + 2],
                            m2_data[offset_position + 3],
                        ]);
                        println!("  Raw offset value: 0x{:08X}", skin_offset);

                        // Check if it's valid
                        if skin_offset as usize >= m2_data.len() {
                            println!("  Offset exceeds file size!");
                        }
                    }
                }
            }
        }
    }

    // Model statistics
    println!("\n=== Model Statistics ===");
    println!("Vertices: {}", model.vertices.len());
    println!("Bones: {}", model.bones.len());
    println!("Materials: {}", model.materials.len());
    println!("Textures: {}", model.textures.len());
    println!("Animations: {}", model.animations.len());

    Ok(())
}
