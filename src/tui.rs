use std::{io, sync::mpsc::Receiver, time::Duration};

use anyhow::Result;
use chrono::offset::Local;
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Row, Table},
    Terminal,
};

use crate::{
    handlers::{config::CompleteConfig, data::Data},
    utils::{app::App, event, text::align_text},
};

pub fn tui(config: CompleteConfig, mut app: App, rx: Receiver<Data>) -> Result<()> {
    let events = event::Events::with_config(event::Config {
        exit_key: Key::Esc,
        tick_rate: Duration::from_millis(config.terminal.tick_delay),
    });

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let username_column_title = align_text(
        "Username",
        &config.frontend.username_alignment,
        &config.frontend.maximum_username_length,
    );

    let mut column_titles = vec![username_column_title.as_str(), "Message content"];

    let mut table_width = vec![
        Constraint::Length(config.frontend.maximum_username_length),
        Constraint::Percentage(100),
    ];

    if config.frontend.date_shown {
        column_titles.insert(0, "Time");

        table_width.insert(
            0,
            Constraint::Length(
                Local::now()
                    .format(config.frontend.date_format.as_str())
                    .to_string()
                    .len() as u16,
            ),
        );
    }

    loop {
        if let Ok(info) = rx.try_recv() {
            app.messages.push_front(info);
        }

        terminal.draw(|f| {
            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(1)].as_ref())
                .split(f.size());

            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(table_width.as_ref())
                .split(f.size());

            // 0'th index because no matter what index is obtained, they're the same height.
            let general_chunk_height = vertical_chunks[0].height as usize - 4;

            // The chunk furthest to the right is the messages, that's the one we want.
            let message_chunk_width = horizontal_chunks[table_width.len() - 1].width as usize - 4;

            // Making sure that messages do have a limit and don't eat up all the RAM.
            &app.messages
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
                    Row::new(column_titles.to_owned())
                        .style(Style::default().fg(Color::Yellow))
                        .bottom_margin(1),
                )
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("[ {}'s chat stream ]", &config.twitch.channel)),
                )
                .widths(table_width.as_ref())
                .column_spacing(1);

            f.render_widget(table, vertical_chunks[0]);
        })?;

        if let event::Event::Input(input) = events.next()? {
            if let Key::Esc = input {
                break;
            }
        }
    }

    Ok(())
}
