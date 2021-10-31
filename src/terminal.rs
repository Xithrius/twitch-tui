use std::{
    io::{stdout, Stdout},
    time::Duration,
};

use anyhow::Result;
use chrono::offset::Local;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tokio::sync::mpsc::{Receiver, Sender};
use tui::{backend::CrosstermBackend, layout::Constraint, Terminal};

use crate::{
    handlers::{
        app::{App, State},
        config::CompleteConfig,
        data::{Data, DataBuilder},
        event::{Config, Event, Events, Key},
    },
    ui::{chat::draw_chat_ui, help::draw_keybinds_ui, input::draw_input_ui},
    utils::text::align_text,
};

pub async fn ui_driver(
    config: &CompleteConfig,
    mut app: App,
    tx: Sender<String>,
    mut rx: Receiver<Data>,
) -> Result<()> {
    let mut events = Events::with_config(Config {
        exit_key: Key::Null,
        tick_rate: Duration::from_millis(config.terminal.tick_delay),
    })
    .await;

    enable_raw_mode().unwrap();

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend).unwrap();

    let username_column_title = align_text(
        "Username",
        &config.frontend.username_alignment,
        config.frontend.maximum_username_length,
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

    app.column_titles = Some(column_titles);
    app.table_constraints = Some(table_constraints);

    terminal.clear().unwrap();

    let data_builder = DataBuilder::new(&config.frontend.date_format);

    let quitting = |mut terminal: Terminal<CrosstermBackend<Stdout>>| {
        disable_raw_mode().unwrap();
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .unwrap();
        terminal.show_cursor().unwrap();
    };

    'outer: loop {
        terminal
            .draw(|mut frame| match app.state {
                State::Normal | State::Input => draw_chat_ui(&mut frame, &mut app, config).unwrap(),
                State::Help => draw_keybinds_ui(&mut frame).unwrap(),
            })
            .unwrap();

        if let Ok(info) = rx.try_recv() {
            app.messages.push_front(info);
        }

        if let Some(Event::Input(key)) = events.next().await {
            match app.state {
                State::Input => {
                    let input_buffer = app.input_boxes.get_mut("chat").unwrap();

                    match key {
                        Key::Esc => {
                            input_buffer.update("", 0);
                            app.state = State::Normal;
                        }
                        Key::Enter => {
                            let input_message = input_buffer.as_str();

                            if !input_message.is_empty() {
                                app.messages.push_front(data_builder.user(
                                    config.twitch.username.to_string(),
                                    input_message.to_string(),
                                ));

                                tx.send(input_message.to_string()).await.unwrap();
                                input_buffer.update("", 0);
                            }
                        }
                        _ => draw_input_ui(&mut app, "chat", key).await,
                    }
                }
                _ => match key {
                    Key::Char('c') => app.state = State::Normal,
                    Key::Char('?') => app.state = State::Help,
                    Key::Char('i') => app.state = State::Input,
                    Key::Char('q') => {
                        quitting(terminal);
                        break 'outer;
                    }
                    Key::Esc => match app.state {
                        State::Normal => {
                            quitting(terminal);
                            break 'outer;
                        }
                        State::Help | State::Input => app.state = State::Normal,
                    },
                    _ => {}
                },
            }
        }
    }

    Ok(())
}
