use anyhow::Result;
use clap::{Parser, Subcommand};
use clio::ClioPath;
use std::time::Duration;

mod cleanup_stale_data;

/// Utilities for maintaining the Essen medical
/// computing platform
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Clean up stale files and directories in a specified directory
    /// (e.g. /local/work or /tmp)
    /// Directories containing a `.keep` file will remain untouched.
    CleanupStaleData {
        /// Directory to clean up (the given dir will not be removed, only its contents)
        /// (e.g. /local/work or /tmp)
        /// The directory must exist and be writable.
        #[arg(short, long)]
        dir: ClioPath,
        /// Age threshold for files and directories to be removed
        /// (e.g. 7d or 5h)
        #[arg(short, long, value_parser = parse_duration::parse)]
        age: Duration,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::CleanupStaleData { dir, age } => {
            cleanup_stale_data::cleanup_stale_data(&dir, &age)?;
        }
    }

    Ok(())
}
