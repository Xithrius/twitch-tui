#[derive(Debug, Clone)]
pub enum TwitchAction {
    Message(String),
    JoinChannel(String),
}
