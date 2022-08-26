use tui::{
    backend::Backend,
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{
    handlers::app::App,
    ui::popups::{centered_popup, WindowType},
    utils::text::{get_cursor_position, suggestion_partition},
};

pub fn switch_channels<T: Backend>(frame: &mut Frame<T>, app: &mut App, channel_suggestions: bool) {
    let input_rect = centered_popup(WindowType::Input(frame.size().height), frame.size());

    let input_buffer = app.current_buffer();

    let suggestion = if channel_suggestions {
        suggestion_partition(
            input_buffer.to_string(),
            app.storage
                .get("channels".to_string())
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
        )
    } else {
        None
    };

    let cursor_pos = get_cursor_position(input_buffer);

    frame.set_cursor(
        (input_rect.x + cursor_pos as u16 + 1)
            .min(input_rect.x + input_rect.width.saturating_sub(2)),
        input_rect.y + 1,
    );

    let paragraph = Paragraph::new(Spans::from(vec![
        Span::raw(input_buffer.as_str()),
        Span::styled(
            if let Some(suggestion_buffer) = suggestion.clone() {
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
            .title("[ Channel ]")
            .border_style(Style::default().fg(Color::Yellow)),
    )
    .scroll((
        0,
        ((cursor_pos + 3) as u16).saturating_sub(input_rect.width),
    ));

    frame.render_widget(Clear, input_rect);
    frame.render_widget(paragraph, input_rect);

    app.buffer_suggestion = suggestion;
}
