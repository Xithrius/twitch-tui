use std::vec;

use serde::{Deserialize, Serialize};

use crate::handlers::config::ToVec;

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

impl ToVec<(String, String)> for TwitchConfig {
    fn to_vec(&self) -> Vec<(String, String)> {
        vec![
            ("Username".to_string(), self.username.clone()),
            ("Channel".to_string(), self.channel.clone()),
            ("Server".to_string(), self.server.clone()),
        ]
    }
}
