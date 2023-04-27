use rustyline::{line_buffer::LineBuffer, At, Word};
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::{
    emotes::Emotes,
    handlers::{
        config::SharedCompleteConfig,
        storage::SharedStorage,
        user_input::events::{Event, Key},
    },
    terminal::TerminalAction,
    ui::{components::Component, statics::LINE_BUFFER_CAPACITY},
    utils::text::{get_cursor_position, title_spans, TitleStyle},
};

pub type InputValidator = Box<dyn Fn(String) -> bool>;
pub type InputSuggester = Box<dyn Fn(SharedStorage, String) -> Option<String>>;

pub struct InputWidget {
    config: SharedCompleteConfig,
    input: LineBuffer,
    title: String,
    focused: bool,
    input_validator: Option<InputValidator>,
    input_suggester: Option<(SharedStorage, InputSuggester)>,
}

impl InputWidget {
    pub fn new(
        config: SharedCompleteConfig,
        title: &str,
        input_validator: Option<InputValidator>,
        input_suggester: Option<(SharedStorage, InputSuggester)>,
    ) -> Self {
        Self {
            config,
            input: LineBuffer::with_capacity(*LINE_BUFFER_CAPACITY),
            title: title.to_string(),
            focused: false,
            input_validator,
            input_suggester,
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

        let suggestion = if self.config.borrow().storage.channels {
            if let Some((storage, suggester)) = &self.input_suggester {
                suggester(storage.clone(), self.input.to_string())
            } else {
                None
            }
        } else {
            None
        };

        let paragraph = Paragraph::new(Spans::from(vec![
            Span::raw(current_input),
            Span::styled(
                suggestion.map_or_else(String::new, |suggestion_buffer| {
                    if suggestion_buffer.len() > current_input.len() {
                        suggestion_buffer[current_input.len()..].to_string()
                    } else {
                        String::new()
                    }
                }),
                Style::default().add_modifier(Modifier::DIM),
            ),
        ]))
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
                Key::Tab => {
                    // TODO: Have this be a shared suggestion so it doesn't have to run twice
                    if self.config.borrow().storage.channels {
                        if let Some((storage, suggester)) = &self.input_suggester {
                            if let Some(suggestion) =
                                suggester(storage.clone(), self.input.to_string())
                            {
                                self.input.update(&suggestion, suggestion.len());
                            }
                        }
                    }
                }
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
