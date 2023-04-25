use std::{collections::VecDeque, slice::Iter};

use chrono::Local;
use log::warn;
use tokio::sync::broadcast::Sender;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::{
    emotes::{emotes_enabled, hide_message_emotes, is_in_rect, show_span_emotes, Emotes},
    handlers::{
        app::SharedMessages,
        config::SharedCompleteConfig,
        data::MessageData,
        filters::SharedFilters,
        state::State,
        user_input::{
            events::{Event, Key},
            input::TerminalAction,
            scrolling::Scrolling,
        },
    },
    twitch::TwitchAction,
    ui::components::Component,
    utils::text::{title_spans, TitleStyle},
};

use super::{utils::centered_rect, ChannelSwitcherWidget, ChatInputWidget};

pub struct ChatWidget {
    config: SharedCompleteConfig,
    messages: SharedMessages,
    chat_input: ChatInputWidget,
    channel_input: ChannelSwitcherWidget,
    filters: SharedFilters,
    pub scroll_offset: Scrolling,
    // theme: Theme,
}

impl ChatWidget {
    pub fn new(
        config: SharedCompleteConfig,
        tx: &Sender<TwitchAction>,
        messages: SharedMessages,
        filters: SharedFilters,
    ) -> Self {
        let chat_input = ChatInputWidget::new(config.clone(), tx.clone());
        let channel_input = ChannelSwitcherWidget::new(config.clone(), tx.clone());
        let scroll_offset = Scrolling::new(config.borrow().frontend.inverted_scrolling);

        Self {
            config,
            messages,
            chat_input,
            channel_input,
            filters,
            scroll_offset,
        }
    }

    pub fn get_messages<'a, B: Backend>(
        &self,
        frame: &mut Frame<B>,
        area: Rect,
        messages_data: &'a VecDeque<MessageData>,
        mut emotes: Emotes,
    ) -> VecDeque<Spans<'a>> {
        // Accounting for not all heights of rows to be the same due to text wrapping,
        // so extra space needs to be used in order to scroll correctly.
        let mut total_row_height: usize = 0;

        let mut messages = VecDeque::new();

        let general_chunk_height = area.height as usize - 2;

        let mut scroll = self.scroll_offset.get_offset();

        // Horizontal chunks represents the list within the main chat window.
        let h_chunk = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1)])
            .split(frame.size());

        let message_chunk_width = h_chunk[0].width as usize;

        let channel_switcher = if self.channel_input.is_focused() {
            Some(centered_rect(60, 20, frame.size()))
        } else {
            None
        };

        let is_behind_channel_switcher =
            |a, b| channel_switcher.map_or(false, |r| is_in_rect(r, a, b));

        let config = self.config.borrow();

        'outer: for data in messages_data.iter() {
            if self
                .filters
                .borrow()
                .contaminated(data.payload.clone().as_str())
            {
                continue;
            }

            // Offsetting of messages for scrolling through said messages
            if scroll > 0 {
                scroll -= 1;
                hide_message_emotes(&data.emotes, &mut emotes.displayed, data.payload.width());
                // let mut map = HashMap::new();
                // hide_message_emotes(&data.emotes, &mut map, data.payload.width());

                continue;
            }

            let username_highlight: Option<&str> = if config.frontend.username_highlight {
                Some(config.twitch.username.as_str())
            } else {
                None
            };

            let spans = data.to_spans(
                &self.config.borrow().frontend,
                message_chunk_width,
                // if input.is_empty() {
                //     None
                // } else {
                //     match app.get_state() {
                //         State::Normal(Some(NormalMode::Search)) => Some(app.input_buffer.as_str()),
                //         _ => None,
                //     }
                // },
                None,
                username_highlight,
            );

            let mut payload = " ".to_string();
            payload.push_str(&data.payload);

            for span in spans.iter().rev() {
                let mut span = span.clone();

                if total_row_height < general_chunk_height {
                    if !data.emotes.is_empty() {
                        let current_row = general_chunk_height - total_row_height;
                        match show_span_emotes(
                            &data.emotes,
                            &mut span,
                            &mut emotes,
                            &payload,
                            self.config.borrow().frontend.margin as usize,
                            current_row as u16,
                            is_behind_channel_switcher,
                        ) {
                            Ok(p) => payload = p,
                            Err(e) => warn!("Unable to display some emotes: {e}"),
                        }
                    }

                    messages.push_front(span);
                    total_row_height += 1;
                } else {
                    if !emotes_enabled(&self.config.borrow().frontend)
                        || emotes.displayed.is_empty()
                    {
                        break 'outer;
                    }

                    // If the current message already had all its emotes deleted, the following messages should
                    // also have had their emotes deleted
                    hide_message_emotes(&data.emotes, &mut emotes.displayed, payload.width());
                    if !data.emotes.is_empty()
                        && !data
                            .emotes
                            .iter()
                            .all(|e| !emotes.displayed.contains_key(&(e.id, e.pid)))
                    {
                        break 'outer;
                    }
                }
            }
        }

        // Padding with empty rows so chat can go from bottom to top.
        if general_chunk_height > total_row_height {
            for _ in 0..(general_chunk_height - total_row_height) {
                messages.push_front(Spans::from(vec![Span::raw("")]));
            }
        }

        messages
    }
}

impl Component for ChatWidget {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Option<Rect>, emotes: Option<Emotes>) {
        // TODO: Don't let this be a thing
        let mut emotes = emotes.unwrap();

        let config = self.config.borrow();

        let area = area.map_or_else(|| f.size(), |a| a);

        let mut v_constraints = vec![Constraint::Min(1)];

