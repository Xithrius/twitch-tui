use anyhow::Result;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    terminal::Frame,
    widgets::{Block, Borders, Row, Table},
};

use crate::{
    handlers::config::CompleteConfig,
    utils::{app::App, colors::WindowStyles},
};

pub fn draw_chat_ui<T>(frame: &mut Frame<T>, app: &mut App, config: CompleteConfig) -> Result<()>
where
    T: Backend,
{
    let table_widths = app.table_constraints.as_ref().unwrap();

    let mut vertical_chunk_constraints = vec![Constraint::Min(1)];

    if config.frontend.input {
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

    // A vector of tuples which contain the length of some message content.
    // This message is contained within the 3rd cell of the row within the tuples.
    // Color and alignment of the username along with message text wrapping is done here.
    let all_messages = &app
        .messages
        .iter()
        .rev()
        .map(|m| m.to_row(&config.frontend, &message_chunk_width))
        .collect::<Vec<(u16, Row)>>();

    let total_row_height: usize = all_messages.iter().map(|r| r.0 as usize).sum();

    let mut all_rows = all_messages.iter().map(|r| r.1.clone()).collect::<Vec<_>>();

    // Accounting for not all heights of rows to be the same due to text wrapping,
    // so extra space needs to be used in order to scroll correctly.
    if total_row_height >= general_chunk_height {
        let mut row_sum = 0;
        let mut final_index = 0;
        for (index, (row_height, _)) in all_messages.iter().rev().enumerate() {
            if row_sum >= general_chunk_height {
                final_index = index;
                break;
            }
            row_sum += *row_height as usize;
        }

        all_rows = all_rows[all_rows.len() - final_index..].to_owned();
    }

    let table = Table::new(all_rows)
        .header(
            Row::new(app.column_titles.as_ref().unwrap().to_owned())
                .style(WindowStyles::new(WindowStyles::ColumnTitle)),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("[ {}'s chat stream ]", &config.twitch.channel))
                .style(WindowStyles::new(WindowStyles::BoarderName)),
        )
        .widths(table_widths.as_ref())
        .column_spacing(1);

    frame.render_widget(table, vertical_chunks[0]);

    Ok(())
}
