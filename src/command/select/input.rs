use ratatui::crossterm::{
    self,
    event::{Event, KeyCode, KeyEventKind, KeyModifiers},
};

use crate::config::Config;

use super::state::State;

pub fn process(config: &Config, state: &mut State) -> anyhow::Result<bool> {
    let Event::Key(key) = crossterm::event::read()? else {
        return Ok(false);
    };
    if key.kind != KeyEventKind::Press {
        return Ok(false);
    }

    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('j')) | (_, KeyCode::Down) => {
            if config.session_selector.inverted {
                state.selection_next()?;
            } else {
                state.selection_prev()?;
            }
        }
        (KeyModifiers::CONTROL, KeyCode::Char('k')) | (_, KeyCode::Up) => {
            if config.session_selector.inverted {
                state.selection_prev()?;
            } else {
                state.selection_next()?;
            }
        }
        (KeyModifiers::CONTROL, KeyCode::Char('h')) | (_, KeyCode::Left) => {
            state.cursor_backward();
        }
        (KeyModifiers::CONTROL, KeyCode::Char('l')) | (_, KeyCode::Right) => {
            state.cursor_forward();
        }
        (_, KeyCode::Tab) => {
            state.selection_next()?;
        }
        (_, KeyCode::BackTab) => {
            state.selection_prev()?;
        }
        (_, KeyCode::Char(char)) => {
            state.char_add(char)?;
        }
        (_, KeyCode::Backspace) => {
            state.char_delete_backward()?;
        }
        (_, KeyCode::Delete) => {
            state.char_delete_forward()?;
        }
        (_, KeyCode::Esc) => {
            return state.abort().map(|()| true);
        }
        (_, KeyCode::Enter) => {
            return state.confirm();
        }
        _ => (),
    }

    Ok(false)
}
