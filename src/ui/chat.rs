use std::cmp::Ordering;

use anyhow::Result;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    terminal::Frame,
    widgets::{Block, Borders, Paragraph, Row, Table},
};
use unicode_width::UnicodeWidthStr;

use crate::{
    handlers::config::CompleteConfig,
    utils::{
        app::{App, State},
        styles,
    },
};

pub fn draw_chat_ui<T>(frame: &mut Frame<T>, app: &mut App, config: &CompleteConfig) -> Result<()>
where
    T: Backend,
{
    let table_widths = app.table_constraints.as_ref().unwrap();

    let mut vertical_chunk_constraints = vec![Constraint::Min(1)];

    if let State::Input = app.state {
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
        let mut rendered_text = app.input_text.as_str();

        let input_text_width = vertical_chunks[1].x + rendered_text.width() as u16;

        let y = vertical_chunks[1].y + 1;

        match input_text_width.cmp(&(vertical_chunks[1].width - 3)) {
            Ordering::Greater => {
                rendered_text =
                    &rendered_text[rendered_text.len() - vertical_chunks[1].width as usize - 3..];
                frame.set_cursor(rendered_text.width() as u16 + 2, y);
            }
            Ordering::Less => frame.set_cursor(input_text_width + 1, y),
            Ordering::Equal => {
                frame.set_cursor(vertical_chunks[1].width - 2, y);
            }
        }

        let paragraph = Paragraph::new(rendered_text)
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("[ Input ]"));

        frame.render_widget(paragraph, vertical_chunks[1]);
    }

    Ok(())
}
