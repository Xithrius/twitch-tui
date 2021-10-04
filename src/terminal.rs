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
use rustyline::Word;
use tokio::sync::mpsc::{Receiver, Sender};
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
        terminal
            .draw(|mut frame| match app.state {
                State::Normal | State::Input => draw_chat_ui(&mut frame, &mut app, config).unwrap(),
                State::KeybindHelp => draw_keybinds_ui(&mut frame).unwrap(),
            })
            .unwrap();

        if let Ok(info) = rx.try_recv() {
            app.messages.push_front(info);
        }

        if let Some(utils::event::Event::Input(input_event)) = events.next().await {
            match app.state {
                State::Input => match input_event.modifiers {
                    KeyModifiers::CONTROL => match input_event.code {
                        KeyCode::Char('u') => {
                            app.input_text.discard_line();
                        }
                        KeyCode::Char('w') => {
                            app.input_text.delete_prev_word(Word::Emacs, 1);
                        }
                        _ => (),
                    },
                    _ => match input_event.code {
                        KeyCode::Enter => {
                            let input_message = app.input_text.as_str();
                            app.messages.push_front(Data::new(
                                Local::now()
                                    .format(config.frontend.date_format.as_str())
                                    .to_string(),
                                config.twitch.username.to_string(),
                                input_message.to_string(),
                                false,
                            ));

                            tx.send(input_message.to_string()).await.unwrap();
                        }
                        KeyCode::Char(c) => {
                            app.input_text.insert(c, 1);
                        }
                        KeyCode::Backspace | KeyCode::Delete => {
                            app.input_text.delete(1);
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
