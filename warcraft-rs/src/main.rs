//! Main entry point for the warcraft-rs CLI

mod cli;
mod commands;
#[cfg(feature = "mpq")]
mod database;
mod utils;

use anyhow::Result;
use clap::CommandFactory;
use clap::Parser;
use clap_complete::{Generator, generate};
use std::io;

use crate::cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Set verbosity
    if cli.verbose > 0 {
        log::set_max_level(match cli.verbose {
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        });
    } else if cli.quiet {
        log::set_max_level(log::LevelFilter::Error);
    }

    // Execute command
    match cli.command {
        #[cfg(feature = "mpq")]
        Commands::Mpq { command } => commands::mpq::execute(command).await,

        #[cfg(feature = "dbc")]
        Commands::Dbc { command } => commands::dbc::execute(command),

        #[cfg(feature = "dbc")]
        Commands::Dbd { command } => command.execute(),

        #[cfg(feature = "blp")]
        Commands::Blp { command } => commands::blp::execute(command),

        #[cfg(feature = "m2")]
        Commands::M2 { command } => commands::m2::execute(command),

        #[cfg(feature = "wmo")]
        Commands::Wmo { command } => commands::wmo::execute(command),

        #[cfg(feature = "adt")]
        Commands::Adt { command } => commands::adt::execute(command),

        #[cfg(feature = "wdt")]
        Commands::Wdt { command } => commands::wdt::execute(command),

        #[cfg(feature = "wdl")]
        Commands::Wdl { command } => commands::wdl::execute(command),

        Commands::Completions { shell } => {
            print_completions(shell, &mut Cli::command());
            Ok(())
        }
    }
}

fn print_completions<G: Generator>(generator: G, cmd: &mut clap::Command) {
    generate(
        generator,
        cmd,
        cmd.get_name().to_string(),
        &mut io::stdout(),
    );
}
