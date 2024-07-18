use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

impl Args {
    pub fn new() -> Self {
        Self::parse()
    }
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Create new tmux sessions for all specified directories or the current
    /// directory.
    Create(Create),
    #[command(hide = true)]
    Select,
    #[command(hide = true)]
    Last,
}

#[derive(Debug, Clone, clap::Args)]
pub struct Create {
    /// The directories to create tmux sessions for.
    pub paths: Vec<PathBuf>,
    /// Create directories, if they do not already exist.
    #[arg(short, long)]
    pub create_dirs: bool,
}