        if self.chat_input.is_focused() {
            v_constraints.push(Constraint::Length(3));
        }

        let v_chunks_binding = Layout::default()
            .direction(Direction::Vertical)
            .margin(self.config.borrow().frontend.margin)
            .constraints(v_constraints)
            .split(area);

        let mut v_chunks: Iter<Rect> = v_chunks_binding.iter();

        let first_v_chunk = v_chunks.next().unwrap();

        if self.messages.borrow().len() > self.config.borrow().terminal.maximum_messages {
            for data in self
                .messages
                .borrow()
                .range(self.config.borrow().terminal.maximum_messages..)
            {
                hide_message_emotes(&data.emotes, &mut emotes.displayed, data.payload.width());
            }
            self.messages
                .borrow_mut()
                .truncate(self.config.borrow().terminal.maximum_messages);
        }

        let messages_data = self.messages.clone().borrow().to_owned();

        let messages = self.get_messages(f, *first_v_chunk, &messages_data, emotes.clone());

        let current_time = Local::now()
            .format(&config.frontend.date_format)
            .to_string();

        let spans = [
            TitleStyle::Combined("Time", &current_time),
            TitleStyle::Combined("Channel", config.twitch.channel.as_str()),
            TitleStyle::Custom(Span::styled(
                if self.filters.borrow().reversed() {
                    "retliF"
                } else {
                    "Filter"
                },
                Style::default().add_modifier(Modifier::BOLD).fg(
                    if self.filters.borrow().enabled() {
                        Color::Green
                    } else {
                        Color::Red
                    },
                ),
            )),
        ];

        let chat_title = if self.config.borrow().frontend.title_shown {
            Spans::from(title_spans(
                &spans,
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ))
        } else {
            Spans::default()
        };

        let mut final_messages = vec![];

        for item in messages {
            final_messages.push(ListItem::new(Text::from(item)));
        }

        let list = List::new(final_messages)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(self.config.borrow().frontend.border_type.clone().into())
                    .title(chat_title), // .style(match self.theme {
                                        //     Theme::Light => BORDER_NAME_LIGHT,
                                        //     _ => BORDER_NAME_DARK,
                                        // }),
            )
            .style(Style::default().fg(Color::White));

        f.render_widget(list, *first_v_chunk);

        if self.chat_input.is_focused() {
            self.chat_input
                .draw(f, v_chunks.next().copied(), Some(emotes));
        } else if self.channel_input.is_focused() {
            self.channel_input.draw(f, None, None);
        }
    }

    fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        if let Event::Input(key) = event {
            let limit =
                self.scroll_offset.get_offset() < self.messages.borrow().len().saturating_sub(1);

            if self.chat_input.is_focused() {
                self.chat_input.event(event);
            } else if self.channel_input.is_focused() {
                self.channel_input.event(event);
            } else {
                match key {
                    Key::Char('i') => self.chat_input.toggle_focus(),
                    Key::Char('s') => self.channel_input.toggle_focus(),
                    Key::Ctrl('t') => self.filters.borrow_mut().toggle(),
                    Key::Ctrl('r') => self.filters.borrow_mut().reverse(),
                    Key::Char('S') => return Some(TerminalAction::SwitchState(State::Dashboard)),
                    Key::Char('?') => return Some(TerminalAction::SwitchState(State::Help)),
                    Key::Char('q') => return Some(TerminalAction::Quit),
                    Key::Esc => {
                        if self.scroll_offset.get_offset() == 0 {
                            return Some(TerminalAction::BackOneLayer);
                        }

                        self.scroll_offset.jump_to(0);
                    }
                    Key::Ctrl('p') => panic!("Manual panic triggered by user."),
                    Key::ScrollUp => {
                        if limit {
                            self.scroll_offset.up();
                        } else if self.scroll_offset.inverted() {
                            self.scroll_offset.down();
                        }
                    }
                    Key::ScrollDown => {
                        if self.scroll_offset.inverted() {
                            if limit {
                                self.scroll_offset.up();
                            }
                        } else {
                            self.scroll_offset.down();
                        }
                    }
                    _ => {}
                }
            }
        }

        None
    }
}

// pub fn render_chat_box<T: Backend>(mention_suggestions: bool) {
//     let input_buffer = &app.input_buffer;

//     let current_input = input_buffer.to_string();

//     let suggestion = if mention_suggestions {
//         input_buffer
//             .chars()
//             .next()
//             .and_then(|start_character| match start_character {
//                 '/' => {
//                     let possible_suggestion = first_similarity(
//                         &COMMANDS
//                             .iter()
//                             .map(ToString::to_string)
//                             .collect::<Vec<String>>(),
//                         &current_input[1..],
//                     );

//                     let default_suggestion = possible_suggestion.clone();

//                     possible_suggestion.map_or(default_suggestion, |s| Some(format!("/{s}")))
//                 }
//                 '@' => {
//                     let possible_suggestion =
//                         first_similarity(&app.storage.get("mentions"), &current_input[1..]);

//                     let default_suggestion = possible_suggestion.clone();

//                     possible_suggestion.map_or(default_suggestion, |s| Some(format!("@{s}")))
//                 }
//                 _ => None,
//             })
//     } else {
//         None
//     };

//     render_insert_box(
//         window,
//         format!(
//             "Message Input: {} / {}",
//             current_input.len(),
//             *TWITCH_MESSAGE_LIMIT
//         )
//         .as_str(),
//         None,
//         suggestion,
//         Some(Box::new(|s: String| -> bool {
//             s.len() < *TWITCH_MESSAGE_LIMIT
//         })),
//     );
// }
