use std::{borrow::ToOwned, collections::VecDeque, slice::Iter};

use chrono::Local;
use color_eyre::{Result, eyre::ContextCompat};
use tokio::sync::mpsc::Sender;
use tui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, TitlePosition},
};

use crate::{
    app::SharedMessages,
    config::SharedCoreConfig,
    emotes::SharedEmotes,
    events::{Event, InternalEvent, Key, TwitchAction, TwitchEvent, get_keybind_text},
    handlers::{data::MessageData, filters::SharedFilters, state::State, storage::SharedStorage},
    twitch::oauth::TwitchOauth,
    ui::components::{
        ChannelSwitcherWidget, ChatInputWidget, Component, FollowingWidget, MessageSearchWidget,
        utils::Scrolling,
    },
    utils::{
        styles::{NO_COLOR, TEXT_DARK_STYLE, TITLE_STYLE},
        text::{TitleStyle, title_line},
    },
};

pub struct ChatWidget {
    config: SharedCoreConfig,
    event_tx: Sender<Event>,
    messages: SharedMessages,
    chat_input: ChatInputWidget,
    channel_input: ChannelSwitcherWidget,
    search_input: MessageSearchWidget,
    following: FollowingWidget,
    filters: SharedFilters,
    pub scroll_offset: Scrolling,
    current_channel_name: String,
    /// In selection mode the selected message is the bottom-most visible one, so scrolling moves it.
    selecting: bool,
    /// (message ID, author) of the bottom-most visible message, refreshed on every draw.
    selected: Option<(String, String)>,
    // theme: Theme,
}

impl ChatWidget {
    pub fn new(
        config: SharedCoreConfig,
        twitch_oauth: TwitchOauth,
        event_tx: Sender<Event>,
        messages: SharedMessages,
        storage: &SharedStorage,
        emotes: &SharedEmotes,
        filters: SharedFilters,
    ) -> Self {
        let current_channel_name = config.twitch.channel.clone();

        let chat_input: ChatInputWidget = ChatInputWidget::new(
            config.clone(),
            event_tx.clone(),
            storage.clone(),
            emotes.clone(),
        );
        let channel_input =
            ChannelSwitcherWidget::new(config.clone(), event_tx.clone(), storage.clone());
        let search_input = MessageSearchWidget::new(config.clone(), event_tx.clone());
        let following = FollowingWidget::new(config.clone(), twitch_oauth, event_tx.clone());

        let scroll_offset = Scrolling::new(config.frontend.inverted_scrolling);

        Self {
            config,
            event_tx,
            messages,
            chat_input,
            channel_input,
            search_input,
            following,
            filters,
            scroll_offset,
            current_channel_name,
            selecting: false,
            selected: None,
        }
    }

    pub const fn is_selecting(&self) -> bool {
        self.selecting
    }

    /// Returns true if the key was consumed by selection mode.
    /// Scroll and quit keys fall through to normal handling; every other key is ignored.
    fn handle_selection_key(&mut self, key: Key) -> bool {
        let keybinds = &self.config.keybinds.normal;
        match key {
            key if keybinds.reply.contains(&key) => {
                if let Some((parent_id, author)) = self.selected.clone() {
                    self.selecting = false;
                    self.chat_input.toggle_focus_reply(parent_id, &author);
                }
                true
            }
            key if keybinds.back_to_previous_window.contains(&key) => {
                self.selecting = false;
                true
            }
            key => {
                !(keybinds.scroll_up.contains(&key)
                    || keybinds.scroll_down.contains(&key)
                    || keybinds.quit.contains(&key))
            }
        }
    }

    pub async fn open_in_player(&self) -> Result<()> {
        let channel_name = self.current_channel_name.as_str();
        if self.config.frontend.view_command.is_empty() {
            webbrowser::open(format!(
            "https://player.twitch.tv/?channel={channel_name}&enableExtensions=true&parent=twitch.tv&quality=chunked",
            ).as_str())?;
        } else {
            self.event_tx
                .send(Event::Internal(InternalEvent::OpenStream(
                    channel_name.to_string(),
                )))
                .await?;
        }

        Ok(())
    }

