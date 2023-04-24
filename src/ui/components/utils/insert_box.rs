use std::{borrow::BorrowMut, cell::RefCell, rc::Rc};

use rustyline::{line_buffer::LineBuffer, At, Word};
use tokio::sync::broadcast::Sender;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::{
    emotes::{Emotes, SharedEmotes},
    handlers::{
        config::SharedCompleteConfig,
        user_input::{
            events::{Event, Key},
            input::TerminalAction,
        },
    },
    twitch::TwitchAction,
    ui::{
        components::{utils::centered_rect, Component},
        statics::LINE_BUFFER_CAPACITY,
    },
    utils::text::{get_cursor_position, title_spans, TitleStyle},
};

#[derive(Debug)]
pub struct InputWidget {
    config: SharedCompleteConfig,
    tx: Sender<TwitchAction>,
    emotes: Option<SharedEmotes>,
    input: LineBuffer,
    title: String,
    // TODO: Implement input buffer validation function
    // input_validation: Option<Box<dyn FnOnce(String) -> bool>>,
    // TODO: Suggestions
    // suggestion: Option<String>,
}

impl InputWidget {
    pub fn new(
        config: SharedCompleteConfig,
        tx: Sender<TwitchAction>,
        emotes: Option<SharedEmotes>,
        title: String,
    ) -> Self {
        Self {
            config,
            tx,
            emotes,
            input: LineBuffer::with_capacity(*LINE_BUFFER_CAPACITY),
            title,
        }
    }

    pub fn update_emotes(&mut self, emotes: Emotes) {
        if let Some(shared_emotes) = self.emotes.borrow_mut() {
            *shared_emotes = Rc::new(RefCell::new(emotes));
        }
    }
}

impl Component for InputWidget {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Option<Rect>) {
        let component_area = area.map_or_else(|| centered_rect(60, 20, f.size()), |a| a);

        let cursor_pos = get_cursor_position(&self.input);

        f.set_cursor(
            (component_area.x + cursor_pos as u16 + 1)
                .min(component_area.x + component_area.width.saturating_sub(2)),
            component_area.y + 1,
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
            .scroll((
                0,
                ((cursor_pos + 3) as u16).saturating_sub(component_area.width),
            ));

        if area.is_some() {
            f.render_widget(Clear, component_area);
        }

        f.render_widget(paragraph, component_area);
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

// pub fn render_insert_box<B: Backend>(
//     f: &mut Frame<B>,
//     area: Rect,
//     config: SharedCompleteConfig,
//     box_title: &str,
//     suggestion: Option<String>,
//     input_validation: Option<Box<dyn FnOnce(String) -> bool>>,
// ) {
//     let cursor_pos = get_cursor_position(buffer);

//     f.set_cursor(
//         (area.x + cursor_pos as u16 + 1).min(area.x + area.width.saturating_sub(2)),
//         area.y + 1,
//     );

//     let current_input = buffer.as_str();

//     let valid_input =
//         input_validation.map_or(true, |check_func| check_func(current_input.to_string()));

//     let binding = [TitleStyle::Single(box_title)];

//     let status_color = if valid_input {
//         Color::Green
//     } else {
//         Color::Red
//     };

//     let paragraph = Paragraph::new(Spans::from(vec![
//         Span::raw(current_input),
//         Span::styled(
//             suggestion
//                 .clone()
//                 .map_or_else(String::new, |suggestion_buffer| {
//                     if suggestion_buffer.len() > current_input.len() {
//                         suggestion_buffer[current_input.len()..].to_string()
//                     } else {
//                         String::new()
//                     }
//                 }),
//             Style::default().add_modifier(Modifier::DIM),
//         ),
//     ]))
//     .block(
//         Block::default()
//             .borders(Borders::ALL)
//             .border_type(frontend.border_type.into())
//             .border_style(Style::default().fg(status_color))
//             .title(title_spans(
//                 &binding,
//                 Style::default()
//                     .fg(status_color)
//                     .add_modifier(Modifier::BOLD),
//             )),
//     )
//     .scroll((0, ((cursor_pos + 3) as u16).saturating_sub(area.width)));

//     // if matches!(app.get_state(), State::ChannelSwitch) {
//     //     frame.render_widget(Clear, area);
//     // }

//     f.render_widget(paragraph, area);

//     // app.buffer_suggestion = suggestion;
// }
