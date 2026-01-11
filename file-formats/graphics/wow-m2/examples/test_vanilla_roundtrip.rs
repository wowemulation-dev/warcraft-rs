//! Test roundtrip parsing and writing of pre-WotLK M2 files.
//!
//! Pre-WotLK M2 files (versions 256-263) contain embedded skin data directly
//! in the .m2 file, unlike WotLK+ which uses separate .skin files.
//!
//! This example demonstrates:
//! - Parsing a pre-WotLK M2 file
//! - Writing it back out
//! - Re-parsing the output to verify integrity
//!
//! Usage: cargo run --example test_vanilla_roundtrip -p wow-m2 -- <input.m2> [output.m2]

use std::env;
use std::fs;
use std::io::Cursor;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <input.m2> [output.m2]", args[0]);
        eprintln!();
        eprintln!("Tests roundtrip parsing and writing of pre-WotLK M2 files.");
        eprintln!("If output path is not provided, uses <input>_roundtrip.m2");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = if args.len() > 2 {
        args[2].clone()
    } else {
        let stem = input_path
            .strip_suffix(".m2")
            .or_else(|| input_path.strip_suffix(".M2"))
            .unwrap_or(input_path);
        format!("{}_roundtrip.m2", stem)
    };

    println!("Testing pre-WotLK M2 roundtrip...");
    println!("Input: {}", input_path);
    println!("Output: {}", output_path);

    // Read original file
    let original_data = match fs::read(input_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to read input file: {}", e);
            std::process::exit(1);
        }
    };
    println!("Original file size: {} bytes", original_data.len());

    // Parse the model
    let mut cursor = Cursor::new(&original_data);
    let model = match wow_m2::M2Model::parse(&mut cursor) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to parse M2: {:?}", e);
            std::process::exit(1);
        }
    };

    println!("Parsed model version: {}", model.header.version);
    println!(
        "Embedded skins count: {}",
        model.raw_data.embedded_skins.len()
    );
    println!(
        "Bone animation data entries: {}",
        model.raw_data.bone_animation_data.len()
    );
    println!(
        "Camera animation data entries: {}",
        model.raw_data.camera_animation_data.len()
    );
    println!(
        "Attachment animation data entries: {}",
        model.raw_data.attachment_animation_data.len()
    );
    println!("Cameras count: {}", model.cameras.len());
    println!("Attachments count: {}", model.attachments.len());
    println!("Events count: {}", model.events.len());

    // Show Vanilla-specific header fields
    if model.header.playable_animation_lookup.is_some() {
        println!(
            "Playable animation lookup: {} entries",
            model
                .header
                .playable_animation_lookup
                .as_ref()
                .unwrap()
                .count
        );
    }

    // Check if this is actually a pre-WotLK model
    if model.header.version >= 264 {
        println!(
            "\nWarning: This is a WotLK+ model (version {}), not pre-WotLK.",
            model.header.version
        );
        println!("Embedded skin handling only applies to pre-WotLK (versions 256-263).");
    }

    println!("\n=== Writing model... ===");

    // Write to output using Cursor for Seek support
    let mut output = Cursor::new(Vec::new());
    if let Err(e) = model.write(&mut output) {
        eprintln!("Failed to write M2: {:?}", e);
        std::process::exit(1);
    }
    let output = output.into_inner();
    println!("Output size: {} bytes", output.len());

    // Write to file
    if let Err(e) = fs::write(&output_path, &output) {
        eprintln!("Failed to write output file: {}", e);
        std::process::exit(1);
    }
    println!("Wrote to: {}", output_path);

    // Try to parse the output
    println!("\n=== Verifying output can be parsed... ===");
    let mut cursor2 = Cursor::new(&output);
    match wow_m2::M2Model::parse(&mut cursor2) {
        Ok(reparsed) => {
            println!(
                "SUCCESS! Reparsed model version: {}",
                reparsed.header.version
            );
            println!(
                "Reparsed embedded skins count: {}",
                reparsed.raw_data.embedded_skins.len()
            );
            println!(
                "Reparsed bone animation data entries: {}",
                reparsed.raw_data.bone_animation_data.len()
            );

            // Verify embedded skin data
            if !reparsed.raw_data.embedded_skins.is_empty() {
                println!("\nEmbedded skin data:");
                for (i, skin) in reparsed.raw_data.embedded_skins.iter().enumerate() {
                    println!(
                        "  Skin {}: model_view={} indices={} triangles={} submeshes={} batches={}",
                        i,
                        skin.model_view.len(),
                        skin.indices.len(),
                        skin.triangles.len(),
                        skin.submeshes.len(),
                        skin.batches.len()
                    );
                }
            }

            // Compare key counts
            let orig_skins = model.raw_data.embedded_skins.len();
            let new_skins = reparsed.raw_data.embedded_skins.len();
            let orig_anims = model.raw_data.bone_animation_data.len();
            let new_anims = reparsed.raw_data.bone_animation_data.len();

            println!("\n=== Comparison ===");
            if orig_skins == new_skins && orig_anims == new_anims {
                println!("PASS: All key data preserved through roundtrip");
            } else {
                println!("WARN: Data counts differ:");
                if orig_skins != new_skins {
                    println!("  Embedded skins: {} -> {}", orig_skins, new_skins);
                }
                if orig_anims != new_anims {
                    println!("  Animation data: {} -> {}", orig_anims, new_anims);
                }
            }
        }
        Err(e) => {
            eprintln!("ERROR: Failed to reparse output: {:?}", e);
            std::process::exit(1);
        }
    }
}
