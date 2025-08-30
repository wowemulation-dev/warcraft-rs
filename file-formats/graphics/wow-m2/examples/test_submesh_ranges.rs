use std::fs;
use std::io::Cursor;
use std::path::Path;
use wow_m2::parse_m2;

fn main() -> anyhow::Result<()> {
    println!("Testing submesh triangle range extraction...");

    let path = Path::new(
        "/home/danielsreichenbach/Repos/github.com/wowemulation-dev/blender-wow-addon/sample_data/1.12.1/m2/Rabbit.m2",
    );

    if !path.exists() {
        println!("❌ Rabbit.m2 not found");
        return Ok(());
    }

    let data = fs::read(path)?;
    let mut cursor = Cursor::new(&data);
    let m2_format = parse_m2(&mut cursor)?;
    let model = m2_format.model();

    match model.parse_embedded_skin(&data, 0) {
        Ok(skin) => {
            let raw_indices = skin.indices();
            let raw_triangles = skin.triangles(); // This might be the FULL triangle array
            let submeshes = skin.submeshes();

            println!("Raw data lengths:");
            println!("  Total indices: {}", raw_indices.len());
            println!("  Total triangles array: {}", raw_triangles.len());
            println!("  Submesh count: {}", submeshes.len());

            // Analyze each submesh
            for (i, submesh) in submeshes.iter().enumerate() {
                println!("\n🔍 Submesh {}:", i);
                println!(
                    "  vertex_start: {}, vertex_count: {}",
                    submesh.vertex_start, submesh.vertex_count
                );
                println!(
                    "  triangle_start: {}, triangle_count: {}",
                    submesh.triangle_start, submesh.triangle_count
                );

                // Calculate expected triangles
                let expected_triangle_indices = submesh.triangle_count;
                let expected_triangles = expected_triangle_indices / 3;
                println!("  Expected triangle indices: {}", expected_triangle_indices);
                println!("  Expected triangles: {}", expected_triangles);

                // Check if we can extract the submesh range
                let triangle_end = submesh.triangle_start + submesh.triangle_count;
                if triangle_end <= raw_triangles.len() as u16 {
                    // Extract the submesh-specific triangle slice
                    let submesh_triangles =
                        &raw_triangles[submesh.triangle_start as usize..triangle_end as usize];
                    println!(
                        "  ✅ Successfully extracted {} triangle indices for submesh",
                        submesh_triangles.len()
                    );
                    println!(
                        "  First 10 submesh triangles: {:?}",
                        &submesh_triangles[..10.min(submesh_triangles.len())]
                    );

                    // Test triangle formation with submesh range
                    println!("\n  Triangle formation with submesh range:");
                    for tri in 0..3.min(submesh_triangles.len() / 3) {
                        let tri_start = tri * 3;
                        let v1_idx = submesh_triangles[tri_start] as usize;
                        let v2_idx = submesh_triangles[tri_start + 1] as usize;
                        let v3_idx = submesh_triangles[tri_start + 2] as usize;

                        if v1_idx < raw_indices.len()
                            && v2_idx < raw_indices.len()
                            && v3_idx < raw_indices.len()
                        {
                            let v1 = raw_indices[v1_idx];
                            let v2 = raw_indices[v2_idx];
                            let v3 = raw_indices[v3_idx];
                            println!(
                                "    Triangle {}: triangles[{}]={} -> vertices({},{},{})",
                                tri, tri_start, submesh_triangles[tri_start], v1, v2, v3
                            );
                        }
                    }
                } else {
                    println!("  ❌ ERROR: triangle range out of bounds!");
                    println!(
                        "     Requested range: {}..{}",
                        submesh.triangle_start, triangle_end
                    );
                    println!("     Available range: 0..{}", raw_triangles.len());
                }
            }

            // Compare with current get_resolved_indices() approach
            println!("\n🆚 Current get_resolved_indices() vs Submesh approach:");
            let current_resolved = skin.get_resolved_indices();
            println!("Current resolved length: {}", current_resolved.len());

            if let Some(submesh) = submeshes.first() {
                let triangle_end = submesh.triangle_start + submesh.triangle_count;
                if triangle_end <= raw_triangles.len() as u16 {
                    let submesh_triangles =
                        &raw_triangles[submesh.triangle_start as usize..triangle_end as usize];

                    // Apply two-level indirection to submesh triangle range
                    let submesh_resolved: Vec<u16> = submesh_triangles
                        .iter()
                        .map(|&tri_idx| {
                            if (tri_idx as usize) < raw_indices.len() {
                                raw_indices[tri_idx as usize]
                            } else {
                                0
                            }
                        })
                        .collect();

                    println!("Submesh resolved length: {}", submesh_resolved.len());
                    println!(
                        "Lengths match: {}",
                        current_resolved.len() == submesh_resolved.len()
                    );

                    if current_resolved.len() != submesh_resolved.len() {
                        println!(
                            "❌ FOUND THE ISSUE: Length mismatch confirms submesh range extraction is needed!"
                        );
                    }
                }
            }
        }
        Err(e) => println!("❌ Failed to parse embedded skin: {}", e),
    }

    Ok(())
}
