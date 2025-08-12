mod cli;
mod ops;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use std::path::Path;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        cli::Commands::Install { file } => {
            ops::install_package(Path::new(&file))?;
        }
        cli::Commands::Remove { name } => {
            ops::remove_package(&name)?;
        }
        cli::Commands::List => {
            ops::list_packages()?;
        }
    }

    Ok(())
}
