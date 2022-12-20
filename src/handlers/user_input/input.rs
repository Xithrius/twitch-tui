use regex::Regex;
use rustyline::{At, Word};
use tokio::sync::mpsc::Sender;

use crate::{
    handlers::{
        app::{App, State},
        config::CompleteConfig,
        data::DataBuilder,
        user_input::events::{Event, Events, Key},
    },
    twitch::TwitchAction,
    ui::statics::{CHANNEL_NAME_REGEX, TWITCH_MESSAGE_LIMIT},
};

pub enum TerminalAction {
    Quitting,
}

struct UserActionAttributes<'a, 'b> {
    app: &'a mut App,
    config: &'b mut CompleteConfig,
    tx: Sender<TwitchAction>,
    key: Key,
}

impl<'a, 'b> UserActionAttributes<'a, 'b> {
    fn new(
        app: &'a mut App,
        config: &'b mut CompleteConfig,
        tx: Sender<TwitchAction>,
        key: Key,
    ) -> Self {
        Self {
            app,
            config,
            tx,
            key,
        }
    }
}

async fn handle_insert_enter_key(action: &mut UserActionAttributes<'_, '_>) {
    let UserActionAttributes {
        app,
        config,
        key: _,
        tx,
    } = action;

    match app.state {
        State::Insert => {
            let input_message = &mut app.input_buffer;

            if input_message.is_empty()
                || app.filters.contaminated(input_message.as_str())
                || input_message.len() > *TWITCH_MESSAGE_LIMIT
            {
                return;
            }

            app.messages.push_front(DataBuilder::user(
                config.twitch.username.to_string(),
                input_message.to_string(),
            ));

            tx.send(TwitchAction::Privmsg(input_message.to_string()))
                .await
                .unwrap();

            if let Some(msg) = input_message.strip_prefix('@') {
                app.storage.add("mentions", msg.to_string());
            }

            let mut possible_command = String::new();

            input_message.clone_into(&mut possible_command);

            input_message.update("", 0);

            if possible_command.as_str() == "/clear" {
                app.clear_messages();
            }
        }
        State::ChannelSwitch => {
            let input_message = &mut app.input_buffer;

            if input_message.is_empty()
                || !Regex::new(&CHANNEL_NAME_REGEX)
                    .unwrap()
                    .is_match(input_message)
            {
                return;
            }

            app.messages.clear();

            tx.send(TwitchAction::Join(input_message.to_string()))
                .await
                .unwrap();

            config.twitch.channel = input_message.to_string();

            app.storage.add("channels", input_message.to_string());

            input_message.update("", 0);

            app.state = State::Normal;
        }
        _ => {}
    }
}

async fn handle_insert_type_movements(action: &mut UserActionAttributes<'_, '_>) {
    let UserActionAttributes {
        app,
        config: _,
        key,
        tx: _,
    } = action;

    let input_buffer = &mut app.input_buffer;

    match key {
        Key::Up => {
            if app.state == State::Insert {
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
        Key::Tab => {
            let suggestion = app.buffer_suggestion.clone();

            if let Some(suggestion_buffer) = suggestion {
                app.input_buffer
                    .update(suggestion_buffer.as_str(), suggestion_buffer.len());
            }
        }
        Key::Enter => handle_insert_enter_key(action).await,
        Key::Char(c) => {
            input_buffer.insert(*c, 1);
        }
        Key::Esc => {
            input_buffer.update("", 0);
            app.state = State::Normal;
        }
        _ => {}
    }
}

fn handle_user_scroll(app: &mut App, key: Key) {
    match app.state {
        State::Insert | State::MessageSearch | State::Normal => {
            let limit = app.scrolling.get_offset() < app.messages.len();

            match key {
                Key::ScrollUp => {
                    if limit {
                        app.scrolling.up();
                    } else if app.scrolling.inverted() {
                        app.scrolling.down();
                    }
                }
                Key::ScrollDown => {
                    if app.scrolling.inverted() {
                        if limit {
                            app.scrolling.up();
                        }
                    } else {
                        app.scrolling.down();
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}

pub async fn handle_stateful_user_input(
    events: &mut Events,
    app: &mut App,
    config: &mut CompleteConfig,
    tx: Sender<TwitchAction>,
) -> Option<TerminalAction> {
    if let Some(Event::Input(key)) = events.next().await {
        handle_user_scroll(app, key);

        match app.state {
            State::Insert | State::ChannelSwitch | State::MessageSearch => {
                let mut action = UserActionAttributes::new(app, config, tx, key);

                handle_insert_type_movements(&mut action).await;
            }
            _ => match key {
                Key::Char('c') => {
                    app.state = State::Normal;
                }
                Key::Char('s') => {
                    app.state = State::ChannelSwitch;
                }
                Key::Ctrl('f') => {
                    app.state = State::MessageSearch;
                }
                Key::Ctrl('t') => {
                    app.filters.toggle();
                }
                Key::Ctrl('r') => {
                    app.filters.reverse();
                }
                Key::Char('i') | Key::Insert => {
                    app.state = State::Insert;
                }
                Key::Char('@' | '/') => {
                    app.state = State::Insert;
                    app.input_buffer.update(&key.to_string(), 1);
                }
                Key::Ctrl('p') => {
                    panic!("Manual panic triggered by user.");
                }
                Key::Char('?') => app.state = State::Help,
                Key::Char('q') => {
                    return Some(TerminalAction::Quitting);
                }
                Key::Esc => {
                    app.scrolling.jump_to(0);

                    app.state = State::Normal;
                }
                _ => {}
            },
        }
    }

    None
}
