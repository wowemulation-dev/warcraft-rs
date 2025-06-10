//! WDL low-resolution terrain command implementations

use anyhow::{Context, Result};
use clap::Subcommand;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use wow_wdl::parser::WdlParser;
use wow_wdl::validation::validate_wdl_file;
use wow_wdl::version::WdlVersion;

#[derive(Subcommand)]
pub enum WdlCommands {
    /// Validate a WDL file
    Validate {
        /// Path to the WDL file
        file: PathBuf,

        /// Explicitly specify the WDL version to validate against
        #[arg(long, value_name = "VERSION")]
        version: Option<String>,
    },

    /// Convert a WDL file from one version to another
    Convert {
        /// Path to the input WDL file
        input: PathBuf,

        /// Path to write the converted WDL file
        output: PathBuf,

        /// Source version (if not auto-detected)
        #[arg(long, value_name = "VERSION")]
        from: Option<String>,

        /// Target version
        #[arg(short, long, value_name = "VERSION")]
        to: String,
    },

    /// Display information about a WDL file
    Info {
        /// Path to the WDL file
        file: PathBuf,
    },
}

/// Maps a version string to a WdlVersion
fn parse_version(version_str: &str) -> Result<WdlVersion> {
    match version_str.to_lowercase().as_str() {
        "vanilla" => Ok(WdlVersion::Vanilla),
        "classic" => Ok(WdlVersion::Vanilla),
        "tbc" => Ok(WdlVersion::Vanilla),
        "wotlk" => Ok(WdlVersion::Wotlk),
        "lich" | "lichking" | "wrath" => Ok(WdlVersion::Wotlk),
        "cata" | "cataclysm" => Ok(WdlVersion::Cataclysm),
        "mop" | "pandaria" => Ok(WdlVersion::Mop),
        "wod" | "draenor" => Ok(WdlVersion::Wod),
        "legion" => Ok(WdlVersion::Legion),
        "bfa" | "battleforazeroth" => Ok(WdlVersion::Bfa),
        "sl" | "shadowlands" => Ok(WdlVersion::Shadowlands),
        "df" | "dragonflight" => Ok(WdlVersion::Dragonflight),
        "latest" => Ok(WdlVersion::Latest),
        _ => anyhow::bail!("Unknown version: {}", version_str),
    }
}

pub fn execute(command: WdlCommands) -> Result<()> {
    match command {
        WdlCommands::Validate { file, version } => execute_validate(file, version),
        WdlCommands::Convert {
            input,
            output,
            from,
            to,
        } => execute_convert(input, output, from, to),
        WdlCommands::Info { file } => execute_info(file),
    }
}

fn execute_validate(path: PathBuf, version: Option<String>) -> Result<()> {
    use console::style;

    let file =
        File::open(&path).with_context(|| format!("Failed to open file: {}", path.display()))?;

    let mut reader = BufReader::new(file);

    // Create parser with specified version if provided
    let parser = if let Some(version_str) = version {
        let version = parse_version(&version_str)?;
        WdlParser::with_version(version)
    } else {
        WdlParser::new()
    };

    // Parse the file
    let wdl_file = parser
        .parse(&mut reader)
        .with_context(|| format!("Failed to parse WDL file: {}", path.display()))?;

    // Validate the file
    match validate_wdl_file(&wdl_file) {
        Ok(_) => {
            println!(
                "✓ WDL file '{}' is valid (version: {})",
                style(path.display()).cyan(),
                style(&wdl_file.version).yellow()
            );
        }
        Err(err) => {
            anyhow::bail!("Validation failed: {}", err);
        }
    }

    Ok(())
}

