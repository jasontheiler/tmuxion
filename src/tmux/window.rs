use std::path::PathBuf;

use serde::Deserialize;
use tmux_interface::{DisplayMessage, SelectLayout, SplitWindow, Tmux};

use crate::tmux::pane::{self, Pane};

pub(super) const FORMAT: &str = "{\"id\":\"#{window_id}\",\"path\":\"#{session_path}\"}";

#[derive(Debug, Clone, Deserialize)]
pub struct Window {
    id: String,
    path: PathBuf,
}

impl Window {
    pub fn select_layout(&self, layout: &Layout) -> anyhow::Result<()> {
        Tmux::with_command(
            SelectLayout::new()
                .target_pane(&self.id)
                .layout_name(layout.to_string()),
        )
        .status()?;
        Ok(())
    }

    pub fn new_pane(&self) -> anyhow::Result<Pane> {
        let output = Tmux::with_command(
            SplitWindow::new()
                .target_window(&self.id)
                .start_directory(self.path.to_string_lossy())
                .full()
                .detached()
                .format(pane::FORMAT)
                .print(),
        )
        .output()?;
        let pane = serde_json::from_str(&output.to_string())?;
        Ok(pane)
    }

    pub fn current_pane(&self) -> anyhow::Result<Pane> {
        let output = Tmux::with_command(
            DisplayMessage::new()
                .target_pane(&self.id)
                .message(pane::FORMAT)
                .print(),
        )
        .output()?;
        let pane = serde_json::from_str(&output.to_string())?;
        Ok(pane)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Layout {
    EvenHorizontal,
    EvenVertical,
    MainHorizontal,
    MainVertical,
    Tiled,
}

impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::EvenHorizontal => "even-horizontal",
            Self::EvenVertical => "even-vertical",
            Self::MainHorizontal => "main-horizontal",
            Self::MainVertical => "main-vertical",
            Self::Tiled => "tiled",
        })
    }
}
