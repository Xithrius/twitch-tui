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
use rustyline::{At, Word};
use tokio::sync::mpsc::{Receiver, Sender};
use tui::{backend::CrosstermBackend, layout::Constraint, Terminal};

use crate::{
    handlers::{
        app::{App, BufferName, State},
        config::CompleteConfig,
        data::{Data, DataBuilder, PayLoad},
        event::{Config, Event, Events, Key},
    },
    twitch::Action,
    ui::draw_ui,
    utils::text::align_text,
};

pub async fn ui_driver(
    mut config: CompleteConfig,
    mut app: App,
    tx: Sender<Action>,
    mut rx: Receiver<Data>,
) -> Result<()> {
    let mut events = Events::with_config(Config {
        exit_key: Key::Null,
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
        if let Ok(info) = rx.try_recv() {
            match info.payload {
                PayLoad::Message(_) => app.messages.push_front(info),

                // If something such as a keypress failed, fallback to the normal state of the application.
                PayLoad::Err(err) => {
                    app.state = State::Normal;
                    app.selected_buffer = BufferName::Chat;

                    app.messages.push_front(data_builder.twitch(err));
                }
            }
        }

        terminal
            .draw(|frame| draw_ui(frame, &mut app, &config))
            .unwrap();

        if let Some(Event::Input(key)) = events.next().await {
            match app.state {
                State::Input | State::ChannelSwitch | State::Search => {
                    let input_buffer = app.current_buffer();

                    match key {
                        Key::Up => {
                            if let State::Input = app.state {
                                app.state = State::Normal;
                            }
                        }
                        Key::Ctrl('f') | Key::Right => {
                            input_buffer.move_forward(1);
                        }
                        Key::Ctrl('b') | Key::Left => {
                            input_buffer.move_backward(1);
                        }
                        Key::Ctrl('a') | Key::Home => {
                            input_buffer.move_home();
                        }
                        Key::Ctrl('e') | Key::End => {
                            input_buffer.move_end();
                        }
                        Key::Alt('f') => {
                            input_buffer.move_to_next_word(At::AfterEnd, Word::Emacs, 1);
                        }
                        Key::Alt('b') => {
                            input_buffer.move_to_prev_word(Word::Emacs, 1);
                        }
                        Key::Ctrl('t') => {
                            input_buffer.transpose_chars();
                        }
                        Key::Alt('t') => {
                            input_buffer.transpose_words(1);
                        }
                        Key::Ctrl('u') => {
                            input_buffer.discard_line();
                        }
                        Key::Ctrl('k') => {
                            input_buffer.kill_line();
                        }
                        Key::Ctrl('w') => {
                            input_buffer.delete_prev_word(Word::Emacs, 1);
                        }
                        Key::Ctrl('d') => {
                            input_buffer.delete(1);
                        }
                        Key::Backspace | Key::Delete => {
                            input_buffer.backspace(1);
                        }
                        Key::Enter => match app.selected_buffer {
                            BufferName::Chat => {
                                let input_message =
                                    app.input_buffers.get_mut(&app.selected_buffer).unwrap();

                                if !input_message.is_empty() {
                                    app.messages.push_front(data_builder.user(
                                        config.twitch.username.to_string(),
                                        input_message.to_string(),
                                    ));

                                    tx.send(Action::Privmsg(input_message.to_string()))
                                        .await
                                        .unwrap();
                                }

                                input_message.update("", 0);
                            }
                            BufferName::Channel => {
                                let input_message =
                                    app.input_buffers.get_mut(&app.selected_buffer).unwrap();

                                if !input_message.is_empty() {
                                    app.messages.clear();

                                    tx.send(Action::Join(input_message.to_string()))
                                        .await
                                        .unwrap();

                                    config.twitch.channel = input_message.to_string();
                                }

                                input_message.update("", 0);

                                app.selected_buffer = BufferName::Chat;
                                app.state = State::Normal;
                            }
                            BufferName::MessageSearch => {}
                        },
                        Key::Char(c) => {
                            input_buffer.insert(c, 1);
                        }
                        Key::Esc => {
                            input_buffer.update("", 0);
                            app.state = State::Normal;
                        }
                        _ => {}
                    }
                }
                _ => match key {
                    Key::Char('c') => {
                        app.state = State::Normal;
                        app.selected_buffer = BufferName::Chat;
                    }
                    Key::Char('C') => {
                        app.state = State::ChannelSwitch;
                        app.selected_buffer = BufferName::Channel;
                    }
                    Key::Char('?') => app.state = State::Help,
                    Key::Char('i') => {
                        app.state = State::Input;
                        app.selected_buffer = BufferName::Chat;
                    }
                    Key::Char('s') => {
                        app.state = State::Search;
                        app.selected_buffer = BufferName::MessageSearch;
                    }
                    Key::Char('q') => {
                        quitting(terminal);
                        break 'outer;
                    }
                    Key::Esc => match app.state {
                        State::Normal => {
                            quitting(terminal);
                            break 'outer;
                        }
                        _ => {
                            app.state = State::Normal;
                            app.selected_buffer = BufferName::Chat;
                        }
                    },
                    _ => {}
                },
            }
        }
    }

    Ok(())
}