    pub fn get_messages<'a>(
        &self,
        frame: &Frame,
        area: Rect,
        messages_data: &'a VecDeque<MessageData>,
    ) -> (VecDeque<Line<'a>>, Option<&'a MessageData>) {
        // Accounting for not all heights of rows to be the same due to text wrapping,
        // so extra space needs to be used in order to scroll correctly.
        let mut total_row_height: usize = 0;

        let mut messages = VecDeque::new();

        let mut general_chunk_height = area.height as usize;
        if !self.config.frontend.hide_chat_border {
            general_chunk_height -= 2;
        }

        let mut scroll = self.scroll_offset.get_offset();

        let mut selected = None;

        // Horizontal chunks represents the list within the main chat window.
        let h_chunk = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1)])
            .split(frame.area());

        let message_chunk_width = h_chunk
            .first()
            .map_or_else(Rect::default, ToOwned::to_owned)
            .width as usize;

        'outer: for data in messages_data {
            let filters = self.filters.borrow();
            let contaminated_message = filters.message.contaminated(&data.payload);
            let contaminated_username = filters.username.contaminated(&data.author.clone());

            if contaminated_message || contaminated_username {
                continue;
            }

            // Offsetting of messages for scrolling through said messages
            if scroll > 0 {
                scroll -= 1;

                continue;
            }

            let username_highlight: Option<&str> = if self.config.frontend.username_highlight {
                Some(self.config.twitch.username.as_str())
            } else {
                None
            };

            let search = self.search_input.to_string();

            let lines = data.to_vec(
                &self.config.frontend,
                message_chunk_width,
                if self.search_input.is_focused() {
                    Some(&search)
                } else {
                    None
                },
                username_highlight,
            );

            // Background only: emotes are encoded in the foreground and underline colors.
            let selection_style = if self.selecting && selected.is_none() {
                Some(if *NO_COLOR {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default().bg(Color::DarkGray)
                })
            } else {
                None
            };
            selected.get_or_insert(data);

            for span in lines.into_iter().rev() {
                if total_row_height < general_chunk_height {
                    messages.push_front(match selection_style {
                        Some(style) => span.patch_style(style),
                        None => span,
                    });
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

        (messages, selected)
    }
}

