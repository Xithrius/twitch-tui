use std::collections::VecDeque;

use tui::layout::Constraint;

use crate::handlers::data::Data;

pub enum State {
    Normal,
    KeybindHelp,
    Input,
}

pub struct App {
    /// Current value of the input box
    pub input: String,
    /// History of recorded messages (time, username, message)
    pub messages: VecDeque<Data>,
    /// Which window the terminal is currently showing
    pub state: State,
    /// The constraints that are set on the table
    pub table_constraints: Option<Vec<Constraint>>,
    /// The titles of the columns within the table of the terminal
    pub column_titles: Option<Vec<String>>,
}

impl App {
    pub fn new(data_limit: usize) -> App {
        App {
            input: String::new(),
            messages: VecDeque::with_capacity(data_limit),
            state: State::Normal,
            table_constraints: None,
            column_titles: None,
        }
    }
}
