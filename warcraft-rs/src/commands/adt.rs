//! ADT terrain command implementations

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum AdtCommands {
    /// Show information about an ADT file
    Info {
        /// Path to the ADT file
        file: String,
    },

    /// Export ADT terrain data
    Export {
        /// Path to the ADT file
        file: String,

        /// Output format (obj, heightmap, json)
        #[arg(short, long, default_value = "obj")]
        format: String,

        /// Output file (derives from input if not specified)
        #[arg(short, long)]
        output: Option<String>,

        /// Include textures
        #[arg(short, long)]
        textures: bool,

        /// Include objects (M2/WMO placements)
        #[arg(short, long)]
        objects: bool,

        /// Include water
        #[arg(short, long)]
        water: bool,
    },

    /// List ADT components
    List {
        /// Path to the ADT file
        file: String,

        /// Component to list (chunks, textures, objects, liquids)
        #[arg(short, long, default_value = "all")]
        component: String,
    },

    /// Extract height map from ADT
    ExtractHeightmap {
        /// Path to the ADT file
        file: String,

        /// Output format (png, raw, csv)
        #[arg(short, long, default_value = "png")]
        format: String,

        /// Output file
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Analyze ADT file
    Analyze {
        /// Path to the ADT file
        file: String,

        /// Show chunk details
        #[arg(short, long)]
        chunks: bool,

        /// Show texture layers
        #[arg(short, long)]
        textures: bool,
    },
}

pub fn execute(command: AdtCommands) -> Result<()> {
    match command {
        AdtCommands::Info { .. } => {
            anyhow::bail!("ADT support not yet implemented");
        }
        AdtCommands::Export { .. } => {
            anyhow::bail!("ADT support not yet implemented");
        }
        AdtCommands::List { .. } => {
            anyhow::bail!("ADT support not yet implemented");
        }
        AdtCommands::ExtractHeightmap { .. } => {
            anyhow::bail!("ADT support not yet implemented");
        }
        AdtCommands::Analyze { .. } => {
            anyhow::bail!("ADT support not yet implemented");
        }
    }
}
