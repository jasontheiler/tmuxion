use fuzzy_matcher::{FuzzyMatcher as _, skim::SkimMatcherV2};

use crate::{args::Args, config::Config, tmux::Session};

pub struct State<'a> {
    args: &'a Args,
    initial_session_opt: Option<Session>,
    sessions: Vec<Session>,
    session_paths: Vec<String>,
    pattern: Vec<char>,
    cursor_pos: usize,
    matcher: SkimMatcherV2,
    matcher_results: Vec<(usize, i64, Vec<usize>)>,
    scroll_pos: usize,
    selection_pos: usize,
}

impl<'a> State<'a> {
    pub fn new(args: &'a Args, config: &Config) -> anyhow::Result<Self> {
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
        let session_paths = sessions.iter().map(sessions_map_fn).collect();
        let matcher_results = sessions
            .iter()
            .enumerate()
            .map(|(i, ..)| (i, 0, Vec::new()))
            .collect();
        Ok(Self {
            args,
            initial_session_opt: Session::current(args.target_client.as_ref())?,
            sessions,
            session_paths,
            pattern: Vec::new(),
            cursor_pos: 0,
            matcher: SkimMatcherV2::default(),
            matcher_results,
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

    pub fn matcher_results_len(&self) -> usize {
        self.matcher_results.len()
    }

    pub fn visible_matcher_results(&self, count: usize) -> &[(usize, i64, Vec<usize>)] {
        let end = (self.scroll_pos + count).min(self.matcher_results.len());
        &self.matcher_results[self.scroll_pos..end]
    }

    pub fn adjust_scroll_pos(&mut self, item_count: usize, mut scrolloff: usize) {
        if item_count == 0 || item_count > self.matcher_results.len() {
            return;
        }

        scrolloff = scrolloff.min((item_count - 1) / 2);

        if self.selection_pos < scrolloff {
            self.scroll_pos = 0;
        } else if self.selection_pos >= self.matcher_results.len() - scrolloff {
            self.scroll_pos = self.matcher_results.len() - item_count;
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
        let matcher_results_len = self.matcher_results.len();
        self.selection_pos = (self.selection_pos + matcher_results_len - 1) % matcher_results_len;
        self.switch_session(false)
    }

    pub fn selection_next(&mut self) -> anyhow::Result<()> {
        self.selection_pos = (self.selection_pos + 1) % self.matcher_results.len();
        self.switch_session(false)
    }

    pub fn is_selected(&self, i: usize) -> bool {
        i == self.selection_pos - self.scroll_pos
    }

    pub fn confirm(&self) -> anyhow::Result<bool> {
        self.switch_session(true)?;
        Ok(!self.matcher_results.is_empty())
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

        let pattern = self.pattern_string();
        let session_paths_filter_map_fn = |(i, session_path): (usize, &String)| {
            self.matcher
                .fuzzy_indices(session_path, &pattern)
                .map(|(score, matched_indices)| (i, score, matched_indices))
        };
        self.matcher_results = self
            .session_paths
            .iter()
            .enumerate()
            .filter_map(session_paths_filter_map_fn)
            .collect::<Vec<_>>();
        self.matcher_results
            .sort_by(|(_, a_score, ..), (_, b_score, ..)| b_score.cmp(a_score));
        self.switch_session(false)
    }

    fn switch_session(&self, save_initial_as_last: bool) -> anyhow::Result<()> {
        let Some(selected_session) = self.get_selected_session()? else {
            return Ok(());
        };
        if save_initial_as_last {
            if let Some(initial_session) = &self.initial_session_opt {
                if initial_session != selected_session {
                    initial_session.save_as_last()?;
                }
            }
        }
        selected_session.switch_to(self.args.target_client.as_ref())?;
        Ok(())
    }

    fn get_selected_session(&self) -> anyhow::Result<Option<&Session>> {
        if self.matcher_results.is_empty() {
            return Ok(None);
        }
        let (i, ..) = self
            .matcher_results
            .get(self.selection_pos)
            .ok_or(anyhow::format_err!("selected match result does not exist"))?;
        let session = self
            .sessions
            .get(*i)
            .ok_or(anyhow::format_err!("selected session does not exist"))?;
        Ok(Some(session))
    }
}
