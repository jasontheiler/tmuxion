mod args;
mod command;
mod config;
mod consts;
mod deserializers;
mod tmux;

use self::{
    args::{Args, Command},
    config::Config,
};

fn main() -> anyhow::Result<()> {
    let args = Args::new();
    let config = Config::new()?;

    match args.command {
        Command::Create(args_create) => command::create(&config, &args_create)?,
        Command::Select => command::select(&config)?,
        Command::Last => command::last(&config)?,
    }

    Ok(())
}
