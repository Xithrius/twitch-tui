use std::{path::PathBuf, vec};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct FiltersConfig {
    /// Filters for chat messages
    pub message: MessageFiltersConfig,
    /// Filters for chat usernames
    pub username: UsernameFiltersConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct MessageFiltersConfig {
    pub path: Option<PathBuf>,
    pub filters: Option<Vec<String>>,
    pub enabled: bool,
    pub reversed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct UsernameFiltersConfig {
    pub path: Option<PathBuf>,
    pub filters: Option<Vec<String>>,
    pub enabled: bool,
    pub reversed: bool,
}

impl From<MessageFiltersConfig> for Vec<(String, String)> {
    fn from(config: MessageFiltersConfig) -> Self {
        vec![
            ("Path".to_string(), format!("{:?}", config.path)),
            ("Filters".to_string(), format!("{:?}", config.filters)),
            ("Enabled".to_string(), config.enabled.to_string()),
            ("Reversed".to_string(), config.reversed.to_string()),
        ]
    }
}

impl From<UsernameFiltersConfig> for Vec<(String, String)> {
    fn from(config: UsernameFiltersConfig) -> Self {
        vec![
            ("Path".to_string(), format!("{:?}", config.path)),
            ("Filters".to_string(), format!("{:?}", config.filters)),
            ("Enabled".to_string(), config.enabled.to_string()),
            ("Reversed".to_string(), config.reversed.to_string()),
        ]
    }
}
