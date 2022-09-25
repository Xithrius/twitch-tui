use std::{
    cmp::{Eq, PartialEq},
    collections::VecDeque,
};

use rustyline::line_buffer::LineBuffer;
use tui::style::Style;

use crate::{
    handlers::{
        config::{CompleteConfig, Theme},
        data::Data,
        filters::Filters,
        storage::Storage,
    },
    utils::styles::{BORDER_NAME_DARK, BORDER_NAME_LIGHT},
};

const INPUT_BUFFER_LIMIT: usize = 4096;

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    Normal,
    Insert,
    Help,
    ChannelSwitch,
    MessageSearch,
}

impl ToString for State {
    fn to_string(&self) -> String {
        match self {
            Self::Normal => "Normal",
            Self::Help => "Help",
            _ => "Input modes",
        }
        .to_string()
    }
}

#[derive(Debug)]
pub struct App {
    /// History of recorded messages (time, username, message, etc.)
    pub messages: VecDeque<Data>,
    /// Data loaded in from a JSON file.
    pub storage: Storage,
    /// Messages to be filtered out
    pub filters: Filters,
    /// Which window the terminal is currently focused on
    pub state: State,
    /// What the user currently has inputted
    pub input_buffer: LineBuffer,
    /// The current suggestion, if any
    pub buffer_suggestion: Option<String>,
    /// Scrolling offset for the main window
    pub scroll_offset: usize,
    /// The theme selected by the user
    pub theme_style: Style,
}

impl App {
    pub fn new(config: &CompleteConfig) -> Self {
        Self {
            messages: VecDeque::with_capacity(config.terminal.maximum_messages),
            storage: Storage::new("storage.json", &config.storage),
            filters: Filters::new("filters.txt", &config.filters),
            state: State::Normal,
            input_buffer: LineBuffer::with_capacity(INPUT_BUFFER_LIMIT),
            buffer_suggestion: None,
            scroll_offset: 0,
            theme_style: match config.frontend.theme {
                Theme::Light => BORDER_NAME_LIGHT,
                _ => BORDER_NAME_DARK,
            },
        }
    }

    pub fn cleanup(&self) {
        self.storage.dump_data();
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();

        self.scroll_offset = 0;
    }
}
