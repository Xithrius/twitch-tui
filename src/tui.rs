use std::{io, sync::mpsc::Receiver, time::Duration};

use anyhow::Result;
use chrono::offset::Local;
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, layout::Constraint, Terminal};

use crate::ui::{chat::draw_chat_ui, keybinds::draw_keybinds_ui};
use crate::{
    handlers::{config::CompleteConfig, data::Data},
    utils::{
        app::{App, State},
        event,
        text::align_text,
    },
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

    let mut column_titles = vec![
        username_column_title.to_owned(),
        "Message content".to_string(),
    ];

    let mut table_constraints = vec![
        Constraint::Length(config.frontend.maximum_username_length),
        Constraint::Percentage(100),
    ];

    if config.frontend.date_shown {
        column_titles.insert(0, "Time".to_string());

        table_constraints.insert(
            0,
            Constraint::Length(
                Local::now()
                    .format(config.frontend.date_format.as_str())
                    .to_string()
                    .len() as u16,
            ),
        );
    }

    app.column_titles = Option::Some(column_titles);
    app.table_constraints = Option::Some(table_constraints);

    let chat_config = config.clone();

    loop {
        if let Ok(info) = rx.try_recv() {
            app.messages.push_front(info);
        }

        terminal.draw(|mut frame| match app.state {
            State::Chat => draw_chat_ui(&mut frame, &mut app, chat_config.to_owned()).unwrap(),
            State::KeybindHelp => draw_keybinds_ui(&mut frame, chat_config.to_owned()).unwrap(),
        })?;

        if let event::Event::Input(input) = events.next()? {
            match input {
                Key::Char('c') => app.state = State::Chat,
                Key::Char('?') => app.state = State::KeybindHelp,
                Key::Esc => break,
                _ => {}
            }
        }
    }

    Ok(())
}
