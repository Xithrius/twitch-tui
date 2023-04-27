use rustyline::{line_buffer::LineBuffer, At, Word};
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::{
    emotes::Emotes,
    handlers::{
        config::SharedCompleteConfig,
        user_input::{
            events::{Event, Key},
            input::TerminalAction,
        },
    },
    ui::{components::Component, statics::LINE_BUFFER_CAPACITY},
    utils::text::{get_cursor_position, title_spans, TitleStyle},
};

pub type InputValidator = Box<dyn Fn(String) -> bool>;

pub struct InputWidget {
    config: SharedCompleteConfig,
    input: LineBuffer,
    title: String,
    focused: bool,
    input_validator: Option<InputValidator>,
    // TODO: Suggestions
    // suggestion: Option<String>,
}

impl InputWidget {
    pub fn new(
        config: SharedCompleteConfig,
        title: &str,
        input_validator: Option<InputValidator>,
    ) -> Self {
        Self {
            config,
            input: LineBuffer::with_capacity(*LINE_BUFFER_CAPACITY),
            title: title.to_string(),
            focused: false,
            input_validator,
        }
    }

    pub fn update(&mut self, s: &str) {
        self.input.update(s, 0);
    }

    pub const fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn toggle_focus(&mut self) {
        self.focused = !self.focused;
    }

    pub fn is_valid(&self) -> bool {
        self.input_validator
            .as_ref()
            .map_or(true, |validator| validator(self.input.to_string()))
    }
}

impl ToString for InputWidget {
    fn to_string(&self) -> String {
        self.input.to_string()
    }
}

impl Component for InputWidget {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect, _emotes: Option<Emotes>) {
        let cursor_pos = get_cursor_position(&self.input);

        f.set_cursor(
            (area.x + cursor_pos as u16 + 1).min(area.x + area.width.saturating_sub(2)),
            area.y + 1,
        );

        let current_input = self.input.as_str();

        let binding = [TitleStyle::Single(&self.title)];

        let status_color = if self.is_valid() {
            Color::Green
        } else {
            Color::Red
        };

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

    fn event(&mut self, event: &Event) -> Option<TerminalAction> {
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
                // TODO: Have enter key be based off of some input widget attribute
                // Key::Enter => {
                // let input_message = &mut self.input;

                // if input_message.is_empty() || input_message.len() > *TWITCH_MESSAGE_LIMIT {
                //     return None;
                // }

                // let mut message = DataBuilder::user(
                //     self.config.borrow().twitch.username.to_string(),
                //     input_message.to_string(),
                // );
                // if let Some(mut emotes) = self.emotes.clone() {
                //     message.parse_emotes(&mut emotes);
                // }

                // app.messages.push_front(message);

                // self.tx
                //     .send(TwitchAction::Privmsg(input_message.to_string()))
                //     .unwrap();

                // if let Some(msg) = input_message.strip_prefix('@') {
                //     app.storage.add("mentions", msg.to_string());
                // }

                // let mut possible_command = String::new();

                // input_message.clone_into(&mut possible_command);

                // input_message.update("", 0);

                // if possible_command.as_str() == "/clear" {
                //     app.clear_messages();
                // }
                // }
                Key::Ctrl('q') => return Some(TerminalAction::Quit),
                Key::Char(c) => {
                    self.input.insert(*c, 1);
                }
                _ => {}
            }
        }

        None
    }
}