impl Component for ChatWidget {
    #[expect(
        clippy::too_many_lines,
        reason = "Chat drawing is long, does not currently need multiple functions"
    )]
    fn draw(&mut self, f: &mut Frame, area: Option<Rect>) -> Result<()> {
        let r = area.unwrap_or_else(|| f.area());

        let mut v_constraints = vec![Constraint::Min(1)];

        if self.chat_input.is_focused() || self.search_input.is_focused() {
            v_constraints.push(Constraint::Length(3));
        }

        let v_chunks_binding = Layout::default()
            .direction(Direction::Vertical)
            .margin(self.config.frontend.margin)
            .constraints(v_constraints)
            .split(r);

        let mut v_chunks: Iter<Rect> = v_chunks_binding.iter();

        let first_v_chunk = v_chunks
            .next()
            .context("Failed to get first vertical chunk")?;

        if self.messages.borrow().len() > self.config.terminal.maximum_messages {
            self.messages
                .borrow_mut()
                .truncate(self.config.terminal.maximum_messages);
        }

        let messages_data = self.messages.borrow();

        let (messages, selected) = self.get_messages(f, *first_v_chunk, &messages_data);
        self.selected =
            selected.and_then(|m| m.message_id.clone().map(|id| (id, m.author.clone())));

        let current_time = Local::now()
            .format(&self.config.frontend.datetime_format)
            .to_string();

        let filters = self.filters.borrow();
        let reversed_filters = filters.message.is_reversed() || filters.username.is_reversed();
        let enabled_filters = filters.message.is_enabled() || filters.username.is_enabled();

        let selection_hint = format!(
            "{} reply, {} cancel",
            get_keybind_text(&self.config.keybinds.normal.reply),
            get_keybind_text(&self.config.keybinds.normal.back_to_previous_window),
        );

        let mut spans = vec![
            TitleStyle::Combined("Time", &current_time),
            TitleStyle::Combined("Channel", self.current_channel_name.as_str()),
            TitleStyle::Custom(Span::styled(
                if reversed_filters { "retliF" } else { "Filter" },
                if *NO_COLOR {
                    Style::default()
                } else {
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(if enabled_filters {
                            Color::Green
                        } else {
                            Color::Red
                        })
                },
            )),
        ];

        if self.selecting {
            spans.push(TitleStyle::Combined("Select", &selection_hint));
        }

        let chat_title = if self.config.frontend.title_shown {
            Line::from(title_line(&spans, *TITLE_STYLE))
        } else {
            Line::default()
        };

        let mut final_messages = vec![];

        for item in messages {
            final_messages.push(ListItem::new(Text::from(item)));
        }

        let list = if self.config.frontend.hide_chat_border {
            List::new(final_messages)
        } else {
            List::new(final_messages).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(self.config.frontend.border_type.clone().into())
                    .title(chat_title),
            )
        }
        .style(*TEXT_DARK_STYLE);

        f.render_widget(list, *first_v_chunk);

        if self.config.frontend.show_scroll_offset {
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
                .border_type(self.config.frontend.border_type.clone().into())
                .title(title_line(&title, Style::default()))
                .title_position(TitlePosition::Bottom)
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
            self.chat_input.draw(f, v_chunks.next().copied())?;
        } else if self.channel_input.is_focused() {
            self.channel_input.draw(f, None)?;
        } else if self.search_input.is_focused() {
            self.search_input.draw(f, v_chunks.next().copied())?;
        } else if self.following.is_focused() {
            self.following.draw(f, None)?;
        }

        Ok(())
    }

    #[expect(
        clippy::too_many_lines,
        reason = "Keybind handling is long, does not currently need multiple functions"
    )]
    async fn event(&mut self, event: &Event) -> Result<()> {
        if let Event::Twitch(TwitchEvent::Action(TwitchAction::JoinChannel(channel))) = event {
            self.current_channel_name.clone_from(channel);
            self.selecting = false;
        }

        // Show the latest messages, where the reply will appear.
        if let Event::Twitch(TwitchEvent::Action(TwitchAction::Reply { .. })) = event {
            self.scroll_offset.jump_to(0);
        }

        if self.chat_input.is_focused() {
            return self.chat_input.event(event).await;
        } else if self.channel_input.is_focused() {
            return self.channel_input.event(event).await;
        } else if self.search_input.is_focused() {
            return self.search_input.event(event).await;
        } else if self.following.is_focused() {
            return self.following.event(event).await;
        }

        if let Event::Input(key) = event {
            let limit =
                self.scroll_offset.get_offset() < self.messages.borrow().len().saturating_sub(1);

            if self.selecting && self.handle_selection_key(*key) {
                return Ok(());
            }

            let keybinds = &self.config.keybinds.normal;
            match key {
                key if keybinds.select_message.contains(key) => self.selecting = true,
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
                key if keybinds.toggle_filters.contains(key) => {
                    let mut filters = self.filters.borrow_mut();
                    filters.message.toggle();
                    filters.username.toggle();
                }
                key if keybinds.reverse_filters.contains(key) => {
                    let mut filters = self.filters.borrow_mut();
                    // TODO: Find a better way to interact with these filters
                    filters.message.reverse();
                    filters.username.reverse();
                }
                key if keybinds.enter_dashboard.contains(key) => {
                    self.event_tx
                        .send(Event::Internal(InternalEvent::SwitchState(
                            State::Dashboard,
                        )))
                        .await?;
                }
                key if keybinds.help.contains(key) => {
                    self.event_tx
                        .send(Event::Internal(InternalEvent::SwitchState(State::Help)))
                        .await?;
                }
                key if keybinds.quit.contains(key) => {
                    self.event_tx
                        .send(Event::Internal(InternalEvent::Quit))
                        .await?;
                }
                key if keybinds.open_in_player.contains(key) => {
                    self.open_in_player().await?;
                }
                key if keybinds.scroll_to_end.contains(key) => {
                    self.scroll_offset.jump_to(0);
                }
                key if keybinds.scroll_to_start.contains(key) => {
                    // TODO: Make this not jump to nothingness
                    self.scroll_offset.jump_to(self.messages.borrow().len());
                }
                key if keybinds.back_to_previous_window.contains(key) => {
                    if self.scroll_offset.get_offset() == 0 {
                        self.event_tx
                            .send(Event::Internal(InternalEvent::BackOneLayer))
                            .await?;
                    }

                    self.scroll_offset.jump_to(0);
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
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::VecDeque, rc::Rc, sync::Arc};

    use chrono::Local;
    use tokio::sync::mpsc::{self, Receiver};
    use tui::{Terminal, backend::TestBackend};

    use super::ChatWidget;
    use crate::{
        config::CoreConfig,
        emotes::Emotes,
        events::{Event, InternalEvent, Key, TwitchAction, TwitchEvent},
        handlers::{data::MessageData, filters::Filters, storage::Storage},
        twitch::oauth::TwitchOauth,
        ui::components::Component,
    };

    fn message(author: &str, id: Option<&str>) -> MessageData {
        MessageData {
            time_sent: Local::now(),
            author: author.to_string(),
            user_id: None,
            system: id.is_none(),
            payload: format!("{author} says hi"),
            emotes: vec![],
            message_id: id.map(ToString::to_string),
            highlight: false,
            badges: None,
        }
    }

    struct Harness {
        chat: ChatWidget,
        rx: Receiver<Event>,
        terminal: Terminal<TestBackend>,
    }

    impl Harness {
        /// Newest message first, like the app's buffer.
        fn new(messages: Vec<MessageData>) -> Self {
            let config = Arc::new(CoreConfig::default());
            let (tx, rx) = mpsc::channel(100);
            let storage = Rc::new(RefCell::new(Storage::new(&config).unwrap()));
            let filters = Rc::new(RefCell::new(Filters::new(&config)));
            let emotes = Rc::new(Emotes::new(false));
            let chat = ChatWidget::new(
                config,
                TwitchOauth::default(),
                tx,
                Rc::new(RefCell::new(VecDeque::from(messages))),
                &storage,
                &emotes,
                filters,
            );
            let terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
            Self { chat, rx, terminal }
        }

        // Draw after every key, as the app loop does; selection is computed while drawing.
        async fn press(&mut self, key: Key) {
            self.chat.event(&Event::Input(key)).await.unwrap();
            self.draw();
        }

        async fn type_text(&mut self, text: &str) {
            for c in text.chars() {
                self.press(Key::Char(c)).await;
            }
        }

        fn draw(&mut self) {
            let chat = &mut self.chat;
            self.terminal.draw(|f| chat.draw(f, None).unwrap()).unwrap();
        }

        fn screen(&self) -> String {
            let buffer = self.terminal.backend().buffer();
            buffer
                .content()
                .iter()
                .map(tui::buffer::Cell::symbol)
                .collect()
        }

        fn sent(&mut self) -> Vec<Event> {
            std::iter::from_fn(|| self.rx.try_recv().ok()).collect()
        }
    }

    fn buffer() -> Vec<MessageData> {
        vec![
            message("carol", Some("id-carol")),
            message("bob", Some("id-bob")),
            message("alice", Some("id-alice")),
        ]
    }

    #[tokio::test]
    async fn reply_to_selected_message_sends_parent_id() {
        let mut h = Harness::new(buffer());
        h.draw();

        h.press(Key::Char('v')).await;
        h.press(Key::Up).await;
        let highlighted: String = h
            .terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .filter(|c| c.bg == tui::style::Color::DarkGray)
            .map(tui::buffer::Cell::symbol)
            .collect();
        assert!(highlighted.contains("bob says hi"), "{highlighted}");
        assert!(!highlighted.contains("carol"), "{highlighted}");

        h.press(Key::Char('r')).await;
        assert!(h.screen().contains("Reply to bob"), "{}", h.screen());

        h.type_text("hello").await;
        h.press(Key::Enter).await;

        assert_eq!(
            h.sent(),
            vec![Event::Twitch(TwitchEvent::Action(TwitchAction::Reply {
                parent_id: "id-bob".to_string(),
                message: "hello".to_string(),
            }))]
        );
    }

    #[tokio::test]
    async fn sending_reply_jumps_to_latest_message() {
        let mut h = Harness::new(buffer());
        h.draw();

        h.press(Key::Char('v')).await;
        h.press(Key::Up).await;
        h.press(Key::Up).await;
        assert_eq!(h.chat.scroll_offset.get_offset(), 2);

        h.press(Key::Char('r')).await;
        h.type_text("hello").await;
        h.press(Key::Enter).await;

        // The app hands every sent action back to the chat widget.
        for event in h.sent() {
            h.chat.event(&event).await.unwrap();
        }
        assert_eq!(h.chat.scroll_offset.get_offset(), 0);
    }

    #[tokio::test]
    async fn message_without_id_cannot_be_replied_to() {
        let mut h = Harness::new(vec![
            message("system", None),
            message("bob", Some("id-bob")),
        ]);
        h.draw();

        h.press(Key::Char('v')).await;
        h.press(Key::Char('r')).await;
        h.type_text("x").await;

        assert!(!h.screen().contains("Reply to"));
        assert_eq!(h.sent(), vec![]);
    }

    #[tokio::test]
    async fn esc_leaves_selection_without_going_back() {
        let mut h = Harness::new(buffer());
        h.draw();

        h.press(Key::Char('v')).await;
        h.press(Key::Esc).await;
        assert_eq!(h.sent(), vec![]);

        // Back in normal mode: `r` is not bound there, `i` sends a plain message.
        h.press(Key::Char('i')).await;
        h.type_text("plain").await;
        h.press(Key::Enter).await;
        assert_eq!(
            h.sent(),
            vec![Event::Twitch(TwitchEvent::Action(TwitchAction::Message(
                "plain".to_string()
            )))]
        );
    }

    #[tokio::test]
    async fn other_keys_ignored_while_selecting() {
        let mut h = Harness::new(buffer());
        h.draw();

        h.press(Key::Char('v')).await;
        h.press(Key::Char('i')).await;
        h.press(Key::Char('q')).await;

        assert!(!h.chat.chat_input.is_focused());
        assert_eq!(h.sent(), vec![Event::Internal(InternalEvent::Quit)]);
    }

    #[tokio::test]
    async fn command_typed_in_reply_mode_runs_as_command() {
        let mut h = Harness::new(buffer());
        h.draw();

        h.press(Key::Char('v')).await;
        h.press(Key::Char('r')).await;
        h.type_text("/clear").await;
        h.press(Key::Enter).await;

        assert_eq!(
            h.sent(),
            vec![Event::Twitch(TwitchEvent::Action(TwitchAction::Message(
                "/clear".to_string()
            )))]
        );
    }

    #[tokio::test]
    async fn cancelled_reply_does_not_leak_into_next_message() {
        let mut h = Harness::new(buffer());
        h.draw();

        h.press(Key::Char('v')).await;
        h.press(Key::Char('r')).await;
        h.press(Key::Esc).await;

        h.press(Key::Char('i')).await;
        assert!(!h.screen().contains("Reply to"));
        h.type_text("plain").await;
        h.press(Key::Enter).await;
        assert_eq!(
            h.sent(),
            vec![Event::Twitch(TwitchEvent::Action(TwitchAction::Message(
                "plain".to_string()
            )))]
        );
    }
}
