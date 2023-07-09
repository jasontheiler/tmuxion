mod input;
mod state;
mod ui;

use crossterm::{
    cursor::SetCursorStyle,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use crate::{config::Config, tmux};

use self::state::State;

pub fn select(config: &Config) -> anyhow::Result<()> {
    tmux::assert_in_session()?;

    let mut state = State::new(config)?;

    crossterm::terminal::enable_raw_mode()?;
    let stdout = std::io::stdout();
    let terminal_backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(terminal_backend)?;
    crossterm::execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        EnableMouseCapture,
        SetCursorStyle::SteadyBar
    )?;
    terminal.hide_cursor()?;

    let res = run(config, &mut state, &mut terminal);

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        SetCursorStyle::DefaultUserShape
    )?;
    terminal.show_cursor()?;

    res
}

fn run<B>(config: &Config, state: &mut State, terminal: &mut Terminal<B>) -> anyhow::Result<()>
where
    B: Backend,
{
    loop {
        terminal.draw(|frame| ui::draw(config, state, frame))?;
        if input::process(config, state)? {
            return Ok(());
        }
    }
}
