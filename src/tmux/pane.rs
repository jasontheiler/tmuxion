use serde::Deserialize;
use tmux_interface::{SendKeys, Tmux};

pub(super) const FORMAT: &str = "{\"id\":\"#{pane_id}\"}";

#[derive(Debug, Clone, Deserialize)]
pub struct Pane {
    id: String,
}

impl Pane {
    pub fn run_command(&self, command: &str) -> anyhow::Result<()> {
        Tmux::with_command(
            SendKeys::new()
                .target_pane(&self.id)
                .disable_lookup()
                .key(format!("{command}\n")),
        )
        .status()?;
        Ok(())
    }
}

impl mlua::UserData for Pane {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("run_command", |_, this, command: String| {
            this.run_command(&command).map_err(mlua::Error::external)
        });
    }
}
