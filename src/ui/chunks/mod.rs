use rustyline::line_buffer::LineBuffer;

use tui::{
    backend::Backend,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    ui::LayoutAttributes,
    utils::text::{get_cursor_position, title_spans, TitleStyle},
};

pub mod chatting;
pub mod message_search;

/// Puts a box for user input at the bottom of the screen,
/// with an interactive cursor.
/// input_validation checks if the user's input is valid, changes window
/// theme to red if invalid, default otherwise.
pub fn insert_box_chunk<T: Backend>(
    frame: &mut Frame<T>,
    layout: LayoutAttributes,
    input_buffer: &LineBuffer,
    suggestion: Option<String>,
    input_validation: Option<Box<dyn FnOnce(String) -> bool>>,
) {
    let cursor_pos = get_cursor_position(input_buffer);
    let input_rect = layout.chunks[layout.constraints.len() - 1];

    frame.set_cursor(
        (input_rect.x + cursor_pos as u16 + 1)
            .min(input_rect.x + input_rect.width.saturating_sub(2)),
        input_rect.y + 1,
    );

    let current_input = input_buffer.as_str();

    let valid_input = if let Some(check_func) = input_validation {
        check_func(current_input.to_string())
    } else {
        true
    };

    let paragraph = Paragraph::new(Spans::from(vec![
        Span::raw(input_buffer.as_str()),
        Span::styled(
            if let Some(suggestion_buffer) = suggestion {
                suggestion_buffer
            } else {
                "".to_string()
            },
            Style::default().add_modifier(Modifier::DIM),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(title_spans(
                vec![TitleStyle::Single("Message input")],
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ))
            .border_style(Style::default().fg(if valid_input {
                Color::Yellow
            } else {
                Color::Red
            })),
    )
    .scroll((
        0,
        ((cursor_pos + 3) as u16).saturating_sub(input_rect.width),
    ));

    frame.render_widget(paragraph, input_rect);
}
