use crate::{config::Config, tmux::{self, Session}};

pub fn last(config: &Config) -> anyhow::Result<()> {
    tmux::assert_in_session()?;

    let last_session_opt = Session::last(config)?;
    let current_session = Session::current(config)?;
    if let Some(last_session) = last_session_opt {
        current_session.save_as_last()?;
        last_session.switch_to()?;
    }

    Ok(())
}
