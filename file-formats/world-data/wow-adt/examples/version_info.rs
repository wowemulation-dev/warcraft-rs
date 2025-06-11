//! Example: ADT Version Detection and Compatibility Analysis
//!
//! This example demonstrates how to detect ADT file versions and analyze
//! compatibility across different World of Warcraft expansions.

use std::env;
use std::fs::File;
use std::path::Path;
use wow_adt::{Adt, AdtVersion};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_adt_file>", args[0]);
        eprintln!();
        eprintln!("This tool analyzes an ADT file's version and compatibility features.");
        eprintln!();
        eprintln!("Example:");
        eprintln!("  {} Azeroth_32_32.adt", args[0]);
        std::process::exit(1);
    }

    let adt_path = &args[1];

    // Check if file exists
    if !Path::new(adt_path).exists() {
        eprintln!("Error: File '{}' does not exist", adt_path);
        std::process::exit(1);
    }

    println!("🔍 ADT Version Analysis: {}", adt_path);
    println!("{}", "=".repeat(60));

    // Open and parse the ADT file
    let file = File::open(adt_path)?;
    let adt = Adt::from_reader(file)?;

    // Display version information
    println!("📊 Version Information:");
    println!("  • Detected version: {:?}", adt.version);
    println!("  • Expansion: {}", get_expansion_name(&adt.version));
    println!(
        "  • Release timeframe: {}",
        get_release_timeframe(&adt.version)
    );

    // Display MVER chunk information
    println!("  • MVER value: {}", adt.mver.version);
    println!("  • Standard MVER: {}", adt.version.to_mver_value());

    if adt.mver.version != adt.version.to_mver_value() {
        println!("    ⚠️  Non-standard MVER value detected!");
    }

    // Analyze version-specific features
    println!();
    println!("🎯 Version-Specific Features:");
    analyze_version_features(&adt);

    // Analyze compatibility
    println!();
    println!("🔄 Compatibility Analysis:");
    analyze_compatibility(&adt);

    // Display chunk presence matrix
    println!();
    println!("📋 Chunk Presence by Version:");
    display_chunk_matrix(&adt);

    // Performance characteristics
    println!();
    println!("⚡ Performance Characteristics:");
    analyze_performance(&adt);

    // Migration information
    println!();
    println!("🔄 Migration Information:");
    provide_migration_info(&adt.version);

    Ok(())
}

fn get_expansion_name(version: &AdtVersion) -> &'static str {
    match version {
        AdtVersion::Vanilla => "World of Warcraft: Classic",
        AdtVersion::TBC => "The Burning Crusade",
        AdtVersion::WotLK => "Wrath of the Lich King",
        AdtVersion::Cataclysm => "Cataclysm",
        AdtVersion::MoP => "Mists of Pandaria",
        AdtVersion::Legion => "Legion",
        AdtVersion::BfA => "Battle for Azeroth",
        AdtVersion::WoD => "Warlords of Draenor",
        AdtVersion::Shadowlands => "Shadowlands",
        AdtVersion::Dragonflight => "Dragonflight",
    }
}

fn get_release_timeframe(version: &AdtVersion) -> &'static str {
    match version {
        AdtVersion::Vanilla => "2004-2007",
        AdtVersion::TBC => "2007-2008",
        AdtVersion::WotLK => "2008-2010",
        AdtVersion::Cataclysm => "2010-2012",
        AdtVersion::MoP => "2012-2014",
        AdtVersion::WoD => "2014-2016",
        AdtVersion::Legion => "2016-2018",
        AdtVersion::BfA => "2018-2020",
        AdtVersion::Shadowlands => "2020-2022",
        AdtVersion::Dragonflight => "2022+",
    }
}

fn analyze_version_features(adt: &Adt) {
    let features = get_version_features(&adt.version);

    for feature in &features {
        let present = check_feature_present(adt, feature);
        let status = if present { "✅" } else { "❌" };
        println!("  {} {}: {}", status, feature.name, feature.description);
    }
}

fn analyze_compatibility(adt: &Adt) {
    let all_versions = [
        AdtVersion::Vanilla,
        AdtVersion::TBC,
        AdtVersion::WotLK,
        AdtVersion::Cataclysm,
        AdtVersion::MoP,
        AdtVersion::Legion,
        AdtVersion::BfA,
    ];

    println!("  Forward compatibility:");
    for version in &all_versions {
        if *version as u8 > adt.version as u8 {
            let compatible = is_forward_compatible(&adt.version, version);
            let status = if compatible { "✅" } else { "❌" };
            println!(
                "    {} {}: {}",
                status,
                get_expansion_name(version),
                if compatible {
                    "Likely compatible"
                } else {
                    "May have issues"
                }
            );
        }
    }

    println!("  Backward compatibility:");
    for version in &all_versions {
        if (*version as u8) < (adt.version as u8) {
            let compatible = is_backward_compatible(&adt.version, version);
            let status = if compatible { "✅" } else { "❌" };
            println!(
                "    {} {}: {}",
                status,
                get_expansion_name(version),
                if compatible {
                    "Should work"
                } else {
                    "Not supported"
                }
            );
        }
    }
}

