#![allow(clippy::use_self)]

use std::{
    cmp::{Eq, PartialEq},
    collections::VecDeque,
};

use rustyline::line_buffer::LineBuffer;
use serde::{Deserialize, Serialize};

use crate::handlers::{
    config::{CompleteConfig, Theme},
    data::MessageData,
    filters::Filters,
    storage::Storage,
};

const INPUT_BUFFER_LIMIT: usize = 4096;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum State {
    Normal,
    Insert,
    Help,
    ChannelSwitch,
    MessageSearch,
}

impl State {
    pub const fn in_insert_mode(&self) -> bool {
        matches!(
            self,
            Self::Insert | Self::ChannelSwitch | Self::MessageSearch
        )
    }

    /// What general category the state can be identified with.
    pub fn category(&self) -> String {
        if self.in_insert_mode() {
            "Insert modes".to_string()
        } else {
            self.to_string()
        }
    }
}

impl ToString for State {
    fn to_string(&self) -> String {
        match self {
            Self::Normal => "Normal",
            Self::Insert => "Insert",
            Self::Help => "Help",
            Self::ChannelSwitch => "Channel",
            Self::MessageSearch => "Search",
        }
        .to_string()
    }
}

pub struct Scrolling {
    /// Offset of scroll
    offset: usize,
    /// If the scrolling is currently inverted
    inverted: bool,
}

impl Scrolling {
    pub const fn new(inverted: bool) -> Self {
        Self {
            offset: 0,
            inverted,
        }
    }

    /// Scrolling upwards, towards the start of the chat
    pub fn up(&mut self) {
        self.offset += 1;
    }

    /// Scrolling downwards, towards the most recent message(s)
    pub fn down(&mut self) {
        self.offset = self.offset.saturating_sub(1);
    }

    pub const fn inverted(&self) -> bool {
        self.inverted
    }

    pub fn jump_to(&mut self, index: usize) {
        self.offset = index;
    }

    pub const fn get_offset(&self) -> usize {
        self.offset
    }
}

pub struct App {
    /// History of recorded messages (time, username, message, etc.)
    pub messages: VecDeque<MessageData>,
    /// Data loaded in from a JSON file.
    pub storage: Storage,
    /// Messages to be filtered out
    pub filters: Filters,
    /// Which window the terminal is currently focused on
    state: State,
    /// What the user currently has inputted
    pub input_buffer: LineBuffer,
    /// The current suggestion, if any
    pub buffer_suggestion: Option<String>,
    /// Interactions with scrolling of the application
    pub scrolling: Scrolling,
    /// The theme selected by the user
    pub theme: Theme,
}

impl App {
    pub fn new(config: &CompleteConfig) -> Self {
        Self {
            messages: VecDeque::with_capacity(config.terminal.maximum_messages),
            storage: Storage::new("storage.json", &config.storage),
            filters: Filters::new("filters.txt", &config.filters),
            state: config.terminal.start_state.clone(),
            input_buffer: LineBuffer::with_capacity(INPUT_BUFFER_LIMIT),
            buffer_suggestion: None,
            theme: config.frontend.theme.clone(),
            scrolling: Scrolling::new(config.frontend.inverted_scrolling),
        }
    }

    pub fn cleanup(&self) {
        self.storage.dump_data();
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();

        self.scrolling.jump_to(0);
    }

    pub fn get_state(&self) -> State {
        self.state.clone()
    }

    pub fn set_state(&mut self, other: State) {
        self.state = other;
    }

    #[allow(dead_code)]
    pub fn rotate_theme(&mut self) {
        todo!("Rotate through different themes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_scroll_overflow_not_inverted() {
        let mut scroll = Scrolling::new(false);
        assert_eq!(scroll.get_offset(), 0);

        scroll.down();
        assert_eq!(scroll.get_offset(), 0);
    }
}
