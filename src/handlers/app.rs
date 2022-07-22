use std::{
    cmp::Eq,
    collections::{HashMap, VecDeque},
    hash::Hash,
};

use enum_iterator::IntoEnumIterator;
use rustyline::line_buffer::LineBuffer;
use tui::layout::Constraint;

use crate::handlers::{config::CompleteConfig, data::Data, filters::Filters, storage::Storage};

pub enum State {
    Normal,
    MessageInput,
    Help,
    ChannelSwitch,
    MessageSearch,
}

#[derive(PartialEq, Eq, Hash, IntoEnumIterator)]
pub enum BufferName {
    Chat,
    Channel,
    MessageHighlighter,
}

pub struct App {
    /// History of recorded messages (time, username, message).
    pub messages: VecDeque<Data>,
    /// Data loaded in from a JSON file.
    pub storage: Storage,
    /// Filtering out messages, no usernames since Twitch does that themselves.
    pub filters: Filters,
    /// Which window the terminal is currently showing.
    pub state: State,
    /// Which input buffer is currently selected.
    pub selected_buffer: BufferName,
    /// The current suggestion for a specific buffer.
    pub buffer_suggestion: String,
    /// Current value of the input box.
    pub input_buffers: HashMap<BufferName, LineBuffer>,
    /// The constraints that are set on the table.
    pub table_constraints: Option<Vec<Constraint>>,
    /// The titles of the columns within the table of the terminal.
    pub column_titles: Option<Vec<String>>,
    /// Scrolling offset for windows.
    pub scroll_offset: usize,
}

impl App {
    pub fn new(config: CompleteConfig) -> Self {
        let mut input_buffers_map = HashMap::new();

        for name in BufferName::into_enum_iter() {
            input_buffers_map.insert(name, LineBuffer::with_capacity(4096));
        }

        Self {
            messages: VecDeque::with_capacity(config.terminal.maximum_messages),
            storage: Storage::new("storage.json", config.storage),
            filters: Filters::new("filters.txt", config.filters),
            state: State::Normal,
            selected_buffer: BufferName::Chat,
            buffer_suggestion: "".to_string(),
            input_buffers: input_buffers_map,
            table_constraints: None,
            column_titles: None,
            scroll_offset: 0,
        }
    }

    pub fn current_buffer(&self) -> &LineBuffer {
        return self.input_buffers.get(&self.selected_buffer).unwrap();
    }

    pub fn current_buffer_mut(&mut self) -> &mut LineBuffer {
        return self.input_buffers.get_mut(&self.selected_buffer).unwrap();
    }

    pub fn cleanup(&self) {
        self.storage.dump_data();
    }
}