fn display_chunk_matrix(adt: &Adt) {
    let chunks = [
        ("MVER", true), // Always present
        ("MHDR", adt.mhdr.is_some()),
        ("MCIN", adt.mcin.is_some()),
        ("MTEX", adt.mtex.is_some()),
        ("MMDX", adt.mmdx.is_some()),
        ("MMID", adt.mmid.is_some()),
        ("MWMO", adt.mwmo.is_some()),
        ("MWID", adt.mwid.is_some()),
        ("MDDF", adt.mddf.is_some()),
        ("MODF", adt.modf.is_some()),
        ("MFBO", adt.mfbo.is_some()),
        ("MH2O", adt.mh2o.is_some()),
        ("MCNK", !adt.mcnk_chunks.is_empty()),
    ];

    for (chunk_name, present) in chunks {
        let status = if present { "✅" } else { "❌" };
        let introduced = get_chunk_introduction(chunk_name);
        println!(
            "  {} {}: {} (introduced in {})",
            status,
            chunk_name,
            if present { "Present" } else { "Missing" },
            introduced
        );
    }
}

fn analyze_performance(adt: &Adt) {
    // Estimate memory usage
    let mut estimated_size = 0;

    if let Some(mcin) = &adt.mcin {
        estimated_size += mcin.entries.len() * 16; // 16 bytes per entry
    }

    if let Some(mtex) = &adt.mtex {
        estimated_size += mtex.filenames.iter().map(|s| s.len()).sum::<usize>();
    }

    if let Some(mmdx) = &adt.mmdx {
        estimated_size += mmdx.filenames.iter().map(|s| s.len()).sum::<usize>();
    }

    println!("  • Estimated parsed size: ~{} KB", estimated_size / 1024);
    println!("  • Terrain chunks: {}", adt.mcnk_chunks.len());

    // Version-specific performance notes
    match adt.version {
        AdtVersion::Vanilla | AdtVersion::TBC => {
            println!("  • Legacy format: Simple structure, fast parsing");
        }
        AdtVersion::WotLK => {
            println!("  • Added water data: Moderate complexity increase");
        }
        AdtVersion::Cataclysm | AdtVersion::MoP => {
            println!("  • Split format: May require loading multiple files");
        }
        _ => {
            println!("  • Modern format: Complex structure, slower parsing");
        }
    }
}

fn provide_migration_info(version: &AdtVersion) {
    match version {
        AdtVersion::Vanilla => {
            println!("  • To upgrade: Add MFBO chunk for TBC compatibility");
            println!("  • Water support: Requires MH2O chunk for WotLK+ features");
        }
        AdtVersion::TBC => {
            println!("  • From Vanilla: Compatible with minimal changes");
            println!("  • To WotLK: Add MH2O chunk for water features");
        }
        AdtVersion::WotLK => {
            println!("  • From TBC: Backward compatible");
            println!("  • To Cataclysm: Major format changes - split ADT files");
        }
        AdtVersion::Cataclysm | AdtVersion::MoP => {
            println!("  • Split format: Requires _tex0.adt, _obj0.adt companions");
            println!("  • Backward compatibility: Limited due to format changes");
        }
        _ => {
            println!("  • Modern format: Significant differences from legacy");
            println!("  • Legacy support: May require format conversion");
        }
    }
}

struct VersionFeature {
    name: &'static str,
    description: &'static str,
    check_fn: fn(&Adt) -> bool,
}

fn get_version_features(version: &AdtVersion) -> Vec<VersionFeature> {
    let mut features = vec![
        VersionFeature {
            name: "Basic terrain",
            description: "MCNK terrain chunks",
            check_fn: |adt| !adt.mcnk_chunks.is_empty(),
        },
        VersionFeature {
            name: "Textures",
            description: "MTEX texture references",
            check_fn: |adt| adt.mtex.is_some(),
        },
        VersionFeature {
            name: "Models",
            description: "MMDX/MMID model system",
            check_fn: |adt| adt.mmdx.is_some() && adt.mmid.is_some(),
        },
    ];

    if *version >= AdtVersion::TBC {
        features.push(VersionFeature {
            name: "Flight boundaries",
            description: "MFBO flight restriction data",
            check_fn: |adt| adt.mfbo.is_some(),
        });
    }

    if *version >= AdtVersion::WotLK {
        features.push(VersionFeature {
            name: "Water system",
            description: "MH2O water and liquid data",
            check_fn: |adt| adt.mh2o.is_some(),
        });
    }

    features
}

fn check_feature_present(adt: &Adt, feature: &VersionFeature) -> bool {
    (feature.check_fn)(adt)
}

fn is_forward_compatible(current: &AdtVersion, target: &AdtVersion) -> bool {
    // Generally, older formats work in newer clients
    (*current as u8) <= (*target as u8)
}

fn is_backward_compatible(current: &AdtVersion, target: &AdtVersion) -> bool {
    // Newer formats generally don't work in older clients
    match (current, target) {
        (AdtVersion::TBC, AdtVersion::Vanilla) => true, // TBC files can work in vanilla with limitations
        (AdtVersion::WotLK, AdtVersion::TBC) => true,   // Some compatibility
        _ => (*current as u8) <= (*target as u8),
    }
}

fn get_chunk_introduction(chunk: &str) -> &'static str {
    match chunk {
        "MVER" | "MHDR" | "MCIN" | "MTEX" | "MMDX" | "MMID" | "MWMO" | "MWID" | "MDDF" | "MODF"
        | "MCNK" => "Vanilla",
        "MFBO" => "TBC",
        "MH2O" => "WotLK",
        _ => "Unknown",
    }
}
