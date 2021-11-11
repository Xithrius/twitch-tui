use std::collections::VecDeque;

use indexmap::IndexMap;
use rustyline::line_buffer::LineBuffer;
use tui::layout::Constraint;

use crate::{
    handlers::{config::CompleteConfig, data::Data},
    ui::statics::INPUT_TAB_TITLES,
};

pub enum State {
    Normal,
    Input,
    Help,
}

pub struct App {
    /// History of recorded messages (time, username, message)
    pub messages: VecDeque<Data>,
    /// Which window the terminal is currently showing
    pub state: State,
    /// Current value of the input box
    pub input_buffers: IndexMap<&'static str, LineBuffer>,
    /// The offset index for tabs
    pub tab_offset: usize,
    /// The constraints that are set on the table
    pub table_constraints: Option<Vec<Constraint>>,
    /// The titles of the columns within the table of the terminal
    pub column_titles: Option<Vec<String>>,
}

impl App {
    pub fn new(config: CompleteConfig) -> App {
        let mut input_map = IndexMap::new();

        let config_values = vec![
            "".to_string(),
            config.twitch.channel,
            config.twitch.username,
            config.twitch.server,
        ];

        for (&k, v) in INPUT_TAB_TITLES.iter().zip(config_values) {
            let mut buffer = LineBuffer::with_capacity(4096);

            buffer.update(&v, v.len());

            input_map.insert(k, buffer);
        }

        App {
            messages: VecDeque::with_capacity(config.terminal.maximum_messages),
            state: State::Normal,
            input_buffers: input_map,
            tab_offset: 0,
            table_constraints: None,
            column_titles: None,
        }
    }
}
