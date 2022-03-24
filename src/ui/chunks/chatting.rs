use tui::{
    backend::Backend,
    style::{Color, Style},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    handlers::app::App,
    ui::{statics::COMMANDS, Verticals},
    utils::text::get_cursor_position,
};

pub fn message_input<T: Backend>(
    frame: &mut Frame<T>,
    app: &mut App,
    verticals: Verticals,
    mention_suggestions: bool,
) {
    let input_buffer = app.current_buffer();

    let suggestion = if let Some(start_character) = input_buffer.chars().next() {
        let first_result = |choices: Vec<String>, choice: String| -> String {
            if let Some(result) = choices
                .iter()
                .filter(|s| s.starts_with(&choice[1..]))
                .collect::<Vec<&String>>()
                .first()
            {
                result.to_string()
            } else {
                "".to_string()
            }
        };

        match start_character {
            '/' => format!(
                "/{}",
                first_result(
                    COMMANDS
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>(),
                    input_buffer.to_string(),
                )
            ),
            '@' => {
                if mention_suggestions {
                    format!(
                        "@{}",
                        first_result(
                            app.database
                                .get_table_content("mentions".to_string())
                                .unwrap(),
                            input_buffer.to_string(),
                        )
                    )
                } else {
                    "".to_string()
                }
            }
            _ => "".to_string(),
        }
    } else {
        "".to_string()
    };

    let cursor_pos = get_cursor_position(input_buffer);
    let input_rect = verticals.chunks[verticals.constraints.len() - 1];

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
            .title("[ Input ]")
            .border_style(Style::default().fg(Color::Yellow)),
    )
    .scroll((
        0,
        ((cursor_pos + 3) as u16).saturating_sub(input_rect.width),
    ));

    frame.render_widget(paragraph, verticals.chunks[verticals.constraints.len() - 1]);

    app.buffer_suggestion = suggestion;
}
