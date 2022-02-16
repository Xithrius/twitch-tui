use std::collections::{HashMap, VecDeque};

use enum_iterator::IntoEnumIterator;
use rustyline::line_buffer::LineBuffer;
use tui::layout::Constraint;

use crate::{
    handlers::{config::CompleteConfig, data::Data, filter::Filter},
    utils::pathing::config_path,
};

#[allow(dead_code)]
pub enum State {
    Normal,
    Input,
    Help,
    ChannelSwitch,
    MessageSearch,
    FilterSearch,
}

#[derive(PartialEq, std::cmp::Eq, std::hash::Hash, IntoEnumIterator)]
pub enum BufferName {
    Chat,
    Channels,
    Messages,
    Filters,
}

pub struct App {
    /// History of recorded messages (time, username, message)
    pub messages: VecDeque<Data>,
    /// A temporary snapshot of current messages
    pub messages_snapshot: VecDeque<Data>,
    /// Which window the terminal is currently showing
    pub state: State,
    /// Which input buffer is currently selected
    pub selected_buffer: BufferName,
    /// Current value of the input box
    pub input_buffers: HashMap<BufferName, LineBuffer>,
    /// The constraints that are set on the table
    pub table_constraints: Option<Vec<Constraint>>,
    /// The titles of the columns within the table of the terminal
    pub column_titles: Option<Vec<String>>,
    /// Scrolling offset for windows
    pub scroll_offset: usize,
    /// Filtering messages
    pub filter: Filter,
}

impl App {
    pub fn new(config: CompleteConfig) -> Self {
        let mut input_buffers_map = HashMap::new();

        for name in BufferName::into_enum_iter() {
            input_buffers_map.insert(name, LineBuffer::with_capacity(4096));
        }

        let filters = if let Ok(filter_file) = std::fs::read_to_string(config_path("filters.txt")) {
            filter_file
                .split('\n')
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        } else {
            vec![]
        };

        Self {
            messages: VecDeque::with_capacity(config.terminal.maximum_messages),
            messages_snapshot: VecDeque::with_capacity(config.terminal.maximum_messages),
            state: State::Normal,
            selected_buffer: BufferName::Chat,
            input_buffers: input_buffers_map,
            table_constraints: None,
            column_titles: None,
            scroll_offset: 0,
            filter: Filter::new(filters, true),
        }
    }

    pub fn current_buffer(&self) -> &LineBuffer {
        return self.input_buffers.get(&self.selected_buffer).unwrap();
    }

    pub fn current_buffer_mut(&mut self) -> &mut LineBuffer {
        return self.input_buffers.get_mut(&self.selected_buffer).unwrap();
    }
}
