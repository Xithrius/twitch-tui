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

use crate::utils::{app::App, event};

pub fn tui(mut app: App, rx: Receiver<Vec<String>>) -> Result<()> {
    let events = event::Events::with_config(event::Config {
        exit_key: Key::Esc,
        tick_rate: Duration::from_millis(30),
    });

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        if let Ok(info) = rx.try_recv() {
            app.messages.push(vec![
                format!("{}", Local::now().format("%a %b %e %T %Y")),
                info[0].to_string(),
                info[1].to_string(),
            ]);
        }

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(1)].as_ref())
                .split(f.size());

            let all_messages = app
                .messages
                .iter()
                .map(|m| Row::new(m.clone()))
                .collect::<Vec<Row>>();

            let chunk_height = chunks[0].height as usize - 4;
            let message_amount = all_messages.len();

            let mut rendered_messages = all_messages;

            if rendered_messages.len() >= chunk_height {
                rendered_messages = rendered_messages[message_amount - chunk_height..].to_owned();
            }

            let table = Table::new(rendered_messages)
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
