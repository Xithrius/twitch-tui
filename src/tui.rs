use std::{
    io,
    sync::mpsc::Receiver,
    time::Duration,
};

use anyhow::Result;
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    Terminal,
    widgets::{Block, Borders, Paragraph, Row, Table},
};
use unicode_width::UnicodeWidthStr;

use crate::utils::{app::App, event};

pub fn tui(mut app: App, rx: Receiver<Vec<String>>) -> Result<()> {
    let events = event::Events::with_config(event::Config {
        exit_key: Key::Esc,
        tick_rate: Duration::from_millis(250),
    });

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        if let Ok(info) = rx.try_recv() {
            app.insert_message(info[0].to_string(), info[1].to_string())
        }

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Min(1),
                    ]
                        .as_ref(),
                )
                .split(f.size());

            let all_messages = app
                .messages
                .iter()
                .map(|m| Row::new(m.clone()))
                .collect::<Vec<Row>>();

            let table = Table::new(all_messages)
                .style(Style::default().fg(Color::White))
                .header(
                    Row::new(vec!["Time", "User", "Message content"])
                        .style(Style::default().fg(Color::Yellow))
                        .bottom_margin(1),
                )
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("[ Table of messages ]"),
                )
                .widths(&[
                    Constraint::Length(26),
                    Constraint::Length(26),
                    Constraint::Min(1),
                ])
                .column_spacing(1);

            f.render_widget(table, chunks[0]);
        })?;

        if let event::Event::Input(input) = events.next()? {
            if let Key::Esc = input {
                break;
            }
        }
    }

    Ok(())
}
