//! DBC database command implementations

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum DbcCommands {
    /// List records in a DBC file
    List {
        /// Path to the DBC file
        file: String,
    },

    /// Export DBC data
    Export {
        /// Path to the DBC file
        file: String,

        /// Output format (json, csv)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Show information about a DBC file
    Info {
        /// Path to the DBC file
        file: String,
    },
}

pub fn execute(command: DbcCommands) -> Result<()> {
    match command {
        DbcCommands::List { .. } => {
            anyhow::bail!("DBC support not yet implemented");
        }
        DbcCommands::Export { .. } => {
            anyhow::bail!("DBC support not yet implemented");
        }
        DbcCommands::Info { .. } => {
            anyhow::bail!("DBC support not yet implemented");
        }
    }
}
