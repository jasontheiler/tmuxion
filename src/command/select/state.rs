use crate::{args::Args, config::Config, tmux::Session};

pub struct State<'a> {
    args: &'a Args,
    initial_session_opt: Option<Session>,
    sessions: Vec<Session>,
    session_paths: Vec<String>,
    pattern: Vec<char>,
    matches: Vec<(usize, Vec<usize>)>,
    cursor_pos: usize,
    scroll_pos: usize,
    selection_pos: usize,
}

impl<'a> State<'a> {
    pub fn new(args: &'a Args, config: &Config) -> anyhow::Result<Self> {
        let initial_session_opt = Session::current(args.target_client.as_ref())?;
        let sessions = Session::all()?;
        let sessions_map_fn = |session: &Session| {
            let mut path = String::new();
            match session
                .path()
                .strip_prefix(std::env::home_dir().unwrap_or_default())
            {
                Ok(path_stripped) if config.session_selector.paths.truncate_home_dir => {
                    path.push_str(&config.session_selector.paths.home_dir_symbol);
                    path.push('/');
                    path.push_str(&path_stripped.to_string_lossy());
                }
                _ => path.push_str(&session.path().to_string_lossy()),
            }
            if config.session_selector.paths.trailing_slash {
                path.push('/');
            }
            path
        };
        let session_paths = sessions.iter().map(sessions_map_fn).collect::<Vec<_>>();
        let matches = session_paths
            .iter()
            .enumerate()
            .map(|(i, _)| (i, Vec::new()))
            .collect();
        Ok(Self {
            args,
            initial_session_opt,
            sessions,
            session_paths,
            pattern: Vec::new(),
            matches,
            cursor_pos: 0,
            scroll_pos: 0,
            selection_pos: 0,
        })
    }

    pub fn sessions_len(&self) -> usize {
        self.sessions.len()
    }

    pub fn get_session_path_by_index(&self, i: usize) -> Option<&String> {
        self.session_paths.get(i)
    }

    pub fn pattern_string(&self) -> String {
        self.pattern.iter().collect()
    }

    pub fn cursor_pos(&self) -> usize {
        self.cursor_pos
    }

    pub fn matches_len(&self) -> usize {
        self.matches.len()
    }

    pub fn visible_matches(&self, count: usize) -> &[(usize, Vec<usize>)] {
        let end = (self.scroll_pos + count).min(self.matches.len());
        &self.matches[self.scroll_pos..end]
    }

    pub fn adjust_scroll_pos(&mut self, item_count: usize, mut scrolloff: usize) {
        if item_count == 0 || item_count > self.matches.len() {
            return;
        }

        scrolloff = scrolloff.min((item_count - 1) / 2);

        if self.selection_pos < scrolloff {
            self.scroll_pos = 0;
        } else if self.selection_pos >= self.matches.len() - scrolloff {
            self.scroll_pos = self.matches.len() - item_count;
        } else if self.selection_pos < self.scroll_pos + scrolloff {
            self.scroll_pos -= 1;
        } else if self.selection_pos >= self.scroll_pos + item_count - scrolloff {
            self.scroll_pos += 1;
        }
    }

    pub fn cursor_backward(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    pub fn cursor_forward(&mut self) {
        if self.cursor_pos < self.pattern.len() {
            self.cursor_pos += 1;
        }
    }

    pub fn char_add(&mut self, char: char) -> anyhow::Result<()> {
        if self.cursor_pos > self.pattern.len() {
            self.pattern.push(char);
        } else {
            self.pattern.insert(self.cursor_pos, char);
        }
        self.cursor_pos += 1;
        self.match_sessions()
    }

    pub fn char_delete_backward(&mut self) -> anyhow::Result<()> {
        if self.cursor_pos > 0 {
            self.pattern.remove(self.cursor_pos - 1);
            self.cursor_pos -= 1;
            self.match_sessions()?;
        }
        Ok(())
    }

    pub fn char_delete_forward(&mut self) -> anyhow::Result<()> {
        if self.cursor_pos < self.pattern.len() {
            self.pattern.remove(self.cursor_pos);
            self.match_sessions()?;
        }
        Ok(())
    }

    pub fn selection_prev(&mut self) -> anyhow::Result<()> {
        let matcher_results_len = self.matches.len();
        self.selection_pos = (self.selection_pos + matcher_results_len - 1) % matcher_results_len;
        self.switch_session(false)
    }

    pub fn selection_next(&mut self) -> anyhow::Result<()> {
        self.selection_pos = (self.selection_pos + 1) % self.matches.len();
        self.switch_session(false)
    }

    pub fn is_selected(&self, i: usize) -> bool {
        i == self.selection_pos - self.scroll_pos
    }

    pub fn confirm(&self) -> anyhow::Result<bool> {
        self.switch_session(true)?;
        Ok(!self.matches.is_empty())
    }

    pub fn abort(&self) -> anyhow::Result<()> {
        if let Some(initial_session) = &self.initial_session_opt {
            return initial_session.switch_to(self.args.target_client.as_ref());
        }
        Ok(())
    }

    fn match_sessions(&mut self) -> anyhow::Result<()> {
        self.scroll_pos = 0;
        self.selection_pos = 0;

        self.matches = frizbee::match_list(
            self.pattern_string(),
            &self.session_paths,
            #[allow(clippy::cast_possible_truncation)]
            frizbee::Options {
                min_score: self.pattern.len() as u16 * 6,
                max_typos: Some(self.pattern.len() as u16 / 4),
                matched_indices: true,
                ..Default::default()
            },
        )
        .iter()
        .map(|m| (m.index_in_haystack, m.indices.clone().unwrap_or_default()))
        .collect();
        self.switch_session(false)
    }

    fn switch_session(&self, save_initial_as_last: bool) -> anyhow::Result<()> {
        let Some(selected_session) = self.get_selected_session()? else {
            return Ok(());
        };
        if save_initial_as_last
            && let Some(initial_session) = &self.initial_session_opt
            && initial_session != selected_session
        {
            initial_session.save_as_last()?;
        }
        selected_session.switch_to(self.args.target_client.as_ref())?;
        Ok(())
    }

    fn get_selected_session(&self) -> anyhow::Result<Option<&Session>> {
        if self.matches.is_empty() {
            return Ok(None);
        }
        let (i, ..) = self
            .matches
            .get(self.selection_pos)
            .ok_or(anyhow::format_err!("selected match result does not exist"))?;
        let session = self
            .sessions
            .get(*i)
            .ok_or(anyhow::format_err!("selected session does not exist"))?;
        Ok(Some(session))
    }
}
