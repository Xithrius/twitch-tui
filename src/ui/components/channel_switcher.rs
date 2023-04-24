use std::string::ToString;

use rustyline::{line_buffer::LineBuffer, At, Word};
use tokio::sync::broadcast::Sender;
use tui::{
    backend::Backend,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::{
    emotes::Emotes,
    handlers::{
        config::SharedCompleteConfig,
        data::DataBuilder,
        user_input::{
            events::{Event, Key},
            input::TerminalAction,
        },
    },
    twitch::TwitchAction,
    ui::{
        components::utils::centered_rect,
        statics::{LINE_BUFFER_CAPACITY, TWITCH_MESSAGE_LIMIT},
    },
    utils::text::{get_cursor_position, title_spans, TitleStyle},
};

#[derive(Debug)]
pub struct ChannelSwitcherWidget {
    config: SharedCompleteConfig,
    tx: Sender<TwitchAction>,
    // TODO: Extract this out to shared [`Rc`]
    emotes: Option<Emotes>,
    input: LineBuffer,
}

impl ChannelSwitcherWidget {
    pub fn new(config: SharedCompleteConfig, tx: Sender<TwitchAction>) -> Self {
        Self {
            config,
            tx,
            emotes: None,
            input: LineBuffer::with_capacity(*LINE_BUFFER_CAPACITY),
        }
    }
}

// let suggestion = if channel_suggestions {
//     first_similarity(
//         &app.storage
//             .get("channels")
//             .iter()
//             .map(ToString::to_string)
//             .collect::<Vec<String>>(),
//         input_buffer,
//     )
// } else {
//     None
// };

// This can't implement the [`Component`] trait due to needing
// emotes to be passed through when drawing.
impl ChannelSwitcherWidget {
    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, emotes: Emotes) {
        self.emotes = Some(emotes);

        let area = centered_rect(60, 20, f.size());

        let cursor_pos = get_cursor_position(&self.input);

        f.set_cursor(
            (area.x + cursor_pos as u16 + 1).min(area.x + area.width.saturating_sub(2)),
            area.y + 1,
        );

        let current_input = self.input.as_str();

        let binding = [TitleStyle::Single("Channel Switcher")];

        let status_color = Color::Green;

        let paragraph = Paragraph::new(current_input)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(self.config.borrow().frontend.border_type.clone().into())
                    .border_style(Style::default().fg(status_color))
                    .title(title_spans(
                        &binding,
                        Style::default()
                            .fg(status_color)
                            .add_modifier(Modifier::BOLD),
                    )),
            )
            .scroll((0, ((cursor_pos + 3) as u16).saturating_sub(area.width)));

        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }

    pub fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        if let Event::Input(key) = event {
            match key {
                Key::Ctrl('f') | Key::Right => {
                    self.input.move_forward(1);
                }
                Key::Ctrl('b') | Key::Left => {
                    self.input.move_backward(1);
                }
                Key::Ctrl('a') | Key::Home => {
                    self.input.move_home();
                }
                Key::Ctrl('e') | Key::End => {
                    self.input.move_end();
                }
                Key::Alt('f') => {
                    self.input.move_to_next_word(At::AfterEnd, Word::Emacs, 1);
                }
                Key::Alt('b') => {
                    self.input.move_to_prev_word(Word::Emacs, 1);
                }
                Key::Ctrl('t') => {
                    self.input.transpose_chars();
                }
                Key::Alt('t') => {
                    self.input.transpose_words(1);
                }
                Key::Ctrl('u') => {
                    self.input.discard_line();
                }
                Key::Ctrl('k') => {
                    self.input.kill_line();
                }
                Key::Ctrl('w') => {
                    self.input.delete_prev_word(Word::Emacs, 1);
                }
                Key::Ctrl('d') => {
                    self.input.delete(1);
                }
                Key::Backspace | Key::Delete => {
                    self.input.backspace(1);
                }
                // Key::Tab => {
                //     let suggestion = app.buffer_suggestion.clone();

                //     if let Some(suggestion_buffer) = suggestion {
                //         app.self.input
                //             .update(suggestion_buffer.as_str(), suggestion_buffer.len());
                //     }
                // }
                Key::Enter => {
                    let input_message = &mut self.input;

                    if input_message.is_empty() || input_message.len() > *TWITCH_MESSAGE_LIMIT {
                        return None;
                    }

                    let mut message = DataBuilder::user(
                        self.config.borrow().twitch.username.to_string(),
                        input_message.to_string(),
                    );
                    if let Some(mut emotes) = self.emotes.clone() {
                        message.parse_emotes(&mut emotes);
                    }

                    // app.messages.push_front(message);

                    self.tx
                        .send(TwitchAction::Privmsg(input_message.to_string()))
                        .unwrap();

                    // if let Some(msg) = input_message.strip_prefix('@') {
                    //     app.storage.add("mentions", msg.to_string());
                    // }

                    let mut possible_command = String::new();

                    input_message.clone_into(&mut possible_command);

                    input_message.update("", 0);

                    // if possible_command.as_str() == "/clear" {
                    //     app.clear_messages();
                    // }
                }
                Key::Ctrl('q') => return Some(TerminalAction::Quitting),
                Key::Char(c) => {
                    self.input.insert(*c, 1);
                }
                Key::Esc => {
                    self.input.update("", 0);

                    return Some(TerminalAction::BackOneLayer);
                }
                _ => {}
            }
        }

        None
    }
}
