use std::vec;

use serde::{Deserialize, Serialize};

pub static DEFAULT_WEBSOCKET_SERVER: &str = "wss://eventsub.wss.twitch.tv/ws";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct TwitchConfig {
    /// The current user's name.
    pub username: String,
    /// The streamer's channel name.
    pub channel: String,
    /// The websocket server to connect to.
    pub server: String,
    /// The authentication token for the websocket server.
    pub token: Option<String>,
    /// Keepalive timeout.
    pub keepalive_timeout_seconds: usize,
    /// Bypass sanitization error for invalid websocket server, try to connect anyway.
    pub bypass_invalid_websocket_server: bool,
}

impl TwitchConfig {
    #[must_use]
    pub fn config_twitch_websocket_url(&self) -> String {
        format!(
            "{}?keepalive_timeout_seconds={}",
            self.server, self.keepalive_timeout_seconds
        )
    }
}

impl Default for TwitchConfig {
    fn default() -> Self {
        Self {
            username: String::new(),
            channel: String::new(),
            server: DEFAULT_WEBSOCKET_SERVER.to_string(),
            token: None,
            keepalive_timeout_seconds: 30,
            bypass_invalid_websocket_server: false,
        }
    }
}

impl From<TwitchConfig> for Vec<(String, String)> {
    fn from(config: TwitchConfig) -> Self {
        vec![
            ("Username".to_string(), config.username.clone()),
            ("Channel".to_string(), config.channel.clone()),
            ("Server".to_string(), config.server),
        ]
    }
}
