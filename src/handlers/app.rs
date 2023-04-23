use std::{cmp::PartialEq, collections::VecDeque};

use rustyline::line_buffer::LineBuffer;
use toml::Table;

use crate::handlers::{
    config::{CompleteConfig, Theme},
    data::MessageData,
    filters::Filters,
    state::State,
    storage::Storage,
};

const INPUT_BUFFER_LIMIT: usize = 4096;

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

#[derive(Debug, PartialEq, Clone)]
pub struct DebugWindow {
    visible: bool,
    pub raw_config: Option<Table>,
}

impl DebugWindow {
    const fn new(visible: bool, raw_config: Option<Table>) -> Self {
        Self {
            visible,
            raw_config,
        }
    }

    pub const fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

pub struct App {
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
    /// If the debug window is visible.
    pub debug: DebugWindow,
}

impl App {
    pub fn new(config: &CompleteConfig, raw_config: Option<Table>) -> Self {
        Self {
            messages: VecDeque::with_capacity(config.terminal.maximum_messages),
            storage: Storage::new("storage.json", &config.storage),
            filters: Filters::new("filters.txt", &config.filters),
            state: config.terminal.start_state.clone(),
            previous_state: None,
            input_buffer: LineBuffer::with_capacity(INPUT_BUFFER_LIMIT),
            buffer_suggestion: None,
            theme: config.frontend.theme.clone(),
            scrolling: Scrolling::new(config.frontend.inverted_scrolling),
            debug: DebugWindow::new(false, raw_config),
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
