use std::path::{Path, PathBuf};

use glob::Pattern;
use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::{
    args,
    config::Config,
    consts::CONFIG_DIR_PATH,
    deserializers,
    tmux::{self, Layout, Session},
};

pub static SESSION_TEMPLATES_FILE_PATH: Lazy<PathBuf> =
    Lazy::new(|| CONFIG_DIR_PATH.join("session_templates.yaml"));

#[allow(unused)]
#[derive(Debug, Clone, Default, Deserialize)]
struct SessionTemplate {
    #[serde(default)]
    conditions: SessionTemplateConditions,
    #[serde(default)]
    windows: Vec<Option<SessionTemplateWindow>>,
}

impl SessionTemplate {
    fn matches(&self, path: &Path) -> bool {
        self.conditions.matches(path)
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
struct SessionTemplateConditions {
    #[serde(default, deserialize_with = "deserializers::pattern_vec")]
    paths: Vec<Pattern>,
    #[serde(default)]
    files: Vec<PathBuf>,
}

impl SessionTemplateConditions {
    fn matches(&self, path: &Path) -> bool {
        let has_path_cond_met = self
            .paths
            .iter()
            .any(|path_cond| path_cond.matches(&path.to_string_lossy()))
            || self.paths.is_empty();
        let has_file_cond_met = self
            .files
            .iter()
            .all(|file_cond| path.join(file_cond).is_file());
        has_path_cond_met && has_file_cond_met
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
struct SessionTemplateWindow {
    #[serde(default)]
    layout: Option<Layout>,
    #[serde(default)]
    panes: Vec<Option<SessionTemplatePane>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct SessionTemplatePane {
    #[serde(default)]
    command: Option<String>,
}

pub fn create(config: &Config, args: &args::Create) -> anyhow::Result<()> {
    let mut paths = args
        .paths
        .iter()
        .map(|path| path.canonicalize())
        .collect::<Result<Vec<_>, _>>()?;
    if paths.is_empty() {
        paths.push(std::env::current_dir()?);
    }

    let content_opt = match std::fs::read_to_string(SESSION_TEMPLATES_FILE_PATH.as_path()) {
        Ok(content) => Some(content),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
        Err(err) => return Err(err.into()),
    };
    let session_templates = content_opt
        .map(|content| serde_yaml::from_str::<Vec<SessionTemplate>>(&content))
        .transpose()?
        .unwrap_or_default();

    let mut session_to_switch_to_opt = Option::<Session>::None;
    for path in &paths {
        let (session, has_existed) = Session::assert(path)?;
        if has_existed {
            session_to_switch_to_opt = Some(session);
            continue;
        }

        tmux::set_up(config)?;

        let session_template = session_templates
            .iter()
            .find(|session_template| session_template.matches(path))
            .cloned()
            .unwrap_or_default();

        for (i, window_template_opt) in session_template.windows.into_iter().enumerate() {
            let window_template = window_template_opt.unwrap_or_default();
            let window = if i == 0 {
                session.current_window()?
            } else {
                session.new_window()?
            };

            if let Some(layout) = window_template.layout {
                window.select_layout(&layout)?;
            }

            for (j, pane_template_opt) in window_template.panes.into_iter().enumerate() {
                let pane_template = pane_template_opt.unwrap_or_default();
                let pane = if j == 0 {
                    window.current_pane()?
                } else {
                    window.new_pane()?
                };

                if let Some(command) = pane_template.command {
                    pane.run_command(&command)?;
                };
            }
        }

        session_to_switch_to_opt = Some(session);
    }

    if let Some(session_to_switch_to) = session_to_switch_to_opt {
        session_to_switch_to.switch_to()?;
    }

    Ok(())
}
