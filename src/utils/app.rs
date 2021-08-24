use chrono::offset::Local;

pub struct App {
    /// Current value of the input box
    pub input: String,
    /// History of recorded messages (time, username, message)
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

impl App {
    pub fn insert_message(&mut self, user: String, message: String) {
        self.messages.push(vec![
            format!("{}", Local::now().format("%a %b %e %T %Y")),
            user,
            message,
        ]);
    }
}
