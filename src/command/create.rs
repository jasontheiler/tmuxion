use std::path::PathBuf;

use crate::{
    args,
    config::Config,
    tmux::{self, Session},
};

pub fn create(config: &Config, args: &args::Create) -> anyhow::Result<()> {
    let current_session_opt = Session::current().ok();

    let paths_map_fn = |path: &PathBuf| {
        if !path.try_exists()? {
            if !args.create_dirs {
                anyhow::bail!("path '{}' does not exist", path.to_string_lossy());
            }
            std::fs::create_dir_all(path)?;
        }
        if path.is_file() {
            anyhow::bail!("path '{}' points to a file", path.to_string_lossy());
        }
        let path = path.canonicalize()?;
        Ok(path)
    };
    let mut paths = args
        .paths
        .iter()
        .map(paths_map_fn)
        .collect::<Result<Vec<_>, _>>()?;
    if paths.is_empty() {
        paths.push(std::env::current_dir()?);
    }

    let mut session_to_switch_to_opt = Option::<Session>::None;
    for path in &paths {
        let (session, has_existed) = Session::new(path)?;

        tmux::set_up(config)?;

        if args.detached {
            continue;
        }
        if has_existed {
            if session_to_switch_to_opt.is_none() {
                session_to_switch_to_opt = Some(session);
            }
            continue;
        }
        session_to_switch_to_opt = Some(session);
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
