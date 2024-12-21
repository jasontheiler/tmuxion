mod input;
mod state;
mod ui;

use ratatui::{
    crossterm::{
        self,
        cursor::SetCursorStyle,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    },
    prelude::*,
};

use crate::{args::Args, config::Config, tmux};

use self::state::State;

pub fn select(args: &Args, config: &Config) -> anyhow::Result<()> {
    tmux::assert_in_session()?;

    let mut state = State::new(args, config)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;

    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        SetCursorStyle::SteadyBar,
    )?;

    let res = run(config, &mut state, &mut terminal);

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        SetCursorStyle::DefaultUserShape,
    )?;

    res
}

fn run<B>(config: &Config, state: &mut State, terminal: &mut Terminal<B>) -> anyhow::Result<()>
where
    B: Backend,
{
    loop {
        terminal.try_draw(|frame| ui::draw(config, state, frame))?;
        if input::process(config, state)? {
            return Ok(());
        }
    }
}
