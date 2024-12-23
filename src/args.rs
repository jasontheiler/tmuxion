use std::path::PathBuf;

use clap::Parser;

use crate::APP_NAME;

#[derive(Debug, Clone, Parser)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
    /// Set configuration file.
    #[arg(long, env = format!("{}_CONFIG_FILE", APP_NAME.to_uppercase()), global = true)]
    pub config_file: Option<PathBuf>,
    /// Set target tmux client.
    #[arg(short, long, env = format!("{}_TARGET_CLIENT", APP_NAME.to_uppercase()), global = true)]
    pub target_client: Option<String>,
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
    #[command(visible_alias = "c")]
    Create(Create),
    #[command(hide = true)]
    Select,
    #[command(hide = true)]
    Last,
}

#[derive(Debug, Clone, clap::Args)]
pub struct Create {
    /// Directories to create tmux sessions for.
    pub paths: Vec<PathBuf>,
    /// Create directories, if they do not already exist.
    #[allow(clippy::struct_field_names)]
    #[arg(short, long)]
    pub create_dirs: bool,
    /// Create tmux sessions in the background.
    #[arg(short, long)]
    pub detached: bool,
}
