use std::vec;

use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::{
    handlers::{config::SharedCompleteConfig, state::State},
    ui::components::utils::centered_rect,
    utils::text::{get_cursor_position, title_spans, TitleStyle},
};

pub fn render_insert_box<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    config: SharedCompleteConfig,
    box_title: &str,
    suggestion: Option<String>,
    input_validation: Option<Box<dyn FnOnce(String) -> bool>>,
) {
    let cursor_pos = get_cursor_position(buffer);

    f.set_cursor(
        (area.x + cursor_pos as u16 + 1).min(area.x + area.width.saturating_sub(2)),
        area.y + 1,
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
            .border_type(frontend.border_type.into())
            .border_style(Style::default().fg(status_color))
            .title(title_spans(
                &binding,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            )),
    )
    .scroll((0, ((cursor_pos + 3) as u16).saturating_sub(area.width)));

    // if matches!(app.get_state(), State::ChannelSwitch) {
    //     frame.render_widget(Clear, area);
    // }

    f.render_widget(paragraph, area);

    // app.buffer_suggestion = suggestion;
}
