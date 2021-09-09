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

    let date_format_length = Local::now()
        .format(config.frontend.date_format.as_str())
        .to_string()
        .len() as u16;

    let username_column_title = align_text(
        "Username",
        &config.frontend.username_alignment,
        &config.frontend.maximum_username_length,
    );

    let column_titles = vec!["Time", &username_column_title, "Message content"];

    let table_width = &[
        Constraint::Length(date_format_length),
        Constraint::Length(config.frontend.maximum_username_length),
        Constraint::Min(1),
    ];

    loop {
        if let Ok(info) = rx.try_recv() {
            app.messages.push(info);
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

            let chunk_height = vertical_chunks[0].height as usize - 4;
            let chunk_width = horizontal_chunks[2].width as usize - 4;

            let all_messages = &app
                .messages
                .iter()
                .map(|m| m.to_row(&config.frontend, &chunk_width))
                .collect::<Vec<(u16, Row)>>();

            let total_row_height: usize = all_messages.iter().map(|r| r.0 as usize).sum();

            let mut all_rows = all_messages.iter().map(|r| r.1.clone()).collect::<Vec<_>>();

            if total_row_height >= chunk_height {
                let mut row_sum = 0;
                let mut final_index = 0;
                for (index, (row_height, _row)) in all_messages.iter().rev().enumerate() {
                    if row_sum >= chunk_height {
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
                        .title("[ Table of messages ]"),
                )
                .widths(table_width)
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
