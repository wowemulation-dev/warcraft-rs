//! M2 model command implementations

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum M2Commands {
    /// Show information about an M2 model
    Info {
        /// Path to the M2 file
        file: String,
    },

    /// Export M2 model data
    Export {
        /// Path to the M2 file
        file: String,

        /// Output format (obj, gltf, json)
        #[arg(short, long, default_value = "obj")]
        format: String,

        /// Output file (derives from input if not specified)
        #[arg(short, long)]
        output: Option<String>,

        /// Include textures
        #[arg(short, long)]
        textures: bool,

        /// Include animations
        #[arg(short, long)]
        animations: bool,
    },

    /// List model components
    List {
        /// Path to the M2 file
        file: String,

        /// Component to list (animations, bones, textures, meshes)
        #[arg(short, long, default_value = "all")]
        component: String,
    },

    /// Extract animations from M2
    ExtractAnim {
        /// Path to the M2 file
        file: String,

        /// Animation ID or name
        #[arg(short, long)]
        animation: Option<String>,

        /// Output directory
        #[arg(short, long)]
        output: Option<String>,
    },
}

pub fn execute(command: M2Commands) -> Result<()> {
    match command {
        M2Commands::Info { .. } => {
            anyhow::bail!("M2 support not yet implemented");
        }
        M2Commands::Export { .. } => {
            anyhow::bail!("M2 support not yet implemented");
        }
        M2Commands::List { .. } => {
            anyhow::bail!("M2 support not yet implemented");
        }
        M2Commands::ExtractAnim { .. } => {
            anyhow::bail!("M2 support not yet implemented");
        }
    }
}
