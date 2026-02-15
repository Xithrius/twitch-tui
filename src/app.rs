use std::{
    cell::RefCell,
    collections::VecDeque,
    process::{Child, Command, Stdio},
    rc::Rc,
};

use color_eyre::Result;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    oneshot::Receiver as OSReceiver,
};
use tracing::{error, warn};
use tui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::{
    config::SharedCoreConfig,
    emotes::{
        ApplyCommand, DecodedEmote, DownloadedEmotes, Emotes, SharedEmotes, display_emote,
        query_emotes,
    },
    events::{Event, Events, InternalEvent, TwitchAction, TwitchEvent, TwitchNotification},
    handlers::{
        data::{KNOWN_CHATTERS, MessageData},
        filters::Filters,
        state::State,
        storage::{SharedStorage, Storage},
    },
    twitch::oauth::TwitchOauth,
    ui::components::{Component, Components},
    utils::sanitization::clean_channel_name,
};

pub type SharedMessages = Rc<RefCell<VecDeque<MessageData>>>;

pub struct App {
    pub running: bool,

    /// UI components
    pub components: Components,

    /// Configuration loaded from file and CLI arguments
    pub config: SharedCoreConfig,

    /// Twitch OAuth client and session info
    pub twitch_oauth: TwitchOauth,
    pub events: Events,
    pub twitch_tx: Sender<TwitchAction>,

    pub messages: SharedMessages,

    /// Data loaded in from a JSON file.
    pub storage: SharedStorage,

    /// States
    state: State,
    previous_state: Option<State>,

    /// Emote encoding pipeline
    pub emotes: SharedEmotes,
    pub emotes_rx: OSReceiver<(DownloadedEmotes, DownloadedEmotes)>,
    pub decoded_emotes_rx: Option<Receiver<Result<DecodedEmote, String>>>,

    pub running_stream: Option<Child>,
}

macro_rules! shared {
    ($expression:expr) => {
        Rc::new(RefCell::new($expression))
    };
}

impl App {
    pub fn new(
        config: SharedCoreConfig,
        twitch_oauth: TwitchOauth,
        events: Events,
        event_tx: Sender<Event>,
        twitch_tx: Sender<TwitchAction>,
        emotes: Rc<Emotes>,
        decoded_emotes_rx: Option<Receiver<Result<DecodedEmote, String>>>,
    ) -> Self {
        let maximum_messages = config.terminal.maximum_messages;
        let first_state = config.terminal.first_state.clone();

        let storage = shared!(Storage::new(&config));
        let filters = shared!(Filters::new(&config));
        let messages = shared!(VecDeque::with_capacity(maximum_messages));

        let components = Components::builder()
            .config(&config)
            .twitch_oauth(twitch_oauth.clone())
            .event_tx(event_tx)
            .storage(storage.clone())
            .filters(filters)
            .messages(messages.clone())
            .emotes(&emotes)
            .build();

        let emotes_rx = query_emotes(&config, twitch_oauth.clone(), config.twitch.channel.clone());

        Self {
            running: true,
            components,
            config,
            twitch_oauth,
            events,
            twitch_tx,
            messages,
            storage,
            state: first_state,
            previous_state: None,
            emotes,
            emotes_rx,
            decoded_emotes_rx,
            running_stream: None,
        }
    }

    pub fn open_stream(&mut self, channel: &str) {
        self.close_current_stream();
        let view_command = &self.config.frontend.view_command;

        if let Some((command, args)) = view_command.split_first() {
            self.running_stream = Command::new(command.clone())
                .args(args)
                .arg(format!("https://twitch.tv/{channel}"))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_or_else(
                    |err| {
                        error!("error spawning view process: {err}");
                        None
                    },
                    Some,
                );
        }
    }

    pub fn close_current_stream(&mut self) {
        if let Some(process) = self.running_stream.as_mut() {
            _ = process
                .kill()
                .inspect_err(|err| error!("failed to kill view process: {err}"));
        }
        self.running_stream = None;
    }

    pub fn cleanup(&mut self) {
        self.close_current_stream();
        self.storage.borrow().dump_data();
        self.emotes.unload();
    }

    pub fn clear_messages(&mut self) {
        self.messages.borrow_mut().clear();

        self.components.chat.scroll_offset.jump_to(0);
    }

    pub fn purge_user_messages(&self, user_id: &str) {
        let messages = self
            .messages
            .borrow_mut()
            .iter()
            .filter(|&m| m.user_id.clone().is_none_or(|user| user != user_id))
            .cloned()
            .collect::<VecDeque<MessageData>>();

        self.messages.replace(messages);
    }

    pub fn remove_message_with(&self, message_id: &str) {
        let index = self
            .messages
            .borrow_mut()
            .iter()
            .position(|f| f.message_id.clone().is_some_and(|id| id == message_id));

        if let Some(i) = index {
            self.messages.borrow_mut().remove(i).unwrap();
        }
    }

    pub fn get_previous_state(&self) -> Option<State> {
        self.previous_state.clone()
    }

    pub fn set_state(&mut self, other: State) {
        self.previous_state = Some(self.state.clone());
        self.state = other;
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let is_emotes_enabled = self.emotes.enabled;

        while self.running {
            if is_emotes_enabled {
                self.handle_emote_events();
            }

            if let Some(event) = self.events.next().await {
                self.event(&event).await?;
            }

            terminal.draw(|f| self.draw(f, Some(f.area()))).unwrap();
        }

        self.cleanup();

        Ok(())
    }

