//! DBD (Database Definition) file handling commands

use clap::Subcommand;
use std::fs;
use std::path::{Path, PathBuf};
use wow_dbc::dbd::{convert_to_yaml_schemas, parse_dbd_file};

#[derive(Debug, Subcommand)]
pub enum DbdCommand {
    /// Convert a DBD file to YAML schemas
    Convert {
        /// Path to the DBD file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output directory for YAML files
        #[arg(short, long, default_value = "./schemas")]
        output: PathBuf,

        /// Only generate schema for specific version (e.g., "3.3.5")
        #[arg(long)]
        version: Option<String>,

        /// Generate schemas for all versions
        #[arg(short, long, default_value_t = false)]
        all: bool,
    },
}

impl DbdCommand {
    /// Execute the DBD command
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            DbdCommand::Convert {
                file,
                output,
                version,
                all,
            } => convert_dbd_to_yaml(file, output, version.as_deref(), *all),
        }
    }
}

/// Convert a DBD file to YAML schemas
fn convert_dbd_to_yaml(
    file: &Path,
    output: &PathBuf,
    version: Option<&str>,
    all: bool,
) -> anyhow::Result<()> {
    // Check if file exists
    if !file.exists() {
        anyhow::bail!("DBD file not found: {}", file.display());
    }

    println!("Converting DBD file: {}", file.display());
    println!();

    // Parse the DBD file
    let dbd_file =
        parse_dbd_file(file).map_err(|e| anyhow::anyhow!("Failed to parse DBD file: {}", e))?;

    let base_name = file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown");

    // Create output directory if it doesn't exist
    fs::create_dir_all(output)
        .map_err(|e| anyhow::anyhow!("Failed to create output directory: {}", e))?;

    // Convert to YAML schemas
    let schemas = convert_to_yaml_schemas(&dbd_file, base_name, version, all);

    if schemas.is_empty() {
        if !all && version.is_none() {
            eprintln!(
                "No schemas generated. Try using --all flag or specify a version with --version."
            );
            eprintln!();
            eprintln!("Common version examples:");
            eprintln!("  --version 1.12  (Classic/Vanilla)");
            eprintln!("  --version 2.4.3 (The Burning Crusade)");
            eprintln!("  --version 3.3.5 (Wrath of the Lich King)");
            eprintln!("  --version 4.3.4 (Cataclysm)");
            eprintln!("  --version 5.4.8 (Mists of Pandaria)");
        } else {
            eprintln!("No schemas found matching the specified criteria.");
        }
    } else {
        println!("Generating YAML schemas...");
        println!();

        let count = schemas.len();
        for (filename, content, version_suffix) in schemas {
            let output_path = output.join(&filename);
            fs::write(&output_path, content).map_err(|e| {
                anyhow::anyhow!("Failed to write file {}: {}", output_path.display(), e)
            })?;
            println!(
                "  Generated: {} ({})",
                output_path.display(),
                version_suffix
            );
        }

        println!();
        println!("Total schemas generated: {}", count);
        println!();
        println!("Note: DBD definitions are sourced from https://github.com/wowdev/WoWDBDefs");
    }

    Ok(())
}
