//! BLP texture command implementations

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum BlpCommands {
    /// Convert BLP to another format
    Convert {
        /// Path to the BLP file
        file: String,

        /// Output format (png, jpg, tga)
        #[arg(short, long, default_value = "png")]
        to: String,

        /// Output file (derives from input if not specified)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Show information about a BLP file
    Info {
        /// Path to the BLP file
        file: String,
    },
}

pub fn execute(command: BlpCommands) -> Result<()> {
    match command {
        BlpCommands::Convert { .. } => {
            anyhow::bail!("BLP support not yet implemented");
        }
        BlpCommands::Info { .. } => {
            anyhow::bail!("BLP support not yet implemented");
        }
    }
}
