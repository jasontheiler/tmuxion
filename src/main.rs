mod args;
mod command;
mod config;
mod deserializers;
mod tmux;

use ratatui::crossterm::style::Stylize as _;

use self::{
    args::{Args, Command},
    config::Config,
};

const APP_NAME: &str = env!("CARGO_PKG_NAME");

fn main() {
    let args = Args::new();
    if let Err(err) = run(args) {
        eprintln!("{} {err}", "error:".dark_red().bold());
        std::process::exit(1);
    }
}

fn run(args: Args) -> anyhow::Result<()> {
    let config = Config::new(&args)?;
    match args.command {
        Command::Create(args_create) => command::create(&config, &args_create),
        Command::Select => command::select(&config),
        Command::Last => command::last(),
    }
}
