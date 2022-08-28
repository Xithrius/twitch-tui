use tui::{
    backend::Backend,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

use crate::{ui::WindowAttributes, utils::text::get_cursor_position};

pub fn ui_search_messages<T: Backend>(window: WindowAttributes<T>) {
    let WindowAttributes { frame, layout, app } = window;

    let input_buffer = app.current_buffer();

    let cursor_pos = get_cursor_position(input_buffer);
    let input_rect = layout.chunks[layout.constraints.len() - 1];

    frame.set_cursor(
        (input_rect.x + cursor_pos as u16 + 1)
            .min(input_rect.x + input_rect.width.saturating_sub(2)),
        input_rect.y + 1,
    );

    let paragraph = Paragraph::new(input_buffer.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("[ Message Search ]")
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .scroll((
            0,
            ((cursor_pos + 3) as u16).saturating_sub(input_rect.width),
        ));

    frame.render_widget(paragraph, input_rect);
}