fn execute_convert(
    input: PathBuf,
    output: PathBuf,
    from: Option<String>,
    to: String,
) -> Result<()> {
    use crate::utils::progress::create_progress_bar;
    use console::style;

    // Open the input file
    let input_file = File::open(&input)
        .with_context(|| format!("Failed to open input file: {}", input.display()))?;

    let mut reader = BufReader::new(input_file);

    // Create parser with specified source version if provided
    let parser = if let Some(version_str) = from {
        let version = parse_version(&version_str)?;
        WdlParser::with_version(version)
    } else {
        WdlParser::new()
    };

    // Parse the input file
    println!("Parsing input file...");
    let wdl_file = parser
        .parse(&mut reader)
        .with_context(|| format!("Failed to parse WDL file: {}", input.display()))?;

    // Parse the target version
    let target_version = parse_version(&to)?;

    // Show progress for conversion
    let pb = create_progress_bar(100, "Converting WDL file");
    pb.set_position(25);

    // Convert the file
    let converted_file = wow_wdl::conversion::convert_wdl_file(&wdl_file, target_version)
        .context("Failed to convert WDL file")?;

    pb.set_position(75);

    // Open the output file
    let output_file = File::create(&output)
        .with_context(|| format!("Failed to create output file: {}", output.display()))?;

    let mut writer = BufWriter::new(output_file);

    // Write the converted file
    let output_parser = WdlParser::with_version(target_version);
    output_parser
        .write(&mut writer, &converted_file)
        .context("Failed to write converted file")?;

    pb.finish_and_clear();

    println!(
        "✓ Successfully converted from {} to {}",
        style(&wdl_file.version).yellow(),
        style(&target_version).green()
    );

    Ok(())
}

fn execute_info(path: PathBuf) -> Result<()> {
    use crate::utils::table::create_table;
    use console::style;
    use prettytable::row;

    // Open the file
    let file =
        File::open(&path).with_context(|| format!("Failed to open file: {}", path.display()))?;

    let mut reader = BufReader::new(file);

    // Parse the file
    let parser = WdlParser::new();
    let wdl_file = parser
        .parse(&mut reader)
        .with_context(|| format!("Failed to parse WDL file: {}", path.display()))?;

    // Display basic information
    println!("\n{}", style("WDL File Information").bold().underlined());
    println!("File: {}", style(path.display()).cyan());
    println!("Version: {}", style(&wdl_file.version).yellow());
    println!("Total Chunks: {}", style(wdl_file.chunks.len()).green());

    // Count map tiles
    let mut tiles_count = 0;
    for &offset in &wdl_file.map_tile_offsets {
        if offset != 0 {
            tiles_count += 1;
        }
    }

    println!(
        "Map Tiles: {}/{}",
        style(tiles_count).green(),
        style(64 * 64).dim()
    );

    // Display WMO information if present
    if !wdl_file.wmo_filenames.is_empty() {
        println!("\n{}", style("WMO Data (Pre-Legion)").bold());
        println!(
            "WMO Models: {}",
            style(wdl_file.wmo_filenames.len()).green()
        );
        println!(
            "WMO Placements: {}",
            style(wdl_file.wmo_placements.len()).green()
        );
    }

    // Display Legion+ model data if present
    if !wdl_file.m2_placements.is_empty() || !wdl_file.wmo_legion_placements.is_empty() {
        println!("\n{}", style("Legion+ Model Data").bold());
        println!(
            "M2 Placements: {}",
            style(wdl_file.m2_placements.len()).green()
        );
        println!(
            "WMO Placements: {}",
            style(wdl_file.wmo_legion_placements.len()).green()
        );
    }

    // Create chunk summary table
    let mut chunk_counts = std::collections::HashMap::new();
    for chunk in &wdl_file.chunks {
        // Convert the 4-byte magic to a string - reverse the bytes to get readable magic
        let chunk_name = if chunk.magic.len() == 4 {
            let reversed = [
                chunk.magic[3],
                chunk.magic[2],
                chunk.magic[1],
                chunk.magic[0],
            ];
            String::from_utf8_lossy(&reversed).to_string()
        } else {
            format!("{:?}", chunk.magic)
        };
        *chunk_counts.entry(chunk_name).or_insert(0) += 1;
    }

    if !chunk_counts.is_empty() {
        println!("\n{}", style("Chunk Summary").bold());
        let mut table = create_table(vec!["Chunk Type", "Count", "Description"]);

        let chunk_descriptions = [
            ("MVER", "Version information"),
            ("MAOF", "Map area offset table"),
            ("MARE", "Map area heightmap data"),
            ("MAHO", "Map area hole data"),
            ("MWMO", "WMO filename strings"),
            ("MWID", "WMO filename indices"),
            ("MODF", "WMO placement data"),
            ("MLDD", "M2 model FileDataIDs"),
            ("MLDX", "M2 model indices"),
            ("MLMD", "M2 model placement data"),
            ("MLMX", "M2 model visibility bounds"),
        ];

        for (chunk_type, description) in &chunk_descriptions {
            if let Some(&count) = chunk_counts.get(*chunk_type) {
                table.add_row(row![
                    style(chunk_type).cyan(),
                    style(count).green(),
                    description
                ]);
            }
        }

        table.printstd();
    }

    Ok(())
}
