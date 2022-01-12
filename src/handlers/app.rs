use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
};

use anyhow::{Context, Result};
use emote_mapper::EmoteMapper;
use enum_iterator::IntoEnumIterator;
use rustyline::line_buffer::LineBuffer;
use tui::layout::Constraint;

use crate::handlers::{config::CompleteConfig, data::Data};

pub enum State {
    Normal,
    MessageInput,
    Help,
    ChannelSwitch,
    MessageSearch,
}

#[derive(PartialEq, std::cmp::Eq, std::hash::Hash, IntoEnumIterator)]
pub enum BufferName {
    Chat,
    Channel,
    MessageHighlighter,
}

pub struct App {
    /// History of recorded messages (time, username, message)
    pub messages: VecDeque<Data>,
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
    /// Maps twitchemotes to emotes to display
    pub emote_mapper: EmoteMapper,
}

impl App {
    pub fn new(config: CompleteConfig) -> Result<Self> {
        let mut input_buffers_map = HashMap::new();

        for name in BufferName::into_enum_iter() {
            input_buffers_map.insert(name, LineBuffer::with_capacity(4096));
        }

        let emote_mapper = if let Some(emote_mapping) = config.frontend.emote_mapper {
            let emote_mapping = shellexpand::tilde(&emote_mapping);
            EmoteMapper::from_path(&*emote_mapping)
                .with_context(|| format!("Loading emote map at {:?}", emote_mapping))?
        } else {
            EmoteMapper::default()
        };

        Ok(Self {
            messages: VecDeque::with_capacity(config.terminal.maximum_messages),
            state: State::Normal,
            selected_buffer: BufferName::Chat,
            input_buffers: input_buffers_map,
            table_constraints: None,
            column_titles: None,
            scroll_offset: 0,
            emote_mapper,
        })
    }

    pub fn current_buffer(&self) -> &LineBuffer {
        return self.input_buffers.get(&self.selected_buffer).unwrap();
    }

    pub fn current_buffer_mut(&mut self) -> &mut LineBuffer {
        return self.input_buffers.get_mut(&self.selected_buffer).unwrap();
    }
}
