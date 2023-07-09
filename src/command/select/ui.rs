use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use unicode_segmentation::UnicodeSegmentation;

use crate::config::Config;

use super::state::State;

pub fn draw(config: &Config, state: &mut State, frame: &mut Frame) {
    let mut constraints = [Constraint::Percentage(100), Constraint::Min(3)];
    if config.session_selector.inverted {
        constraints.reverse();
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(frame.size());

    draw_results(
        config,
        state,
        frame,
        if config.session_selector.inverted {
            chunks[1]
        } else {
            chunks[0]
        },
    );
    draw_prompt(
        config,
        state,
        frame,
        if config.session_selector.inverted {
            chunks[0]
        } else {
            chunks[1]
        },
    );
}

fn draw_results(config: &Config, state: &mut State, frame: &mut Frame, chunk: Rect) {
    state.adjust_scroll_pos(chunk.height as usize, config.session_selector.scrolloff);

    let items = state
        .visible_matcher_results(chunk.height as usize)
        .iter()
        .filter_map(|(i, _, char_indices)| {
            state
                .get_session_by_index(*i)
                .map(|session| (session.path_pretty().unwrap_or_default(), char_indices))
        })
        .enumerate()
        .map(|(i, (session_path, char_indices))| {
            get_results_item(config, session_path, char_indices, state.is_selected(i))
        })
        .collect::<Vec<_>>();
    let block = Block::new()
        .style(config.session_selector.results.style)
        .borders(Borders::ALL)
        .border_set(config.session_selector.results.border)
        .border_style(config.session_selector.results.border_style)
        .title(config.session_selector.results.title.clone())
        .title_alignment(Alignment::Center)
        .title_style(config.session_selector.results.title_style);
    let widget = List::new(items)
        .block(block)
        .start_corner(if config.session_selector.inverted {
            Corner::TopLeft
        } else {
            Corner::BottomLeft
        });

    frame.render_widget(widget, chunk);
}

fn get_results_item<'a>(
    config: &'a Config,
    session_path: &'a str,
    char_indices: &'a [usize],
    is_selected: bool,
) -> ListItem<'a> {
    let mut spans = Vec::with_capacity(session_path.len() + 1);

    spans.push(if is_selected {
        Span::styled(
            config.session_selector.results.selection_prefix.clone(),
            config.session_selector.results.selection_prefix_style,
        )
    } else {
        let selection_prefix_len = config
            .session_selector
            .results
            .selection_prefix
            .graphemes(true)
            .count();
        Span::raw(String::from(' ').repeat(selection_prefix_len))
    });

    for (i, c) in session_path.chars().enumerate() {
        let mut style = config.session_selector.results.item_style;
        if is_selected {
            style = style.patch(config.session_selector.results.selection_style);
        }
        if char_indices.binary_search(&i).is_ok() {
            style = style.patch(config.session_selector.results.item_match_style);
        }
        spans.push(Span::styled(String::from(c), style));
    }

    let mut widget = ListItem::new(Line::from(spans));
    if is_selected {
        widget = widget.style(config.session_selector.results.selection_style);
    }
    widget
}

fn draw_prompt(config: &Config, state: &State, frame: &mut Frame, chunk: Rect) {
    let line = Line::from(vec![
        Span::styled(
            config.session_selector.prompt.pattern_prefix.clone(),
            config
                .session_selector
                .prompt
                .pattern_style
                .patch(config.session_selector.prompt.pattern_prefix_style),
        ),
        Span::styled(
            state.pattern_string(),
            config.session_selector.prompt.pattern_style,
        ),
    ]);
    let block = Block::default()
        .style(config.session_selector.prompt.style)
        .borders(Borders::ALL)
        .border_set(config.session_selector.prompt.border)
        .border_style(config.session_selector.prompt.border_style)
        .title(config.session_selector.prompt.title.clone())
        .title_alignment(Alignment::Center)
        .title_style(config.session_selector.prompt.title_style);
    let widget = Paragraph::new(line).block(block);

    frame.render_widget(widget, chunk);

    let pattern_prefix_len = config
        .session_selector
        .prompt
        .pattern_prefix
        .graphemes(true)
        .count();
    #[allow(clippy::cast_possible_truncation)]
    frame.set_cursor(
        chunk.x + 1 + pattern_prefix_len as u16 + state.cursor_pos() as u16,
        chunk.y + 1,
    );
}
