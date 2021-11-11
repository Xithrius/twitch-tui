use anyhow::Result;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    terminal::Frame,
    widgets::{Block, Borders, Paragraph, Row, Table},
};

use crate::{
    handlers::{
        app::{App, State},
        config::CompleteConfig,
    },
    ui::statics::COMMANDS,
    utils::{styles, text::get_cursor_position},
};

pub fn draw_chat_ui<T>(frame: &mut Frame<T>, app: &mut App, config: &CompleteConfig) -> Result<()>
where
    T: Backend,
{
    let table_widths = app.table_constraints.as_ref().unwrap();

    let mut vertical_chunk_constraints = vec![Constraint::Min(1)];

    if let State::Input = app.state {
        if app.input_buffer.starts_with('/') {
            vertical_chunk_constraints.push(Constraint::Length(9));
        }

        vertical_chunk_constraints.push(Constraint::Length(3));
    }

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(vertical_chunk_constraints.as_ref())
        .split(frame.size());

    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(table_widths.as_ref())
        .split(frame.size());

    // 0'th index because no matter what index is obtained, they're the same height.
    let general_chunk_height = vertical_chunks[0].height as usize - 3;

    // The chunk furthest to the right is the messages, that's the one we want.
    let message_chunk_width = horizontal_chunks[table_widths.len() - 1].width as usize - 4;

    // Making sure that messages do have a limit and don't eat up all the RAM.
    app.messages
        .truncate(config.terminal.maximum_messages as usize);

    // Accounting for not all heights of rows to be the same due to text wrapping,
    // so extra space needs to be used in order to scroll correctly.
    let mut total_row_height: usize = 0;
    let mut display_rows = std::collections::VecDeque::new();

    for data in app.messages.iter() {
        let (msg_height, row) = data.to_row(&config.frontend, &message_chunk_width);
        let row_height = total_row_height + msg_height as usize;

        if row_height > general_chunk_height {
            break;
        }
        total_row_height = row_height;
        display_rows.push_front(row);
    }

    let table = Table::new(display_rows)
        .header(
            Row::new(app.column_titles.as_ref().unwrap().to_owned()).style(styles::COLUMN_TITLE),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("[ {}'s chat stream ]", &config.twitch.channel))
                .style(styles::BORDER_NAME),
        )
        .widths(table_widths.as_ref())
        .column_spacing(1);

    frame.render_widget(table, vertical_chunks[0]);

    if let State::Input = app.state {
        if app.input_buffer.starts_with('/') {
            let suggested_commands = COMMANDS
                .iter()
                .map(|f| format!("/{}", f))
                .filter(|f| f.starts_with(app.input_buffer.as_str()))
                .collect::<Vec<String>>()
                .join("\n");

            let suggestions_paragraph = Paragraph::new(suggested_commands)
                .style(Style::default().fg(Color::Blue))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("[ Command suggestions ]"),
                );

            frame.render_widget(suggestions_paragraph, vertical_chunks[1]);
        }

        let text = app.input_buffer.as_str();
        let cursor_pos = get_cursor_position(&app.input_buffer);
        let input_rect = vertical_chunks[vertical_chunk_constraints.len() - 1];

        frame.set_cursor(
            (input_rect.x + cursor_pos as u16 + 1)
                .min(input_rect.x + input_rect.width.saturating_sub(2)),
            input_rect.y + 1,
        );

        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("[ Input ]"))
            .scroll((
                0,
                ((cursor_pos + 3) as u16).saturating_sub(input_rect.width),
            ));

        frame.render_widget(
            paragraph,
            vertical_chunks[vertical_chunk_constraints.len() - 1],
        );
    }

    Ok(())
}
