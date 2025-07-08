use std::{collections::VecDeque, slice::Iter};

use chrono::Local;
use tui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, block::Position},
};

use crate::{
    emotes::SharedEmotes,
    handlers::{
        config::SharedCoreConfig,
        context::SharedMessages,
        data::MessageData,
        filters::SharedFilters,
        state::State,
        storage::SharedStorage,
        user_input::{events::Event, scrolling::Scrolling},
    },
    terminal::TerminalAction,
    ui::components::{
        ChannelSwitcherWidget, ChatInputWidget, Component, MessageSearchWidget,
        following::FollowingWidget,
    },
    utils::{
        styles::{NO_COLOR, TEXT_DARK_STYLE, TITLE_STYLE},
        text::{TitleStyle, title_line},
    },
};

pub struct ChatWidget {
    config: SharedCoreConfig,
    messages: SharedMessages,
    chat_input: ChatInputWidget,
    channel_input: ChannelSwitcherWidget,
    search_input: MessageSearchWidget,
    following: FollowingWidget,
    filters: SharedFilters,
    pub scroll_offset: Scrolling,
    // theme: Theme,
}

impl ChatWidget {
    pub fn new(
        config: SharedCoreConfig,
        messages: SharedMessages,
        storage: &SharedStorage,
        emotes: &SharedEmotes,
        filters: SharedFilters,
    ) -> Self {
        let chat_input = ChatInputWidget::new(config.clone(), storage.clone(), emotes.clone());
        let channel_input = ChannelSwitcherWidget::new(config.clone(), storage.clone());
        let search_input = MessageSearchWidget::new(config.clone());
        let following = FollowingWidget::new(config.clone());

        let scroll_offset = Scrolling::new(config.borrow().frontend.inverted_scrolling);

        Self {
            config,
            messages,
            chat_input,
            channel_input,
            search_input,
            following,
            filters,
            scroll_offset,
        }
    }

    pub fn open_in_player(&self) -> Option<TerminalAction> {
        let config = self.config.borrow();
        //TODO dedupe #3
        let channel_name = if config.frontend.only_get_live_followed_channels {
            config
                .twitch
                .channel
                .split(':')
                .next()
                .map_or_else(|| config.twitch.channel.as_str(), |name| name.trim_end())
        } else {
            config.twitch.channel.as_str()
        };

        let has_non_empty_view_command = if let Some(view_command) = &config.frontend.view_command {
            !view_command.is_empty()
        } else {
            false
        };
        if has_non_empty_view_command {
            Some(TerminalAction::OpenStream(channel_name.to_string()))
        } else {
            webbrowser::open(format!(
            "https://player.twitch.tv/?channel={channel_name}&enableExtensions=true&parent=twitch.tv&quality=chunked",
            ).as_str()).unwrap();
            None
        }
    }

    pub fn get_messages<'a>(
        &self,
        frame: &Frame,
        area: Rect,
        messages_data: &'a VecDeque<MessageData>,
    ) -> VecDeque<Line<'a>> {
        // Accounting for not all heights of rows to be the same due to text wrapping,
        // so extra space needs to be used in order to scroll correctly.
        let mut total_row_height: usize = 0;

        let mut messages = VecDeque::new();

        let mut general_chunk_height = area.height as usize;
        if !self.config.borrow().frontend.hide_chat_border {
            general_chunk_height -= 2;
        }

        let mut scroll = self.scroll_offset.get_offset();

        // Horizontal chunks represents the list within the main chat window.
        let h_chunk = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1)])
            .split(frame.area());

        let message_chunk_width = h_chunk[0].width as usize;

        let config = self.config.borrow();

        'outer: for data in messages_data {
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

                continue;
            }

            let username_highlight: Option<&str> = if config.frontend.username_highlight {
                Some(config.twitch.username.as_str())
            } else {
                None
            };

            let search = self.search_input.to_string();

            let lines = data.to_vec(
                &self.config.borrow().frontend,
                message_chunk_width,
                if self.search_input.is_focused() {
                    Some(&search)
                } else {
                    None
                },
                username_highlight,
            );

            for span in lines.into_iter().rev() {
                if total_row_height < general_chunk_height {
                    messages.push_front(span);
                    total_row_height += 1;
                } else {
                    break 'outer;
                }
            }
        }

        // Padding with empty rows so chat can go from bottom to top.
        if general_chunk_height > total_row_height {
            for _ in 0..(general_chunk_height - total_row_height) {
                messages.push_front(Line::from(vec![Span::raw("")]));
            }
        }

        messages
    }
}

