//! WDL low-resolution terrain command implementations

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum WdlCommands {
    /// Show information about a WDL file
    Info {
        /// Path to the WDL file
        file: String,
    },

    /// Export WDL terrain data
    Export {
        /// Path to the WDL file
        file: String,

        /// Output format (obj, heightmap, json)
        #[arg(short, long, default_value = "obj")]
        format: String,

        /// Output file (derives from input if not specified)
        #[arg(short, long)]
        output: Option<String>,

        /// Include water data
        #[arg(short, long)]
        water: bool,
    },

    /// Extract low-res height map
    ExtractHeightmap {
        /// Path to the WDL file
        file: String,

        /// Output format (png, raw, csv)
        #[arg(short, long, default_value = "png")]
        format: String,

        /// Output file
        #[arg(short, long)]
        output: Option<String>,

        /// Scale factor for output
        #[arg(short, long, default_value = "1")]
        scale: u32,
    },

    /// Compare WDL with high-res ADT data
    Compare {
        /// Path to the WDL file
        file: String,

        /// Path to corresponding WDT file
        #[arg(short, long)]
        wdt: String,

        /// Show differences only
        #[arg(short, long)]
        diff: bool,
    },

    /// Generate overview image
    Overview {
        /// Path to the WDL file
        file: String,

        /// Output image file
        #[arg(short, long)]
        output: Option<String>,

        /// Color mode (height, water, combined)
        #[arg(short, long, default_value = "combined")]
        mode: String,
    },
}

pub fn execute(command: WdlCommands) -> Result<()> {
    match command {
        WdlCommands::Info { .. } => {
            anyhow::bail!("WDL support not yet implemented");
        }
        WdlCommands::Export { .. } => {
            anyhow::bail!("WDL support not yet implemented");
        }
        WdlCommands::ExtractHeightmap { .. } => {
            anyhow::bail!("WDL support not yet implemented");
        }
        WdlCommands::Compare { .. } => {
            anyhow::bail!("WDL support not yet implemented");
        }
        WdlCommands::Overview { .. } => {
            anyhow::bail!("WDL support not yet implemented");
        }
    }
}
