//! Example: ADT Version Detection and Compatibility Analysis
//!
//! This example demonstrates how to detect ADT file versions and analyze
//! compatibility across different World of Warcraft expansions.

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use wow_adt::{AdtVersion, ParsedAdt, RootAdt, parse_adt};

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    if !Path::new(adt_path).exists() {
        eprintln!("Error: File '{adt_path}' does not exist");
        std::process::exit(1);
    }

    println!("ADT Version Analysis: {adt_path}");
    println!("{}", "=".repeat(60));

    let file = File::open(adt_path)?;
    let mut reader = BufReader::new(file);
    let parsed = parse_adt(&mut reader)?;

    let adt = match parsed {
        ParsedAdt::Root(root) => root,
        _ => {
            eprintln!("Error: This example only works with root ADT files");
            std::process::exit(1);
        }
    };

    println!("Version Information:");
    println!("  Detected version: {:?}", adt.version);
    println!("  Expansion: {}", get_expansion_name(&adt.version));
    println!(
        "  Release timeframe: {}",
        get_release_timeframe(&adt.version)
    );

    println!();
    println!("Version-Specific Features:");
    analyze_version_features(&adt);

    println!();
    println!("Compatibility Analysis:");
    analyze_compatibility(&adt);

    println!();
    println!("Chunk Presence by Version:");
    display_chunk_matrix(&adt);

    println!();
    println!("Performance Characteristics:");
    analyze_performance(&adt);

    println!();
    println!("Migration Information:");
    provide_migration_info(&adt.version);

    Ok(())
}

fn get_expansion_name(version: &AdtVersion) -> &'static str {
    match version {
        AdtVersion::VanillaEarly => "World of Warcraft: Classic (Early)",
        AdtVersion::VanillaLate => "World of Warcraft: Classic (Late)",
        AdtVersion::TBC => "The Burning Crusade",
        AdtVersion::WotLK => "Wrath of the Lich King",
        AdtVersion::Cataclysm => "Cataclysm",
        AdtVersion::MoP => "Mists of Pandaria",
    }
}

fn get_release_timeframe(version: &AdtVersion) -> &'static str {
    match version {
        AdtVersion::VanillaEarly => "2004-2005 (1.0-1.8.4)",
        AdtVersion::VanillaLate => "2005-2007 (1.9+)",
        AdtVersion::TBC => "2007-2008",
        AdtVersion::WotLK => "2008-2010",
        AdtVersion::Cataclysm => "2010-2012",
        AdtVersion::MoP => "2012-2014",
    }
}

fn analyze_version_features(adt: &RootAdt) {
    let features = get_version_features(&adt.version);

    for feature in &features {
        let present = check_feature_present(adt, feature);
        let status = if present { "[YES]" } else { "[NO]" };
        println!("  {} {}: {}", status, feature.name, feature.description);
    }
}

fn analyze_compatibility(adt: &RootAdt) {
    let all_versions = [
        AdtVersion::VanillaEarly,
        AdtVersion::VanillaLate,
        AdtVersion::TBC,
        AdtVersion::WotLK,
        AdtVersion::Cataclysm,
        AdtVersion::MoP,
    ];

    println!("  Forward compatibility:");
    for version in &all_versions {
        if *version as u8 > adt.version as u8 {
            let compatible = is_forward_compatible(&adt.version, version);
            let status = if compatible { "[YES]" } else { "[NO]" };
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
            let status = if compatible { "[YES]" } else { "[NO]" };
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

fn display_chunk_matrix(adt: &RootAdt) {
    let chunks = [
        ("MVER", true),
        ("MHDR", true),
        ("MCIN", true),
        ("MTEX", !adt.textures.is_empty()),
        ("MMDX", !adt.models.is_empty()),
        ("MMID", !adt.model_indices.is_empty()),
        ("MWMO", !adt.wmos.is_empty()),
        ("MWID", !adt.wmo_indices.is_empty()),
        ("MDDF", !adt.doodad_placements.is_empty()),
        ("MODF", !adt.wmo_placements.is_empty()),
        ("MFBO", adt.flight_bounds.is_some()),
        ("MH2O", adt.water_data.is_some()),
        ("MCNK", !adt.mcnk_chunks.is_empty()),
    ];

    for (chunk_name, present) in chunks {
        let status = if present { "[YES]" } else { "[NO]" };
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

fn analyze_performance(adt: &RootAdt) {
    let mut estimated_size = 0;

    estimated_size += adt.mcin.entries.len() * 16;
    estimated_size += adt.textures.iter().map(|s| s.len()).sum::<usize>();
    estimated_size += adt.models.iter().map(|s| s.len()).sum::<usize>();

    println!("  Estimated parsed size: ~{} KB", estimated_size / 1024);
    println!("  Terrain chunks: {}", adt.mcnk_chunks.len());

    match adt.version {
        AdtVersion::VanillaEarly | AdtVersion::VanillaLate | AdtVersion::TBC => {
            println!("  Legacy format: Simple structure, fast parsing");
        }
        AdtVersion::WotLK => {
            println!("  Added water data: Moderate complexity increase");
        }
        AdtVersion::Cataclysm | AdtVersion::MoP => {
            println!("  Split format: May require loading multiple files");
        }
    }
}

fn provide_migration_info(version: &AdtVersion) {
    match version {
        AdtVersion::VanillaEarly | AdtVersion::VanillaLate => {
            println!("  To upgrade: Add MFBO chunk for TBC compatibility");
            println!("  Water support: Requires MH2O chunk for WotLK+ features");
        }
        AdtVersion::TBC => {
            println!("  From Vanilla: Compatible with minimal changes");
            println!("  To WotLK: Add MH2O chunk for water features");
        }
        AdtVersion::WotLK => {
            println!("  From TBC: Backward compatible");
            println!("  To Cataclysm: Major format changes - split ADT files");
        }
        AdtVersion::Cataclysm | AdtVersion::MoP => {
            println!("  Split format: Requires _tex0.adt, _obj0.adt companions");
            println!("  Backward compatibility: Limited due to format changes");
        }
    }
}

struct VersionFeature {
    name: &'static str,
    description: &'static str,
    check_fn: fn(&RootAdt) -> bool,
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
            check_fn: |adt| !adt.textures.is_empty(),
        },
        VersionFeature {
            name: "Models",
            description: "MMDX/MMID model system",
            check_fn: |adt| !adt.models.is_empty() && !adt.model_indices.is_empty(),
        },
    ];

    if *version >= AdtVersion::TBC {
        features.push(VersionFeature {
            name: "Flight boundaries",
            description: "MFBO flight restriction data",
            check_fn: |adt| adt.flight_bounds.is_some(),
        });
    }

    if *version >= AdtVersion::WotLK {
        features.push(VersionFeature {
            name: "Water system",
            description: "MH2O water and liquid data",
            check_fn: |adt| adt.water_data.is_some(),
        });
    }

    features
}

fn check_feature_present(adt: &RootAdt, feature: &VersionFeature) -> bool {
    (feature.check_fn)(adt)
}

fn is_forward_compatible(current: &AdtVersion, target: &AdtVersion) -> bool {
    (*current as u8) <= (*target as u8)
}

fn is_backward_compatible(current: &AdtVersion, target: &AdtVersion) -> bool {
    match (current, target) {
        (AdtVersion::TBC, AdtVersion::VanillaLate) => true,
        (AdtVersion::WotLK, AdtVersion::TBC) => true,
        _ => current <= target,
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
