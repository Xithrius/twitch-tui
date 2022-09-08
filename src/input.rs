use regex::Regex;
use rustyline::{At, Word};
use tokio::sync::mpsc::Sender;

use crate::{
    handlers::{
        app::{App, BufferName, State},
        config::CompleteConfig,
        data::DataBuilder,
        event::{Event, Events, Key},
    },
    twitch::TwitchAction,
    ui::statics::{CHANNEL_NAME_REGEX, TWITCH_MESSAGE_LIMIT},
};

pub enum TerminalAction {
    Quitting,
}

pub async fn handle_user_input(
    events: &mut Events,
    app: &mut App,
    config: &mut CompleteConfig,
    tx: Sender<TwitchAction>,
) -> Option<TerminalAction> {
    if let Some(Event::Input(key)) = events.next().await {
        match app.state {
            State::Insert | State::MessageSearch | State::Normal => match key {
                Key::ScrollUp => {
                    if app.scroll_offset < app.messages.len() {
                        app.scroll_offset += 1;
                    }
                }
                Key::ScrollDown => {
                    if app.scroll_offset > 0 {
                        app.scroll_offset -= 1;
                    }
                }
                _ => {}
            },
            _ => {}
        }

        match app.state {
            State::Insert | State::ChannelSwitch | State::MessageSearch => {
                let input_buffer = app.current_buffer_mut();

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
                            app.input_buffers
                                .get_mut(&app.selected_buffer)
                                .unwrap()
                                .update(suggestion_buffer.as_str(), suggestion_buffer.len());
                        }
                    }
                    Key::Enter => match app.selected_buffer {
                        BufferName::Chat => {
                            let input_message =
                                app.input_buffers.get_mut(&app.selected_buffer).unwrap();

                            if input_message.is_empty()
                                || app.filters.contaminated(input_message)
                                || input_message.len() > *TWITCH_MESSAGE_LIMIT
                            {
                                return None;
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

                            input_message.update("", 0);
                        }
                        BufferName::Channel => {
                            let input_message =
                                app.input_buffers.get_mut(&app.selected_buffer).unwrap();

                            if input_message.is_empty()
                                || !Regex::new(*CHANNEL_NAME_REGEX)
                                    .unwrap()
                                    .is_match(input_message)
                            {
                                return None;
                            }

                            app.messages.clear();

                            tx.send(TwitchAction::Join(input_message.to_string()))
                                .await
                                .unwrap();

                            config.twitch.channel = input_message.to_string();

                            app.storage.add("channels", input_message.to_string());

                            input_message.update("", 0);

                            app.selected_buffer = BufferName::Chat;
                            app.state = State::Normal;
                        }
                        BufferName::MessageHighlighter => {}
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
                Key::Char('s') => {
                    app.state = State::ChannelSwitch;
                    app.selected_buffer = BufferName::Channel;
                }
                Key::Ctrl('f') => {
                    app.state = State::MessageSearch;
                    app.selected_buffer = BufferName::MessageHighlighter;
                }
                Key::Ctrl('t') => {
                    app.filters.toggle();
                }
                Key::Ctrl('r') => {
                    app.filters.reverse();
                }
                Key::Char('i') | Key::Insert => {
                    app.state = State::Insert;
                    app.selected_buffer = BufferName::Chat;
                }
                Key::Ctrl('p') => {
                    panic!("Manual panic triggered by user.");
                }
                Key::Char('?') => app.state = State::Help,
                Key::Char('q') => {
                    if app.state == State::Normal {
                        return Some(TerminalAction::Quitting);
                    }
                }
                Key::Esc => {
                    app.scroll_offset = 0;
                    app.state = State::Normal;
                    app.selected_buffer = BufferName::Chat;
                }
                _ => {}
            },
        }
    }

    None
}
