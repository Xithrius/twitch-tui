use std::{
    io::{stdout, Stdout},
    time::Duration,
};

use anyhow::Result;
use chrono::offset::Local;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::FutureExt;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::unconstrained,
};
use tui::{backend::CrosstermBackend, layout::Constraint, Terminal};

use crate::{
    handlers::{config::CompleteConfig, data::Data},
    ui::{chat::draw_chat_ui, keybinds::draw_keybinds_ui},
    utils::{
        self,
        app::{App, State},
        text::align_text,
    },
};

pub async fn ui_driver(
    config: &CompleteConfig,
    mut app: App,
    tx: Sender<String>,
    mut rx: Receiver<Data>,
) -> Result<()> {
    let mut events = utils::event::Events::with_config(utils::event::Config {
        exit_key: KeyCode::Null,
        tick_rate: Duration::from_millis(config.terminal.tick_delay),
    })
    .await;

    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

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

    app.column_titles = Option::Some(column_titles);
    app.table_constraints = Option::Some(table_constraints);

    terminal.clear().unwrap();

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
        if let Some(Some(info)) = unconstrained(rx.recv()).now_or_never() {
            app.messages.push_front(info);
        }

        terminal
            .draw(|mut frame| match app.state {
                State::Normal | State::Input => draw_chat_ui(&mut frame, &mut app, config).unwrap(),
                State::KeybindHelp => draw_keybinds_ui(&mut frame).unwrap(),
            })
            .unwrap();

        if let Some(utils::event::Event::Input(input_event)) = &events.next().await {
            match app.state {
                State::Input => match input_event.modifiers {
                    KeyModifiers::CONTROL => match input_event.code {
                        KeyCode::Char('u') => {
                            app.input_text.truncate(0);
                        }
                        KeyCode::Char('w') => {
                            // If the last character is whitespace, truncate to the whitespace
                            // before the last non-whitespace character
                            // Else, truncate to the last whitespace character
                            let mut text: &str = &app.input_text;

                            // Get to the last non-whitespace character
                            if text.ends_with(char::is_whitespace) {
                                if let Some(n) = text.rfind(|c: char| !c.is_whitespace()) {
                                    text = &text[..n];
                                }
                            }
                            // Truncate to the last whitespace character
                            let truncate_location = match text.rfind(char::is_whitespace) {
                                Some(n) => n + 1,
                                None => 0,
                            };
                            app.input_text.truncate(truncate_location);
                        }
                        _ => (),
                    },
                    _ => match input_event.code {
                        KeyCode::Enter => {
                            let input_message: String = app.input_text.drain(..).collect();
                            app.messages.push_front(Data::new(
                                Local::now()
                                    .format(config.frontend.date_format.as_str())
                                    .to_string(),
                                config.twitch.username.to_string(),
                                input_message.clone(),
                                false,
                            ));

                            tx.send(input_message).await.unwrap();
                        }
                        KeyCode::Char(c) => {
                            app.input_text.push(c);
                        }
                        KeyCode::Backspace | KeyCode::Delete => {
                            app.input_text.pop();
                        }
                        KeyCode::Esc => {
                            app.state = State::Normal;
                        }
                        _ => {}
                    },
                },
                _ => match input_event.code {
                    KeyCode::Char('c') => app.state = State::Normal,
                    KeyCode::Char('?') => app.state = State::KeybindHelp,
                    KeyCode::Char('i') => app.state = State::Input,
                    KeyCode::Char('q') => {
                        quitting(terminal);
                        break 'outer;
                    }
                    KeyCode::Esc => match app.state {
                        State::Normal => {
                            quitting(terminal);
                            break 'outer;
                        }
                        State::KeybindHelp | State::Input => app.state = State::Normal,
                    },
                    _ => {}
                },
            }
        }
    }

    Ok(())
}
