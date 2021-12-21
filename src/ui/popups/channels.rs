use tui::{
    backend::Backend,
    style::{Color, Style},
    terminal::Frame,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{
    handlers::app::App,
    ui::popups::{centered_popup, Centering},
    utils::text::get_cursor_position,
};

pub fn switch_channels<T: Backend>(frame: &mut Frame<T>, app: &mut App) {
    let input_rect = centered_popup(Centering::Input(None), frame.size());

    let input_buffer = app.current_buffer();

    let cursor_pos = get_cursor_position(input_buffer);

    frame.set_cursor(
        (input_rect.x + cursor_pos as u16 + 1)
            .min(input_rect.x + input_rect.width.saturating_sub(2)),
        input_rect.y + 1,
    );

    let paragraph = Paragraph::new(input_buffer.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("[ Channel ]"))
        .scroll((
            0,
            ((cursor_pos + 3) as u16).saturating_sub(input_rect.width),
        ));

    frame.render_widget(Clear, input_rect);
    frame.render_widget(paragraph, input_rect);
}
