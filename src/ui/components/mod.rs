use std::vec;

use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{
    handlers::app::State,
    ui::WindowAttributes,
    utils::text::{get_cursor_position, title_spans, TitleStyle},
};

pub mod chunks;
pub mod popups;

pub use chunks::{chatting::render_chat_box, help::render_help_window, states::render_state_tabs};
pub use popups::channels::render_channel_switcher;

/// Puts a box for user input at the bottom of the screen,
/// with an interactive cursor.
/// `input_validation` checks if the user's input is valid, changes window
/// theme to red if invalid, default otherwise.
pub fn render_insert_box<T: Backend>(
    window: WindowAttributes<T>,
    box_title: &str,
    input_rectangle: Option<Rect>,
    suggestion: Option<String>,
    input_validation: Option<Box<dyn FnOnce(String) -> bool>>,
) {
    let WindowAttributes { frame, layout, app } = window;

    let buffer = &app.input_buffer;

    let cursor_pos = get_cursor_position(buffer);

    let input_rect = if let Some(r) = input_rectangle {
        r
    } else {
        layout.chunks[layout.constraints.len() - 2]
    };

    frame.set_cursor(
        (input_rect.x + cursor_pos as u16 + 1)
            .min(input_rect.x + input_rect.width.saturating_sub(2)),
        input_rect.y + 1,
    );

    let current_input = buffer.as_str();

    let valid_input =
        input_validation.map_or(true, |check_func| check_func(current_input.to_string()));

    let paragraph = Paragraph::new(Spans::from(vec![
        Span::raw(current_input),
        Span::styled(
            suggestion.clone().map_or_else(
                || "".to_string(),
                |suggestion_buffer| {
                    if suggestion_buffer.len() > current_input.len() {
                        suggestion_buffer[current_input.len()..].to_string()
                    } else {
                        "".to_string()
                    }
                },
            ),
            Style::default().add_modifier(Modifier::DIM),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(title_spans(
                vec![TitleStyle::Single(box_title)],
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

    if matches!(app.state, State::ChannelSwitch) {
        frame.render_widget(Clear, input_rect);
    }

    frame.render_widget(paragraph, input_rect);

    app.buffer_suggestion = suggestion;
}
