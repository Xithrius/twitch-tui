pub struct App {
    /// Current value of the input box.
    pub input: String,
    /// History of recorded messages (time, username, message).
    pub messages: Vec<Vec<String>>,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            messages: Vec::new(),
        }
    }
}
