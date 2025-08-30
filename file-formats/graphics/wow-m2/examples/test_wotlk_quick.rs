use std::fs;
use std::io::Cursor;
use std::path::Path;
use wow_m2::parse_m2;

fn main() -> anyhow::Result<()> {
    println!("Testing WotLK HumanMale model...");

    let path = Path::new(
        "/home/danielsreichenbach/Repos/github.com/wowemulation-dev/blender-wow-addon/sample_data/3.3.5a/m2/HumanMale.M2",
    );

    if !path.exists() {
        println!("❌ File not found: {}", path.display());
        return Ok(());
    }

    let data = fs::read(path)?;
    println!("File size: {} bytes", data.len());

    let mut cursor = Cursor::new(&data);
    match parse_m2(&mut cursor) {
        Ok(m2_format) => {
            let model = m2_format.model();
            println!("✅ Successfully parsed M2 file");
            println!("Version: {}", model.header.version);
            println!(
                "Model format: {}",
                if m2_format.is_legacy() {
                    "Legacy (embedded skins)"
                } else {
                    "Chunked (external skins)"
                }
            );

            // Try to check if it has embedded skins
            if model.has_embedded_skins() {
                println!("⚠️ Parser thinks this has embedded skins - this might be wrong for v264");
            } else {
                println!("✅ Parser correctly identifies no embedded skins");
            }

            // Try enhanced parsing
            match model.parse_all_data(&data) {
                Ok(enhanced_data) => {
                    println!("✅ Enhanced parsing successful");
                    println!("  Vertices: {}", enhanced_data.vertices.len());
                    println!("  Bones: {}", enhanced_data.bones.len());
                    println!("  Animations: {}", enhanced_data.animations.len());
                    println!("  Textures: {}", enhanced_data.textures.len());
                }
                Err(e) => {
                    println!("❌ Enhanced parsing failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to parse M2 file: {}", e);
        }
    }

    Ok(())
}
