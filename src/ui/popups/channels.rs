use tui::{
    backend::Backend,
    style::{Color, Style},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{
    handlers::app::App,
    ui::popups::{centered_popup, WindowType},
    utils::text::get_cursor_position,
};

pub fn switch_channels<T: Backend>(frame: &mut Frame<T>, app: &mut App, channel_suggestions: bool) {
    let input_rect = centered_popup(WindowType::Input(frame.size().height), frame.size());

    let input_buffer = app.current_buffer();

    let suggestion = if channel_suggestions {
        if let Some(result) = app
            .storage
            .get("channels".to_string())
            .iter()
            .filter(|s| s.starts_with(input_buffer.as_str()))
            .collect::<Vec<&String>>()
            .first()
        {
            result.to_string()
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
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
            if suggestion.len() > input_buffer.as_str().len() {
                &suggestion[input_buffer.as_str().len()..]
            } else {
                ""
            },
            Style::default().fg(Color::LightYellow),
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
