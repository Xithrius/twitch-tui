use std::vec;

use serde::{Deserialize, Serialize};

use crate::{config::LogLevel, handlers::state::State};

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

impl From<TerminalConfig> for Vec<(String, String)> {
    fn from(config: TerminalConfig) -> Self {
        vec![
            ("Current channel".to_string(), config.delay.to_string()),
            (
                "Max messages".to_string(),
                config.maximum_messages.to_string(),
            ),
            (
                "Log file".to_string(),
                config
                    .log_file
                    .clone()
                    .unwrap_or_else(|| "None".to_string()),
            ),
            ("First state".to_string(), config.first_state.to_string()),
        ]
    }
}
