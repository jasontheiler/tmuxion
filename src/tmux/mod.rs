mod pane;
mod session;
mod window;

use tmux_interface::{BindKey, DisplayPopup, RunShell, SelectLayout, SetHook, Tmux, TmuxCommands};

use crate::{config::Config, consts::SELF_FILE_PATH};

pub use self::session::Session;

const ENV_VAR_KEY: &str = "TMUX";
const CMD_SELECT_LAYOUT_HOOK_NAMES: &[&str] = &[
    "after-split-window[69]",
    "window-resized[69]",
    "pane-exited[69]",
];

pub fn assert_in_session() -> anyhow::Result<()> {
    std::env::var(ENV_VAR_KEY)
        .map(|_| ())
        .map_err(|_| anyhow::format_err!("you are not in a tmux session"))
}

pub fn set_up(config: &Config) -> anyhow::Result<()> {
    let mut tmux_cmds = TmuxCommands::new();

    let cmd_select_layout = SelectLayout::new().build();
    for &hook_name in CMD_SELECT_LAYOUT_HOOK_NAMES {
        tmux_cmds.push(
            SetHook::new()
                .global()
                .hook_name(hook_name)
                .command(cmd_select_layout.to_string()),
        );
    }

    let mut tmux_keybind_cmds = TmuxCommands::new();

    let cmd_session_selector = DisplayPopup::new()
        .width(config.session_selector.width.clone())
        .height(config.session_selector.height.clone())
        .no_border()
        .shell_command(format!("\"{} select\"", SELF_FILE_PATH.to_string_lossy()))
        .close_on_exit()
        .build();
    for key in &config.keybinds.session_selector {
        tmux_keybind_cmds.push(
            BindKey::new()
                .key(key)
                .command(cmd_session_selector.to_string()),
        );
    }

    let cmd_last_session = RunShell::new()
        .shell_command(format!("\"{} last\"", SELF_FILE_PATH.to_string_lossy()))
        .build();
    for key in &config.keybinds.last_session {
        tmux_keybind_cmds.push(
            BindKey::new()
                .key(key)
                .command(cmd_last_session.to_string()),
        );
    }

    let cmd_keybinds = SetHook::new()
        .global()
        .hook_name("client-attached[69]")
        .command(tmux_keybind_cmds.to_string());
    tmux_cmds.push(cmd_keybinds);

    Tmux::with_commands(tmux_cmds).status()?;

    Ok(())
}
