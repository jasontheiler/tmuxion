use std::path::{Path, PathBuf};

use mlua::LuaSerdeExt as _;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tmux_interface::{
    AttachSession, DisplayMessage, HasSession, ListSessions, NewSession, NewWindow, SwitchClient,
    Tmux,
};

use crate::{
    consts::CACHE_DIR_PATH,
    tmux::{
        self,
        window::{self, Window},
    },
};

pub(super) const FORMAT: &str = r##"{"id":"#{session_id}","path":"#{session_path}"}"##;
static LAST_SESSION_FILE_PATH: Lazy<PathBuf> =
    Lazy::new(|| CACHE_DIR_PATH.join("last_session.json"));

#[derive(Debug, Clone, Eq, Deserialize, Serialize)]
pub struct Session {
    id: String,
    #[serde(default, skip_serializing)]
    path: PathBuf,
}

impl Session {
    pub fn new(path: &Path) -> anyhow::Result<(Self, bool)> {
        let sessions = Self::all()?;
        let session_opt = sessions
            .iter()
            .find(|session| session.path.as_path() == path)
            .cloned();
        if let Some(session) = session_opt {
            return Ok((session, true));
        }
        let output = Tmux::with_command(
            NewSession::new()
                .start_directory(path.to_string_lossy())
                .detached()
                .format(FORMAT)
                .print(),
        )
        .output()?;
        let session = serde_json::from_str(&output.to_string())?;
        Ok((session, false))
    }

    pub fn current() -> anyhow::Result<Self> {
        let output = Tmux::with_command(DisplayMessage::new().message(FORMAT).print()).output()?;
        let session = serde_json::from_str(&output.to_string())?;
        Ok(session)
    }

    pub fn last() -> anyhow::Result<Option<Self>> {
        let content = match std::fs::read_to_string(LAST_SESSION_FILE_PATH.as_path()) {
            Ok(content) => content,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(err.into()),
        };
        let session = serde_json::from_str::<Self>(&content)?;
        let exists = Tmux::with_command(HasSession::new().target_session(&session.id))
            .status()?
            .success();
        Ok(exists.then_some(session))
    }

    pub fn all() -> anyhow::Result<Vec<Self>> {
        let output = Tmux::with_command(ListSessions::new().format(FORMAT)).output()?;
        let mut sessions = output
            .to_string()
            .lines()
            .map(serde_json::from_str)
            .collect::<Result<Vec<_>, _>>()?;
        sessions.sort();
        Ok(sessions)
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn save_as_last(&self) -> anyhow::Result<()> {
        if let Some(last_session_file_path_parent) = LAST_SESSION_FILE_PATH.parent() {
            std::fs::create_dir_all(last_session_file_path_parent)?;
        }
        std::fs::write(
            LAST_SESSION_FILE_PATH.as_path(),
            serde_json::to_string(&self)?,
        )?;
        Ok(())
    }

    pub fn switch_to(&self) -> anyhow::Result<()> {
        if tmux::assert_in_session().is_ok() {
            Tmux::with_command(SwitchClient::new().target_session(&self.id)).status()?;
        } else {
            Tmux::with_command(AttachSession::new().target_session(&self.id)).status()?;
        }
        Ok(())
    }

    pub fn new_window(&self) -> anyhow::Result<Window> {
        let output = Tmux::with_command(
            NewWindow::new()
                .target_window(&self.id)
                .start_directory(self.path.to_string_lossy())
                .detached()
                .format(window::FORMAT)
                .print(),
        )
        .output()?;
        let window = serde_json::from_str(&output.to_string())?;
        Ok(window)
    }

    pub fn current_window(&self) -> anyhow::Result<Window> {
        let output = Tmux::with_command(
            DisplayMessage::new()
                .target_pane(&self.id)
                .message(window::FORMAT)
                .print(),
        )
        .output()?;
        let window = serde_json::from_str(&output.to_string())?;
        Ok(window)
    }

    pub fn globs<S: AsRef<str>>(
        &self,
        patterns: &[S],
        opts: impl Into<Option<GlobsOpts>>,
    ) -> anyhow::Result<impl Iterator<Item = PathBuf>> {
        let mut glob_walker_builder =
            globwalk::GlobWalkerBuilder::from_patterns(&self.path, patterns);
        if let Some(opts) = opts.into() {
            if let Some(min_depth) = opts.min_depth {
                glob_walker_builder = glob_walker_builder.min_depth(min_depth);
            }
            if let Some(max_depth) = opts.max_depth {
                glob_walker_builder = glob_walker_builder.max_depth(max_depth);
            }
            glob_walker_builder = glob_walker_builder.follow_links(opts.follow_links);
            glob_walker_builder = glob_walker_builder.case_insensitive(opts.case_insensitive);
        }
        let paths_iter = glob_walker_builder
            .build()?
            .filter_map(Result::ok)
            .map(globwalk::DirEntry::into_path);
        Ok(paths_iter)
    }
}

impl PartialEq for Session {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for Session {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Session {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.cmp(&other.path)
    }
}

impl mlua::UserData for Session {
    fn add_fields<'lua, F: mlua::prelude::LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("path", |_, this| {
            Ok(this.path.to_string_lossy().to_string())
        });
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("new_window", |_, this, ()| {
            this.new_window().map_err(mlua::Error::external)
        });

        methods.add_method("current_window", |_, this, ()| {
            this.current_window().map_err(mlua::Error::external)
        });

        methods.add_method(
            "globs",
            |lua, this, (patterns, opts_val): (Vec<String>, mlua::Value)| {
                let opts = lua.from_value::<Option<GlobsOpts>>(opts_val)?;
                let paths = this
                    .globs(&patterns, opts)
                    .map_err(mlua::Error::external)?
                    .map(|path| path.to_string_lossy().to_string())
                    .collect::<Vec<_>>();
                Ok(paths)
            },
        );
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct GlobsOpts {
    min_depth: Option<usize>,
    max_depth: Option<usize>,
    follow_links: bool,
    case_insensitive: bool,
}
