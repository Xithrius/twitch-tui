use rustyline::line_buffer::LineBuffer;

use tui::{
    backend::Backend,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    ui::{statics::TWITCH_MESSAGE_LIMIT, LayoutAttributes},
    utils::text::{get_cursor_position, title_spans},
};

pub mod chatting;
pub mod message_search;

// TODO: Have closure checker for inputted text.
pub fn insert_box_chunk<T: Backend>(
    frame: &mut Frame<T>,
    layout: LayoutAttributes,
    input_buffer: LineBuffer,
    suggestion: Option<String>,
) {
    let cursor_pos = get_cursor_position(&input_buffer);
    let input_rect = layout.chunks[layout.constraints.len() - 1];

    frame.set_cursor(
        (input_rect.x + cursor_pos as u16 + 1)
            .min(input_rect.x + input_rect.width.saturating_sub(2)),
        input_rect.y + 1,
    );

    let suggestion_buffer = if let Some(s) = suggestion {
        if s.len() > input_buffer.as_str().len() {
            &s[input_buffer.as_str().len()..]
        } else {
            ""
        }
    } else {
        ""
    };

    let current_input = input_buffer.as_str();

    let paragraph = Paragraph::new(Spans::from(vec![
        Span::raw(input_buffer.as_str()),
        Span::styled(
            suggestion_buffer,
            Style::default().add_modifier(Modifier::DIM),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(title_spans(
                vec![vec![
                    "Message limit",
                    format!("{} / {}", current_input.len(), *TWITCH_MESSAGE_LIMIT).as_str(),
                ]],
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ))
            .border_style(
                Style::default().fg(if current_input.len() > *TWITCH_MESSAGE_LIMIT {
                    Color::Red
                } else {
                    Color::Yellow
                }),
            ),
    )
    .scroll((
        0,
        ((cursor_pos + 3) as u16).saturating_sub(input_rect.width),
    ));

    frame.render_widget(paragraph, input_rect);
}
