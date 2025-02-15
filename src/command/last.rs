use crate::{
    args::Args,
    tmux::{self, Session},
};

pub fn last(args: &Args) -> anyhow::Result<()> {
    tmux::assert_in_session()?;

    let last_session_opt = Session::last()?;
    let current_session = Session::current(args.target_client.as_ref())?;
    if let Some(last_session) = last_session_opt {
        current_session.save_as_last()?;
        last_session.switch_to(args.target_client.as_ref())?;
    }

    Ok(())
}
