use std::vec;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct StorageConfig {
    /// If previous channels switched to should be tracked.
    pub channels: bool,
    /// If previous username mentions should be tracked.
    pub mentions: bool,
    /// If chatters previously in a room should be tracked
    pub chatters: bool,
}

impl From<StorageConfig> for Vec<(String, String)> {
    fn from(config: StorageConfig) -> Self {
        vec![
            ("Channels enabled".to_string(), config.channels.to_string()),
            ("Mentions enabled".to_string(), config.mentions.to_string()),
            ("Chatters enabled".to_string(), config.chatters.to_string()),
        ]
    }
}
