use std::path::PathBuf;

use crate::{
    args,
    config::Config,
    tmux::{self, Session},
};

pub fn create(config: &Config, args: &args::Create) -> anyhow::Result<()> {
    let current_session_opt = Session::current(config).ok();

    let path_map_fn = |path: &PathBuf| {
        if !path.try_exists()? {
            if !args.create_dirs {
                anyhow::bail!("path `{}` does not exist", path.to_string_lossy());
            }
            std::fs::create_dir_all(path)?;
        }
        if path.is_file() {
            anyhow::bail!("path `{}` is a file", path.to_string_lossy());
        }
        let path = path.canonicalize()?;
        Ok(path)
    };
    let mut paths = args
        .paths
        .iter()
        .map(path_map_fn)
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
