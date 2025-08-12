use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "RetroPKG - Lightweight package manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install a package
    Install {
        #[arg(short, long)]
        file: String,
    },
    
    /// Remove a package
    Remove {
        #[arg(short, long)]
        name: String,
    },
    
    /// List installed packages
    List,
}
