use std::vec;

use serde::{Deserialize, Serialize};

use crate::handlers::config::ToVec;

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

impl ToVec<(String, String)> for StorageConfig {
    fn to_vec(&self) -> Vec<(String, String)> {
        vec![
            ("Channels enabled".to_string(), self.channels.to_string()),
            ("Mentions enabled".to_string(), self.mentions.to_string()),
            ("Chatters enabled".to_string(), self.chatters.to_string()),
        ]
    }
}
