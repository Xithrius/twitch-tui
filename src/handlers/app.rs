use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use rustyline::line_buffer::LineBuffer;
use tokio::sync::broadcast::Sender;
use toml::Table;
use tui::{backend::Backend, Frame};

use crate::{
    emotes::Emotes,
    handlers::{
        config::{CompleteConfig, Theme},
        data::MessageData,
        filters::Filters,
        state::State,
        storage::Storage,
        user_input::{events::Event, input::TerminalAction, scrolling::Scrolling},
    },
    twitch::TwitchAction,
    ui::{
        components::{Component, Components},
        statics::LINE_BUFFER_CAPACITY,
    },
};

pub struct App<'a> {
    /// All the available components.
    pub components: Components<'a>,
    /// History of recorded messages (time, username, message, etc).
    pub messages: VecDeque<MessageData>,
    /// Data loaded in from a JSON file.
    pub storage: Storage,
    /// Messages to be filtered out.
    pub filters: Filters,
    /// Which window the terminal is currently focused on.
    state: State,
    /// The previous state, if any.
    previous_state: Option<State>,
    /// What the user currently has inputted.
    pub input_buffer: LineBuffer,
    /// The current suggestion, if any.
    pub buffer_suggestion: Option<String>,
    /// Interactions with scrolling of the application.
    pub scrolling: Scrolling,
    /// The theme selected by the user.
    pub theme: Theme,
}

impl App<'_> {
    pub fn new(
        config: CompleteConfig,
        raw_config: Option<Table>,
        tx: Sender<TwitchAction>,
    ) -> Self {
        let shared_config = Rc::new(RefCell::new(config));

        let shared_config_borrow = shared_config.borrow();

        let storage = Storage::new("storage.json", &shared_config_borrow.storage);

        let components = Components::new(&shared_config, raw_config, tx, storage);

        Self {
            components,
            messages: VecDeque::with_capacity(shared_config_borrow.terminal.maximum_messages),
            storage,
            filters: Filters::new("filters.txt", &shared_config_borrow.filters),
            state: shared_config_borrow.terminal.start_state.clone(),
            previous_state: None,
            input_buffer: LineBuffer::with_capacity(*LINE_BUFFER_CAPACITY),
            buffer_suggestion: None,
            theme: shared_config_borrow.frontend.theme.clone(),
            scrolling: Scrolling::new(shared_config_borrow.frontend.inverted_scrolling),
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, emotes: Emotes) {
        let size = f.size();

        if size.height < 10 || size.width < 60 {
            self.components.error.draw(f, Some(size));
        } else {
            match self.state {
                State::Dashboard => todo!(),
                State::Normal => todo!(),
                State::Insert => todo!(),
                State::Help => todo!(),
                State::ChannelSwitch => self.components.channel_switcher.draw(f, emotes),
                State::MessageSearch => todo!(),
            }
        }
        // } else if app.get_state() == State::Dashboard
        //     || (Some(State::Dashboard) == app.get_previous_state()
        //         && State::ChannelSwitch == app.get_state())
        // {
        //     render_dashboard_ui(f, &mut app, &config);
        // } else {
        //     render_chat_ui(f, &mut app, &config, &mut emotes);
        // }
    }

    pub fn event(&mut self, event: Event) -> Option<TerminalAction> {
        match self.state {
            State::Dashboard => todo!(),
            State::Normal => todo!(),
            State::Insert => todo!(),
            State::Help => todo!(),
            State::ChannelSwitch => self.components.channel_switcher.event(event),
            State::MessageSearch => todo!(),
        }
    }

    pub fn cleanup(&self) {
        self.storage.dump_data();
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();

        self.scrolling.jump_to(0);
    }

    pub fn get_previous_state(&self) -> Option<State> {
        self.previous_state.clone()
    }

    pub fn get_state(&self) -> State {
        self.state.clone()
    }

    pub fn set_state(&mut self, other: State) {
        self.previous_state = Some(self.state.clone());
        self.state = other;
    }

    #[allow(dead_code)]
    pub fn rotate_theme(&mut self) {
        todo!("Rotate through different themes")
    }
}
