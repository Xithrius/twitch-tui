use std::vec;

use serde::{Deserialize, Serialize};

use crate::handlers::{
    config::{LogLevel, ToVec},
    state::State,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct TerminalConfig {
    /// The delay in milliseconds between terminal updates.
    pub delay: u64,
    /// The maximum amount of messages before truncation.
    pub maximum_messages: usize,
    /// The file path to log to.
    pub log_file: Option<String>,
    /// Which log level the tracing library should be set to.
    pub log_level: LogLevel,
    /// What state the application should start in.
    pub first_state: State,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            delay: 30,
            maximum_messages: 500,
            log_file: None,
            log_level: LogLevel::INFO,
            first_state: State::default(),
        }
    }
}

impl ToVec<(String, String)> for TerminalConfig {
    fn to_vec(&self) -> Vec<(String, String)> {
        vec![
            ("Current channel".to_string(), self.delay.to_string()),
            (
                "Max messages".to_string(),
                self.maximum_messages.to_string(),
            ),
            (
                "Log file".to_string(),
                self.log_file.clone().unwrap_or_else(|| "None".to_string()),
            ),
            ("First state".to_string(), self.first_state.to_string()),
        ]
    }
}
