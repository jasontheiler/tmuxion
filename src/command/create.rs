use crate::{
    args,
    config::Config,
    tmux::{self, Session},
};

pub fn create(config: &Config, args: &args::Create) -> anyhow::Result<()> {
    let current_session_opt = Session::current(config).ok();

    let mut paths = args
        .paths
        .iter()
        .map(|path| path.canonicalize())
        .collect::<Result<Vec<_>, _>>()?;
    if paths.is_empty() {
        paths.push(std::env::current_dir()?);
    }

    let mut session_to_switch_to_opt = Option::<Session>::None;
    for path in &paths {
        let (session, has_existed) = Session::new(config, path)?;
        if has_existed {
            if session_to_switch_to_opt.is_none() {
                session_to_switch_to_opt = Some(session);
            }
            continue;
        }
        session_to_switch_to_opt = Some(session.clone());

        tmux::set_up(config)?;

        if let Some(on_session_created) = &config.on_session_created {
            on_session_created.call(session)?;
        };
    }

    if let Some(session_to_switch_to) = session_to_switch_to_opt {
        if let Some(current_session) = current_session_opt {
            if current_session != session_to_switch_to {
                current_session.save_as_last()?;
            }
        }
        session_to_switch_to.switch_to()?;
    }

    Ok(())
}
