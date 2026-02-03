use std::vec;

use serde::{Deserialize, Serialize};

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
    /// Keepalive timeout
    pub keepalive_timeout_seconds: usize,
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
            server: "wss://eventsub.wss.twitch.tv/ws".to_string(),
            token: None,
            keepalive_timeout_seconds: 30,
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
