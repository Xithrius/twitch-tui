use std::collections::{HashMap, VecDeque};

use maplit::hashmap;
use rustyline::line_buffer::LineBuffer;
use tui::layout::Constraint;

use crate::handlers::data::Data;

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
    /// Current value of different input boxes
    pub input_boxes: HashMap<&'static str, LineBuffer>,
    /// The constraints that are set on the table
    pub table_constraints: Option<Vec<Constraint>>,
    /// The titles of the columns within the table of the terminal
    pub column_titles: Option<Vec<String>>,
}

impl App {
    pub fn new(data_limit: usize) -> App {
        App {
            messages: VecDeque::with_capacity(data_limit),
            state: State::Normal,
            input_boxes: hashmap! {
                "chat" => LineBuffer::with_capacity(4096),
            },
            table_constraints: None,
            column_titles: None,
        }
    }
}
