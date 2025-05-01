use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use serde::{Deserialize, Serialize};
use tmux_interface::{
    AttachSession, DisplayMessage, HasSession, ListSessions, NewSession, RenameSession,
    SwitchClient, Tmux,
};

use crate::{APP_NAME, tmux};

const FORMAT: &str =
    r##"{"id":"#{session_id}","name":"#{session_name}","path":"#{session_path}"}"##;
static NAME_PREFIX: LazyLock<String> = LazyLock::new(|| format!("{APP_NAME}_"));
static LAST_SESSION_FILE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    std::env::home_dir()
        .unwrap_or_default()
        .join(".cache")
        .join(APP_NAME)
        .join("last_session.json")
});

#[derive(Debug, Clone, Eq, Deserialize, Serialize)]
pub struct Session {
    id: String,
    #[serde(default, skip_serializing)]
    name: String,
    #[serde(default, skip_serializing)]
    path: PathBuf,
}

impl Session {
    pub fn new(path: &Path) -> anyhow::Result<(Self, bool)> {
        let sessions = Self::all()?;
        let sessions_find_fn = |session: &Session| {
            let Ok(session_path) = session.path.canonicalize() else {
                return false;
            };
            session_path == path
        };
        let session_opt = sessions.into_iter().find(sessions_find_fn);
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
        let session = serde_json::from_str::<Self>(&output.to_string())?;
        Tmux::with_command(
            RenameSession::new()
                .target_session(&session.id)
                .new_name(format!("{}{}", NAME_PREFIX.as_str(), session.id)),
        )
        .status()?;
        Ok((session, false))
    }

    pub fn current(target_client_opt: Option<&String>) -> anyhow::Result<Option<Self>> {
        let mut display_message = DisplayMessage::new().message(FORMAT).print();
        if let Some(target_client) = target_client_opt {
            // For the `display-message` command the `target-client` option only
            // controls in which client's status line the message is displayed
            // if the output is not printed to stdout.
            display_message = display_message.target_pane(target_client);
        }
        let output = Tmux::with_command(display_message).output()?;
        let session = serde_json::from_str::<Self>(&output.to_string())?;
        Ok(session
            .name
            .starts_with(NAME_PREFIX.as_str())
            .then_some(session))
    }

    pub fn last() -> anyhow::Result<Option<Self>> {
        let content = match std::fs::read_to_string(LAST_SESSION_FILE_PATH.as_path()) {
            Ok(content) => content,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => anyhow::bail!(err),
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
            .collect::<Result<Vec<Self>, _>>()?
            .into_iter()
            .filter(|session| session.name.starts_with(NAME_PREFIX.as_str()))
            .collect::<Vec<_>>();
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

    pub fn switch_to(&self, target_client_opt: Option<&String>) -> anyhow::Result<()> {
        if tmux::assert_in_session().is_ok() {
            let mut switch_client = SwitchClient::new().target_session(&self.id);
            if let Some(target_client) = target_client_opt {
                switch_client = switch_client.target_client(target_client);
            }
            Tmux::with_command(switch_client).status()?;
        } else {
            Tmux::with_command(AttachSession::new().target_session(&self.id)).status()?;
        }
        Ok(())
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
