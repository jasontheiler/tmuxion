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
    /// Creates new tmux sessions for all specified directories or the current
    /// directory.
    Create(Create),
    #[command(hide = true)]
    Select,
    #[command(hide = true)]
    Last,
}

#[derive(Debug, Clone, clap::Args)]
pub struct Create {
    pub paths: Vec<PathBuf>,
}
