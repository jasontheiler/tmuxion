use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListDirection, ListItem},
};
use unicode_segmentation::UnicodeSegmentation as _;

use crate::config::Config;

use super::state::State;

pub fn draw(config: &Config, state: &mut State, frame: &mut Frame) {
    let mut constraints = [Constraint::Percentage(100), Constraint::Min(3)];
    if config.session_selector.inverted {
        constraints.reverse();
    };

    let layout = Layout::vertical(constraints).split(frame.size());

    draw_results(
        config,
        state,
        frame,
        if config.session_selector.inverted {
            layout[1]
        } else {
            layout[0]
        },
    );
    draw_prompt(
        config,
        state,
        frame,
        if config.session_selector.inverted {
            layout[0]
        } else {
            layout[1]
        },
    );
}

fn draw_results(config: &Config, state: &mut State, frame: &mut Frame, area: Rect) {
    state.adjust_scroll_pos(area.height as usize, config.session_selector.scrolloff);

    let items = state
        .visible_matcher_results(area.height as usize)
        .iter()
        .filter_map(|(i, _, char_indices)| {
            state
                .get_session_by_index(*i)
                .map(|session| (session.path_str(), char_indices))
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
    let list = List::new(items)
        .block(block)
        .direction(if config.session_selector.inverted {
            ListDirection::TopToBottom
        } else {
            ListDirection::BottomToTop
        });
    frame.render_widget(list, area);
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

    let mut list_item = ListItem::new(Line::from(spans));
    if is_selected {
        list_item = list_item.style(config.session_selector.results.selection_style);
    }
    list_item
}

fn draw_prompt(config: &Config, state: &State, frame: &mut Frame, area: Rect) {
    let block = Block::new()
        .style(config.session_selector.prompt.style)
        .borders(Borders::ALL)
        .border_set(config.session_selector.prompt.border)
        .border_style(config.session_selector.prompt.border_style)
        .title(config.session_selector.prompt.title.clone())
        .title_alignment(Alignment::Center)
        .title_style(config.session_selector.prompt.title_style);
    let block_inner = block.inner(area);
    frame.render_widget(block, area);

    let stats = config
        .session_selector
        .prompt
        .stats_template
        .replace("{results}", &state.matcher_results_len().to_string())
        .replace("{sessions}", &state.sessions_len().to_string());

    let pattern_prefix_len = config
        .session_selector
        .prompt
        .pattern_prefix
        .graphemes(true)
        .count();
    let stats_len = stats.graphemes(true).count();
    let layout = Layout::horizontal([
        #[allow(clippy::cast_possible_truncation)]
        Constraint::Min(pattern_prefix_len as u16),
        Constraint::Percentage(100),
        #[allow(clippy::cast_possible_truncation)]
        Constraint::Min(stats_len as u16),
    ])
    .split(block_inner);

    let span_pattern_prefix = Span::styled(
        config.session_selector.prompt.pattern_prefix.clone(),
        config
            .session_selector
            .prompt
            .pattern_style
            .patch(config.session_selector.prompt.pattern_prefix_style),
    );
    frame.render_widget(span_pattern_prefix, layout[0]);

    let span_pattern = Span::styled(
        state.pattern_string(),
        config.session_selector.prompt.pattern_style,
    );
    frame.render_widget(span_pattern, layout[1]);

    let span_stats = Span::styled(
        stats,
        config
            .session_selector
            .prompt
            .pattern_style
            .patch(config.session_selector.prompt.stats_style),
    );
    frame.render_widget(span_stats, layout[2]);

    #[allow(clippy::cast_possible_truncation)]
    frame.set_cursor(
        layout[1].x + layout[1].width.min(state.cursor_pos() as u16),
        layout[1].y,
    );
}
