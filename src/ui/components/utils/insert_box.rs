use rustyline::{line_buffer::LineBuffer, At, Word};
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
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
    utils::text::{get_cursor_position, title_line, TitleStyle},
};

pub type InputValidator = Box<dyn Fn(String) -> bool>;
pub type VisualValidator = Box<dyn Fn(String) -> String>;
pub type InputSuggester = Box<dyn Fn(SharedStorage, String) -> Option<String>>;

pub struct InputWidget {
    config: SharedCompleteConfig,
    input: LineBuffer,
    title: String,
    focused: bool,
    input_validator: Option<InputValidator>,
    visual_indicator: Option<VisualValidator>,
    input_suggester: Option<(SharedStorage, InputSuggester)>,
    suggestion: Option<String>,
}

impl InputWidget {
    pub fn new(
        config: SharedCompleteConfig,
        title: &str,
        input_validator: Option<InputValidator>,
        visual_indicator: Option<VisualValidator>,
        input_suggester: Option<(SharedStorage, InputSuggester)>,
    ) -> Self {
        Self {
            config,
            input: LineBuffer::with_capacity(LINE_BUFFER_CAPACITY),
            title: title.to_string(),
            focused: false,
            input_validator,
            visual_indicator,
            input_suggester,
            suggestion: None,
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
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, _emotes: Option<&mut Emotes>) {
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

        self.suggestion = if self.config.borrow().storage.channels {
            if let Some((storage, suggester)) = &self.input_suggester {
                suggester(storage.clone(), self.input.to_string())
            } else {
                None
            }
        } else {
            None
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(self.config.borrow().frontend.border_type.clone().into())
            .border_style(Style::default().fg(status_color))
            .title(title_line(
                &binding,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ));

        let paragraph_lines = Line::from(vec![
            Span::raw(current_input),
            Span::styled(
                self.suggestion
                    .as_ref()
                    .map_or_else(String::new, |suggestion_buffer| {
                        if suggestion_buffer.len() > current_input.len() {
                            suggestion_buffer[current_input.len()..].to_string()
                        } else {
                            String::new()
                        }
                    }),
                Style::default().add_modifier(Modifier::DIM),
            ),
        ]);

        let paragraph = Paragraph::new(paragraph_lines)
            .block(block)
            .scroll((0, ((cursor_pos + 3) as u16).saturating_sub(area.width)));

        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);

        if let Some(visual) = &self.visual_indicator {
            let contents = visual(self.input.to_string());

            let title = [TitleStyle::Single(&contents)];

            let bottom_block = Block::default()
                .title(title_line(
                    &title,
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ))
                .title_on_bottom()
                .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                .border_type(self.config.borrow().frontend.border_type.clone().into());

            // This is only supposed to render on the very bottom line of the area.
            // If some rendering breaks for input boxes, this is a possible source.
            let rect = Rect::new(area.x, area.bottom() - 1, area.width, 1);
            f.render_widget(bottom_block, rect);
        }
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
                    if self.config.borrow().storage.channels {
                        if let Some(suggestion) = &self.suggestion {
                            self.input.update(suggestion, suggestion.len());
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
