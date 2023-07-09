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
