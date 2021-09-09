use std::collections::VecDeque;

use crate::handlers::data::Data;

pub struct App {
    /// Current value of the input box
    pub input: String,
    /// History of recorded messages (time, username, message)
    pub messages: VecDeque<Data>,
}

impl App {
    pub fn new(data_limit: usize) -> App {
        App {
            input: String::new(),
            messages: VecDeque::with_capacity(data_limit),
        }
    }
}
