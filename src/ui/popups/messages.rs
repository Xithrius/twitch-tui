use std::collections::VecDeque;

use tui::{
    backend::Backend,
    layout::Constraint,
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph, Row, Table},
};

use fuzzy_matcher::skim::SkimMatcherV2;
use lazy_static::lazy_static;

use crate::{
    handlers::{app::App, data::PayLoad},
    ui::popups::{centered_popup, scroll_view, Centering},
    utils::{styles, text::get_cursor_position},
};
use fuzzy_matcher::FuzzyMatcher;

const MAX_MESSAGE_SEARCH: u16 = 10;

lazy_static! {
    pub static ref FUZZY_FINDER: SkimMatcherV2 = SkimMatcherV2::default();
}

pub fn search_messages<T: Backend>(frame: &mut Frame<T>, app: &mut App) {
    let input_rect = centered_popup(Centering::Input(Some(MAX_MESSAGE_SEARCH)), frame.size());
    let window_rect = centered_popup(Centering::Window(MAX_MESSAGE_SEARCH), frame.size());

    let input_buffer = app.current_buffer();

    let cursor_pos = get_cursor_position(input_buffer);

    frame.set_cursor(
        (input_rect.x + cursor_pos as u16 + 1)
            .min(input_rect.x + input_rect.width.saturating_sub(2)),
        input_rect.y + 1,
    );

    let input_text = &input_buffer.as_str();

    let input_paragraph = Paragraph::new(*input_text)
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("[ Message Search ]"),
        )
        .scroll((
            0,
            ((cursor_pos + 3) as u16).saturating_sub(input_rect.width),
        ));

    frame.render_widget(Clear, input_rect);
    frame.render_widget(input_paragraph, input_rect);

    let all_messages = app
        .messages_snapshot
        .iter()
        .flat_map(|f| match &f.payload {
            PayLoad::Message(m) => Some(m.as_str()),
            _ => None,
        })
        .collect::<VecDeque<&str>>();

    if all_messages.is_empty() {
        let window_paragraph = Table::new(vec![])
            .block(Block::default().borders(Borders::ALL).title("[ Results ]"))
            .column_spacing(2)
            .style(styles::BORDER_NAME);

        frame.render_widget(Clear, window_rect);
        frame.render_widget(window_paragraph, window_rect);

        return;
    }

    let maximum_message_length = *all_messages
        .iter()
        .map(|v| v.len())
        .collect::<Vec<usize>>()
        .iter()
        .max()
        .unwrap() as u16;

    let table_widths = all_messages
        .iter()
        .map(|_| Constraint::Min(maximum_message_length))
        .collect::<Vec<Constraint>>();

    let render_messages = scroll_view(
        all_messages,
        app.scroll_offset,
        MAX_MESSAGE_SEARCH as usize + 2,
    )
    .iter()
    .flat_map(|&f| {
        let chars = f.chars();

        if let Some((_, indices)) = FUZZY_FINDER.fuzzy_indices(f, input_text) {
            Some(Row::new(vec![Spans::from(
                chars
                    .enumerate()
                    .map(|(i, s)| {
                        if indices.contains(&i) {
                            Span::styled(
                                s.to_string(),
                                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                            )
                        } else {
                            Span::raw(s.to_string())
                        }
                    })
                    .collect::<Vec<Span>>(),
            )]))
        } else {
            None
        }
    })
    .collect::<Vec<Row>>();

    let window_paragraph = Table::new(render_messages)
        .block(Block::default().borders(Borders::ALL).title("[ Results ]"))
        .column_spacing(2)
        .widths(&table_widths)
        .style(styles::BORDER_NAME);

    frame.render_widget(Clear, window_rect);
    frame.render_widget(window_paragraph, window_rect);
}