    pub fn handle_emote_events(&mut self) {
        // Check if we have received any emotes
        if let Ok((user_emotes, global_emotes)) = self.emotes_rx.try_recv() {
            *self.emotes.user_emotes.borrow_mut() = user_emotes;
            *self.emotes.global_emotes.borrow_mut() = global_emotes;

            for message in &mut *self.messages.borrow_mut() {
                message.reparse_emotes(&self.emotes);
            }
        }

        // Check if we need to load a decoded emote
        if let Some(rx) = &mut self.decoded_emotes_rx {
            if let Ok(r) = rx.try_recv() {
                match r {
                    Ok(d) => {
                        if let Err(e) = d.apply() {
                            warn!("Unable to send command to load emote. {e}");
                        } else if let Err(e) = display_emote(d.id(), 1, d.cols()) {
                            warn!("Unable to send command to display emote. {e}");
                        }
                    }
                    Err(name) => {
                        warn!("Unable to load emote: {name}.");
                        self.emotes.user_emotes.borrow_mut().remove(&name);
                        self.emotes.global_emotes.borrow_mut().remove(&name);
                        self.emotes.info.borrow_mut().remove(&name);
                    }
                }
            }
        }
    }
}

impl Component for App {
    fn draw(&mut self, f: &mut Frame, area: Option<Rect>) {
        let mut size = area.unwrap_or_else(|| f.area());

        if self.config.frontend.state_tabs {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(size.height - 1), Constraint::Length(1)])
                .split(f.area());

            size = layout[0];

            self.components.tabs.draw(f, Some(layout[1]), &self.state);
        }

        if (size.height < 10 || size.width < 60)
            && self.config.frontend.show_unsupported_screen_size
        {
            self.components.window_size_error.draw(f, Some(f.area()));
        } else {
            match self.state {
                State::Dashboard => self.components.dashboard.draw(f, None),
                State::Normal => self.components.chat.draw(f, None),
                State::Help => self.components.help.draw(f, None),
            }
        }

        if self.components.debug.is_focused() {
            let new_rect = Rect::new(size.x, size.y + 1, size.width - 1, size.height - 2);

            let rect = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(new_rect)[1];

            self.components.debug.draw(f, Some(rect));
        }
    }

    async fn event(&mut self, event: &Event) -> Result<()> {
        match event.clone() {
            Event::Internal(internal_event) => match internal_event {
                InternalEvent::Quit => self.running = false,
                InternalEvent::BackOneLayer => {
                    if let Some(previous_state) = self.get_previous_state() {
                        self.set_state(previous_state);
                    } else {
                        self.set_state(self.config.terminal.first_state.clone());
                    }
                }
                InternalEvent::SwitchState(state) => {
                    if state == State::Normal {
                        self.clear_messages();
                    }

                    self.set_state(state);
                }
                InternalEvent::OpenStream(channel) => {
                    self.open_stream(&channel);
                }
                InternalEvent::SelectEmote(_) => {}
            },
            Event::Twitch(twitch_event) => match twitch_event {
                TwitchEvent::Action(twitch_action) => match twitch_action {
                    TwitchAction::JoinChannel(channel) => {
                        let channel = clean_channel_name(&channel);
                        self.clear_messages();
                        self.emotes.unload();

                        self.twitch_tx
                            .send(TwitchAction::JoinChannel(channel.clone()))
                            .await?;

                        if self.config.frontend.autostart_view_command {
                            self.open_stream(&channel);
                        }
                        self.emotes_rx =
                            query_emotes(&self.config, self.twitch_oauth.clone(), channel);
                        self.set_state(State::Normal);
                    }
                    TwitchAction::Message(message) => {
                        self.twitch_tx.send(TwitchAction::Message(message)).await?;
                    }
                },
                TwitchEvent::Notification(twitch_notification) => {
                    match twitch_notification {
                        TwitchNotification::Message(m) => {
                            let message_data = MessageData::from_twitch_message(m, &self.emotes);
                            if !KNOWN_CHATTERS.contains(&message_data.author.as_str())
                                && self.config.twitch.username != message_data.author
                            {
                                self.storage
                                    .borrow_mut()
                                    .add("chatters", message_data.author.clone());
                            }
                            self.messages.borrow_mut().push_front(message_data);

                            // If scrolling is enabled, pad for more messages.
                            if self.components.chat.scroll_offset.get_offset() > 0 {
                                self.components.chat.scroll_offset.up();
                            }
                        }
                        TwitchNotification::ClearChat(user_id) => {
                            if let Some(user) = user_id {
                                self.purge_user_messages(user.as_str());
                            } else {
                                self.clear_messages();
                            }
                        }
                        TwitchNotification::DeleteMessage(message_id) => {
                            self.remove_message_with(message_id.as_str());
                        }
                    }
                }
            },
            Event::Input(key) => {
                if self.components.debug.is_focused() {
                    return self.components.debug.event(event).await;
                }

                if self.config.keybinds.toggle_debug_focus.contains(&key) {
                    self.components.debug.toggle_focus();
                }
            }
            Event::Tick => {}
        }

        match self.state {
            State::Dashboard => self.components.dashboard.event(event).await,
            State::Normal => self.components.chat.event(event).await,
            State::Help => self.components.help.event(event).await,
        }
    }
}
