use std::{
    cell::RefCell,
    collections::VecDeque,
    process::{Child, Command, Stdio},
    rc::Rc,
};

use color_eyre::Result;
use tokio::sync::mpsc::Sender;
use tracing::error;
use tui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::{
    config::SharedCoreConfig,
    emotes::{Emotes, SharedEmotes},
    events::Event,
    handlers::{
        data::MessageData,
        filters::Filters,
        state::State,
        storage::{SharedStorage, Storage},
    },
    twitch::oauth::TwitchOauth,
    ui::components::{Component, Components},
};

pub type SharedMessages = Rc<RefCell<VecDeque<MessageData>>>;

pub struct App {
    /// All the available components.
    pub components: Components,
    /// Shared core config loaded from file and CLI arguments.
    pub config: SharedCoreConfig,
    /// History of recorded messages (time, username, message, etc).
    pub messages: SharedMessages,
    /// Data loaded in from a JSON file.
    pub storage: SharedStorage,
    /// Which window the terminal is currently focused on.
    state: State,
    /// The previous state, if any.
    previous_state: Option<State>,
    /// Emotes
    pub emotes: SharedEmotes,
    /// Running stream
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
        event_tx: Sender<Event>,
        emotes: Rc<Emotes>,
    ) -> Self {
        let maximum_messages = config.terminal.maximum_messages;
        let first_state = config.terminal.first_state.clone();

        let storage = shared!(Storage::new(&config));
        let filters = shared!(Filters::new(&config));
        let messages = shared!(VecDeque::with_capacity(maximum_messages));

        let components = Components::builder()
            .config(&config)
            .twitch_oauth(twitch_oauth)
            .event_tx(event_tx)
            .storage(storage.clone())
            .filters(filters)
            .messages(messages.clone())
            .emotes(&emotes)
            .build();

        Self {
            components,
            config,
            messages,
            storage,
            state: first_state,
            previous_state: None,
            emotes,
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
        if let Event::Input(key) = event {
            if self.components.debug.is_focused() {
                return self.components.debug.event(event).await;
            }

            if self.config.keybinds.toggle_debug_focus.contains(key) {
                self.components.debug.toggle_focus();
            }
        }

        match self.state {
            State::Dashboard => self.components.dashboard.event(event).await,
            State::Normal => self.components.chat.event(event).await,
            State::Help => self.components.help.event(event).await,
        }
    }
}
