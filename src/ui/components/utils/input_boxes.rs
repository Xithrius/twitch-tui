use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{
    handlers::app::State,
    ui::{components::utils::popups::centered_popup, WindowAttributes},
    utils::text::{get_cursor_position, title_spans, TitleStyle},
};

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
    let WindowAttributes {
        frame,
        layout,
        app,
        show_state_tabs,
    } = window;

    let buffer = &app.input_buffer;

    let cursor_pos = get_cursor_position(buffer);

    let input_rect = input_rectangle.map_or_else(
        || {
            if let Some(l) = layout {
                l.chunks[l.constraints.len() - (if show_state_tabs { 2 } else { 1 })]
            } else {
                centered_popup(frame.size(), frame.size().height)
            }
        },
        |r| r,
    );

    frame.set_cursor(
        (input_rect.x + cursor_pos as u16 + 1)
            .min(input_rect.x + input_rect.width.saturating_sub(2)),
        input_rect.y + 1,
    );

    let current_input = buffer.as_str();

    let valid_input =
        input_validation.map_or(true, |check_func| check_func(current_input.to_string()));

    let binding = [TitleStyle::Single(box_title)];

    let status_color = if valid_input {
        Color::Green
    } else {
        Color::Red
    };

    let paragraph = Paragraph::new(Spans::from(vec![
        Span::raw(current_input),
        Span::styled(
            suggestion
                .clone()
                .map_or_else(String::new, |suggestion_buffer| {
                    if suggestion_buffer.len() > current_input.len() {
                        suggestion_buffer[current_input.len()..].to_string()
                    } else {
                        String::new()
                    }
                }),
            Style::default().add_modifier(Modifier::DIM),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(title_spans(
                &binding,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ))
            .border_style(Style::default().fg(status_color)),
    )
    .scroll((
        0,
        ((cursor_pos + 3) as u16).saturating_sub(input_rect.width),
    ));

    if matches!(app.get_state(), State::ChannelSwitch) {
        frame.render_widget(Clear, input_rect);
    }

    frame.render_widget(paragraph, input_rect);

    app.buffer_suggestion = suggestion;
}
