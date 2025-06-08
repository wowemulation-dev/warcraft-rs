//! WMO world map object command implementations

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum WmoCommands {
    /// Show information about a WMO file
    Info {
        /// Path to the WMO file
        file: String,
    },

    /// Export WMO data
    Export {
        /// Path to the WMO file
        file: String,

        /// Output format (obj, gltf, json)
        #[arg(short, long, default_value = "obj")]
        format: String,

        /// Output directory
        #[arg(short, long)]
        output: Option<String>,

        /// Include textures
        #[arg(short, long)]
        textures: bool,

        /// Include doodads
        #[arg(short, long)]
        doodads: bool,
    },

    /// List WMO components
    List {
        /// Path to the WMO file
        file: String,

        /// Component to list (groups, doodads, portals, lights)
        #[arg(short, long, default_value = "all")]
        component: String,
    },

    /// Extract WMO groups
    ExtractGroups {
        /// Path to the WMO file
        file: String,

        /// Group indices to extract (comma-separated, or "all")
        #[arg(short, long, default_value = "all")]
        groups: String,

        /// Output directory
        #[arg(short, long)]
        output: Option<String>,
    },
}

pub fn execute(command: WmoCommands) -> Result<()> {
    match command {
        WmoCommands::Info { .. } => {
            anyhow::bail!("WMO support not yet implemented");
        }
        WmoCommands::Export { .. } => {
            anyhow::bail!("WMO support not yet implemented");
        }
        WmoCommands::List { .. } => {
            anyhow::bail!("WMO support not yet implemented");
        }
        WmoCommands::ExtractGroups { .. } => {
            anyhow::bail!("WMO support not yet implemented");
        }
    }
}