impl Component for ChatWidget {
    fn draw(&mut self, f: &mut Frame, area: Option<Rect>) {
        let r = area.map_or_else(|| f.area(), |a| a);

        let config = self.config.borrow();

        let mut v_constraints = vec![Constraint::Min(1)];

        if self.chat_input.is_focused() || self.search_input.is_focused() {
            v_constraints.push(Constraint::Length(3));
        }

        let v_chunks_binding = Layout::default()
            .direction(Direction::Vertical)
            .margin(self.config.borrow().frontend.margin)
            .constraints(v_constraints)
            .split(r);

        let mut v_chunks: Iter<Rect> = v_chunks_binding.iter();

        let first_v_chunk = v_chunks.next().unwrap();

        if self.messages.borrow().len() > self.config.borrow().terminal.maximum_messages {
            self.messages
                .borrow_mut()
                .truncate(self.config.borrow().terminal.maximum_messages);
        }

        let messages_data = self.messages.borrow();

        let messages = self.get_messages(f, *first_v_chunk, &messages_data);

        let current_time = Local::now()
            .format(&config.frontend.datetime_format)
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
                if *NO_COLOR {
                    Style::default()
                } else {
                    Style::default().add_modifier(Modifier::BOLD).fg(
                        if self.filters.borrow().enabled() {
                            Color::Green
                        } else {
                            Color::Red
                        },
                    )
                },
            )),
        ];

        let chat_title = if self.config.borrow().frontend.title_shown {
            Line::from(title_line(&spans, *TITLE_STYLE))
        } else {
            Line::default()
        };

        let mut final_messages = vec![];

        for item in messages {
            final_messages.push(ListItem::new(Text::from(item)));
        }

        let list = if self.config.borrow().frontend.hide_chat_border {
            List::new(final_messages)
        } else {
            List::new(final_messages).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(self.config.borrow().frontend.border_type.clone().into())
                    .title(chat_title),
            )
        }
        .style(*TEXT_DARK_STYLE);

        f.render_widget(list, *first_v_chunk);

        if self.config.borrow().frontend.show_scroll_offset {
            // Cannot scroll past the first message
            let message_amount = messages_data.len().saturating_sub(1);

            let title_binding = format!(
                "{} / {}",
                self.scroll_offset.get_offset(),
                message_amount.to_string().as_str()
            );

            let title = [TitleStyle::Single(&title_binding)];

            let bottom_block = Block::default()
                .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                .border_type(self.config.borrow().frontend.border_type.clone().into())
                .title(title_line(&title, Style::default()))
                .title_position(Position::Bottom)
                .title_alignment(Alignment::Right);

            let rect = Rect::new(
                first_v_chunk.x,
                first_v_chunk.bottom() - 1,
                first_v_chunk.width,
                1,
            );

            f.render_widget(bottom_block, rect);
        }

        if self.chat_input.is_focused() {
            self.chat_input.draw(f, v_chunks.next().copied());
        } else if self.channel_input.is_focused() {
            self.channel_input.draw(f, None);
        } else if self.search_input.is_focused() {
            self.search_input.draw(f, v_chunks.next().copied());
        } else if self.following.is_focused() {
            self.following.draw(f, None);
        }
    }

    #[allow(clippy::cognitive_complexity)]
    async fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        if let Event::Input(key) = event {
            let limit =
                self.scroll_offset.get_offset() < self.messages.borrow().len().saturating_sub(1);

            if self.chat_input.is_focused() {
                self.chat_input.event(event).await
            } else if self.channel_input.is_focused() {
                self.channel_input.event(event).await
            } else if self.search_input.is_focused() {
                self.search_input.event(event).await
            } else if self.following.is_focused() {
                self.following.event(event).await
            } else {
                let keybinds = self.config.borrow().keybinds.normal.clone();
                match key {
                    key if keybinds.enter_insert.contains(key) => self.chat_input.toggle_focus(),
                    key if keybinds.enter_insert_with_mention.contains(key) => {
                        self.chat_input.toggle_focus_with("@");
                    }
                    key if keybinds.enter_insert_with_command.contains(key) => {
                        self.chat_input.toggle_focus_with("/");
                    }
                    key if keybinds.recent_channels_search.contains(key) => {
                        self.channel_input.toggle_focus();
                    }
                    key if keybinds.search_messages.contains(key) => {
                        self.search_input.toggle_focus();
                    }
                    key if keybinds.followed_channels_search.contains(key) => {
                        self.following.toggle_focus().await;
                    }
                    key if keybinds.toggle_message_filter.contains(key) => {
                        self.filters.borrow_mut().toggle();
                    }
                    key if keybinds.reverse_message_filter.contains(key) => {
                        self.filters.borrow_mut().reverse();
                    }
                    key if keybinds.enter_dashboard.contains(key) => {
                        return Some(TerminalAction::SwitchState(State::Dashboard));
                    }
                    key if keybinds.help.contains(key) => {
                        return Some(TerminalAction::SwitchState(State::Help));
                    }
                    key if keybinds.quit.contains(key) => return Some(TerminalAction::Quit),
                    key if keybinds.open_in_player.contains(key) => return self.open_in_player(),
                    key if keybinds.scroll_to_end.contains(key) => {
                        self.scroll_offset.jump_to(0);
                    }
                    key if keybinds.scroll_to_start.contains(key) => {
                        // TODO: Make this not jump to nothingness
                        self.scroll_offset.jump_to(self.messages.borrow().len());
                    }
                    key if keybinds.back_to_previous_window.contains(key) => {
                        if self.scroll_offset.get_offset() == 0 {
                            return Some(TerminalAction::BackOneLayer);
                        }

                        self.scroll_offset.jump_to(0);
                    }
                    key if keybinds.crash_application.contains(key) => {
                        panic!("Manual panic triggered by user.")
                    }
                    key if keybinds.scroll_up.contains(key) => {
                        if limit {
                            self.scroll_offset.up();
                        } else if self.scroll_offset.is_inverted() {
                            self.scroll_offset.down();
                        }
                    }
                    key if keybinds.scroll_down.contains(key) => {
                        if self.scroll_offset.is_inverted() {
                            if limit {
                                self.scroll_offset.up();
                            }
                        } else {
                            self.scroll_offset.down();
                        }
                    }
                    _ => {}
                }

                None
            }
        } else {
            None
        }
    }
}
