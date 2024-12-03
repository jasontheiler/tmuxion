mod session;

use tmux_interface::{BindKey, DisplayPopup, RunShell, Tmux, TmuxCommands};

use crate::config::Config;

pub use self::session::Session;

const ENV_VAR_KEY: &str = "TMUX";

pub fn assert_in_session() -> anyhow::Result<()> {
    std::env::var(ENV_VAR_KEY)
        .map(|_| ())
        .map_err(|_| anyhow::format_err!("you are not in a tmux session"))
}

pub fn set_up(config: &Config) -> anyhow::Result<()> {
    let mut tmux_cmds = TmuxCommands::new();

    let cmd_select_session = DisplayPopup::new()
        .width(config.session_selector.width.clone())
        .height(config.session_selector.height.clone())
        .no_border()
        .shell_command(format!(
            r#""{} select""#,
            std::env::current_exe()?.to_string_lossy()
        ))
        .close_on_exit()
        .build();
    for key in &config.keybinds.select_session {
        tmux_cmds.push(
            BindKey::new()
                .key(key)
                .command(cmd_select_session.to_string()),
        );
    }

    let cmd_last_session = RunShell::new()
        .shell_command(format!(
            r#""{} last""#,
            std::env::current_exe()?.to_string_lossy()
        ))
        .build();
    for key in &config.keybinds.last_session {
        tmux_cmds.push(
            BindKey::new()
                .key(key)
                .command(cmd_last_session.to_string()),
        );
    }

    Tmux::with_commands(tmux_cmds).status()?;

    Ok(())
}
