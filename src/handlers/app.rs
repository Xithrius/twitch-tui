use std::{
    cmp::{Eq, PartialEq},
    collections::VecDeque,
    hash::Hash,
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

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum State {
    Normal,
    Insert,
    Help,
    ChannelSwitch,
    MessageSearch,
}

#[derive(Debug)]
pub struct App {
    /// History of recorded messages (time, username, message).
    pub messages: VecDeque<Data>,
    /// Data loaded in from a JSON file.
    pub storage: Storage,
    /// Filtering out messages, no usernames since Twitch does that themselves.
    pub filters: Filters,
    /// Which window the terminal is currently showing.
    pub state: State,
    /// The state of the user's input.
    pub input_buffer: LineBuffer,
    /// The current suggestion for a specific buffer.
    pub buffer_suggestion: Option<String>,
    /// Scrolling offset for windows.
    pub scroll_offset: usize,
    /// The styling for the theme.
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
            buffer_suggestion: Some("".to_string()),
            scroll_offset: 0,
            theme_style: match config.frontend.theme {
                Theme::Light => BORDER_NAME_LIGHT,
                _ => BORDER_NAME_DARK,
            },
        }
    }

    // pub fn current_buffer(&self) -> &LineBuffer {
    //     return self.input_buffers.get(&self.selected_buffer).unwrap();
    // }

    // pub fn current_buffer_mut(&mut self) -> &mut LineBuffer {
    //     return self.input_buffers.get_mut(&self.selected_buffer).unwrap();
    // }

    pub fn cleanup(&self) {
        self.storage.dump_data();
    }
}
