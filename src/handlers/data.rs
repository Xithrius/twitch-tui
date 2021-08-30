#[derive(Debug, Clone)]
pub struct Data {
    pub time_sent: String,
    pub author: String,
    pub message: String,
}

impl Data {
    pub fn to_vec(&self) -> Vec<String> {
        return vec![self.time_sent.to_string(), self.author.to_string(), self.message.to_string()]
    }
}