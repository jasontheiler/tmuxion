use crate::tmux::{self, Session};

pub fn last() -> anyhow::Result<()> {
    tmux::assert_in_session()?;

    let last_session_opt = Session::last()?;
    let current_session = Session::current()?;
    if let Some(last_session) = last_session_opt {
        last_session.switch_to()?;
        current_session.save_as_last()?;
    }

    Ok(())
}
