use tui::{
    backend::Backend,
    style::{Color, Style},
    terminal::Frame,
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    handlers::app::App,
    ui::{statics::COMMANDS, Verticals},
    utils::text::get_cursor_position,
};

pub fn message_input<T: Backend>(frame: &mut Frame<T>, app: &mut App, verticals: Verticals) {
    let input_buffer = app.current_buffer();

    if input_buffer.starts_with('/') {
        let suggested_commands = COMMANDS
            .iter()
            .map(|f| format!("/{}", f))
            .filter(|f| f.starts_with(input_buffer.as_str()))
            .collect::<Vec<String>>()
            .join("\n");

        let suggestions_paragraph = Paragraph::new(suggested_commands)
            .style(Style::default().fg(Color::Blue))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("[ Command suggestions ]"),
            );

        frame.render_widget(suggestions_paragraph, verticals.chunks[1]);
    }

    let cursor_pos = get_cursor_position(input_buffer);
    let input_rect = verticals.chunks[verticals.constraints.len() - 1];

    frame.set_cursor(
        (input_rect.x + cursor_pos as u16 + 1)
            .min(input_rect.x + input_rect.width.saturating_sub(2)),
        input_rect.y + 1,
    );

    let paragraph = Paragraph::new(input_buffer.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("[ Input ]"))
        .scroll((
            0,
            ((cursor_pos + 3) as u16).saturating_sub(input_rect.width),
        ));

    frame.render_widget(
        paragraph,
        verticals.chunks[verticals.constraints.len() - 1],
    );
}
