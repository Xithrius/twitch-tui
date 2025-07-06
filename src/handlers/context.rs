use std::{
    cell::RefCell,
    collections::VecDeque,
    process::{Child, Command, Stdio},
    rc::Rc,
};

use chrono::{DateTime, Local};
use tui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::{
    emotes::{Emotes, SharedEmotes},
    handlers::{
        config::{CoreConfig, SharedCoreConfig},
        data::MessageData,
        filters::Filters,
        state::State,
        storage::{SharedStorage, Storage},
        user_input::events::{Event, Key},
    },
    terminal::TerminalAction,
    ui::components::{Component, Components},
};

use tracing::{error};

pub type SharedMessages = Rc<RefCell<VecDeque<MessageData>>>;

pub struct Context {
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
    ///Running stream
    pub running_stream: Option<Child>,
}

macro_rules! shared {
    ($expression:expr) => {
        Rc::new(RefCell::new($expression))
    };
}

impl Context {
    pub fn new(config: CoreConfig, startup_time: DateTime<Local>) -> Self {
        let shared_config = shared!(config.clone());

        let shared_config_borrow = shared_config.borrow();

        // TODO: Storage path should be specified in the config, default next to config.toml
        let storage = shared!(Storage::new("storage.json", &shared_config_borrow.storage));

        if !storage
            .borrow()
            .contains("channels", &config.twitch.channel)
        {
            storage.borrow_mut().add("channels", config.twitch.channel);
        }

        // TODO: Filters path should be specified in the config, default next to config.toml
        let filters = shared!(Filters::new("filters.txt", &shared_config_borrow.filters));

        let messages = shared!(VecDeque::with_capacity(
            shared_config_borrow.terminal.maximum_messages,
        ));

        let emotes_enabled: bool = shared_config.borrow().frontend.is_emotes_enabled();
        let emotes = Rc::new(Emotes::new(emotes_enabled));

        let components = Components::new(
            &shared_config,
            storage.clone(),
            filters,
            messages.clone(),
            &emotes,
            startup_time,
        );

        Self {
            components,
            config: shared_config.clone(),
            messages,
            storage,
            state: shared_config_borrow.terminal.first_state.clone(),
            previous_state: None,
            emotes,
            running_stream: None,
        }
    }

    pub fn draw(&mut self, f: &mut Frame) {
        let mut size = f.area();

        if self.config.borrow().frontend.state_tabs {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(size.height - 1), Constraint::Length(1)])
                .split(f.area());

            size = layout[0];

            self.components.tabs.draw(f, Some(layout[1]), &self.state);
        }

        if (size.height < 10 || size.width < 60)
            && self.config.borrow().frontend.show_unsupported_screen_size
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

    pub async fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        if let Event::Input(key) = event {
            if self.components.debug.is_focused() {
                return self.components.debug.event(event).await;
            }

            match key {
                // Global keybinds
                Key::Ctrl('d') => {
                    self.components.debug.toggle_focus();
                }
                _ => {
                    return match self.state {
                        State::Dashboard => self.components.dashboard.event(event).await,
                        State::Normal => self.components.chat.event(event).await,
                        State::Help => self.components.help.event(event).await,
                    };
                }
            }
        }

        None
    }

    //TODO error handling
    pub fn open_stream(&mut self, channel: &str) {
        self.close_current_stream();
        let config = self.config.borrow();
        if let Some(view_command) = config.frontend.view_command.as_ref() {
            self.running_stream = Command::new(view_command)
                .arg(format!("https://twitch.tv/{channel}"))
                .args(
                    config
                        .frontend
                        .view_command_args
                        .as_ref()
                        .map_or_else(|| &[] as &[String], |view_args| view_args.as_slice()),
                )
                .stdout(Stdio::null())
                .spawn()
                .map_or_else(
                    |err| {
                        error!("error spawning view process: {err}");
                        None
                    },
                    |process| Some(process),
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

    //TODO keep in mind this mut in case i have to make it a refcell
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
