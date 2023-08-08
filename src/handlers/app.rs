use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use chrono::{DateTime, Local};
use rustyline::line_buffer::LineBuffer;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::{
    emotes::Emotes,
    handlers::{
        config::{CompleteConfig, SharedCompleteConfig, Theme},
        data::MessageData,
        filters::{Filters, SharedFilters},
        state::State,
        storage::{SharedStorage, Storage},
        user_input::events::{Event, Key},
    },
    terminal::TerminalAction,
    ui::{
        components::{Component, Components},
        statics::LINE_BUFFER_CAPACITY,
    },
};

pub type SharedMessages = Rc<RefCell<VecDeque<MessageData>>>;

pub struct App {
    /// All the available components.
    pub components: Components,
    /// A config for the app and components to share.
    pub config: SharedCompleteConfig,
    /// History of recorded messages (time, username, message, etc).
    pub messages: SharedMessages,
    /// Data loaded in from a JSON file.
    pub storage: SharedStorage,
    /// Messages to be filtered out.
    pub filters: SharedFilters,
    /// Which window the terminal is currently focused on.
    state: State,
    /// The previous state, if any.
    previous_state: Option<State>,
    /// What the user currently has inputted.
    pub input_buffer: LineBuffer,
    /// The current suggestion, if any.
    pub buffer_suggestion: Option<String>,
    /// The theme selected by the user.
    pub theme: Theme,
    /// Emotes
    pub emotes: Emotes,
}

macro_rules! shared {
    ($expression:expr) => {
        Rc::new(RefCell::new($expression))
    };
}

impl App {
    pub fn new(config: CompleteConfig, startup_time: DateTime<Local>) -> Self {
        let shared_config = shared!(config.clone());

        let shared_config_borrow = shared_config.borrow();

        let storage = shared!(Storage::new("storage.json", &shared_config_borrow.storage));

        if !storage
            .borrow()
            .contains("channels", &config.twitch.channel)
        {
            storage.borrow_mut().add("channels", config.twitch.channel);
        }

        let filters = shared!(Filters::new("filters.txt", &shared_config_borrow.filters));

        let messages = shared!(VecDeque::with_capacity(
            shared_config_borrow.terminal.maximum_messages,
        ));

        let components = Components::new(
            &shared_config,
            storage.clone(),
            filters.clone(),
            messages.clone(),
            startup_time,
        );

        Self {
            components,
            config: shared_config.clone(),
            messages,
            storage,
            filters,
            state: shared_config_borrow.terminal.first_state.clone(),
            previous_state: None,
            input_buffer: LineBuffer::with_capacity(LINE_BUFFER_CAPACITY),
            buffer_suggestion: None,
            theme: shared_config_borrow.frontend.theme.clone(),
            emotes: Emotes::default(),
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        let mut size = f.size();

        if self.config.borrow().frontend.state_tabs {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(size.height - 1), Constraint::Length(1)])
                .split(f.size());

            size = layout[0];

            self.components.tabs.draw(f, Some(layout[1]), &self.state);
        }

        if size.height < 10 || size.width < 60 {
            self.components.error.draw(f, f.size(), None);
        } else {
            match self.state {
                State::Dashboard => self.components.dashboard.draw(f, size, None),
                State::Normal => self.components.chat.draw(f, size, Some(&mut self.emotes)),
                State::Help => self.components.help.draw(f, size, None),
            }
        }

        if self.components.debug.is_focused() {
            let new_rect = Rect::new(size.x, size.y + 1, size.width - 1, size.height - 2);

            let rect = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(new_rect)[1];

            self.components.debug.draw(f, rect, None);
        }
    }

    pub fn event(&mut self, event: &Event) -> Option<TerminalAction> {
        if let Event::Input(key) = event {
            if self.components.debug.is_focused() {
                return self.components.debug.event(event);
            }

            match key {
                // Global keybinds
                Key::Ctrl('d') => {
                    self.components.debug.toggle_focus();
                }
                _ => {
                    return match self.state {
                        State::Dashboard => self.components.dashboard.event(event),
                        State::Normal => self.components.chat.event(event),
                        State::Help => self.components.help.event(event),
                    };
                }
            }
        }

        None
    }

    pub fn cleanup(&self) {
        self.storage.borrow().dump_data();
    }

    pub fn clear_messages(&mut self) {
        self.emotes.clear();
        self.messages.borrow_mut().clear();

        self.components.chat.scroll_offset.jump_to(0);
    }

    pub fn remove_message_with(&mut self, message_id: &str) {
        let index = self
            .messages
            .borrow_mut()
            .iter()
            .position(|f| f.message_id.clone().map_or(false, |id| id == message_id));

        if let Some(i) = index {
            self.messages.borrow_mut().remove(i).unwrap();
        }
    }

    pub fn get_previous_state(&self) -> Option<State> {
        self.previous_state.clone()
    }

    #[allow(dead_code)]
    pub fn get_state(&self) -> State {
        self.state.clone()
    }

    pub fn set_state(&mut self, other: State) {
        self.emotes.clear();
        self.previous_state = Some(self.state.clone());
        self.state = other;
    }

    #[allow(dead_code)]
    pub fn rotate_theme(&mut self) {
        todo!("Rotate through different themes")
    }
}
