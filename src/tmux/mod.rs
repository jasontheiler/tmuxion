mod pane;
mod session;
mod window;

use tmux_interface::{BindKey, DisplayPopup, RunShell, SelectLayout, SetHook, Tmux, TmuxCommands};

use crate::{config::Config, consts::SELF_FILE_PATH};

pub use self::{session::Session, window::Layout};

const ENV_VAR_KEY: &str = "TMUX";

pub fn assert_in_session() -> anyhow::Result<()> {
    std::env::var(ENV_VAR_KEY)
        .map(|_| ())
        .map_err(|_| anyhow::format_err!("you are not in a tmux session"))
}

pub fn set_up(config: &Config) -> anyhow::Result<()> {
    let mut tmux_cmds = TmuxCommands::new();

    let cmd_select_layout_string = SelectLayout::new().build().to_string();
    for hook_name in [
        "after-split-window[69]",
        "window-resized[69]",
        "pane-exited[69]",
    ] {
        tmux_cmds.push(
            SetHook::new()
                .global()
                .hook_name(hook_name)
                .command(cmd_select_layout_string.clone()),
        );
    }

    let mut tmux_keybind_cmds = TmuxCommands::new();

    let cmd_session_selector_string = DisplayPopup::new()
        .width(config.session_selector.width.clone())
        .height(config.session_selector.height.clone())
        .no_border()
        .shell_command(format!("\"{} select\"", SELF_FILE_PATH.to_string_lossy()))
        .close_on_exit()
        .build()
        .to_string();
    for key in &config.keybinds.session_selector {
        tmux_keybind_cmds.push(
            BindKey::new()
                .key(key)
                .command(cmd_session_selector_string.clone()),
        );
    }

    let cmd_last_session_string = RunShell::new()
        .shell_command(format!("\"{} last\"", SELF_FILE_PATH.to_string_lossy()))
        .build()
        .to_string();
    for key in &config.keybinds.last_session {
        tmux_keybind_cmds.push(
            BindKey::new()
                .key(key)
                .command(cmd_last_session_string.clone()),
        );
    }

    tmux_cmds.push(
        SetHook::new()
            .global()
            .hook_name("client-attached[69]")
            .command(tmux_keybind_cmds.to_string()),
    );

    Tmux::with_commands(tmux_cmds).status()?;

    Ok(())
}
