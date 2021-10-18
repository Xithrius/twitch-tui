use std::collections::VecDeque;

use rustyline::line_buffer::LineBuffer;
use tui::layout::Constraint;

use crate::handlers::data::Data;

pub enum State {
    Normal,
    KeybindHelp,
    Input,
}

pub struct App {
    /// History of recorded messages (time, username, message)
    pub messages: VecDeque<Data>,
    /// Which window the terminal is currently showing
    pub state: State,
    /// Current value of the input box
    pub input_text: LineBuffer,
    /// The constraints that are set on the table
    pub table_constraints: Option<Vec<Constraint>>,
    /// The titles of the columns within the table of the terminal
    pub column_titles: Option<Vec<String>>,
    /// How many lines have been scrolled up/down
    pub scroll_offset: usize,
    /// If the amount of messages is 1 more than the window height for chat.
    pub allow_scrolling: bool,
    /// How large the chat window is.
    pub chat_chunk_height: Option<usize>,
}

impl App {
    pub fn new(data_limit: usize) -> App {
        App {
            messages: VecDeque::with_capacity(data_limit),
            state: State::Normal,
            input_text: LineBuffer::with_capacity(4096),
            table_constraints: None,
            column_titles: None,
            scroll_offset: 0,
            allow_scrolling: false,
            chat_chunk_height: None,
        }
    }
}
