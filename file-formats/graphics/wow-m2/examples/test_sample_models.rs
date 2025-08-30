use std::fs;
use std::io::Cursor;
use std::path::Path;
use wow_m2::parse_m2;

fn test_model(path: &Path) -> anyhow::Result<()> {
    println!("\n═══════════════════════════════════════════════════════");
    println!("Testing: {}", path.display());
    println!("═══════════════════════════════════════════════════════");

    // Read the M2 file
    let data = fs::read(path)?;
    println!("File size: {} bytes", data.len());

    // Parse the M2 model
    let mut cursor = Cursor::new(&data);
    let m2_format = parse_m2(&mut cursor)?;
    let model = m2_format.model();

    // Extract all data
    println!("\n📊 Extracting comprehensive model data...");
    let enhanced_data = model.parse_all_data(&data)?;

    // Display comprehensive information
    model.display_info(&enhanced_data);

    // Additional validation
    println!("\n✅ Validation Results:");

    // Check vertex data
    if !enhanced_data.vertices.is_empty() {
        println!("  ✓ Vertices parsed: {}", enhanced_data.vertices.len());

        // Check bone weights
        let weighted_verts = enhanced_data
            .vertices
            .iter()
            .filter(|v| v.bone_weights[0] > 0)
            .count();
        println!(
            "  ✓ Weighted vertices: {}/{}",
            weighted_verts,
            enhanced_data.vertices.len()
        );
    }

    // Check bone hierarchy
    if !enhanced_data.bones.is_empty() {
        println!("  ✓ Bones parsed: {}", enhanced_data.bones.len());

        let root_bones = enhanced_data
            .bones
            .iter()
            .filter(|b| b.bone.parent_bone == -1)
            .count();
        println!("  ✓ Root bones: {}", root_bones);
    }

    // Check animations
    if !enhanced_data.animations.is_empty() {
        println!("  ✓ Animations parsed: {}", enhanced_data.animations.len());

        let looping_anims = enhanced_data
            .animations
            .iter()
            .filter(|a| a.is_looping)
            .count();
        println!(
            "  ✓ Looping animations: {}/{}",
            looping_anims,
            enhanced_data.animations.len()
        );
    }

    // Check textures
    if !enhanced_data.textures.is_empty() {
        println!("  ✓ Textures parsed: {}", enhanced_data.textures.len());
    }

    // Check embedded skins
    if !enhanced_data.embedded_skins.is_empty() {
        println!("  ✓ Embedded skin data found:");
        for (i, skin) in enhanced_data.embedded_skins.iter().enumerate() {
            let indices_count = skin.indices().len();
            let triangles_count = skin.triangles().len();
            let submeshes_count = skin.submeshes().len();
            println!(
                "    Skin {}: {} indices, {} triangles, {} submeshes",
                i, indices_count, triangles_count, submeshes_count
            );
        }
    }

    println!("\n✅ Model successfully parsed and validated!");

    Ok(())
}

fn main() -> anyhow::Result<()> {
    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║     M2 Enhanced Parser - Sample Models Test          ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    // Define sample model paths
    let sample_dir = Path::new(
        "/home/danielsreichenbach/Repos/github.com/wowemulation-dev/blender-wow-addon/sample_data/1.12.1/m2",
    );

    let models = ["Rabbit.m2", "HumanMale.m2", "OrcMale.m2"];

    let mut success_count = 0;
    let mut failure_count = 0;

    for model_name in &models {
        let model_path = sample_dir.join(model_name);

        if !model_path.exists() {
            println!("\n⚠️  Model not found: {}", model_path.display());
            continue;
        }

        match test_model(&model_path) {
            Ok(_) => success_count += 1,
            Err(e) => {
                println!("\n❌ Error testing {}: {}", model_name, e);
                failure_count += 1;
            }
        }
    }

    // Summary
    println!("\n╔═══════════════════════════════════════════════════════╗");
    println!("║                    Test Summary                       ║");
    println!("╚═══════════════════════════════════════════════════════╝");
    println!("  Successful: {}", success_count);
    println!("  Failed: {}", failure_count);
    println!("  Total: {}", success_count + failure_count);

    if failure_count == 0 {
        println!("\n🎉 All models parsed successfully!");
    } else {
        println!("\n⚠️  Some models failed to parse.");
    }

    Ok(())
}
